use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use tauri::{Emitter, Manager};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionMode {
    Backend,
    Native(crate::public_setup::SpeechProvider),
}

#[derive(Clone)]
pub struct CallStreamSessionSnapshot {
    pub access_token: String,
    pub backend_url: String,
    pub user_id: String,
    pub mode: SessionMode,
    generation: u64,
}
#[derive(Default)]
struct Session {
    current: Option<CallStreamSessionSnapshot>,
    generation: u64,
}
static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
fn session() -> &'static Mutex<Session> {
    SESSION.get_or_init(|| Mutex::new(Session::default()))
}
#[derive(Serialize, Deserialize)]
struct Connection {
    backend_url: String,
    user_id: String,
}
#[cfg(not(debug_assertions))]
const KEYCHAIN_SERVICE: &str = "org.destroy.dictation.community.backend";
#[cfg(debug_assertions)]
const KEYCHAIN_SERVICE: &str = "org.destroy.dictation.community.dev.backend";
pub fn backend_url() -> String {
    cached_call_stream_session_snapshot()
        .map(|a| a.backend_url)
        .unwrap_or_default()
}
pub fn cached_call_stream_session_snapshot() -> Option<CallStreamSessionSnapshot> {
    session().lock().ok()?.current.clone()
}
pub(crate) fn current_generation() -> Result<u64, String> {
    Ok(session().lock().map_err(|_| "Session busy")?.generation)
}

