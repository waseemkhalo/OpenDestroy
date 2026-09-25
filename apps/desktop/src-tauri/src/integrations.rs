//! Personal, opt-in provider integrations.
//!
//! Provider credentials never cross the IPC boundary into JavaScript. They are
//! read and written only through the macOS Keychain, and every asynchronous
//! request captures the integration generation before it leaves the process.
//! A key change invalidates the captured generation, so a response belonging to
//! an earlier account or credential set cannot update the UI.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Mutex, OnceLock};

const COMPOSIO_API: &str = "https://backend.composio.dev/api/v3.1";
const COMPOSIO_TOOLKIT: &str = "googledrive";
const COMPOSIO_FIND_FILE_TOOL: &str = "GOOGLEDRIVE_FIND_FILE";
const GOOGLE_DRIVE_METADATA_SCOPE: &str = "https://www.googleapis.com/auth/drive.metadata.readonly";
const GOOGLE_DRIVE_FILE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";
const GIPHY_API: &str = "https://api.giphy.com/v1";
const REQUEST_TIMEOUT_SECS: u64 = 20;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_QUERY_CHARS: usize = 50;
const MAX_RESULTS: usize = 3;

#[cfg(not(debug_assertions))]
const KEYCHAIN_SERVICE: &str = "org.destroy.dictation.community.integrations";
#[cfg(debug_assertions)]
const KEYCHAIN_SERVICE: &str = "org.destroy.dictation.community.dev.integrations";

const KEYCHAIN_COMPOSIO: &str = "composio-api-key";
const KEYCHAIN_GIPHY: &str = "giphy-api-key";
const KEYCHAIN_USER: &str = "integration-user-id";

#[derive(Debug, Clone, Default)]
struct IntegrationState {
    generation: u64,
    pending_connection: Option<PendingConnection>,
    drive_account: Option<DriveAccount>,
}

#[derive(Debug, Clone)]
struct PendingConnection {
    generation: u64,
    connected_account_id: String,
}

#[derive(Debug, Clone)]
struct DriveAccount {
    id: String,
    label: Option<String>,
}

#[derive(Debug, Clone)]
struct IntegrationSnapshot {
    generation: u64,
    user_id: String,
    composio_key: String,
}

static STATE: OnceLock<Mutex<IntegrationState>> = OnceLock::new();

fn state() -> &'static Mutex<IntegrationState> {
    STATE.get_or_init(|| Mutex::new(IntegrationState::default()))
}

