//! Hold-to-dictate target capture and safe delivery.
//!
//! Ordinary dictation audio remains in the webview only long enough to call
//! STT. An explicitly requested voice note is the one exception: it becomes a
//! temporary attachment handoff file with bounded size and deletion. This
//! module otherwise keeps only privacy-safe app identity for the active key
//! hold and forgets it immediately after delivery or cancellation.
//!
//! Latency shapes the design. Identity comes from `NSWorkspace` and is instant,
//! so recording starts as soon as the original app/field target is captured.
//! The native Accessibility probe is deliberately completed in that shortcut
//! callback: exact target identity must exist before any HUD/window work can
//! race it. Delivery performs a fresh probe and accepts only the same target.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::clipboard::{
    copy_to_clipboard, paste_file, paste_prepared_preserving_clipboard, paste_preserving_clipboard,
    undo_last_edit, ActError,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use futures_util::StreamExt;
use serde::Serialize;
use tauri::State;

use crate::focus::{
    frontmost_app_identity, probe_dictation_target, same_dictation_field, DictationFocusTarget,
};

/// Upper bound on how long delivery waits for the accessibility probe. The
/// probe overlaps with speech, so this only guards a hung `osascript`.
const PROBE_WAIT: Duration = Duration::from_millis(1_200);
const MAX_MEDIA_BYTES: usize = 8 * 1024 * 1024;
const MAX_VOICE_NOTE_BYTES: usize = 6 * 1024 * 1024;
const MAX_VOICE_NOTE_BASE64_BYTES: usize = 8 * 1024 * 1024 + 4;
const UNDO_WINDOW: Duration = Duration::from_secs(30);
const VOICE_NOTE_RETENTION: Duration = Duration::from_secs(600);
#[cfg(not(debug_assertions))]
const VOICE_NOTE_DIR: &str = "destroy-dictation-community-voice-notes";
#[cfg(debug_assertions)]
const VOICE_NOTE_DIR: &str = "destroy-dictation-community-dev-voice-notes";

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Development-only, bounded lifecycle breadcrumbs. Never accepts customer
/// content, target metadata or session IDs. The file is created exclusively and
/// subsequent writes use the same handle, so a symlink cannot redirect them.
#[cfg(all(debug_assertions, not(test)))]
fn session_trace(event: &'static str, caller: u32) {
    use std::collections::VecDeque;
    use std::io::{Seek, SeekFrom, Write};
    use std::sync::OnceLock;
    struct Trace {
        file: Option<std::fs::File>,
        events: VecDeque<String>,
        sequence: u64,
    }
    static TRACE: OnceLock<Mutex<Trace>> = OnceLock::new();
    let mut trace = lock(TRACE.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("destroy-session-{}.log", std::process::id()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        Mutex::new(Trace {
            file: options.open(path).ok(),
            events: VecDeque::new(),
            sequence: 0,
        })
    }));
    trace.sequence += 1;
    let sequence = trace.sequence;
    if trace.events.len() == 64 {
        trace.events.pop_front();
    }
    trace
        .events
        .push_back(format!("{sequence} {event} line={caller}\n"));
    let content: String = trace.events.iter().map(String::as_str).collect();
    if let Some(file) = &mut trace.file {
        let _ = file.seek(SeekFrom::Start(0));
        let _ = file.write_all(content.as_bytes());
        let _ = file.set_len(content.len() as u64);
        let _ = file.flush();
    }
}

#[cfg(any(not(debug_assertions), test))]
fn session_trace(_event: &'static str, _caller: u32) {}

fn voice_note_directory() -> std::path::PathBuf {
    std::env::temp_dir().join(VOICE_NOTE_DIR)
}

fn cleanup_stale_voice_notes() {
    let directory = voice_note_directory();
    let Ok(directory_metadata) = std::fs::symlink_metadata(&directory) else {
        return;
    };
    if directory_metadata.file_type().is_symlink() || !directory_metadata.is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return;
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() || file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let is_ours = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("Destroy Voice Note "));
        let expired = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age >= VOICE_NOTE_RETENTION);
        if is_ours && expired {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn write_private_file(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let directory = path
        .parent()
        .ok_or_else(|| "Could not prepare the voice note attachment".to_string())?;
    match std::fs::symlink_metadata(directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err("Could not secure the voice note directory".into());
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(directory)
                .map_err(|_| "Could not prepare the voice note attachment".to_string())?;
        }
        Err(_) => return Err("Could not prepare the voice note attachment".into()),
    }
    let metadata = std::fs::symlink_metadata(directory)
        .map_err(|_| "Could not secure the voice note directory".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Could not secure the voice note directory".into());
    }
    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};

        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| "Could not secure the voice note directory".to_string())?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|_| "Could not prepare the voice note attachment".to_string())?;
        file.write_all(bytes)
            .map_err(|_| "Could not prepare the voice note attachment".to_string())?;
        file.sync_all()
            .map_err(|_| "Could not prepare the voice note attachment".to_string())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, bytes)
            .map_err(|_| "Could not prepare the voice note attachment".to_string())?;
    }
    Ok(())
}

/// A one-shot cell the accessibility probe publishes into.
#[derive(Default)]
struct TargetSlot {
    resolved: Mutex<Option<DictationFocusTarget>>,
    ready: Condvar,
}

