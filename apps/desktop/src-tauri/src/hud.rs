//! A separate, non-key presentation window. Sessions and account state stay in main.
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub struct HudState {
    snapshot: Mutex<Option<Snapshot>>,
    escape_owned: AtomicBool,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Snapshot {
    revision: u64,
    visible: bool,
    phase: String,
    #[serde(flatten)]
    content: serde_json::Map<String, serde_json::Value>,
}
impl Snapshot {
    fn clear_hidden(&mut self) {
        if !self.visible {
            self.phase = "idle".into();
            self.content.clear();
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionKind {
    Cancel,
    Open,
    Copy,
    Accessibility,
    Choose,
    Favorite,
    Rotate,
    Kind,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Action {
    revision: u64,
    action: ActionKind,
    index: Option<u8>,
    kind: Option<String>,
}

pub fn allowed_command(window: &str, command: &str) -> bool {
    window == "main"
        || (window == "dictation-hud" && matches!(command, "hud_snapshot" | "hud_action"))
}

#[tauri::command]
pub fn hud_snapshot(state: tauri::State<HudState>) -> Option<Snapshot> {
    state.snapshot.lock().ok()?.clone()
}
#[tauri::command]
pub fn hud_action(
    app: tauri::AppHandle,
    state: tauri::State<HudState>,
    action: Action,
) -> Result<(), String> {
    let current = state.snapshot.lock().map_err(|_| "HUD unavailable")?;
    if !current
        .as_ref()
        .is_some_and(|s| s.visible && s.revision == action.revision)
    {
        return Ok(());
    }
    if action.index.is_some_and(|i| i > 2)
        || action
            .kind
            .as_ref()
            .is_some_and(|s| s != "gif" && s != "sticker")
    {
        return Err("Invalid HUD action".into());
    }
    app.emit_to("main", "destroy://hud-action", action)
        .map_err(|_| "Cannot send HUD action".into())
}

#[tauri::command]
pub fn update_hud(
    app: tauri::AppHandle,
    state: tauri::State<HudState>,
    mut snapshot: Snapshot,
) -> Result<(), String> {
    let mut current = state.snapshot.lock().map_err(|_| "HUD unavailable")?;
    if current
        .as_ref()
        .is_some_and(|s| s.revision >= snapshot.revision)
    {
        return Ok(());
    }
    let was_visible = current.as_ref().is_some_and(|s| s.visible);
    snapshot.clear_hidden();
    if !snapshot.visible {
        // Release the previous payload even if hiding or notifying the webview fails.
        *current = Some(snapshot.clone());
    }
    let window = app
        .get_webview_window("dictation-hud")
        .ok_or("HUD unavailable")?;
    if snapshot.visible {
        let (width, height) =
            crate::geometry::position_hud(&window, &snapshot.phase, !was_visible)?;
        snapshot.content.insert(
            "camera".into(),
            serde_json::json!({"width":width,"height":height}),
        );
    } else {
        window.hide().map_err(|_| "Cannot hide HUD")?;
    }
    app.emit_to("dictation-hud", "destroy://hud", &snapshot)
        .map_err(|_| "Cannot update HUD")?;
    if snapshot.visible {
        crate::geometry::show_without_focus(&window)?;
    }
    let mut shortcut_error = None;
    if snapshot.visible != was_visible {
        let escape: Shortcut = "Escape".parse().map_err(|_| "Escape unavailable")?;
        if snapshot.visible {
            // Never replace a user-configured shortcut, including Escape itself.
            if !app.global_shortcut().is_registered(escape) {
                match app.global_shortcut().on_shortcut(escape, |app, _, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app.emit_to("main", "destroy://dictation-cancel", ());
                    }
                }) {
                    Ok(()) => state.escape_owned.store(true, Ordering::Relaxed),
                    Err(_) => {
                        shortcut_error =
                            Some("Escape is unavailable; click the dictation indicator to cancel")
                    }
                }
            }
        } else if state.escape_owned.swap(false, Ordering::Relaxed) {
            let _ = app.global_shortcut().unregister(escape);
        }
    }
    *current = Some(snapshot);
    shortcut_error.map_or(Ok(()), |error| Err(error.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hidden_snapshot_releases_prior_session_payload_and_keeps_revision() {
        let mut snapshot: Snapshot = serde_json::from_value(serde_json::json!({
            "revision": 42, "visible": false, "phase": "picker",
            "target": {"appName": "Private destination"},
            "message": "Prior delivery", "error": "Prior provider error",
            "leading": "Prior recognized text", "links": [{"url": "https://private.example"}]
        }))
        .unwrap();
        snapshot.clear_hidden();
        assert_eq!(
            serde_json::to_value(snapshot).unwrap(),
            serde_json::json!({
                "revision": 42, "visible": false, "phase": "idle"
            })
        );
    }
    #[test]
    fn hud_cannot_call_account_capture_or_clipboard_commands() {
        for command in [
            "configure_backend",
            "backend_api",
            "destroy_dictation_begin",
            "destroy_dictation_deliver",
            "destroy_dictation_copy",
            "update_hud",
            "quit_app",
        ] {
            assert!(!allowed_command("dictation-hud", command));
            assert!(allowed_command("main", command));
        }
        assert!(allowed_command("dictation-hud", "hud_snapshot"));
        assert!(allowed_command("dictation-hud", "hud_action"));
        assert!(!allowed_command("other", "hud_action"));
    }
}