pub(crate) fn clear_local_credentials() -> Result<(), String> {
    native_keychain::delete(KEYCHAIN_COMPOSIO)?;
    native_keychain::delete(KEYCHAIN_GIPHY)?;
    native_keychain::delete(KEYCHAIN_USER)?;
    let mut guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    guard.generation = guard.generation.wrapping_add(1);
    guard.pending_connection = None;
    guard.drive_account = None;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationStatus {
    #[serde(rename = "composioKeyPresent")]
    pub composio_key_present: bool,
    #[serde(rename = "driveConnected")]
    pub drive_connected: bool,
    #[serde(rename = "driveAccountLabel")]
    pub drive_account_label: Option<String>,
    #[serde(rename = "giphyKeyPresent")]
    pub giphy_key_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleDriveSearchResult {
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleDriveSearchResponse {
    pub results: Vec<GoogleDriveSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoogleDriveConnectResponse {
    pub pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiphySearchResult {
    pub id: String,
    pub title: String,
    pub alt_text: String,
    pub preview_url: String,
    pub content_url: String,
    pub source_url: String,
    pub width: u32,
    pub height: u32,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiphySearchResponse {
    pub query: String,
    pub kind: String,
    pub provider: String,
    pub attribution: String,
    pub results: Vec<GiphySearchResult>,
}

#[cfg(target_os = "macos")]
mod native_keychain {
    use super::KEYCHAIN_SERVICE;

    pub fn get(account: &str) -> Result<Option<String>, String> {
        match security_framework::passwords::get_generic_password(KEYCHAIN_SERVICE, account) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| "Stored integration credential is invalid".to_string()),
            Err(error) if error.code() == -25300 => Ok(None),
            Err(_) => Err("Cannot read integration credential from Keychain".to_string()),
        }
    }

    pub fn set(account: &str, value: &str) -> Result<(), String> {
        match security_framework::passwords::set_generic_password(
            KEYCHAIN_SERVICE,
            account,
            value.as_bytes(),
        ) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25299 => {
                delete(account)?;
                security_framework::passwords::set_generic_password(
                    KEYCHAIN_SERVICE,
                    account,
                    value.as_bytes(),
                )
                .map_err(|_| "Cannot save integration credential in Keychain".to_string())
            }
            Err(_) => Err("Cannot save integration credential in Keychain".to_string()),
        }
    }

    pub fn delete(account: &str) -> Result<(), String> {
        match security_framework::passwords::delete_generic_password(KEYCHAIN_SERVICE, account) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()),
            Err(_) => Err("Cannot clear integration credential from Keychain".to_string()),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod native_keychain {
    pub fn get(_account: &str) -> Result<Option<String>, String> {
        Err("Personal integrations require the native macOS Keychain".to_string())
    }

    pub fn set(_account: &str, _value: &str) -> Result<(), String> {
        Err("Personal integrations require the native macOS Keychain".to_string())
    }

    pub fn delete(_account: &str) -> Result<(), String> {
        Err("Personal integrations require the native macOS Keychain".to_string())
    }
}

fn validate_secret(value: &str, label: &str, max_len: usize) -> Result<String, String> {
    let value = value.trim();
    if !(8..=max_len).contains(&value.len()) || value.chars().any(char::is_whitespace) {
        return Err(format!("Enter a valid {label} key"));
    }
    Ok(value.to_owned())
}

fn integration_user_id() -> Result<String, String> {
    if let Some(value) = native_keychain::get(KEYCHAIN_USER)? {
        if validate_user_id(&value) {
            return Ok(value);
        }
        return Err("Stored integration identity is invalid".to_string());
    }
    let value = format!("destroy-community-{}", uuid::Uuid::new_v4());
    native_keychain::set(KEYCHAIN_USER, &value)?;
    Ok(value)
}

fn validate_user_id(value: &str) -> bool {
    (20..=100).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn capture_composio_snapshot() -> Result<IntegrationSnapshot, String> {
    let guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    let composio_key = native_keychain::get(KEYCHAIN_COMPOSIO)?
        .ok_or_else(|| "Save a Composio project key first".to_string())?;
    let user_id = integration_user_id()?;
    let snapshot = IntegrationSnapshot {
        generation: guard.generation,
        user_id,
        composio_key,
    };
    Ok(snapshot)
}

fn capture_giphy_snapshot() -> Result<(u64, String, String), String> {
    let guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    let key = native_keychain::get(KEYCHAIN_GIPHY)?
        .ok_or_else(|| "Save a GIPHY key first".to_string())?;
    let user_id = integration_user_id()?;
    let generation = guard.generation;
    Ok((generation, key, user_id))
}

fn assert_current(generation: u64) -> Result<(), String> {
    state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())
        .and_then(|guard| {
            if guard.generation == generation {
                Ok(())
            } else {
                Err("Integration changed; discarded response".to_string())
            }
        })
}

fn local_status() -> Result<IntegrationStatus, String> {
    let composio_key_present = native_keychain::get(KEYCHAIN_COMPOSIO)?.is_some();
    let giphy_key_present = native_keychain::get(KEYCHAIN_GIPHY)?.is_some();
    let drive_account = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?
        .drive_account
        .clone();
    Ok(IntegrationStatus {
        composio_key_present,
        drive_connected: drive_account.is_some(),
        drive_account_label: drive_account.and_then(|account| account.label),
        giphy_key_present,
    })
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|_| "Cannot create provider connection".to_string())
}

async fn bounded_body(response: reqwest::Response) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err("Provider response exceeds size limit".to_string());
    }
    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Provider response interrupted".to_string())?;
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err("Provider response exceeds size limit".to_string());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn composio_json(
    snapshot: &IntegrationSnapshot,
    method: reqwest::Method,
    path: &str,
    query: &[(&str, &str)],
    body: Option<Value>,
) -> Result<Value, String> {
    if !path.starts_with('/') || path.contains('?') || path.contains('#') || path.contains("..") {
        return Err("Invalid Composio path".to_string());
    }
    let request = client()?
        .request(method, format!("{COMPOSIO_API}{path}"))
        .header("x-api-key", &snapshot.composio_key)
        .query(query);
    let response = match body {
        Some(body) => request.json(&body).send().await,
        None => request.send().await,
    }
    .map_err(|_| "Composio request unavailable".to_string())?;
    let status = response.status();
    let bytes = bounded_body(response).await?;
    if !status.is_success() {
        return Err("Composio request was not accepted".to_string());
    }
    serde_json::from_slice(&bytes).map_err(|_| "Composio returned an invalid response".to_string())
}

async fn composio_delete(snapshot: &IntegrationSnapshot, id: &str) -> Result<(), String> {
    if !validate_resource_id(id) {
        return Err("Invalid connected account".to_string());
    }
    let response = client()?
        .delete(format!("{COMPOSIO_API}/connected_accounts/{id}"))
        .header("x-api-key", &snapshot.composio_key)
        .send()
        .await
        .map_err(|_| "Composio request unavailable".to_string())?;
    let status = response.status();
    let _ = bounded_body(response).await?;
    if status.is_success() {
        Ok(())
    } else {
        Err("Composio request was not accepted".to_string())
    }
}

fn validate_resource_id(value: &str) -> bool {
    (1..=200).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn toolkit_slug(item: &Value) -> Option<&str> {
    item["toolkit"]["slug"]
        .as_str()
        .or_else(|| item["toolkit_slug"].as_str())
}

fn account_label(item: &Value) -> Option<String> {
    [
        item["alias"].as_str(),
        item["data"]["email"].as_str(),
        item["state"]["val"]["email"].as_str(),
        item["state"]["val"]["user"]["email"].as_str(),
    ]
    .into_iter()
    .flatten()
    .find(|value| !value.trim().is_empty())
    .map(str::to_owned)
}

fn account_items(value: &Value) -> &[Value] {
    value["items"]
        .as_array()
        .or_else(|| value["data"].as_array())
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn drive_files(value: &Value) -> &[Value] {
    value["files"]
        .as_array()
        .map(|values| values.as_slice())
        .or_else(|| {
            value["data"]["files"]
                .as_array()
                .map(|values| values.as_slice())
        })
        .or_else(|| value["results"].as_array().map(|values| values.as_slice()))
        .or_else(|| value["items"].as_array().map(|values| values.as_slice()))
        .unwrap_or(&[])
}

fn active_drive_account(value: &Value, user_id: &str) -> Option<DriveAccount> {
    account_items(value).iter().find_map(|item| {
        let id = item["id"].as_str()?;
        if !validate_resource_id(id)
            || toolkit_slug(item) != Some(COMPOSIO_TOOLKIT)
            || item["user_id"].as_str() != Some(user_id)
            || item["status"].as_str()?.to_ascii_uppercase() != "ACTIVE"
        {
            return None;
        }
        Some(DriveAccount {
            id: id.to_owned(),
            label: account_label(item),
        })
    })
}

async fn list_drive_accounts(snapshot: &IntegrationSnapshot) -> Result<Value, String> {
    composio_json(
        snapshot,
        reqwest::Method::GET,
        "/connected_accounts",
        &[
            ("toolkit_slugs", COMPOSIO_TOOLKIT),
            ("user_ids", snapshot.user_id.as_str()),
            ("limit", "20"),
        ],
        None,
    )
    .await
}

fn suitable_managed_drive_auth_config(value: &Value) -> Option<String> {
    account_items(value).iter().find_map(|item| {
        let restrictions = item["restrict_to_following_tools"].as_array()?;
        let restricted_to_find =
            restrictions.len() == 1 && restrictions[0].as_str() == Some(COMPOSIO_FIND_FILE_TOOL);
        let read_only_scope = item["credentials"]["scopes"]
            .as_str()
            .is_some_and(|scopes| {
                scopes.split(',').map(str::trim).all(|scope| {
                    scope == GOOGLE_DRIVE_METADATA_SCOPE || scope == GOOGLE_DRIVE_FILE_SCOPE
                })
            });
        (toolkit_slug(item) == Some(COMPOSIO_TOOLKIT)
            && item["is_composio_managed"].as_bool() == Some(true)
            && item["status"].as_str() != Some("DISABLED")
            && restricted_to_find
            && read_only_scope)
            .then(|| item["id"].as_str())
            .flatten()
            .filter(|id| validate_resource_id(id))
            .map(str::to_owned)
    })
}

async fn google_drive_auth_config_id(snapshot: &IntegrationSnapshot) -> Result<String, String> {
    let response = composio_json(
        snapshot,
        reqwest::Method::GET,
        "/auth_configs",
        &[
            ("toolkit_slug", COMPOSIO_TOOLKIT),
            ("is_composio_managed", "true"),
            ("show_disabled", "false"),
            ("limit", "50"),
        ],
        None,
    )
    .await?;
    if let Some(id) = suitable_managed_drive_auth_config(&response) {
        return Ok(id.to_owned());
    }

    // Create a project-local managed config restricted to the sole read-only
    // tool used by this module. This avoids silently selecting a pre-existing
    // config with broader OAuth scope or unrelated tool access.
    let created = composio_json(
        snapshot,
        reqwest::Method::POST,
        "/auth_configs",
        &[],
        Some(json!({
            "toolkit": {"slug": COMPOSIO_TOOLKIT},
            "auth_config": {
                "type": "use_composio_managed_auth",
                "credentials": {"scopes": GOOGLE_DRIVE_METADATA_SCOPE},
                "restrict_to_following_tools": [COMPOSIO_FIND_FILE_TOOL]
            }
        })),
    )
    .await?;
    created["auth_config"]["id"]
        .as_str()
        .or_else(|| created["id"].as_str())
        .filter(|id| validate_resource_id(id))
        .map(str::to_owned)
        .ok_or("Composio did not return the new Google Drive auth config".to_string())
}

fn is_composio_connect_url(raw: &str) -> bool {
    reqwest::Url::parse(raw).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("connect.composio.dev")
            && url.username().is_empty()
            && url.password().is_none()
            && url.path().starts_with("/link/")
            && url.path().len() > "/link/".len()
    })
}

#[cfg(target_os = "macos")]
fn open_fixed_url(url: &str) -> Result<(), String> {
    let status = std::process::Command::new("/usr/bin/open")
        .arg(url)
        .status()
        .map_err(|_| "Cannot open the setup page".to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Cannot open the setup page".to_string())
    }
}

#[cfg(not(target_os = "macos"))]
fn open_fixed_url(_url: &str) -> Result<(), String> {
    Err("Opening provider setup requires macOS".to_string())
}

#[tauri::command]
pub async fn integration_status() -> Result<IntegrationStatus, String> {
    let snapshot = match capture_composio_snapshot() {
        Ok(snapshot) => snapshot,
        Err(_) => return local_status(),
    };
    let accounts = list_drive_accounts(&snapshot).await?;
    assert_current(snapshot.generation)?;
    let drive_account = active_drive_account(&accounts, &snapshot.user_id);
    {
        let mut guard = state()
            .lock()
            .map_err(|_| "Integration state busy".to_string())?;
        guard.drive_account = drive_account.clone();
    }
    let giphy_key_present = native_keychain::get(KEYCHAIN_GIPHY)?.is_some();
    Ok(IntegrationStatus {
        composio_key_present: true,
        drive_connected: drive_account.is_some(),
        drive_account_label: drive_account.and_then(|account| account.label),
        giphy_key_present,
    })
}

#[tauri::command]
pub async fn save_composio_key(api_key: String) -> Result<IntegrationStatus, String> {
    let key = validate_secret(&api_key, "Composio project", 512)?;
    {
        let mut guard = state()
            .lock()
            .map_err(|_| "Integration state busy".to_string())?;
        native_keychain::set(KEYCHAIN_COMPOSIO, &key)?;
        guard.generation = guard.generation.wrapping_add(1);
        guard.pending_connection = None;
        guard.drive_account = None;
    }
    local_status()
}

#[tauri::command]
pub async fn connect_google_drive() -> Result<GoogleDriveConnectResponse, String> {
    let snapshot = capture_composio_snapshot()?;
    let auth_config_id = google_drive_auth_config_id(&snapshot).await?;
    assert_current(snapshot.generation)?;
    let response = composio_json(
        &snapshot,
        reqwest::Method::POST,
        "/connected_accounts/link",
        &[],
        Some(json!({
            "auth_config_id": auth_config_id,
            "user_id": snapshot.user_id,
        })),
    )
    .await?;
    assert_current(snapshot.generation)?;
    let redirect_url = response["redirect_url"]
        .as_str()
        .or_else(|| response["redirectUrl"].as_str())
        .filter(|value| is_composio_connect_url(value))
        .ok_or("Composio returned an unsafe authorization link".to_string())?;
    let connected_account_id = response["connected_account_id"]
        .as_str()
        .or_else(|| response["id"].as_str())
        .filter(|value| validate_resource_id(value))
        .ok_or("Composio returned an invalid connection".to_string())?;
    open_fixed_url(redirect_url)?;
    let mut guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    if guard.generation != snapshot.generation {
        return Err("Integration changed; authorization cancelled".to_string());
    }
    guard.pending_connection = Some(PendingConnection {
        generation: snapshot.generation,
        connected_account_id: connected_account_id.to_owned(),
    });
    Ok(GoogleDriveConnectResponse { pending: true })
}

#[tauri::command]
pub async fn refresh_google_drive() -> Result<IntegrationStatus, String> {
    let snapshot = match capture_composio_snapshot() {
        Ok(snapshot) => snapshot,
        Err(_) => return local_status(),
    };
    let accounts = list_drive_accounts(&snapshot).await?;
    assert_current(snapshot.generation)?;
    let drive_account = active_drive_account(&accounts, &snapshot.user_id);
    let mut guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    if guard.generation != snapshot.generation {
        return Err("Integration changed; discarded response".to_string());
    }
    guard.drive_account = drive_account.clone();
    if guard
        .pending_connection
        .as_ref()
        .is_some_and(|pending| pending.generation == snapshot.generation && drive_account.is_some())
    {
        guard.pending_connection = None;
    }
    Ok(IntegrationStatus {
        composio_key_present: true,
        drive_connected: drive_account.is_some(),
        drive_account_label: drive_account.and_then(|account| account.label),
        giphy_key_present: native_keychain::get(KEYCHAIN_GIPHY)?.is_some(),
    })
}

#[tauri::command]
pub async fn disconnect_google_drive() -> Result<IntegrationStatus, String> {
    let snapshot = match capture_composio_snapshot() {
        Ok(snapshot) => snapshot,
        Err(_) => return local_status(),
    };
    let accounts = list_drive_accounts(&snapshot).await?;
    let ids: Vec<String> = account_items(&accounts)
        .iter()
        .filter(|item| {
            toolkit_slug(item) == Some(COMPOSIO_TOOLKIT)
                && item["user_id"].as_str() == Some(snapshot.user_id.as_str())
        })
        .filter_map(|item| item["id"].as_str().filter(|id| validate_resource_id(id)))
        .map(str::to_owned)
        .collect();
    for id in ids {
        assert_current(snapshot.generation)?;
        composio_delete(&snapshot, &id).await?;
    }
    assert_current(snapshot.generation)?;
    let mut guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    guard.pending_connection = None;
    guard.drive_account = None;
    drop(guard);
    local_status()
}

fn drive_query(query: &str) -> Result<String, String> {
    if query.trim().is_empty() || query.chars().count() > 200 {
        return Err("Enter a Google Drive search query".to_string());
    }
    // Spoken input is always treated as literal name text. Drive's query
    // language is intentionally not exposed through the voice finder.
    let escaped = query.replace('\\', "\\\\").replace('\'', "\\'");
    Ok(format!("name contains '{escaped}' and trashed = false"))
}

fn valid_drive_url(raw: &str) -> bool {
    reqwest::Url::parse(raw).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && matches!(
                url.host_str(),
                Some("drive.google.com")
                    | Some("docs.google.com")
                    | Some("drive.usercontent.google.com")
            )
    })
}