impl TargetSlot {
    fn publish(&self, target: DictationFocusTarget) {
        let mut guard = lock(&self.resolved);
        *guard = Some(target);
        self.ready.notify_all();
    }

    /// Blocks until the probe lands, or returns `None` at `timeout`.
    fn wait(&self, timeout: Duration) -> Option<DictationFocusTarget> {
        let deadline = Instant::now() + timeout;
        let mut guard = lock(&self.resolved);
        while guard.is_none() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let (next, _) = self
                .ready
                .wait_timeout(guard, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard = next;
        }
        guard.clone()
    }
}

/// State for the active key hold. Cleared on delivery or cancellation.
pub struct DictationSession {
    active: Mutex<Option<ActiveDictation>>,
    /// The last successful inline insertion. Content stays in process memory
    /// only long enough to support an explicit “undo that” or correction.
    last_inline: Mutex<Option<InlineInsertion>>,
}

#[derive(Clone)]
struct ActiveDictation {
    id: String,
    /// Exact app/AX target captured before Destroy changes its own windows.
    started: DictationFocusTarget,
    /// Accessibility target captured in the same shortcut callback as `started`.
    probe: Arc<TargetSlot>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationStart {
    pub session_id: String,
    pub target: DictationFocusTarget,
    /// Direct insertion requires macOS Accessibility. Exposing the state with
    /// the captured app identity lets the notch explain the missing capability
    /// while the user is speaking instead of pretending the destination is
    /// unknown after transcription finishes.
    pub needs_accessibility: bool,
}

#[derive(Clone)]
struct InlineInsertion {
    target: DictationFocusTarget,
    text: String,
    inserted_at: Instant,
}

impl DictationSession {
    /// Captures the app and focused field before Destroy changes its own window.
    /// The global hotkey handler calls this synchronously, ahead of the event
    /// that expands the notch, so the destination cannot race the webview.
    pub(crate) fn capture_target(&self) -> DictationStart {
        let identity = frontmost_app_identity();
        let needs_accessibility = !crate::permissions::accessibility_granted();
        let captured = if needs_accessibility {
            identity.clone()
        } else {
            captured_target(&identity, probe_dictation_target())
        };
        let slot = Arc::new(TargetSlot::default());
        slot.publish(captured.clone());
        let session_id = uuid::Uuid::new_v4().to_string();
        *lock(&self.active) = Some(ActiveDictation {
            id: session_id.clone(),
            started: captured.clone(),
            probe: Arc::clone(&slot),
        });
        session_trace("capture", line!());
        DictationStart {
            session_id,
            target: captured,
            needs_accessibility,
        }
    }

    /// Returns the native hotkey capture when one already exists. Direct UI
    /// starts still work by capturing here as a fallback.
    pub(crate) fn current_or_capture_target(&self) -> DictationStart {
        if let Some(active) = lock(&self.active).clone() {
            session_trace("reuse", line!());
            return DictationStart {
                session_id: active.id,
                target: active.started,
                needs_accessibility: !crate::permissions::accessibility_granted(),
            };
        }
        self.capture_target()
    }

    pub(crate) fn is_active(&self, session_id: &str) -> bool {
        self.active(session_id).is_some()
    }

    fn active(&self, session_id: &str) -> Option<ActiveDictation> {
        let current = lock(&self.active);
        if current.is_none() {
            session_trace("lookup-missing", line!());
        } else if current
            .as_ref()
            .is_some_and(|active| active.id != session_id)
        {
            session_trace("lookup-mismatched", line!());
        }
        current
            .as_ref()
            .filter(|active| active.id == session_id)
            .cloned()
    }

    /// Atomically claims the matching session immediately before a native
    /// mutation. A cancellation or newer hold makes the old session unusable.
    #[track_caller]
    fn take_active(&self, session_id: &str) -> Option<ActiveDictation> {
        let mut active = lock(&self.active);
        if active.as_ref().is_some_and(|entry| entry.id == session_id) {
            session_trace("claim", std::panic::Location::caller().line());
            active.take()
        } else {
            session_trace("claim-rejected", std::panic::Location::caller().line());
            None
        }
    }

    fn cancel(&self, session_id: &str) -> bool {
        session_trace("cancel-request", line!());
        self.take_active(session_id).is_some()
    }

    pub(crate) fn purge_sensitive_state(&self) {
        session_trace("account-reset", line!());
        *lock(&self.active) = None;
        *lock(&self.last_inline) = None;
    }

    fn remember_inline(&self, target: &DictationFocusTarget, text: &str) {
        *lock(&self.last_inline) = Some(InlineInsertion {
            target: target.clone(),
            text: text.to_string(),
            inserted_at: Instant::now(),
        });
    }

    pub(crate) fn expire_sensitive_memory(&self) {
        let _ = self.recent_inline();
        cleanup_stale_voice_notes();
    }

