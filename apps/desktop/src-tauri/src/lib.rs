mod app_identity;
mod auth;
mod clipboard;
#[cfg(all(debug_assertions, target_os = "macos"))]
mod dev_snapshot;
mod dictation;
mod dictation_stream;
mod focus;
mod geometry;
mod hud;
mod panel_bounds;
mod permissions;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
struct ShortcutConfig(Mutex<String>);
fn register(app: &tauri::AppHandle, value: &str) -> Result<(), String> {
    let shortcut: Shortcut = value
        .parse()
        .map_err(|_| "Invalid shortcut; try CommandOrControl+Backquote")?;
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _, event| {
            if event.state == ShortcutState::Pressed {
                // The visible picker owns its original target while another hold selects a choice.
                let state = app.state::<dictation::DictationSession>();
                let target = state.current_or_capture_target();
                let _ = app.emit("destroy://dictation-start", target);
            } else {
                let _ = app.emit("destroy://dictation-stop", ());
            }
        })
        .map_err(|_| "Shortcut is unavailable or used by another app".into())
}
#[tauri::command]
fn set_shortcut(
    app: tauri::AppHandle,
    value: String,
    state: tauri::State<ShortcutConfig>,
) -> Result<(), String> {
    let mut old = state.0.lock().map_err(|_| "Shortcut busy")?;
    if value == *old
        && value
            .parse::<Shortcut>()
            .is_ok_and(|v| app.global_shortcut().is_registered(v))
    {
        return Ok(());
    }
    register(&app, &value)?;
    if value != *old {
        if let Ok(s) = old.parse::<Shortcut>() {
            let _ = app.global_shortcut().unregister(s);
        }
    }
    *old = value.clone();
    let path = app_identity::config_path().with_file_name("shortcut.txt");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "Cannot save shortcut")?;
    }
    std::fs::write(path, value).map_err(|_| "Cannot save shortcut")?;
    Ok(())
}
#[tauri::command]
fn shortcut_status(
    app: tauri::AppHandle,
    state: tauri::State<ShortcutConfig>,
) -> Result<(), String> {
    let value = state.0.lock().map_err(|_| "Shortcut busy")?;
    if value
        .parse::<Shortcut>()
        .is_ok_and(|s| app.global_shortcut().is_registered(s))
    {
        Ok(())
    } else {
        Err(
            "Shortcut could not be registered. Choose another shortcut in Connection & device."
                .into(),
        )
    }
}
#[tauri::command]
fn get_shortcut(state: tauri::State<ShortcutConfig>) -> String {
    state.0.lock().map(|s| s.clone()).unwrap_or_default()
}
#[tauri::command]
fn show_panel(app: tauri::AppHandle, expanded: bool, focus: Option<bool>) -> Result<(), String> {
    let w = app.get_webview_window("main").ok_or("Window unavailable")?;
    if expanded {
        if focus.unwrap_or(false) && w.is_minimized().unwrap_or(false) {
            w.unminimize()
                .map_err(|_| "Cannot restore settings window")?;
        }
        geometry::position_settings(&w)?;
        geometry::show_without_focus(&w)?;
        if focus.unwrap_or(false) {
            w.set_focus().map_err(|_| "Cannot focus settings window")?;
        }
    } else {
        geometry::remember_panel(&w);
        app.state::<panel_bounds::PanelBounds>().flush();
        w.hide().map_err(|_| "Cannot hide settings window")?;
    }
    Ok(())
}
#[tauri::command]
fn updater_available(app: tauri::AppHandle) -> bool {
    app.config().plugins.0.contains_key("updater")
}
#[tauri::command]
fn notch_log(_message: String) {} // No transcript or preference logging.
#[tauri::command]
fn list_audio_input_devices() -> Vec<serde_json::Value> {
    vec![]
} // WebKit selection owns IDs; native bridge isn't used.
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.state::<panel_bounds::PanelBounds>().flush();
    app.exit(0)
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Arc::new(Mutex::new(Default::default())) as dictation_stream::DictationStreamHandle)
        .manage(dictation::DictationSession::default())
        .manage(panel_bounds::PanelBounds::default())
        .manage(hud::HudState::default())
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if matches!(
                event,
                tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_)
            ) {
                if let Some(w) = window.app_handle().get_webview_window("main") {
                    geometry::remember_panel(&w);
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
            if matches!(
                event,
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
            ) {
                window.state::<panel_bounds::PanelBounds>().flush();
            }
        })
        .manage(ShortcutConfig(Mutex::new(
            "CommandOrControl+Backquote".into(),
        )))
        .setup(|app| {
            if app.config().plugins.0.contains_key("updater") {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            if app.config().identifier != app_identity::BUNDLE_ID {
                return Err("Build identity mismatch. Use npm run tauri.".into());
            }
            let value =
                std::fs::read_to_string(app_identity::config_path().with_file_name("shortcut.txt"))
                    .unwrap_or("CommandOrControl+Backquote".into());
            let value = value.trim().to_owned();
            *app.state::<ShortcutConfig>().0.lock().unwrap() = value.clone();
            if let Err(e) = register(app.handle(), &value) {
                eprintln!("{e}")
            };
            show_panel(app.handle().clone(), true, Some(false))?;
            #[cfg(all(debug_assertions, target_os = "macos"))]
            dev_snapshot::schedule(app.handle());
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    handle.state::<panel_bounds::PanelBounds>().flush();
                    handle
                        .state::<dictation::DictationSession>()
                        .expire_sensitive_memory();
                }
            });
            Ok(())
        })
        .invoke_handler(|invoke| {
            if !hud::allowed_command(
                invoke.message.webview_ref().label(),
                invoke.message.command(),
            ) {
                invoke
                    .resolver
                    .reject("This window cannot call that command");
                return true;
            }
            let handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![
                hud::update_hud,
                hud::hud_snapshot,
                hud::hud_action,
                auth::configure_backend,
                auth::restore_connection,
                auth::disconnect_backend,
                auth::backend_api,
                auth::export_personal_data,
                auth::destroy_transcribe_dictation,
                dictation::destroy_dictation_begin,
                dictation::destroy_dictation_target,
                dictation::destroy_dictation_selected_text,
                dictation::destroy_dictation_previous_text,
                dictation::destroy_dictation_cancel,
                dictation::destroy_dictation_copy,
                dictation::destroy_dictation_clear_sensitive_state,
                dictation::destroy_dictation_undo,
                dictation::destroy_dictation_replace_last,
                dictation::destroy_dictation_deliver,
                dictation::destroy_dictation_deliver_picker_text,
                dictation::destroy_dictation_deliver_voice_note,
                dictation::destroy_dictation_deliver_media,
                dictation_stream::destroy_dictation_stream_start,
                dictation_stream::destroy_dictation_stream_send_audio,
                dictation_stream::destroy_dictation_stream_finish,
                dictation_stream::destroy_dictation_stream_cancel,
                permissions::get_permission_status,
                permissions::request_microphone_access,
                permissions::request_accessibility_access,
                permissions::check_accessibility_access,
                shortcut_status,
                geometry::notch_geometry,
                set_shortcut,
                get_shortcut,
                show_panel,
                updater_available,
                notch_log,
                list_audio_input_devices,
                quit_app
            ];
            handler(invoke)
        })
        .build(tauri::generate_context!())
        .expect("Cannot start Destroy Dictation")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if matches!(&event, tauri::RunEvent::Reopen { .. }) {
                let _ = show_panel(app.clone(), true, Some(true));
            }
        });
}