fn map_drive_results(value: &Value) -> Vec<GoogleDriveSearchResult> {
    drive_files(value)
        .iter()
        .filter_map(|file| {
            let name = file["name"].as_str()?.to_owned();
            let url = file["webViewLink"]
                .as_str()
                .or_else(|| file["web_view_link"].as_str())
                .or_else(|| file["url"].as_str())?;
            if !valid_drive_url(url) {
                return None;
            }
            let keywords = file["keywords"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .filter(|items| !items.is_empty());
            Some(GoogleDriveSearchResult {
                name,
                url: url.to_owned(),
                keywords,
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

#[tauri::command]
pub async fn search_google_drive(query: String) -> Result<GoogleDriveSearchResponse, String> {
    let snapshot = capture_composio_snapshot()?;
    let accounts = list_drive_accounts(&snapshot).await?;
    assert_current(snapshot.generation)?;
    let account = active_drive_account(&accounts, &snapshot.user_id)
        .ok_or("Connect Google Drive before searching")?;
    let q = drive_query(&query)?;
    let response = composio_json(
        &snapshot,
        reqwest::Method::POST,
        "/tools/execute/GOOGLEDRIVE_FIND_FILE",
        &[],
        Some(json!({
            "connected_account_id": account.id,
            "user_id": snapshot.user_id,
            "version": "latest",
            "arguments": {
                "q": q
            }
        })),
    )
    .await?;
    assert_current(snapshot.generation)?;
    if response["successful"].as_bool() == Some(false) {
        return Err("Google Drive search was not accepted".to_string());
    }
    Ok(GoogleDriveSearchResponse {
        results: map_drive_results(&response["data"]),
    })
}

fn giphy_url(raw: &str) -> bool {
    reqwest::Url::parse(raw).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url
                .host_str()
                .is_some_and(|host| host == "giphy.com" || host.ends_with(".giphy.com"))
    })
}

fn number(value: &Value) -> u32 {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| value.as_str()?.parse().ok())
        .unwrap_or(0)
}

fn map_giphy_results(value: &Value, kind: &str) -> Vec<GiphySearchResult> {
    value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let id = item["id"].as_str()?.to_owned();
            let preview = item["images"]["fixed_height_small"]["url"]
                .as_str()
                .or_else(|| item["images"]["fixed_width_small"]["url"].as_str())?;
            let content = item["images"]["original"]["url"].as_str()?;
            let source = item["url"].as_str()?;
            if !giphy_url(preview) || !giphy_url(content) || !giphy_url(source) {
                return None;
            }
            Some(GiphySearchResult {
                id,
                title: item["title"].as_str().unwrap_or("").to_owned(),
                alt_text: item["alt_text"].as_str().unwrap_or("GIF").to_owned(),
                preview_url: preview.to_owned(),
                content_url: content.to_owned(),
                source_url: source.to_owned(),
                width: number(&item["images"]["original"]["width"]),
                height: number(&item["images"]["original"]["height"]),
                kind: kind.to_owned(),
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

#[tauri::command]
pub async fn save_giphy_key(api_key: String) -> Result<IntegrationStatus, String> {
    let key = validate_secret(&api_key, "GIPHY", 256)?;
    {
        let mut guard = state()
            .lock()
            .map_err(|_| "Integration state busy".to_string())?;
        native_keychain::set(KEYCHAIN_GIPHY, &key)?;
        guard.generation = guard.generation.wrapping_add(1);
        guard.pending_connection = None;
        guard.drive_account = None;
    }
    local_status()
}

#[tauri::command]
pub fn disconnect_giphy() -> Result<IntegrationStatus, String> {
    let mut guard = state()
        .lock()
        .map_err(|_| "Integration state busy".to_string())?;
    native_keychain::delete(KEYCHAIN_GIPHY)?;
    guard.generation = guard.generation.wrapping_add(1);
    guard.pending_connection = None;
    guard.drive_account = None;
    drop(guard);
    local_status()
}

/// GIPHY's search endpoints are plural (`/v1/gifs/search`); the singular form is a 404.
fn giphy_search_path(kind: &str) -> Option<&'static str> {
    match kind {
        "gif" => Some("gifs"),
        "sticker" => Some("stickers"),
        _ => None,
    }
}

/// Names the fix for the GIPHY failures a user can act on; the key itself is never echoed.
fn giphy_rejection(status: u16) -> String {
    match status {
        401 | 403 => "GIPHY rejected the API key. Check it in Settings → Connections.".to_string(),
        429 => "GIPHY's rate limit was reached. Try again in a few minutes.".to_string(),
        _ => format!("GIPHY search was not accepted (HTTP {status})"),
    }
}

#[tauri::command]
pub async fn search_giphy(
    query: String,
    kind: String,
    offset: u32,
) -> Result<GiphySearchResponse, String> {
    if query.trim().is_empty() || query.chars().count() > MAX_QUERY_CHARS {
        return Err("Enter a GIPHY search query".to_string());
    }
    let path = giphy_search_path(&kind).ok_or("Choose GIF or sticker search")?;
    if offset > 4999 {
        return Err("GIPHY search offset is out of range".to_string());
    }
    let (generation, key, user_id) = capture_giphy_snapshot()?;
    let endpoint = format!("{GIPHY_API}/{path}/search");
    let offset = offset.to_string();
    let response = client()?
        .get(endpoint)
        .query(&[
            ("api_key", key.as_str()),
            ("q", query.as_str()),
            ("limit", "3"),
            ("offset", offset.as_str()),
            ("rating", "g"),
            ("customer_id", user_id.as_str()),
        ])
        .send()
        .await
        .map_err(|_| "GIPHY search unavailable".to_string())?;
    let status = response.status();
    let bytes = bounded_body(response).await?;
    if !status.is_success() {
        return Err(giphy_rejection(status.as_u16()));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| "GIPHY returned an invalid response".to_string())?;
    assert_current(generation)?;
    Ok(GiphySearchResponse {
        query,
        kind: kind.clone(),
        provider: "giphy".to_string(),
        attribution: "Powered by GIPHY".to_string(),
        results: map_giphy_results(&value, &kind),
    })
}

#[tauri::command]
pub fn open_setup_link(destination: String) -> Result<(), String> {
    let url = match destination.as_str() {
        "composio" => "https://dashboard.composio.dev/",
        "giphy" => "https://developers.giphy.com/dashboard/",
        "openai" => "https://platform.openai.com/api-keys",
        "gemini" => "https://aistudio.google.com/apikey",
        _ => return Err("Unknown setup destination".to_string()),
    };
    open_fixed_url(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn giphy_search_uses_the_plural_endpoints() {
        assert_eq!(giphy_search_path("gif"), Some("gifs"));
        assert_eq!(giphy_search_path("sticker"), Some("stickers"));
        assert_eq!(giphy_search_path("gifs"), None);
    }

    #[test]
    fn giphy_rejections_name_the_fix() {
        assert!(giphy_rejection(401).contains("rejected the API key"));
        assert!(giphy_rejection(403).contains("rejected the API key"));
        assert!(giphy_rejection(429).contains("rate limit"));
        assert_eq!(
            giphy_rejection(500),
            "GIPHY search was not accepted (HTTP 500)"
        );
    }

    #[test]
    fn status_uses_the_frontend_contract_names() {
        let value = serde_json::to_value(IntegrationStatus {
            composio_key_present: true,
            drive_connected: false,
            drive_account_label: None,
            giphy_key_present: true,
        })
        .unwrap();
        assert_eq!(
            value,
            json!({
                "composioKeyPresent": true,
                "driveConnected": false,
                "driveAccountLabel": null,
                "giphyKeyPresent": true
            })
        );
    }

    #[test]
    fn provider_urls_are_https_and_allowlisted() {
        assert!(giphy_url("https://media1.giphy.com/media/id/giphy.gif?x=1"));
        assert!(!giphy_url("http://media.giphy.com/media/id.gif"));
        assert!(!giphy_url("https://evil.example/media.gif"));
        assert!(valid_drive_url("https://drive.google.com/file/d/id/view"));
        assert!(!valid_drive_url("https://evil.example/file"));
        assert!(is_composio_connect_url(
            "https://connect.composio.dev/link/ln_abc"
        ));
        assert!(!is_composio_connect_url(
            "https://backend.composio.dev/link/ln_abc"
        ));
    }

    #[test]
    fn giphy_mapping_preserves_order_and_full_urls() {
        let value = json!({"data": [
            {"id":"one","title":"One","alt_text":"one","url":"https://giphy.com/gifs/one","images":{"fixed_height_small":{"url":"https://media.giphy.com/one-small.gif"},"original":{"url":"https://media.giphy.com/one.gif?x=1","width":"320","height":"200"}}},
            {"id":"two","title":"Two","alt_text":"two","url":"https://giphy.com/gifs/two","images":{"fixed_height_small":{"url":"https://media.giphy.com/two-small.gif"},"original":{"url":"https://media.giphy.com/two.gif?x=2","width":"640","height":"480"}}}
        ]});
        let results = map_giphy_results(&value, "gif");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "one");
        assert_eq!(
            results[1].content_url,
            "https://media.giphy.com/two.gif?x=2"
        );
    }

    #[test]
    fn drive_mapping_is_capped_and_rejects_non_drive_links() {
        let execution = json!({
            "successful": true,
            "data": {"files": [
                {"id":"one","name":"one","mimeType":"text/plain","webViewLink":"https://drive.google.com/file/d/one/view"},
                {"id":"bad","name":"bad","mimeType":"text/plain","webViewLink":"https://evil.example/bad"},
                {"id":"two","name":"two","mimeType":"application/pdf","webViewLink":"https://docs.google.com/document/d/two/edit"},
                {"id":"three","name":"three","mimeType":"text/plain","webViewLink":"https://drive.google.com/file/d/three/view"},
                {"id":"four","name":"four","mimeType":"text/plain","webViewLink":"https://drive.google.com/file/d/four/view"}
            ]}
        });
        let results = map_drive_results(&execution["data"]);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "one");
        assert_eq!(results[2].name, "three");
    }

    #[test]
    fn realistic_connected_account_response_is_fenced_to_user_and_toolkit() {
        let response = json!({"items": [
            {"id":"ca-other","toolkit":{"slug":"googledrive"},"user_id":"other","status":"ACTIVE"},
            {"id":"ca-wrong-toolkit","toolkit":{"slug":"gmail"},"user_id":"owner","status":"ACTIVE"},
            {"id":"ca-pending","toolkit":{"slug":"googledrive"},"user_id":"owner","status":"INITIALIZING"},
            {"id":"ca-owner","toolkit":{"slug":"googledrive"},"user_id":"owner","status":"ACTIVE","alias":"owner@example.com"}
        ]});
        let account = active_drive_account(&response, "owner").unwrap();
        assert_eq!(account.id, "ca-owner");
        assert_eq!(account.label.as_deref(), Some("owner@example.com"));
    }

    #[test]
    fn auth_config_selection_rejects_wider_or_unrelated_configs() {
        let response = json!({"items": [
            {"id":"ac-wide","toolkit":{"slug":"googledrive"},"is_composio_managed":true,"status":"ENABLED","credentials":{"scopes":"https://www.googleapis.com/auth/drive"},"restrict_to_following_tools":[]},
            {"id":"ac-gmail","toolkit":{"slug":"gmail"},"is_composio_managed":true,"status":"ENABLED","credentials":{"scopes":"https://www.googleapis.com/auth/drive.metadata.readonly"},"restrict_to_following_tools":["GOOGLEDRIVE_FIND_FILE"]},
            {"id":"ac-safe","toolkit":{"slug":"googledrive"},"is_composio_managed":true,"status":"ENABLED","credentials":{"scopes":"https://www.googleapis.com/auth/drive.metadata.readonly"},"restrict_to_following_tools":["GOOGLEDRIVE_FIND_FILE"]}
        ]});
        assert_eq!(
            suitable_managed_drive_auth_config(&response).as_deref(),
            Some("ac-safe")
        );
    }

    #[test]
    fn drive_plain_text_becomes_a_literal_name_query() {
        assert_eq!(
            drive_query("annual report").unwrap(),
            "name contains 'annual report' and trashed = false"
        );
        assert_eq!(
            drive_query("name contains 'report'").unwrap(),
            "name contains 'name contains \\'report\\'' and trashed = false"
        );
        assert_eq!(
            drive_query("mimeType = 'application/pdf'").unwrap(),
            "name contains 'mimeType = \\'application/pdf\\'' and trashed = false"
        );
    }

    #[test]
    fn resource_ids_and_users_are_fenced() {
        assert!(validate_resource_id("ca_abc-123"));
        assert!(!validate_resource_id("ca/abc"));
        assert!(validate_user_id(
            "destroy-community-12345678-1234-1234-1234-123456789012"
        ));
        assert!(!validate_user_id("other-user"));
    }
}