    fn recent_inline(&self) -> Option<InlineInsertion> {
        let mut recent = lock(&self.last_inline);
        if recent
            .as_ref()
            .is_some_and(|entry| entry.inserted_at.elapsed() > UNDO_WINDOW)
        {
            *recent = None;
        }
        recent.clone()
    }
}

impl Default for DictationSession {
    fn default() -> Self {
        cleanup_stale_voice_notes();
        Self {
            active: Mutex::new(None),
            last_inline: Mutex::new(None),
        }
    }
}

/// Combines the immediate app identity with the AX probe captured in the same
/// shortcut callback. A probe from another app is never allowed to become the
/// target, even if the user switched focus during the probe.
fn captured_target(
    identity: &DictationFocusTarget,
    probed: DictationFocusTarget,
) -> DictationFocusTarget {
    if !identity.bundle_id.is_empty()
        && probed.bundle_id == identity.bundle_id
        && !probed.focus_signature.is_empty()
    {
        DictationFocusTarget {
            app_icon_data_url: identity.app_icon_data_url.clone(),
            ..probed
        }
    } else {
        identity.clone()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationDelivery {
    pub delivery: &'static str,
    pub reason: Option<&'static str>,
    pub app_kind: String,
}

/// Captures the dictation target before the HUD can change app/window state.
///
/// When Accessibility is available, the returned target includes the exact
/// focused AX field and its window identity. Without it, only the safe app
/// identity is returned and delivery reports the permission fallback.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_begin(state: State<'_, DictationSession>) -> DictationStart {
    state.current_or_capture_target()
}

/// Waits for the accessibility probe so the UI can refine the target icon —
/// a browser window resolves to Gmail or Notion once its title is known.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_target(
    state: State<'_, DictationSession>,
    session_id: String,
) -> Option<DictationFocusTarget> {
    let active = state.active(&session_id)?;
    let mut target = active.probe.wait(PROBE_WAIT)?;
    if target.app_icon_data_url.is_none() {
        target.app_icon_data_url = active.started.app_icon_data_url;
    }
    Some(target)
}

/// Returns only an explicit selection captured at key-down. The selection is
/// never logged or persisted and is cleared with the dictation session.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_selected_text(
    state: State<'_, DictationSession>,
    session_id: String,
) -> Option<String> {
    state
        .active(&session_id)
        .and_then(|active| active.probe.wait(PROBE_WAIT))
        .filter(|target| target.can_paste)
        .and_then(|target| target.selected_text)
        .filter(|value| !value.trim().is_empty())
}

/// Memory-only previous insertion used for an explicit spoken correction.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_previous_text(
    state: State<'_, DictationSession>,
    session_id: String,
) -> Option<String> {
    let active = state.active(&session_id)?;
    let captured = active.probe.wait(PROBE_WAIT)?;
    state
        .recent_inline()
        .filter(|entry| same_dictation_field(&entry.target, &captured))
        .map(|entry| entry.text)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_cancel(
    app: tauri::AppHandle,
    state: State<'_, DictationSession>,
    session_id: String,
    reason: Option<String>,
) -> bool {
    // Only fixed codes reach the development trace, never arbitrary IPC text.
    session_trace(
        match reason.as_deref() {
            Some("no-account") => "cancel-no-account",
            Some("capture-start-error") => "cancel-capture-start-error",
            Some("processing-error") => "cancel-processing-error",
            Some("dispose") => "cancel-dispose",
            Some("user") => "cancel-user",
            _ => "cancel-unspecified",
        },
        line!(),
    );
    let cancelled = state.cancel(&session_id);
    use tauri::Manager;
    app.state::<crate::native_audio::NativeAudio>()
        .cancel(&session_id);
    cancelled
}

/// Places dashboard Quick Copy text on the system clipboard.
///
/// The dashboard runs in the Tauri webview, where the browser Clipboard API
/// can be unavailable even though Destroy has the native clipboard permissions it
/// uses for dictation delivery.
#[tauri::command]
pub fn destroy_dictation_copy(text: String) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("There is no dictation to copy".into());
    }
    copy_to_clipboard(text).map_err(|error| error.to_string())
}

/// Account/team privacy boundary used by sign-out and account deletion.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_clear_sensitive_state(state: State<'_, DictationSession>) {
    state.purge_sensitive_state();
}

/// Undoes only Destroy's most recent inline insertion, only while the same app is
/// still frontmost and inside the short correction window.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_undo(
    state: State<'_, DictationSession>,
    session_id: String,
) -> Result<bool, String> {
    let Some(entry) = state.recent_inline() else {
        return Ok(false);
    };
    if state.active(&session_id).is_none() {
        return Ok(false);
    }
    let current = probe_dictation_target();
    if !same_dictation_field(&current, &entry.target) {
        return Ok(false);
    }
    if state.take_active(&session_id).is_none() {
        return Ok(false);
    }
    undo_last_edit().map_err(|error| error.to_string())?;
    *lock(&state.last_inline) = None;
    Ok(true)
}

/// Replaces Destroy's most recent inline insertion after the server has applied a
/// spoken correction. This is one explicit undo followed by one paste; it does
/// not read or rewrite unrelated field content.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_replace_last(
    state: State<'_, DictationSession>,
    session_id: String,
    text: String,
) -> Result<DictationDelivery, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("Corrected dictation is empty".into());
    }
    let Some(entry) = state.recent_inline() else {
        return Err("The previous dictation is no longer available to correct".into());
    };
    if state.active(&session_id).is_none() {
        return Err("That dictation session was cancelled".into());
    }
    let current = probe_dictation_target();
    if !same_dictation_field(&current, &entry.target) {
        return Err("Return to the original field before correcting that dictation".into());
    }
    if state.take_active(&session_id).is_none() {
        return Err("That dictation session was cancelled".into());
    }
    undo_last_edit().map_err(|error| error.to_string())?;
    std::thread::sleep(Duration::from_millis(80));
    if paste_preserving_clipboard(text).is_err() {
        *lock(&state.last_inline) = None;
        copy_to_clipboard(text).map_err(|error| error.to_string())?;
        return Ok(DictationDelivery {
            delivery: "clipboard",
            reason: Some("paste_failed"),
            app_kind: entry.target.app_kind,
        });
    }
    state.remember_inline(&entry.target, text);
    Ok(DictationDelivery {
        delivery: "inline",
        reason: None,
        app_kind: entry.target.app_kind,
    })
}

