use base64::{engine::general_purpose::STANDARD, Engine as _};
use dictation_protocol::{
    DictationPreferences, DictationTransformOperation, DictationTransformRequest,
};
use futures_util::StreamExt;
use regex_lite::Regex;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Write},
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe::Hint,
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub const LOCAL_USER_ID: &str = "local-user";
pub const PUBLIC_BACKEND_URL: &str = "https://local.destroy.invalid";
const MAX_AUDIO_BYTES: usize = 6 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeechProvider {
    Local,
    Openai,
    Gemini,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct NativeProfile {
    provider: SpeechProvider,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSetupStatus {
    pub provider: Option<String>,
    pub ready: bool,
    pub model_ready: bool,
    pub model_downloading: bool,
    pub openai_key_present: bool,
    pub gemini_key_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_model_id: Option<String>,
    #[serde(rename = "user_id")]
    pub user_id: Option<String>,
    #[serde(rename = "backend_url")]
    pub backend_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

static STORAGE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
fn storage_lock() -> &'static Mutex<()> {
    STORAGE_LOCK.get_or_init(|| Mutex::new(()))
}

fn native_profile_path() -> PathBuf {
    crate::app_identity::config_path().with_file_name("native-profile.json")
}

fn local_data_path() -> PathBuf {
    crate::app_identity::config_path().with_file_name("public-local-data.json")
}

fn keychain_service() -> &'static str {
    #[cfg(debug_assertions)]
    {
        "org.destroy.dictation.community.dev.speech"
    }
    #[cfg(not(debug_assertions))]
    {
        "org.destroy.dictation.community.speech"
    }
}

#[cfg(target_os = "macos")]
fn read_secret(account: &str) -> Option<Vec<u8>> {
    security_framework::passwords::get_generic_password(keychain_service(), account).ok()
}

#[cfg(not(target_os = "macos"))]
fn read_secret(_account: &str) -> Option<Vec<u8>> {
    None
}

#[cfg(target_os = "macos")]
fn write_secret(account: &str, value: &[u8]) -> Result<(), String> {
    security_framework::passwords::set_generic_password(keychain_service(), account, value)
        .map_err(|_| "Cannot save speech key in Keychain".into())
}

#[cfg(not(target_os = "macos"))]
fn write_secret(_account: &str, _value: &[u8]) -> Result<(), String> {
    Err("Native provider keys require the macOS Keychain".into())
}

#[cfg(target_os = "macos")]
fn delete_secret(account: &str) -> Result<(), String> {
    match security_framework::passwords::delete_generic_password(keychain_service(), account) {
        Ok(()) => Ok(()),
        Err(error) if error.code() == -25300 => Ok(()),
        Err(_) => Err("Cannot remove speech key from Keychain".into()),
    }
}

#[cfg(not(target_os = "macos"))]
fn delete_secret(_account: &str) -> Result<(), String> {
    Ok(())
}

fn secret_present(account: &str) -> bool {
    read_secret(account).is_some_and(|value| !value.is_empty())
}

fn secret_string(account: &str) -> Result<String, String> {
    let value = read_secret(account).ok_or("Speech key is not available in Keychain")?;
    String::from_utf8(value).map_err(|_| "Speech key in Keychain is invalid".into())
}

pub(crate) fn clear_local_speech_credentials() -> Result<(), String> {
    delete_secret("openai-api-key")?;
    delete_secret("gemini-api-key")?;
    clear_saved_native_profile()
}

fn parse_provider(value: &str) -> Result<SpeechProvider, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "local" => Ok(SpeechProvider::Local),
        "openai" => Ok(SpeechProvider::Openai),
        "gemini" => Ok(SpeechProvider::Gemini),
        _ => Err("Choose local, openai, or gemini speech".into()),
    }
}

fn provider_name(provider: SpeechProvider) -> &'static str {
    match provider {
        SpeechProvider::Local => "local",
        SpeechProvider::Openai => "openai",
        SpeechProvider::Gemini => "gemini",
    }
}

fn default_local_data() -> serde_json::Value {
    serde_json::json!({
        "preferences": {
            "self_correction": true,
            "remove_fillers": true,
            "app_formatting": true,
            "selected_text_editing": true,
            "language": "auto",
            "style_note": ""
        },
        "snippets": [], "terms": [], "variables": {}, "links": [], "favorites": [], "days": {}
    })
}

fn load_local_data() -> Result<serde_json::Value, String> {
    let path = local_data_path();
    if !path.exists() {
        return Ok(default_local_data());
    }
    let data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).map_err(|_| "Cannot read local data")?)
            .map_err(|_| "Local data is invalid")?;
    if !data.is_object() {
        return Err("Local data is invalid".into());
    }
    Ok(data)
}