/// Invalidates live sessions and transient dictation work while retaining
/// persisted connection credentials. Used by the non-destructive onboarding
/// restart flow.
pub(crate) fn invalidate_session(app: &tauri::AppHandle) -> Result<u64, String> {
    let mut s = session().lock().map_err(|_| "Session busy")?;
    s.generation = s.generation.wrapping_add(1);
    let generation = s.generation;
    s.current = None;
    drop(s);
    crate::dictation_stream::prepare_account_boundary(app)?;
    app.state::<crate::native_audio::NativeAudio>().clear();
    crate::dictation::destroy_dictation_clear_sensitive_state(app.state());
    let _ = app.emit("destroy://account-changed", ());
    Ok(generation)
}
pub(crate) fn install_native_session(
    generation: u64,
    provider: crate::public_setup::SpeechProvider,
    key: String,
) -> Result<(), String> {
    let mut guard = session().lock().map_err(|_| "Session busy")?;
    if guard.generation != generation {
        return Err("Connection changed; try again".into());
    }
    guard.current = Some(CallStreamSessionSnapshot {
        access_token: key,
        backend_url: crate::public_setup::PUBLIC_BACKEND_URL.into(),
        user_id: crate::public_setup::LOCAL_USER_ID.into(),
        mode: SessionMode::Native(provider),
        generation,
    });
    Ok(())
}
pub fn call_stream_session_snapshot_is_current(auth: &CallStreamSessionSnapshot) -> bool {
    session().lock().is_ok_and(|s| {
        s.generation == auth.generation
            && s.current
                .as_ref()
                .is_some_and(|a| a.user_id == auth.user_id && a.backend_url == auth.backend_url)
    })
}
pub async fn ensure_fresh_session() -> Result<(), String> {
    match cached_call_stream_session_snapshot() {
        Some(snapshot) if matches!(snapshot.mode, SessionMode::Backend) => Ok(()),
        Some(_) => Err("Live dictation is unavailable in native speech mode".into()),
        None => Err("Connect your dictation backend in Settings".into()),
    }
}
fn validate_url(raw: &str) -> Result<String, String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "Enter a valid backend URL")?;
    if url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err("Backend URL must contain only an origin".into());
    }
    if url.scheme() != "https"
        && !(url.scheme() == "http"
            && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")))
    {
        return Err("Use HTTPS, or HTTP on loopback for self-hosting".into());
    }
    Ok(url.to_string().trim_end_matches('/').to_string())
}
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(40))
        .build()
        .map_err(|_| "Cannot create backend connection".into())
}
async fn request(
    snapshot: &CallStreamSessionSnapshot,
    method: &str,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if !path.starts_with("/v1/") || path.contains("..") || path.contains('?') || path.contains('#')
    {
        return Err("Invalid backend path".into());
    }
    let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|_| "Invalid method")?;
    let request = client()?
        .request(method, format!("{}{path}", snapshot.backend_url))
        .bearer_auth(&snapshot.access_token);
    let response = request
        .json(&body)
        .send()
        .await
        .map_err(|_| "Backend unavailable; check your connection")?;
    let status = response.status();
    let mut data = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Backend response interrupted")?;
        if data.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err("Backend response exceeds size limit".into());
        }
        data.extend_from_slice(&chunk);
    }
    let value: serde_json::Value =
        serde_json::from_slice(&data).map_err(|_| "Backend returned an invalid response")?;
    if !status.is_success() {
        return Err(value["error"]
            .as_str()
            .unwrap_or("Backend request failed")
            .chars()
            .take(300)
            .collect());
    }
    Ok(value)
}
pub(crate) fn boundary(app: &tauri::AppHandle) -> Result<u64, String> {
    let mut s = session().lock().map_err(|_| "Session busy")?;
    s.generation += 1;
    let generation = s.generation;
    s.current = None;
    #[cfg(target_os = "macos")]
    let keychain_clear = match security_framework::passwords::delete_generic_password(
        KEYCHAIN_SERVICE,
        "access-token",
    ) {
        Ok(()) => Ok(()),
        Err(e) if e.code() == -25300 => Ok(()),
        Err(_) => Err("Cannot clear backend token from Keychain".to_string()),
    };
    #[cfg(not(target_os = "macos"))]
    let keychain_clear: Result<(), String> = Ok(());
    let config_clear = match std::fs::remove_file(crate::app_identity::config_path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Cannot clear saved backend connection".to_string()),
    };
    drop(s);
    let streams = crate::dictation_stream::prepare_account_boundary(app);
    app.state::<crate::native_audio::NativeAudio>().clear();
    crate::dictation::destroy_dictation_clear_sensitive_state(app.state());
    let _ = app.emit("destroy://account-changed", ());
    streams?;
    keychain_clear?;
    config_clear?;
    Ok(generation)
}
#[tauri::command]
pub async fn configure_backend(
    app: tauri::AppHandle,
    backend_url: String,
    token: String,
) -> Result<serde_json::Value, String> {
    let url = validate_url(&backend_url)?;
    if !(32..=1024).contains(&token.len()) {
        return Err("Enter the backend access token (not a provider key)".into());
    }
    let generation = boundary(&app)?;
    let mut snapshot = CallStreamSessionSnapshot {
        access_token: token.clone(),
        backend_url: url.clone(),
        user_id: String::new(),
        mode: SessionMode::Backend,
        generation,
    };
    let result = request(&snapshot, "GET", "/v1/account", serde_json::Value::Null).await?;
    snapshot.user_id = result["user_id"]
        .as_str()
        .filter(|v| !v.is_empty())
        .ok_or("Backend did not identify the account")?
        .to_owned();
    let mut s = session().lock().map_err(|_| "Session busy")?;
    if s.generation != generation {
        return Err("Connection changed; try again".into());
    }
    #[cfg(target_os = "macos")]
    security_framework::passwords::set_generic_password(
        KEYCHAIN_SERVICE,
        "access-token",
        token.as_bytes(),
    )
    .map_err(|_| "Cannot save access token in Keychain")?;
    let path = crate::app_identity::config_path();
    let parent = path.parent().ok_or("Cannot find app storage")?;
    std::fs::create_dir_all(parent).map_err(|_| "Cannot create app storage")?;
    let config = serde_json::to_vec(&Connection {
        backend_url: url,
        user_id: snapshot.user_id.clone(),
    })
    .map_err(|_| "Cannot encode connection")?;
    std::fs::write(&path, config).map_err(|_| "Cannot save connection")?;
    crate::public_setup::clear_saved_native_profile()?;
    s.current = Some(snapshot);
    Ok(result)
}
#[tauri::command]
pub async fn restore_connection() -> Result<serde_json::Value, String> {
    if let Some(a) = cached_call_stream_session_snapshot() {
        return Ok(crate::public_setup::status_for_session(&a));
    }
    if crate::public_setup::has_saved_native_profile() {
        return crate::public_setup::restore_native_connection().await;
    }
    let (generation, c, url, token) = {
        let guard = session().lock().map_err(|_| "Session busy")?;
        let generation = guard.generation;
        let path = crate::app_identity::config_path();
        let c: Connection =
            serde_json::from_slice(&std::fs::read(path).map_err(|_| "Connect a backend to begin")?)
                .map_err(|_| "Saved connection invalid")?;
        let url = validate_url(&c.backend_url)?;
        #[cfg(target_os = "macos")]
        let token =
            security_framework::passwords::get_generic_password(KEYCHAIN_SERVICE, "access-token")
                .map_err(|_| "Reconnect to restore Keychain access")?;
        #[cfg(not(target_os = "macos"))]
        let token: Vec<u8> = vec![];
        (generation, c, url, token)
    };
    let a = CallStreamSessionSnapshot {
        access_token: String::from_utf8(token).map_err(|_| "Invalid access token")?,
        backend_url: url.clone(),
        user_id: c.user_id.clone(),
        mode: SessionMode::Backend,
        generation,
    };
    let result = request(&a, "GET", "/v1/account", serde_json::Value::Null).await?;
    if result["user_id"] != c.user_id {
        return Err("Backend account changed; reconnect explicitly".into());
    }
    let mut s = session().lock().map_err(|_| "Session busy")?;
    if s.generation != generation {
        return Err("Connection changed".into());
    }
    s.current = Some(a);
    Ok(serde_json::json!({"user_id":c.user_id,"backend_url":url}))
}
#[tauri::command]
pub fn disconnect_backend(app: tauri::AppHandle) -> Result<(), String> {
    boundary(&app)?;
    crate::public_setup::clear_saved_native_profile()?;
    Ok(())
}
pub(crate) fn validate_expected_account(
    snapshot: &CallStreamSessionSnapshot,
    user: Option<&str>,
    origin: Option<&str>,
) -> Result<(), String> {
    match (user, origin) {
        (None, None) => Ok(()),
        (Some(user), Some(origin))
            if user == snapshot.user_id && validate_url(origin)? == snapshot.backend_url =>
        {
            Ok(())
        }
        _ => Err("Account changed; request was not sent".into()),
    }
}
#[tauri::command]
pub async fn backend_api(
    method: String,
    path: String,
    body: Option<serde_json::Value>,
    expected_user_id: Option<String>,
    expected_backend_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let snapshot = cached_call_stream_session_snapshot().ok_or("Connect a backend first")?;
    validate_expected_account(
        &snapshot,
        expected_user_id.as_deref(),
        expected_backend_url.as_deref(),
    )?;
    let body = body.unwrap_or(serde_json::Value::Null);
    let result = match snapshot.mode {
        SessionMode::Backend => request(&snapshot, &method, &path, body).await?,
        SessionMode::Native(_) => {
            crate::public_setup::native_backend_api(&snapshot, &method, &path, body).await?
        }
    };
    if !call_stream_session_snapshot_is_current(&snapshot) {
        return Err("Account changed; discarded response".into());
    }
    Ok(result)
}
#[tauri::command]
pub async fn destroy_transcribe_dictation(
    audio_base64: String,
    mime_type: String,
    expected_user_id: Option<String>,
    expected_backend_url: Option<String>,
) -> Result<serde_json::Value, String> {
    backend_api(
        "POST".into(),
        "/v1/dictation/transcribe".into(),
        Some(serde_json::json!({"audio_base64":audio_base64,"mime_type":mime_type})),
        expected_user_id,
        expected_backend_url,
    )
    .await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queued_requests_require_the_captured_account_and_origin() {
        let snapshot = CallStreamSessionSnapshot {
            access_token: "test".into(),
            backend_url: "https://api.example.com".into(),
            user_id: "owner-a".into(),
            mode: SessionMode::Backend,
            generation: 0,
        };
        assert!(validate_expected_account(
            &snapshot,
            Some("owner-a"),
            Some("https://api.example.com/")
        )
        .is_ok());
        assert!(validate_expected_account(
            &snapshot,
            Some("owner-b"),
            Some("https://api.example.com")
        )
        .is_err());
        assert!(validate_expected_account(
            &snapshot,
            Some("owner-a"),
            Some("https://other.example.com")
        )
        .is_err());
        assert!(validate_expected_account(&snapshot, Some("owner-a"), None).is_err());
        assert!(
            validate_expected_account(&snapshot, None, Some("https://api.example.com")).is_err()
        );
    }
    #[test]
    fn backend_origin_boundary() {
        assert!(validate_url("http://127.0.0.1:8787").is_ok());
        for url in [
            "http://example.com",
            "https://user:pass@example.com",
            "https://example.com/v1",
            "https://example.com?secret=1",
        ] {
            assert!(validate_url(url).is_err())
        }
    }
}