/// Chooses how transcribed text reaches the user, and delivers it.
///
/// Inline paste requires that the app is unchanged *and* that its focused field
/// accepted dictation. Everything else falls back to the clipboard — dictated
/// text is never silently dropped.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_deliver(
    state: State<'_, DictationSession>,
    session_id: String,
    text: String,
) -> Result<DictationDelivery, String> {
    deliver_text(&state, &session_id, &text, false)
}

/// Delivers a text choice made inside Destroy's picker. Unlike ordinary dictation,
/// clicking a choice may activate Destroy, so this explicit path returns focus to
/// the app captured at key-down before it pastes.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_deliver_picker_text(
    state: State<'_, DictationSession>,
    session_id: String,
    text: String,
) -> Result<DictationDelivery, String> {
    deliver_text(&state, &session_id, &text, true)
}

fn current_delivery_target(has_accessibility: bool) -> DictationFocusTarget {
    if has_accessibility {
        probe_dictation_target()
    } else {
        // App identity remains available without AX and is enough to prevent a
        // clipboard fallback from being attributed to the wrong application.
        frontmost_app_identity()
    }
}

fn deliver_text(
    state: &DictationSession,
    session_id: &str,
    text: &str,
    reactivate_picker_target: bool,
) -> Result<DictationDelivery, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("No speech was transcribed".into());
    }

    let active = state
        .active(session_id)
        .ok_or_else(|| "That dictation session was cancelled".to_string())?;
    let probed = active.probe.wait(PROBE_WAIT);

    if reactivate_picker_target {
        if frontmost_app_identity().bundle_id != active.started.bundle_id
            && reactivate_dictation_target(&active.started.bundle_id)
        {
            std::thread::sleep(Duration::from_millis(120));
        }
    }
    let has_accessibility = crate::permissions::accessibility_granted();
    let current = current_delivery_target(has_accessibility);
    let mut plan = DeliveryPlan::resolve(Some(&active.started), probed.as_ref(), &current);

    // Typing into another app needs Accessibility. Attempting the paste without
    // it fires macOS's consent alert mid-delivery, which steals focus and loses
    // the target the user was dictating into. Route to the clipboard instead and
    // let the UI ask for permission at a moment the user controls.
    if !has_accessibility {
        plan.needs_accessibility = true;
    }

    // Claim only after every slow probe/reactivation. Cancel or a newer hold
    // wins until the instant before the clipboard/keystroke mutation.
    if state.take_active(session_id).is_none() {
        return Err("That dictation session was cancelled".into());
    }

    if plan.can_paste_inline() && paste_preserving_clipboard(text).is_ok() {
        state.remember_inline(&current, text);
        return Ok(DictationDelivery {
            delivery: "inline",
            reason: None,
            app_kind: plan.app_kind,
        });
    }

    copy_to_clipboard(text).map_err(|error| error.to_string())?;
    Ok(DictationDelivery {
        delivery: "clipboard",
        reason: Some(plan.fallback_reason()),
        app_kind: plan.app_kind,
    })
}

fn is_giphy_asset_url(value: &str) -> bool {
    value.len() <= 2_048
        && reqwest::Url::parse(value).is_ok_and(|url| {
            url.scheme() == "https"
                && url.host_str().is_some_and(|host| {
                    let host = host.to_ascii_lowercase();
                    host == "giphy.com" || host.ends_with(".giphy.com")
                })
        })
}