fn save_local_data(data: &serde_json::Value) -> Result<(), String> {
    let path = local_data_path();
    let parent = path.parent().ok_or("Cannot find local data folder")?;
    std::fs::create_dir_all(parent).map_err(|_| "Cannot create local data folder")?;
    let temp = parent.join(format!(".public-local-data-{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec(data).map_err(|_| "Cannot encode local data")?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let write_result = (|| -> Result<(), String> {
        let mut file = options
            .open(&temp)
            .map_err(|_| "Cannot create local data")?;
        file.write_all(&bytes)
            .map_err(|_| "Cannot write local data")?;
        file.sync_all().map_err(|_| "Cannot sync local data")?;
        std::fs::rename(&temp, &path).map_err(|_| "Cannot commit local data".to_string())
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    write_result
}

fn saved_provider() -> Option<SpeechProvider> {
    serde_json::from_slice::<NativeProfile>(&std::fs::read(native_profile_path()).ok()?)
        .ok()
        .map(|profile| profile.provider)
}

pub(crate) fn has_saved_native_profile() -> bool {
    saved_provider().is_some()
}

pub(crate) fn clear_saved_native_profile() -> Result<(), String> {
    match std::fs::remove_file(native_profile_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Cannot clear native speech profile".into()),
    }
}

fn save_native_profile(provider: SpeechProvider) -> Result<(), String> {
    let path = native_profile_path();
    let parent = path.parent().ok_or("Cannot find native profile folder")?;
    std::fs::create_dir_all(parent).map_err(|_| "Cannot create native profile folder")?;
    let bytes = serde_json::to_vec(&NativeProfile { provider })
        .map_err(|_| "Cannot encode native speech profile")?;
    let temp = parent.join(format!(".native-profile-{}.tmp", uuid::Uuid::new_v4()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<(), String> {
        let mut file = options
            .open(&temp)
            .map_err(|_| "Cannot create native speech profile")?;
        file.write_all(&bytes)
            .map_err(|_| "Cannot write native speech profile")?;
        file.sync_all()
            .map_err(|_| "Cannot sync native speech profile")?;
        std::fs::rename(&temp, &path).map_err(|_| "Cannot commit native speech profile".to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn model_ready() -> bool {
    crate::local_models::selected_model_ready()
}

fn setup_status(provider: Option<SpeechProvider>, current_key: Option<&str>) -> PublicSetupStatus {
    let local_model_ready = model_ready();
    let local_models = crate::local_models::status();
    let openai_key_present = secret_present("openai-api-key");
    let gemini_key_present = secret_present("gemini-api-key");
    let ready = match provider {
        Some(SpeechProvider::Local) => local_model_ready,
        Some(SpeechProvider::Openai) => {
            current_key.is_some_and(|key| !key.is_empty()) || openai_key_present
        }
        Some(SpeechProvider::Gemini) => {
            current_key.is_some_and(|key| !key.is_empty()) || gemini_key_present
        }
        None => false,
    };
    // Download failures belong to local setup only. Keeping them on a shared
    // status object made a failed optional local download look like a failed
    // cloud provider during startup.
    let error = (provider == Some(SpeechProvider::Local))
        .then_some(local_models.error.clone())
        .flatten();
    let local_model_id = local_models.selected_model_id;
    PublicSetupStatus {
        provider: provider.map(provider_name).map(str::to_owned),
        ready,
        model_ready: match provider {
            Some(SpeechProvider::Local) => local_model_ready,
            Some(SpeechProvider::Openai | SpeechProvider::Gemini) => true,
            None => false,
        },
        model_downloading: local_models.downloading_model_id.is_some(),
        openai_key_present,
        gemini_key_present,
        user_id: provider.map(|_| LOCAL_USER_ID.to_owned()),
        backend_url: provider
            .map(|_| PUBLIC_BACKEND_URL.to_owned())
            .unwrap_or_default(),
        local_model_id: Some(local_model_id),
        error,
    }
}

pub(crate) fn status_for_session(
    snapshot: &crate::auth::CallStreamSessionSnapshot,
) -> serde_json::Value {
    let status = match snapshot.mode {
        crate::auth::SessionMode::Backend => PublicSetupStatus {
            provider: Some("backend".into()),
            ready: true,
            model_ready: true,
            // A local model download is unrelated to a connected backend.
            // Reporting it here can keep onboarding in a local-only pending
            // state even though backend dictation is ready.
            model_downloading: false,
            openai_key_present: secret_present("openai-api-key"),
            gemini_key_present: secret_present("gemini-api-key"),
            local_model_id: None,
            user_id: Some(snapshot.user_id.clone()),
            backend_url: snapshot.backend_url.clone(),
            error: None,
        },
        crate::auth::SessionMode::Native(provider) => {
            setup_status(Some(provider), Some(&snapshot.access_token))
        }
    };
    serde_json::to_value(status)
        .unwrap_or_else(|_| serde_json::json!({"provider":null,"ready":false}))
}

#[tauri::command]
pub fn public_setup_status() -> serde_json::Value {
    if let Some(snapshot) = crate::auth::cached_call_stream_session_snapshot() {
        return status_for_session(&snapshot);
    }
    // The public UI probes status during startup. Restore a saved native
    // profile here as well so a status-first flow cannot leave the router
    // disconnected from the selected provider.
    if let Some(provider) = saved_provider() {
        let key = native_profile_key(provider).unwrap_or_default();
        if let Ok(generation) = crate::auth::current_generation() {
            if crate::auth::install_native_session(generation, provider, key).is_ok() {
                if let Some(snapshot) = crate::auth::cached_call_stream_session_snapshot() {
                    return status_for_session(&snapshot);
                }
            }
        }
    }
    serde_json::to_value(setup_status(saved_provider(), None))
        .unwrap_or_else(|_| serde_json::json!({"provider":null,"ready":false}))
}

fn native_profile_key(provider: SpeechProvider) -> Result<String, String> {
    match provider {
        SpeechProvider::Local => Ok(String::new()),
        SpeechProvider::Openai => secret_string("openai-api-key"),
        SpeechProvider::Gemini => secret_string("gemini-api-key"),
    }
}

pub(crate) async fn restore_native_connection() -> Result<serde_json::Value, String> {
    let provider = saved_provider().ok_or("Native speech profile is invalid")?;
    let key = native_profile_key(provider).unwrap_or_default();
    let generation = crate::auth::current_generation()?;
    crate::auth::install_native_session(generation, provider, key)?;
    Ok(public_setup_status())
}

#[tauri::command]
pub async fn public_configure_speech(
    app: tauri::AppHandle,
    provider: String,
    api_key: Option<String>,
) -> Result<serde_json::Value, String> {
    let provider = parse_provider(&provider)?;
    let key = match provider {
        SpeechProvider::Local => {
            if api_key
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                return Err("Local speech does not use an API key".into());
            }
            String::new()
        }
        SpeechProvider::Openai | SpeechProvider::Gemini => {
            let value = match api_key {
                Some(value) => value,
                None => native_profile_key(provider)
                    .map_err(|_| "Speech key is not available in Keychain")?,
            };
            let value = value.trim().to_owned();
            if !(10..=4096).contains(&value.len()) {
                return Err("Provider API key length is invalid".into());
            }
            value
        }
    };
    let generation = crate::auth::boundary(&app)?;
    match provider {
        SpeechProvider::Local => {}
        SpeechProvider::Openai => write_secret("openai-api-key", key.as_bytes())?,
        SpeechProvider::Gemini => write_secret("gemini-api-key", key.as_bytes())?,
    }
    save_native_profile(provider)?;
    crate::auth::install_native_session(generation, provider, key)?;
    Ok(public_setup_status())
}

#[tauri::command]
pub async fn public_download_model(model_id: Option<String>) -> Result<serde_json::Value, String> {
    crate::local_models::start_download(model_id).await?;
    Ok(public_setup_status())
}

#[tauri::command]
pub fn public_local_models() -> serde_json::Value {
    serde_json::to_value(crate::local_models::status())
        .unwrap_or_else(|_| serde_json::json!({"selectedModelId":"whisper-base-en","models":[]}))
}

#[tauri::command]
pub fn public_select_local_model(model_id: String) -> Result<serde_json::Value, String> {
    crate::local_models::select(&model_id)?;
    Ok(public_local_models())
}

#[tauri::command]
pub fn public_remove_local_model(model_id: String) -> Result<serde_json::Value, String> {
    Ok(
        serde_json::to_value(crate::local_models::remove(&model_id)?).unwrap_or_else(
            |_| serde_json::json!({"selectedModelId":"whisper-base-en","models":[]}),
        ),
    )
}

pub(crate) async fn native_backend_api(
    snapshot: &crate::auth::CallStreamSessionSnapshot,
    method: &str,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let provider = match snapshot.mode {
        crate::auth::SessionMode::Native(provider) => provider,
        crate::auth::SessionMode::Backend => {
            return Err("Native route called for backend session".into())
        }
    };
    if method == "POST" && path == "/v1/dictation/transcribe" {
        return native_transcribe(provider, &snapshot.access_token, &body).await;
    }
    native_route(method, path, body)
}

fn native_route(
    method: &str,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if !path.starts_with("/v1/") || path.contains("..") || path.contains('?') || path.contains('#')
    {
        return Err("Invalid native route".into());
    }
    if serde_json::to_vec(&body)
        .map_err(|_| "Invalid request")?
        .len()
        > 512 * 1024
    {
        return Err("Request exceeds size limit".into());
    }
    let _guard = storage_lock().lock().map_err(|_| "Local storage busy")?;
    if method == "DELETE" && path == "/v1/account/data" {
        match std::fs::remove_file(local_data_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Cannot delete local data".into()),
        }
        return Ok(serde_json::json!({"deleted":true}));
    }
    let mut data = load_local_data()?;
    let result = match (method, path) {
        ("GET", "/v1/account") | ("GET", "/v1/me") => serde_json::json!({"user_id":LOCAL_USER_ID}),
        ("GET", "/v1/account/export") => data.clone(),
        ("GET", "/v1/dictation/preferences") => data["preferences"].clone(),
        ("GET", "/v1/dictation/snippets") => {
            serde_json::json!({"snippets":data["snippets"].as_array().cloned().unwrap_or_default(),"max_snippets":50})
        }
        ("GET", "/v1/dictation/vocabulary") => {
            serde_json::json!({"terms":data["terms"].as_array().cloned().unwrap_or_default(),"max_terms":100})
        }
        ("GET", "/v1/dictation/usage") => local_usage(&data),
        ("GET", "/v1/dictation/media/status") => serde_json::json!({"enabled":false}),
        ("GET", "/v1/dictation/media/favorites") => {
            serde_json::json!({"favorites":data["favorites"].as_array().cloned().unwrap_or_default()})
        }
        ("GET", "/v1/variables") => data["variables"].clone(),
        ("GET", "/v1/links") => data["links"].clone(),
        ("PUT", "/v1/dictation/preferences") => {
            validate_preferences(&body)?;
            data["preferences"] = body.clone();
            body
        }
        ("PUT", "/v1/dictation/vocabulary") => {
            validate_vocabulary(&body)?;
            data["terms"] = body["terms"].clone();
            serde_json::json!({"terms":data["terms"],"max_terms":100})
        }
        ("PUT", "/v1/variables") => {
            validate_variables(&body)?;
            data["variables"] = body.clone();
            body
        }
        ("PUT", "/v1/links") => {
            validate_links(&body)?;
            data["links"] = body.clone();
            body
        }
        ("PUT", "/v1/dictation/snippets") => {
            let saved = save_snippet(&mut data, &body)?;
            saved
        }
        ("POST", "/v1/dictation/usage") => {
            update_usage(&mut data, &body)?;
            local_usage(&data)
        }
        ("POST", "/v1/links/search") => search_links(&data, &body),
        ("POST", "/v1/dictation/transform") => local_transform(&mut data, &body)?,
        ("POST", "/v1/dictation/style/learn") => {
            return Err("Writing-style learning is unavailable in native mode".into())
        }
        ("POST", "/v1/dictation/media/search") => {
            return Err("Media search is unavailable in native mode".into())
        }
        _ if method == "DELETE" && path.strip_prefix("/v1/dictation/snippets/").is_some() => {
            delete_row(
                &mut data,
                "snippets",
                path.strip_prefix("/v1/dictation/snippets/").unwrap_or(""),
            );
            serde_json::json!({"deleted":true})
        }
        _ if method == "DELETE"
            && path
                .strip_prefix("/v1/dictation/media/favorites/")
                .is_some() =>
        {
            delete_row(
                &mut data,
                "favorites",
                path.strip_prefix("/v1/dictation/media/favorites/")
                    .unwrap_or(""),
            );
            serde_json::json!({"deleted":true})
        }
        _ => return Err("Unknown native endpoint".into()),
    };
    if method == "PUT" || method == "POST" || method == "DELETE" {
        save_local_data(&data)?;
    }
    Ok(result)
}

fn validate_preferences(value: &serde_json::Value) -> Result<(), String> {
    let object = value.as_object().ok_or("Invalid preferences")?;
    let language = object
        .get("language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("auto");
    if !matches!(
        language,
        "auto" | "en" | "fr" | "de" | "es" | "it" | "pt" | "nl" | "ja" | "ko" | "zh"
    ) || object
        .get("style_note")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|v| v.chars().count() > 400)
    {
        return Err("Unsupported language or style note exceeds 400 characters".into());
    }
    Ok(())
}

fn normalized_command(value: &str) -> String {
    value
        .trim()
        .trim_matches(|character: char| character.is_ascii_punctuation())
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matching_snippet<'a>(
    spoken: &str,
    rows: &'a [serde_json::Value],
) -> Option<&'a serde_json::Value> {
    let spoken = normalized_command(spoken);
    rows.iter().find(|row| {
        let trigger = normalized_command(row["trigger"].as_str().unwrap_or(""));
        spoken == trigger
            || ["insert", "add", "use", "paste", "snippet"]
                .iter()
                .any(|verb| spoken == format!("{verb} {trigger}"))
    })
}

fn local_transform(
    data: &mut serde_json::Value,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request: DictationTransformRequest =
        serde_json::from_value(body.clone()).map_err(|_| "Invalid transformation")?;
    if request.text.trim().is_empty()
        || request.text.chars().count() > 12_000
        || request
            .selected_text
            .as_ref()
            .is_some_and(|value| value.chars().count() > 16_000)
        || request
            .previous_text
            .as_ref()
            .is_some_and(|value| value.chars().count() > 16_000)
    {
        return Err("Text length exceeds transformation limits".into());
    }
    let prefs: DictationPreferences =
        serde_json::from_value(data["preferences"].clone()).unwrap_or_default();
    match request.operation {
        DictationTransformOperation::Dictate => {
            if let Some(snippet) = matching_snippet(&request.text, data["snippets"].as_array().unwrap_or(&vec![])) {
                let mut result = snippet["body"].as_str().unwrap_or("").to_owned();
                if let Some(variables) = data["variables"].as_object() { for (key, value) in variables { result = result.replace(&format!("{{{{{key}}}}}"), value.as_str().unwrap_or("")); } }
                if result.contains("{{") { return Err("Define the missing snippet variable in Settings".into()); }
                return Ok(serde_json::json!({"text":result,"applied":["snippet"]}));
            }
            let mut fallback = request.text.trim().to_owned(); let mut applied = vec!["local_fallback"];
            if prefs.self_correction { fallback = apply_simple_self_correction(&fallback); applied.push("self_correction"); }
            if prefs.remove_fillers { fallback = remove_fillers_and_repetition(&fallback); applied.push("filler_cleanup"); }
            Ok(serde_json::json!({"text":fallback,"applied":applied}))
        }
        DictationTransformOperation::Rewrite | DictationTransformOperation::EditSelected | DictationTransformOperation::CorrectPrevious => Err("Explicit writing transformations require a configured backend; local mode never sends them to the cloud".into()),
    }
}

fn apply_simple_self_correction(input: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let markers = [
        "—actually ",
        " - actually ",
        ", actually ",
        "; actually ",
        "—sorry ",
        " - sorry ",
        ", sorry, ",
        "—i mean ",
        " - i mean ",
        ", i mean, ",
        " actually ",
        " i mean ",
    ];
    let Some((index, marker)) = markers
        .iter()
        .filter_map(|marker| lower.rfind(marker).map(|index| (index, *marker)))
        .max_by_key(|(index, _)| *index)
    else {
        return input.trim().to_owned();
    };
    let correction = input[index + marker.len()..]
        .trim()
        .trim_matches(|character| matches!(character, ',' | '.' | ';' | '—' | '-'))
        .trim();
    if correction.is_empty() {
        return input.trim().to_owned();
    }
    let mut prefix: Vec<&str> = input[..index]
        .trim()
        .trim_end_matches(|character| matches!(character, ',' | '.' | ';' | '—' | '-'))
        .split_whitespace()
        .collect();
    let correction_words: Vec<&str> = correction.split_whitespace().collect();
    let Some(previous) = prefix.last().copied() else {
        return correction.to_owned();
    };
    if correction_words.len() != 1 || !same_correction_class(previous, correction_words[0]) {
        return input.trim().to_owned();
    }
    prefix.pop();
    if prefix.is_empty() {
        correction.to_owned()
    } else {
        format!("{} {correction}", prefix.join(" "))
    }
}

fn same_correction_class(previous: &str, correction: &str) -> bool {
    fn normalized(value: &str) -> String {
        value
            .trim_matches(|character: char| character.is_ascii_punctuation())
            .to_ascii_lowercase()
    }
    let previous = normalized(previous);
    let correction = normalized(correction);
    let classes: [&[&str]; 4] = [
        &[
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ],
        &[
            "january",
            "february",
            "march",
            "april",
            "may",
            "june",
            "july",
            "august",
            "september",
            "october",
            "november",
            "december",
        ],
        &[
            "red", "orange", "yellow", "green", "blue", "purple", "pink", "black", "white", "gray",
            "grey", "brown",
        ],
        &["yes", "no"],
    ];
    classes
        .iter()
        .any(|values| values.contains(&previous.as_str()) && values.contains(&correction.as_str()))
        || (previous.chars().any(|character| character.is_ascii_digit())
            && correction
                .chars()
                .any(|character| character.is_ascii_digit()))
}

fn remove_fillers_and_repetition(input: &str) -> String {
    let filler =
        Regex::new(r"(?i)(^|[\s,])(?:um+|uh+|erm+)($|[\s,])").expect("static filler regex");
    let you_know = Regex::new(r"(?i),\s*you know\s*,").expect("static filler regex");
    let mut cleaned = format!(" {} ", you_know.replace_all(input, ", "));
    for _ in 0..2 {
        cleaned = filler.replace_all(&cleaned, "$1$2").into_owned();
    }
    let mut words = Vec::new();
    let mut last = String::new();
    for word in cleaned.split_whitespace() {
        let normalized = word
            .trim_matches(|character: char| character.is_ascii_punctuation())
            .to_ascii_lowercase();
        let safe = matches!(
            normalized.as_str(),
            "i" | "we" | "a" | "an" | "the" | "to" | "and" | "but" | "so"
        );
        if safe && normalized == last {
            continue;
        }
        last = normalized;
        words.push(word);
    }
    words.join(" ").trim().to_owned()
}

fn validate_vocabulary(value: &serde_json::Value) -> Result<(), String> {
    let terms = value["terms"].as_array().ok_or("Invalid vocabulary")?;
    if terms.len() > 100
        || terms.iter().any(|term| {
            term.as_str()
                .is_none_or(|v| v.trim().is_empty() || v.chars().count() > 80)
        })
    {
        return Err("Vocabulary allows 100 terms of 1–80 characters".into());
    }
    Ok(())
}

fn validate_variables(value: &serde_json::Value) -> Result<(), String> {
    let object = value.as_object().ok_or("Variables must be an object")?;
    if object.len() > 50
        || object.iter().any(|(key, value)| {
            key.is_empty()
                || key.len() > 40
                || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || value.as_str().is_none_or(|v| v.len() > 4000)
        })
    {
        return Err("Invalid snippet variables".into());
    }
    Ok(())
}

fn validate_links(value: &serde_json::Value) -> Result<(), String> {
    let links = value
        .as_array()
        .filter(|links| links.len() <= 100)
        .ok_or("Maximum 100 links")?;
    if links.iter().any(|link| {
        let name = link["name"].as_str().unwrap_or("");
        let url = link["url"].as_str().unwrap_or("");
        name.trim().is_empty()
            || name.chars().count() > 200
            || !reqwest::Url::parse(url).is_ok_and(|url| {
                url.scheme() == "https"
                    && url.host_str().is_some()
                    && url.username().is_empty()
                    && url.password().is_none()
            })
            || link["keywords"]
                .as_str()
                .is_some_and(|value| value.chars().count() > 1000)
    }) {
        return Err("Every link needs a name and HTTPS URL".into());
    }
    Ok(())
}

fn save_snippet(
    data: &mut serde_json::Value,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let object = body.as_object().ok_or("Invalid snippet")?;
    let title = object
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let trigger = object
        .get("trigger")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let content = object
        .get("body")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if trigger.trim().is_empty()
        || trigger.chars().count() > 160
        || title.chars().count() > 120
        || content.is_empty()
        || content.chars().count() > 4000
    {
        return Err("Invalid snippet size".into());
    }
    let id = object
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let saved = serde_json::json!({"id":id,"title":title,"trigger":trigger,"body":content});
    let rows = data["snippets"]
        .as_array_mut()
        .ok_or("Local snippets are invalid")?;
    rows.retain(|row| row["id"] != id);
    if rows.len() >= 50 {
        return Err("Maximum 50 snippets".into());
    }
    rows.push(saved.clone());
    Ok(saved)
}

fn delete_row(data: &mut serde_json::Value, key: &str, id: &str) {
    if let Some(rows) = data[key].as_array_mut() {
        rows.retain(|row| row["id"].as_str() != Some(id));
    }
}

fn update_usage(data: &mut serde_json::Value, body: &serde_json::Value) -> Result<(), String> {
    let day = body["day"].as_str().ok_or("Local day is required")?;
    chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").map_err(|_| "Invalid date")?;
    let words = body["words"]
        .as_u64()
        .filter(|value| *value <= 200_000)
        .ok_or("Invalid word count")?;
    let speaking_ms = body["speaking_ms"]
        .as_u64()
        .filter(|value| *value <= 86_400_000)
        .ok_or("Invalid speaking time")?;
    let days = data["days"]
        .as_object_mut()
        .ok_or("Local usage is invalid")?;
    let previous = days
        .get(day)
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    days.insert(day.to_owned(), serde_json::json!({"words":words.max(previous["words"].as_u64().unwrap_or(0)),"speaking_ms":speaking_ms.max(previous["speaking_ms"].as_u64().unwrap_or(0))}));
    Ok(())
}

fn local_usage(data: &serde_json::Value) -> serde_json::Value {
    let mut words = 0_u64;
    let mut speaking_ms = 0_u64;
    let mut days = Vec::new();
    if let Some(entries) = data["days"].as_object() {
        for (day, value) in entries {
            let day_words = value["words"].as_u64().unwrap_or(0);
            let day_ms = value["speaking_ms"].as_u64().unwrap_or(0);
            words += day_words;
            speaking_ms += day_ms;
            days.push(serde_json::json!({"day":day,"words":day_words,"speaking_ms":day_ms}));
        }
    }
    serde_json::json!({"words":words,"speaking_ms":speaking_ms,"days":days})
}

fn search_links(data: &serde_json::Value, body: &serde_json::Value) -> serde_json::Value {
    let query = body["query"].as_str().unwrap_or("").to_ascii_lowercase();
    let terms: Vec<&str> = query.split_whitespace().collect();
    let mut scored: Vec<(usize, serde_json::Value)> = data["links"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|link| {
            let text = format!(
                "{} {}",
                link["name"].as_str().unwrap_or(""),
                link["keywords"].as_str().unwrap_or("")
            )
            .to_ascii_lowercase();
            let score = terms.iter().filter(|term| text.contains(**term)).count();
            (score > 0).then(|| (score, link.clone()))
        })
        .collect();
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    serde_json::json!({"results":scored.into_iter().take(3).map(|(_, link)| link).collect::<Vec<_>>()})
}

fn audio_mime(mime: &str) -> Result<(&str, &'static str), String> {
    match mime.split(';').next().unwrap_or("").trim() {
        "audio/wav" => Ok(("audio/wav", "wav")),
        "audio/mp4" => Ok(("audio/mp4", "m4a")),
        "audio/webm" => Ok(("audio/webm", "webm")),
        _ => Err("Unsupported recording type".into()),
    }
}

async fn native_transcribe(
    provider: SpeechProvider,
    key: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let encoded = body["audio_base64"].as_str().ok_or("Audio is required")?;
    if encoded.len() > 8 * 1024 * 1024 + 4 {
        return Err("Audio exceeds size limit".into());
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "Invalid audio encoding")?;
    if bytes.is_empty() || bytes.len() > MAX_AUDIO_BYTES {
        return Err("Audio is empty or too large".into());
    }
    let mime = body["mime_type"].as_str().unwrap_or("audio/wav");
    let (mime, extension) = audio_mime(mime)?;
    match provider {
        SpeechProvider::Local => {
            // Snapshot the selected model before decoding/spawning. The lease
            // keeps removal from racing this transcription, and the path is
            // never re-resolved if the user changes selection meanwhile.
            let selected = crate::local_models::snapshot_selected_model()?;
            let samples = decode_audio(&bytes, mime)?;
            let text = tokio::task::spawn_blocking(move || match selected.engine {
                crate::local_models::ModelEngine::Whisper => {
                    transcribe_whisper(&selected.id, &selected.path, &samples)
                }
                crate::local_models::ModelEngine::Parakeet => {
                    crate::parakeet::transcribe(&selected.path, &samples)
                }
            })
            .await
            .map_err(|_| "Local transcription failed")??;
            Ok(serde_json::json!({"text":text,"path":"local"}))
        }
        SpeechProvider::Openai => transcribe_openai(key, bytes, mime, extension).await,
        SpeechProvider::Gemini => transcribe_gemini(key, encoded, mime).await,
    }
}

async fn transcribe_openai(
    key: &str,
    bytes: Vec<u8>,
    mime: &str,
    extension: &str,
) -> Result<serde_json::Value, String> {
    if key.is_empty() {
        return Err("OpenAI speech key is not configured".into());
    }
    let form = Form::new().text("model", "gpt-4o-mini-transcribe").part(
        "file",
        Part::bytes(bytes)
            .file_name(format!("dictation.{extension}"))
            .mime_str(mime)
            .map_err(|_| "Unsupported recording type")?,
    );
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|_| "Cannot create transcription client")?;
    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(key)
        .multipart(form)
        .send()
        .await
        .map_err(|_| "OpenAI transcription unavailable")?;
    let value = bounded_json(response).await?;
    let text = value["text"]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .ok_or("OpenAI returned no transcription")?;
    Ok(serde_json::json!({"text":text,"path":"cloud"}))
}

async fn transcribe_gemini(
    key: &str,
    encoded: &str,
    mime: &str,
) -> Result<serde_json::Value, String> {
    if key.is_empty() {
        return Err("Gemini speech key is not configured".into());
    }
    let body = serde_json::json!({"contents":[{"parts":[{"text":"Transcribe the attached audio accurately. Return only the spoken words, with no summary, explanation, timestamps, or invented text."},{"inline_data":{"mime_type":mime,"data":encoded}}]}]});
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|_| "Cannot create transcription client")?;
    let response = client.post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent").header("x-goog-api-key", key).json(&body).send().await.map_err(|_| "Gemini transcription unavailable")?;
    let value = bounded_json(response).await?;
    let text = value["candidates"]
        .as_array()
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate["content"]["parts"].as_array())
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part["text"].as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|text| !text.trim().is_empty())
        .ok_or("Gemini returned no transcription")?;
    Ok(serde_json::json!({"text":text,"path":"cloud"}))
}

async fn bounded_json(response: reqwest::Response) -> Result<serde_json::Value, String> {
    let status = response.status();
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Provider response interrupted")?;
        if bytes.len() + chunk.len() > 256 * 1024 {
            return Err("Provider response exceeds size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        return Err("Speech provider rejected the request".into());
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| "Speech provider returned an invalid response".into())
}

fn transcribe_whisper(
    model_id: &str,
    model: &std::path::Path,
    samples: &[f32],
) -> Result<String, String> {
    let mut context_params = WhisperContextParameters::default();
    context_params.use_gpu(true);
    let context = WhisperContext::new_with_params(model, context_params)
        .map_err(|_| "Cannot load local speech model")?;
    let mut state = context
        .create_state()
        .map_err(|_| "Cannot create local speech state")?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    if model_id.ends_with("-en") {
        params.set_language(Some("en"));
        params.set_detect_language(false);
    } else {
        params.set_language(None);
        params.set_detect_language(true);
    }
    params.set_no_timestamps(true);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_n_threads(4);
    state
        .full(params, &samples)
        .map_err(|_| "Local transcription failed")?;
    let mut text = String::new();
    for segment in state.as_iter() {
        let part = segment
            .to_str_lossy()
            .map_err(|_| "Local transcription returned invalid text")?;
        text.push_str(part.trim());
        text.push(' ');
    }
    let text = text.trim().to_owned();
    if text.is_empty() {
        return Err("Local speech returned no transcription".into());
    }
    Ok(text)
}

/// Browser containers can leave channel/sample-rate fields unset on the
/// probed track and only expose them on the first decoded packet. Keep the
/// container metadata when it is complete, but resolve missing fields from
/// the decoder's concrete signal spec before touching the samples.
fn resolve_decoded_layout(
    track_sample_rate: Option<u32>,
    track_channels: Option<usize>,
    decoded_sample_rate: u32,
    decoded_channels: usize,
) -> Result<(usize, usize), String> {
    let sample_rate = track_sample_rate
        .or((decoded_sample_rate > 0).then_some(decoded_sample_rate))
        .ok_or("Recording sample rate is missing")? as usize;
    let channels = track_channels
        .or((decoded_channels > 0).then_some(decoded_channels))
        .ok_or("Recording channels are missing")?;
    if sample_rate == 0 || channels == 0 {
        return Err("Recording audio metadata is invalid".into());
    }
    Ok((sample_rate, channels))
}

fn exceeds_duration_limit(sample_count: usize, sample_rate: usize) -> bool {
    sample_count > sample_rate.saturating_mul(180)
}

fn decode_audio(bytes: &[u8], mime: &str) -> Result<Vec<f32>, String> {
    let mut hint = Hint::new();
    hint.with_extension(match mime {
        "audio/wav" => "wav",
        "audio/mp4" => "m4a",
        "audio/webm" => "webm",
        _ => "",
    });
    let source =
        MediaSourceStream::new(Box::new(Cursor::new(bytes.to_owned())), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            source,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| "This recording format cannot be decoded locally")?;
    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or("Recording has no audio track")?;
    let track_id = track.id;
    let track_sample_rate = track.codec_params.sample_rate;
    let track_channels = track.codec_params.channels.map(|channels| channels.count());
    let mut resolved_sample_rate = track_sample_rate;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|_| "This recording codec cannot be decoded locally")?;
    let mut mono = Vec::new();
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = decoder
            .decode(&packet)
            .map_err(|_| "This recording cannot be decoded locally")?;
        let (sample_rate, channels) = resolve_decoded_layout(
            track_sample_rate,
            track_channels,
            decoded.spec().rate,
            decoded.spec().channels.count(),
        )?;
        resolved_sample_rate.get_or_insert(sample_rate as u32);
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        for frame in buffer.samples().chunks(channels) {
            mono.push(frame.iter().copied().sum::<f32>() / channels as f32);
            if exceeds_duration_limit(mono.len(), sample_rate) {
                return Err("Recording exceeds the local 3-minute limit".into());
            }
        }
    }
    if mono.is_empty() {
        return Err("Recording contains no decodable audio".into());
    }
    let sample_rate = resolved_sample_rate
        .filter(|rate| *rate > 0)
        .ok_or("Recording sample rate is missing")? as usize;
    if sample_rate == 16_000 {
        return Ok(mono);
    }
    let output_len = mono.len().saturating_mul(16_000) / sample_rate;
    let mut output = Vec::with_capacity(output_len);
    for index in 0..output_len {
        let position = index as f64 * sample_rate as f64 / 16_000.0;
        let left = position.floor() as usize;
        let right = (left + 1).min(mono.len() - 1);
        let fraction = (position - left as f64) as f32;
        output.push(mono[left] * (1.0 - fraction) + mono[right] * fraction);
    }
    Ok(output)
}