/// Exports only after a user clicks Export. File is new, owner-readable, and scoped
/// to the captured account; a connection change prevents writing a stale result.
#[tauri::command]
pub async fn export_personal_data(app: tauri::AppHandle) -> Result<String, String> {
    use std::io::Write;
    let snapshot = cached_call_stream_session_snapshot().ok_or("Connect a backend first")?;
    let data = match snapshot.mode {
        SessionMode::Backend => {
            request(
                &snapshot,
                "GET",
                "/v1/account/export",
                serde_json::Value::Null,
            )
            .await?
        }
        SessionMode::Native(_) => {
            crate::public_setup::native_backend_api(
                &snapshot,
                "GET",
                "/v1/account/export",
                serde_json::Value::Null,
            )
            .await?
        }
    };
    let bytes = serde_json::to_vec_pretty(&data).map_err(|_| "Cannot encode export")?;
    let directory = app
        .path()
        .download_dir()
        .map_err(|_| "Downloads folder unavailable")?;
    let path = directory.join(format!(
        "destroy-dictation-personal-data-{}.json",
        uuid::Uuid::new_v4()
    ));
    let locked = session().lock().map_err(|_| "Session busy")?;
    if locked.generation != snapshot.generation || locked.current.is_none() {
        return Err("Connection changed; export cancelled".into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .map_err(|_| "Cannot create export in Downloads")?;
    if file
        .write_all(&bytes)
        .and_then(|_| file.sync_all())
        .is_err()
    {
        let _ = std::fs::remove_file(&path);
        return Err("Cannot finish export".into());
    }
    Ok(path.to_string_lossy().into_owned())
}