/// Attaches an explicitly recorded voice note and then inserts its transcript.
/// The raw note exists only as a temporary handoff file and is deleted after
/// the receiving app has had time to finish its upload.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn destroy_dictation_deliver_voice_note(
    state: State<'_, DictationSession>,
    session_id: String,
    audio_base64: String,
    content_type: String,
    transcript: String,
) -> Result<DictationDelivery, String> {
    let active = state
        .active(&session_id)
        .ok_or_else(|| "That dictation session was cancelled".to_string())?;
    let probed = active.probe.wait(PROBE_WAIT);
    if frontmost_app_identity().bundle_id != active.started.bundle_id
        && reactivate_dictation_target(&active.started.bundle_id)
    {
        std::thread::sleep(Duration::from_millis(120));
    }
    let has_accessibility = crate::permissions::accessibility_granted();
    let current = current_delivery_target(has_accessibility);
    let mut plan = DeliveryPlan::resolve(Some(&active.started), probed.as_ref(), &current);
    if !has_accessibility {
        plan.needs_accessibility = true;
    }
    if !plan.can_paste_inline() {
        if state.take_active(&session_id).is_none() {
            return Err("That dictation session was cancelled".into());
        }
        copy_to_clipboard(transcript.trim()).map_err(|error| error.to_string())?;
        return Ok(DictationDelivery {
            delivery: "clipboard",
            reason: Some(plan.fallback_reason()),
            app_kind: plan.app_kind,
        });
    }

    let encoded_audio = audio_base64.trim();
    if encoded_audio.len() > MAX_VOICE_NOTE_BASE64_BYTES {
        return Err("Voice note must be no larger than 6 MB".into());
    }
    let bytes = STANDARD
        .decode(encoded_audio)
        .map_err(|_| "Voice note audio is invalid".to_string())?;
    if bytes.is_empty() || bytes.len() > MAX_VOICE_NOTE_BYTES {
        return Err("Voice note must be between 1 byte and 6 MB".into());
    }
    let extension = match content_type.split(';').next().unwrap_or_default().trim() {
        "audio/mp4" | "audio/m4a" => "m4a",
        "audio/mpeg" => "mp3",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/webm" => "webm",
        _ => return Err("This voice note audio format is not supported".into()),
    };
    let directory = voice_note_directory();
    let path = directory.join(format!(
        "Destroy Voice Note {}.{extension}",
        uuid::Uuid::new_v4()
    ));
    write_private_file(&path, &bytes)?;
    // Encoding/file I/O may take long enough for focus to change. Re-probe
    // immediately before claiming the session and attaching anything.
    let has_accessibility = crate::permissions::accessibility_granted();
    let current = current_delivery_target(has_accessibility);
    plan = DeliveryPlan::resolve(Some(&active.started), probed.as_ref(), &current);
    if !has_accessibility {
        plan.needs_accessibility = true;
    }
    if !plan.can_paste_inline() {
        if state.take_active(&session_id).is_none() {
            let _ = std::fs::remove_file(&path);
            return Err("That dictation session was cancelled".into());
        }
        let _ = std::fs::remove_file(&path);
        copy_to_clipboard(transcript.trim()).map_err(|error| error.to_string())?;
        return Ok(DictationDelivery {
            delivery: "clipboard",
            reason: Some(plan.fallback_reason()),
            app_kind: plan.app_kind,
        });
    }
    if state.take_active(&session_id).is_none() {
        let _ = std::fs::remove_file(&path);
        return Err("That dictation session was cancelled".into());
    }
    if let Err(error) = paste_file(&path) {
        let _ = std::fs::remove_file(&path);
        return Err(error.to_string());
    }
    // Schedule deletion as soon as the receiving app has the file reference,
    // including when the optional transcript paste fails afterward.
    let cleanup_path = path.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(VOICE_NOTE_RETENTION).await;
        let _ = std::fs::remove_file(cleanup_path);
    });
    std::thread::sleep(Duration::from_millis(800));
    if !transcript.trim().is_empty() {
        if paste_preserving_clipboard(&format!("Transcript: {}", transcript.trim())).is_err() {
            copy_to_clipboard(&format!("Transcript: {}", transcript.trim()))
                .map_err(|error| error.to_string())?;
            return Ok(DictationDelivery {
                delivery: "clipboard",
                reason: Some("voice_note_attached_transcript_copied"),
                app_kind: plan.app_kind,
            });
        }
    }
    Ok(DictationDelivery {
        delivery: "inline",
        reason: None,
        app_kind: plan.app_kind,
    })
}