#[tauri::command]
pub fn public_disconnect_speech(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    crate::auth::boundary(&app)?;
    delete_secret("openai-api-key")?;
    delete_secret("gemini-api-key")?;
    clear_saved_native_profile()?;
    Ok(public_setup_status())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Generated once with ffmpeg from a 440 Hz lavfi sine wave (0.25 s,
    // AAC-LC, 32 kbps, MP4 container). Keeping the tiny encoded fixture here
    // makes contributor and CI tests independent of an ffmpeg installation.
    const SYNTHETIC_AAC_MP4_B64: &str = "AAAAHGZ0eXBpc29tAAACAGlzb21pc28ybXA0MQAAAz5tb292AAAAbG12aGQAAAAAAAAAAAAAAAAAAKxEAAArEQABAAABAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAACaXRyYWsAAABcdGtoZAAAAAMAAAAAAAAAAAAAAAEAAAAAAAArEQAAAAAAAAAAAAAAAQEAAAAAAQAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAACRlZHRzAAAAHGVsc3QAAAAAAAAAAQAAKxEAAAQAAAEAAAAAAeFtZGlhAAAAIG1kaGQAAAAAAAAAAAAAAAAAAKxEAAAvEVXEAAAAAAAtaGRscgAAAAAAAAAAc291bgAAAAAAAAAAAAAAAFNvdW5kSGFuZGxlcgAAAAGMbWluZgAAABBzbWhkAAAAAAAAAAAAAAAkZGluZgAAABxkcmVmAAAAAAAAAAEAAAAMdXJsIAAAAAEAAAFQc3RibAAAAH5zdHNkAAAAAAAAAAEAAABubXA0YQAAAAAAAAABAAAAAAAAAAAAAQAQAAAAAKxEAAAAAAA2ZXNkcwAAAAADgICAJQABAASAgIAXQBUAAAAAAH0AAAB8jgWAgIAFEghW5QAGgICAAQIAAAAUYnRydAAAAAAAAH0AAAB8jgAAACBzdHRzAAAAAAAAAAIAAAALAAAEAAAAAAEAAAMRAAAAHHN0c2MAAAAAAAAAAQAAAAEAAAAMAAAAAQAAAERzdHN6AAAAAAAAAAAAAAAMAAAAlwAAAKIAAAA8AAAAPgAAAEEAAABCAAAAXQAAAF0AAABLAAAAVAAAAFoAAABYAAAAFHN0Y28AAAAAAAAAAQAAA2oAAAAac2dwZAEAAAByb2xsAAAAAgAAAAH//wAAABxzYmdwAAAAAHJvbGwAAAABAAAADAAAAAEAAABhdWR0YQAAAFltZXRhAAAAAAAAACFoZGxyAAAAAAAAAABtZGlyYXBwbAAAAAAAAAAAAAAAACxpbHN0AAAAJKl0b28AAAAcZGF0YQAAAAEAAAAATGF2ZjYzLjEuMTAxAAAACGZyZWUAAARJbWRhdNwATGF2YzYzLjEuMTAxAAJgpVSQ2nIy88ennjVecuWq6yRHlJIvzCQbdwPwXqvYu+4bLalpH+PS83ver8ngbUxVY2zWmvPrJSaaqmK5iqMmlLJSyWmlJoyUKUZKMlEmtDaG0MDGzYMDIkQMbNmzaJEilNmzcUUUUUUUUUUUUUUUSIooooookREREhEUURFFEiKKIoooouABOJTayV2UU6sp1ZLpz/Ptxps8av/61+/XGuNXr/+14/nzxrjV6//i9/588a61qw3+tjR9FBgLoEywhZ+p1m8pjbnAZzgMDO4TMDAzuEhIMDO4SEgwMDO7hISsGBgZ3cJCTkJsJd/hCrDdT5y5XpSt6UJvZm5QkqSzBpQkJJXgaUJCSV5wY2bCQkqDZuDAxsJCSrHLxDUUUUGooooo3G10UcABOvKLGteOe/2fv7fFumml6qXI45JJIiQO5/fOm7NmNn9yGT4MM/uBk+DDP7wj4+AM/uBw+AM/uBk+AOABCDKJ4p1rK+f/7P8f+v/td8XeqvfXv9b8ffbty6lUXmtihdCMCKig1FFBqBQug1o2EsxLNMzrm/NT4O35nAEIMooifUCHSMr1//fv/9fxc41xk87rx8V4+Nu4VrKTLCedRzqKedU88551Cec9WUqm+lx7qp+V4BzxeJimChVwAQoyigKNCIdWqvf/+1+f/X/yXxqXJ169fd8/jbs8Ml4q6mD5RRLfMaKKKKKKKKrigkRFSq9YlL+hTSmQvQ7ZgF+AAQYylONdGIdEQdCIVV6//pv/X/aXxc1nWb7+/e/ad3DwklMAurow5/BwenXr/LXKbXPPN9KLGXMPVUs9h2i1d/b/SWWAtaI3tyMNOt4NdjJutxJxxCIDJI3ZGtRwAQwyizohDoiDoRFox6//pd/+3/ter1JajU/T98jrby7yolD6fT6SJuL6fT6fSj6fT6UDXBz+cDbcCx7hkDEPDeFANsMTzcp0s6Q90WMUWgJWmO+c1LOBe+tNWsXAAQAynOiBXoRDqWZ3//d7/7/9bnE1eq3x/H78503Ty0Kky8CJiYaTsPQxsBh48eXMvDNIIvhnO0aW21ia+os1HWEpsH8EoCoMZDaOAQoykJY0OKtEIdEIdEIdIN//Sf7/UdS5IevjX7fzt17qpJVEAGBgbvGU7Ax+Jbj8AYleBiQNmOclX5vdCDmlApTTS+JMvCUL9cBiVoKzsD+bdBevATwyizpyDoSDoRCVv+v6f+3xqXd3LXJcS3Da0ntCDHpkXIY1V9W2O42Nfd0yYdl9V++6PTXjX3YSeFV+/bHdHjXjX3YSeFV++/uwQKr6rqJL5BUyOCu2rR14AVAyixpCDoiDoSDo1VXdzrV3cuXJLkkcUk6k1ckBrNo5/d8027LufzuNOaG4W+aCbdldfacazaG4eeaCbc+7kacazQtwuNNwt80G7K6+0404+6+040yHgA==";

    #[test]
    fn local_dictate_transform_is_deterministic_and_never_cloud_backed() {
        let mut data = default_local_data();
        let result = local_transform(&mut data, &serde_json::json!({"text":"Um I I think Tuesday actually Wednesday","app_kind":"generic","operation":"dictate"})).expect("local dictate");
        assert_eq!(result["text"], "I think Wednesday");
        assert!(result["applied"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item == "local_fallback")));
    }

    #[test]
    fn local_explicit_transform_fails_without_cloud_fallback() {
        let mut data = default_local_data();
        let result = local_transform(
            &mut data,
            &serde_json::json!({"text":"rewrite this","app_kind":"generic","operation":"rewrite"}),
        );
        assert!(result
            .unwrap_err()
            .contains("never sends them to the cloud"));
    }

    #[test]
    fn generated_wav_is_decodable_for_the_local_runtime() {
        let sample_rate = 16_000_u32;
        let pcm = vec![0_u8; sample_rate as usize * 2];
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36_u32 + pcm.len() as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        wav.extend_from_slice(&pcm);
        assert_eq!(
            decode_audio(&wav, "audio/wav")
                .expect("generated wav")
                .len(),
            sample_rate as usize
        );
    }

    #[test]
    fn browser_packet_layout_fills_missing_track_channels() {
        // WebKit's MediaRecorder output can probe with an incomplete track
        // CodecParameters record, while the first decoded packet has the
        // concrete 48 kHz mono signal spec used by browser microphones.
        assert_eq!(
            resolve_decoded_layout(None, None, 48_000, 1).expect("decoded browser layout"),
            (48_000, 1)
        );
        assert_eq!(
            resolve_decoded_layout(Some(16_000), Some(2), 48_000, 1)
                .expect("declared track layout"),
            (16_000, 2)
        );
    }

    #[test]
    fn duration_limit_uses_input_sample_rate_before_resampling() {
        assert!(!exceeds_duration_limit(48_000 * 180, 48_000));
        assert!(exceeds_duration_limit(48_000 * 180 + 1, 48_000));
        assert!(!exceeds_duration_limit(16_000 * 180, 16_000));
    }

    #[test]
    fn generated_aac_mp4_is_decodable_with_missing_layout_fallback() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(SYNTHETIC_AAC_MP4_B64)
            .expect("embedded AAC/MP4 fixture");
        let decoded = decode_audio(&bytes, "audio/mp4").expect("generated AAC/MP4 decode");
        assert!(!decoded.is_empty());
        assert_eq!(
            resolve_decoded_layout(None, None, 48_000, 1).unwrap(),
            (48_000, 1)
        );
    }

    #[test]
    #[ignore = "requires the verified 148 MiB model to be downloaded"]
    fn installed_model_loads_with_bundled_whisper_runtime() {
        assert!(model_ready());
    }

    #[test]
    fn local_link_validation_requires_https() {
        assert!(
            validate_links(&serde_json::json!([{"name":"x","url":"http://example.com"}])).is_err()
        );
        assert!(
            validate_links(&serde_json::json!([{ "name":"x", "url":"https://example.com" }]))
                .is_ok()
        );
    }

    #[test]
    fn local_status_never_reports_a_provider_key_value() {
        let value = serde_json::to_string(&setup_status(
            Some(SpeechProvider::Openai),
            Some("secret-value"),
        ))
        .unwrap();
        assert!(!value.contains("secret-value"));
    }

    #[test]
    fn setup_status_serializes_stable_readiness_fields() {
        let value = serde_json::to_value(setup_status(Some(SpeechProvider::Local), None)).unwrap();
        assert!(value.get("modelDownloading").is_some());
        assert!(value.get("modelReady").is_some());
        assert_eq!(value["provider"], "local");
    }
}