pub(crate) async fn download_giphy_asset(url: &str) -> Result<(Vec<u8>, String), String> {
    if !is_giphy_asset_url(url) {
        return Err("Destroy refused a media URL outside GIPHY".into());
    }
    let response = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(4))
        .timeout(Duration::from_secs(15))
        // Validate every hop, not only the final URL. Otherwise a trusted
        // media host could redirect the desktop through a local/private URL
        // before returning to GIPHY, turning attachment delivery into SSRF.
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 || !is_giphy_asset_url(attempt.url().as_str()) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|_| "Could not prepare the media download".to_string())?
        .get(url)
        .send()
        .await
        .map_err(|_| "Could not download this GIPHY result".to_string())?;
    if !response.status().is_success() || !is_giphy_asset_url(response.url().as_str()) {
        return Err("GIPHY did not return a usable media asset".into());
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_MEDIA_BYTES as u64)
    {
        return Err("This GIPHY result is too large to paste".into());
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/gif")
        .split(';')
        .next()
        .unwrap_or("image/gif")
        .trim()
        .to_ascii_lowercase();
    if !matches!(
        content_type.as_str(),
        "image/gif" | "image/webp" | "image/png"
    ) {
        return Err("GIPHY returned an unsupported media format".into());
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "The GIPHY download was interrupted".to_string())?;
        if bytes.len().saturating_add(chunk.len()) > MAX_MEDIA_BYTES {
            return Err("This GIPHY result is too large to paste".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.is_empty() {
        return Err("GIPHY returned an empty media asset".into());
    }
    Ok((bytes, content_type))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn plain_media_fallback(leading_text: &str, source_url: &str, content_url: &str) -> String {
    let link = if source_url.trim().is_empty() {
        content_url.trim()
    } else {
        source_url.trim()
    };
    if leading_text.trim().is_empty() {
        link.to_string()
    } else {
        format!("{}\n{}", leading_text.trim(), link)
    }
}

#[cfg(target_os = "macos")]
fn reactivate_dictation_target(bundle_id: &str) -> bool {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    use objc2_foundation::NSString;

    if bundle_id.trim().is_empty() {
        return false;
    }
    let bundle = NSString::from_str(bundle_id);
    let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle);
    applications.firstObject().is_some_and(|application| {
        application.activateWithOptions(NSApplicationActivationOptions::empty())
    })
}

#[cfg(not(target_os = "macos"))]
fn reactivate_dictation_target(_bundle_id: &str) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn write_media_clipboard(
    bytes: &[u8],
    content_type: &str,
    leading_text: &str,
    content_url: &str,
    source_url: &str,
    alt_text: &str,
) -> Result<(), String> {
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSData, NSString};

    let webp_type = NSString::from_str("org.webmproject.webp");
    let gif_type = NSString::from_str("com.compuserve.gif");
    let png_type = NSString::from_str("public.png");
    let html_type = NSString::from_str("public.html");
    let string_type = NSString::from_str("public.utf8-plain-text");
    let media_type = match content_type {
        "image/png" => &*png_type,
        "image/webp" => &*webp_type,
        _ => &*gif_type,
    };
    let plain = plain_media_fallback(leading_text, source_url, content_url);
    let body = if leading_text.trim().is_empty() {
        String::new()
    } else {
        format!("<p>{}</p>", escape_html(leading_text.trim()))
    };
    let html = format!(
        "{body}<img src=\"{}\" alt=\"{}\">",
        escape_html(content_url),
        escape_html(alt_text)
    );
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    let media_data = NSData::with_bytes(bytes);
    let html_data = NSData::with_bytes(html.as_bytes());
    let plain_string = NSString::from_str(&plain);
    if !pasteboard.setData_forType(Some(&media_data), media_type) {
        return Err("Could not place GIPHY media on the clipboard".into());
    }
    let _ = pasteboard.setData_forType(Some(&html_data), &html_type);
    let _ = pasteboard.setString_forType(&plain_string, &string_type);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn write_media_clipboard(
    _bytes: &[u8],
    _content_type: &str,
    leading_text: &str,
    content_url: &str,
    source_url: &str,
    _alt_text: &str,
) -> Result<(), String> {
    copy_to_clipboard(&plain_media_fallback(leading_text, source_url, content_url))
        .map_err(|error| error.to_string())
}

/// Downloads a selected GIPHY result, writes a multi-flavor pasteboard payload
/// (animated media + HTML + plain fallback), and pastes only when the original
/// target is still focused and editable. It never sends the resulting draft.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn destroy_dictation_deliver_media(
    state: State<'_, DictationSession>,
    session_id: String,
    leading_text: String,
    content_url: String,
    source_url: String,
    alt_text: String,
) -> Result<DictationDelivery, String> {
    // Check before the network call so an already-cancelled picker does no
    // provider work. The matching session is checked and atomically claimed
    // again after the download, immediately before clipboard mutation.
    let source_url = if source_url.is_empty() || is_giphy_asset_url(&source_url) {
        source_url
    } else {
        String::new()
    };
    let alt_text: String = alt_text.chars().take(512).collect();
    let active = state
        .active(&session_id)
        .ok_or_else(|| "That dictation session was cancelled".to_string())?;
    let (bytes, content_type) = download_giphy_asset(&content_url).await?;
    let active = state
        .active(&session_id)
        .filter(|candidate| candidate.id == active.id)
        .ok_or_else(|| "That dictation session was cancelled".to_string())?;
    let probed = active.probe.wait(PROBE_WAIT);
    // Choosing a result can make Destroy frontmost. The picker is an extension of
    // the original field, so return focus to that captured app before ⌘V.
    // Plain dictation never activates an app; this is specific to an explicit
    // picker interaction.
    if frontmost_app_identity().bundle_id != active.started.bundle_id
        && reactivate_dictation_target(&active.started.bundle_id)
    {
        std::thread::sleep(Duration::from_millis(120));
    }
    let has_accessibility = crate::permissions::accessibility_granted();
    let current = current_delivery_target(has_accessibility);
    let mut plan = DeliveryPlan::resolve(Some(&active.started), probed.as_ref(), &current);
    if !has_accessibility {
        plan.needs_accessibility = true;
    }

    if state.take_active(&session_id).is_none() {
        return Err("That dictation session was cancelled".into());
    }

    if plan.can_paste_inline()
        && paste_prepared_preserving_clipboard(|| {
            write_media_clipboard(
                &bytes,
                &content_type,
                &leading_text,
                &content_url,
                &source_url,
                &alt_text,
            )
            .map_err(ActError::Exec)
        })
        .is_ok()
    {
        return Ok(DictationDelivery {
            delivery: "inline",
            reason: None,
            app_kind: plan.app_kind,
        });
    }
    write_media_clipboard(
        &bytes,
        &content_type,
        &leading_text,
        &content_url,
        &source_url,
        &alt_text,
    )?;
    Ok(DictationDelivery {
        delivery: "clipboard",
        reason: Some(plan.fallback_reason()),
        app_kind: plan.app_kind,
    })
}

/// The delivery decision, split out so it can be tested without a live desktop.
#[derive(Debug, PartialEq, Eq)]
struct DeliveryPlan {
    target_app_captured: bool,
    same_target_app: bool,
    same_target_field: bool,
    field_accepts_paste: bool,
    field_was_observed: bool,
    needs_accessibility: bool,
    app_kind: String,
}

impl DeliveryPlan {
    fn resolve(
        started: Option<&DictationFocusTarget>,
        probed: Option<&DictationFocusTarget>,
        current: &DictationFocusTarget,
    ) -> Self {
        let started_bundle = started
            .map(|target| target.bundle_id.as_str())
            .unwrap_or("");
        let target_app_captured = !started_bundle.is_empty();
        let same_target_app = !started_bundle.is_empty() && started_bundle == current.bundle_id;

        // Only trust the probe if it described the app we actually targeted —
        // the user may have switched apps while it was still running.
        let probe_matches = probed
            .is_some_and(|target| target.bundle_id == started_bundle && !started_bundle.is_empty());
        let same_target_field =
            probe_matches && probed.is_some_and(|target| same_dictation_field(target, current));
        let field_accepts_paste =
            same_target_field && probed.is_some_and(|target| target.can_paste) && current.can_paste;
        // A read-only field still has an accessibility signature. When both
        // probes lack one, macOS did not let Destroy observe the focused element at
        // all; do not misreport that as the user switching fields.
        let field_was_observed = probed.is_some_and(|target| {
            target.bundle_id == started_bundle && !target.focus_signature.is_empty()
        }) || (same_target_app && !current.focus_signature.is_empty());

        // The probe knows the window title, so it can tell Gmail from a plain
        // browser window. Fall back to the key-down identity when it cannot.
        let refined = probed
            .filter(|_| probe_matches)
            .map(|target| target.app_kind.clone())
            .filter(|kind| kind != "generic");
        let app_kind = refined
            .or_else(|| started.map(|target| target.app_kind.clone()))
            .unwrap_or_else(|| current.app_kind.clone());

        Self {
            target_app_captured,
            same_target_app,
            same_target_field,
            field_accepts_paste,
            field_was_observed,
            needs_accessibility: false,
            app_kind,
        }
    }

    fn can_paste_inline(&self) -> bool {
        !self.needs_accessibility
            && self.same_target_app
            && self.same_target_field
            && self.field_accepts_paste
    }

    /// Why the text went to the clipboard. The UI turns this into a next step,
    /// so order it most-actionable first.
    fn fallback_reason(&self) -> &'static str {
        if self.needs_accessibility
            && self.target_app_captured
            && (!self.field_was_observed || (self.same_target_app && self.field_accepts_paste))
        {
            "needs_accessibility"
        } else if !self.same_target_app {
            "target_changed"
        } else if !self.same_target_field {
            "field_changed"
        } else if !self.field_accepts_paste {
            "field_not_editable"
        } else {
            "paste_failed"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        captured_target, ActiveDictation, DeliveryPlan, DictationSession, InlineInsertion,
        TargetSlot, UNDO_WINDOW,
    };
    use crate::focus::DictationFocusTarget;
    use std::sync::Arc;
    use std::time::Duration;

    fn target(bundle: &str, kind: &str, can_paste: bool) -> DictationFocusTarget {
        DictationFocusTarget {
            can_paste,
            app_name: bundle.into(),
            bundle_id: bundle.into(),
            app_kind: kind.into(),
            app_icon_data_url: None,
            focus_signature: format!("{bundle}:field"),
            selected_text: None,
        }
    }

    #[test]
    fn initial_ax_probe_becomes_the_exact_target_before_hud_work() {
        let mut identity = target("com.apple.TextEdit", "generic", false);
        identity.app_icon_data_url = Some("data:image/png;base64,icon".into());
        let probed = target("com.apple.TextEdit", "editor", true);
        let captured = captured_target(&identity, probed.clone());

        assert_eq!(captured.focus_signature, probed.focus_signature);
        assert!(captured.can_paste);
        assert_eq!(captured.app_kind, "editor");
        assert_eq!(
            captured.app_icon_data_url.as_deref(),
            Some("data:image/png;base64,icon")
        );
    }

    #[test]
    fn probe_from_a_changed_app_is_rejected_as_the_original_target() {
        let identity = target("com.apple.TextEdit", "editor", false);
        let probed = target("com.apple.Notes", "generic", true);
        let captured = captured_target(&identity, probed);

        assert_eq!(captured, identity);
    }

    #[test]
    fn pastes_inline_when_app_and_field_both_hold() {
        let started = target("com.tinyspeck.slackmacgap", "slack", false);
        let probed = target("com.tinyspeck.slackmacgap", "slack", true);
        let plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &probed);
        assert!(plan.can_paste_inline());
        assert_eq!(plan.app_kind, "slack");
    }

    #[test]
    fn media_choice_hold_preserves_the_original_target() {
        let session = DictationSession::default();
        let original = target("com.google.Chrome", "gmail", false);
        let slot = Arc::new(TargetSlot::default());
        *super::lock(&session.active) = Some(ActiveDictation {
            id: "session-1".into(),
            started: original.clone(),
            probe: slot,
        });

        let start = session.current_or_capture_target();
        assert_eq!(start.session_id, "session-1");
        assert_eq!(start.target, original);
    }

    #[test]
    fn stale_session_id_cannot_claim_or_cancel_a_new_hold() {
        let session = DictationSession::default();
        *super::lock(&session.active) = Some(ActiveDictation {
            id: "new-session".into(),
            started: target("com.apple.mail", "mail", false),
            probe: Arc::new(TargetSlot::default()),
        });
        assert!(!session.cancel("old-session"));
        assert!(session.active("new-session").is_some());
        assert!(session.take_active("old-session").is_none());
        assert!(session.take_active("new-session").is_some());
    }

    #[test]
    fn expired_correction_text_is_actually_erased() {
        let session = DictationSession::default();
        *super::lock(&session.last_inline) = Some(InlineInsertion {
            target: target("com.apple.mail", "mail", true),
            text: "private customer text".into(),
            inserted_at: std::time::Instant::now() - UNDO_WINDOW - Duration::from_secs(1),
        });
        assert!(session.recent_inline().is_none());
        assert!(super::lock(&session.last_inline).is_none());
    }

    #[test]
    fn switching_apps_mid_transcription_falls_back_to_clipboard() {
        let started = target("com.google.Chrome", "browser", false);
        let probed = target("com.google.Chrome", "gmail", true);
        let current = target("com.apple.Terminal", "generic", false);
        let plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &current);
        assert!(!plan.can_paste_inline());
        assert_eq!(plan.fallback_reason(), "target_changed");
    }

    #[test]
    fn read_only_field_reports_why_it_used_the_clipboard() {
        let started = target("com.apple.Preview", "generic", false);
        let probed = target("com.apple.Preview", "generic", false);
        let plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &started);
        assert!(!plan.can_paste_inline());
        assert_eq!(plan.fallback_reason(), "field_not_editable");
    }

    #[test]
    fn switching_fields_inside_one_app_never_pastes_or_undoes() {
        let started = target("com.google.Chrome", "browser", false);
        let probed = target("com.google.Chrome", "gmail", true);
        let mut current = target("com.google.Chrome", "gmail", true);
        current.focus_signature = "com.google.Chrome:other-field".into();
        let plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &current);
        assert!(!plan.can_paste_inline());
        assert_eq!(plan.fallback_reason(), "field_changed");
    }

    /// A probe describing a different app than the one we targeted says nothing
    /// about our target's field, so it must not authorise a paste.
    #[test]
    fn stale_probe_never_authorises_a_paste() {
        let started = target("com.google.Chrome", "browser", false);
        let stale = target("com.apple.Terminal", "generic", true);
        let plan = DeliveryPlan::resolve(Some(&started), Some(&stale), &started);
        assert!(!plan.can_paste_inline());
        // The icon still reflects the app the user actually spoke into.
        assert_eq!(plan.app_kind, "browser");
    }

    /// The probe refines a bare browser window into the site being composed in.
    #[test]
    fn probe_refines_the_target_icon() {
        let started = target("com.google.Chrome", "browser", false);
        let probed = target("com.google.Chrome", "gmail", true);
        let plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &started);
        assert_eq!(plan.app_kind, "gmail");
    }

    /// Without Accessibility, Destroy must not attempt the paste at all — the
    /// consent alert would steal the very focus the paste depends on.
    #[test]
    fn missing_accessibility_routes_to_the_clipboard_with_a_next_step() {
        let started = target("com.tinyspeck.slackmacgap", "slack", false);
        let probed = target("com.tinyspeck.slackmacgap", "slack", true);
        let mut plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &probed);
        assert!(plan.can_paste_inline());

        plan.needs_accessibility = true;
        assert!(!plan.can_paste_inline());
        assert_eq!(plan.fallback_reason(), "needs_accessibility");
    }

    /// A permission prompt is only the right next step when the field could not
    /// be observed or permission is the only thing missing. A positively
    /// observed read-only field stays a read-only field.
    #[test]
    fn permission_hint_does_not_mask_a_field_that_cannot_take_text() {
        let started = target("com.apple.Preview", "generic", false);
        let probed = target("com.apple.Preview", "generic", false);
        let mut plan = DeliveryPlan::resolve(Some(&started), Some(&probed), &started);
        plan.needs_accessibility = true;
        assert_eq!(plan.fallback_reason(), "field_not_editable");
    }

    /// This is the production failure mode: without Accessibility, app
    /// identity still comes from NSWorkspace but both AX probes have no focused
    /// element. The user needs the permission action, not "target changed".
    #[test]
    fn missing_accessibility_is_actionable_when_ax_cannot_see_the_field() {
        let started = target("com.openai.chat", "generic", false);
        let empty = target("", "generic", false);
        let mut plan = DeliveryPlan::resolve(Some(&started), Some(&empty), &empty);
        plan.needs_accessibility = true;

        assert!(!plan.can_paste_inline());
        assert_eq!(plan.fallback_reason(), "needs_accessibility");
    }

    #[test]
    fn slot_times_out_instead_of_blocking_delivery_forever() {
        let slot = TargetSlot::default();
        assert!(slot.wait(Duration::from_millis(10)).is_none());
    }

    #[test]
    fn slot_hands_back_a_late_probe() {
        let slot = Arc::new(TargetSlot::default());
        let writer = Arc::clone(&slot);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            writer.publish(target("org.destroy.test", "generic", true));
        });
        let resolved = slot.wait(Duration::from_secs(2));
        assert_eq!(resolved.map(|target| target.can_paste), Some(true));
    }
}
