#[allow(unsafe_code)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[cfg(target_os = "macos")]
#[must_use]
pub fn camera_notch_size(window: &tauri::WebviewWindow) -> Option<(u32, u32)> {
    use objc2_app_kit::NSWindow;

    let ptr = window.ns_window().ok()?;
    if ptr.is_null() {
        return None;
    }

    // SAFETY: Tauri owns this live NSWindow. Callers resolve the geometry while
    // showing or collapsing the window on Tauri's application event loop.
    let ns_window = unsafe { &*ptr.cast::<NSWindow>() };
    let screen = ns_window.screen()?;
    let left = screen.auxiliaryTopLeftArea();
    let right = screen.auxiliaryTopRightArea();
    if left.size.width <= 0.0 || right.size.width <= 0.0 {
        return None;
    }

    let camera_left = left.origin.x + left.size.width;
    let width = right.origin.x - camera_left;
    let height = left.size.height.max(right.size.height);

    // Reject zero/implausible rectangles (including non-notched external
    // displays) rather than letting a bad AppKit reading swallow the hit area.
    if !(120.0..=360.0).contains(&width) || !(20.0..=64.0).contains(&height) {
        return None;
    }

    Some((width.round() as u32, height.round() as u32))
}

#[cfg(not(target_os = "macos"))]
#[must_use]
pub fn camera_notch_size(_window: &tauri::WebviewWindow) -> Option<(u32, u32)> {
    None
}

#[tauri::command]
pub fn notch_geometry(app: tauri::AppHandle) -> serde_json::Value {
    use tauri::Manager;
    let size = app
        .get_webview_window("dictation-hud")
        .and_then(|w| camera_notch_size(&w));
    let (width, height) = size.unwrap_or((210, 34));
    serde_json::json!({"width":width,"height":height})
}

#[cfg(target_os = "macos")]
pub fn remember_panel(window: &tauri::WebviewWindow) {
    use tauri::Manager;
    if window.label() != "main" || window.is_maximized().unwrap_or(false) {
        return;
    }
    let Ok(ptr) = window.ns_window() else { return };
    if ptr.is_null() {
        return;
    }
    // SAFETY: called from native window events or the synchronous UI command.
    let frame = unsafe { &*ptr.cast::<objc2_app_kit::NSWindow>() }.frame();
    window
        .state::<crate::panel_bounds::PanelBounds>()
        .remember(crate::panel_bounds::Bounds {
            x: frame.origin.x,
            y: frame.origin.y,
            width: frame.size.width,
            height: frame.size.height,
        });
}
#[cfg(not(target_os = "macos"))]
pub fn remember_panel(_window: &tauri::WebviewWindow) {}

#[cfg(target_os = "macos")]
pub fn position_settings(window: &tauri::WebviewWindow) -> Result<(), String> {
    use crate::panel_bounds::{restore, Bounds, PanelBounds};
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSScreen, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    use tauri::Manager;
    if window.label() != "main" {
        return Err("Settings window required".into());
    }
    let mtm = MainThreadMarker::new().ok_or("Panel must be positioned on the UI thread")?;
    let ptr = window.ns_window().map_err(|_| "Native panel unavailable")?;
    if ptr.is_null() {
        return Err("Native panel unavailable".into());
    }
    // SAFETY: the main-thread marker and Tauri guarantee a live UI-owned window.
    let native = unsafe { &*ptr.cast::<NSWindow>() };
    let state = window.state::<PanelBounds>();
    remember_panel(window);
    let screens = NSScreen::screens(mtm);
    let mouse = NSEvent::mouseLocation();
    let mut preferred = 0;
    let mut areas = Vec::new();
    for (i, screen) in screens.iter().enumerate() {
        let frame = screen.visibleFrame();
        if mouse.x >= frame.origin.x
            && mouse.x <= frame.origin.x + frame.size.width
            && mouse.y >= frame.origin.y
            && mouse.y <= frame.origin.y + frame.size.height
        {
            preferred = i;
        }
        areas.push(Bounds {
            x: frame.origin.x,
            y: frame.origin.y,
            width: frame.size.width,
            height: frame.size.height,
        });
    }
    let saved = state
        .0
        .lock()
        .map_err(|_| "Panel state unavailable")?
        .bounds;
    let target = restore(saved, &areas, preferred).ok_or("Display unavailable")?;
    // An existing, reachable settings frame is never changed by dictation or results.
    if !window.is_maximized().unwrap_or(false)
        && (!state
            .0
            .lock()
            .map_err(|_| "Panel state unavailable")?
            .expanded
            || saved != Some(target))
    {
        native.setFrame_display(
            NSRect::new(
                NSPoint::new(target.x, target.y),
                NSSize::new(target.width, target.height),
            ),
            true,
        );
    }
    state
        .0
        .lock()
        .map_err(|_| "Panel state unavailable")?
        .expanded = true;
    remember_panel(window);
    state.flush();
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn position_settings(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn position_hud(
    window: &tauri::WebviewWindow,
    phase: &str,
    choose_display: bool,
) -> Result<(u32, u32), String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSEvent, NSMainMenuWindowLevel, NSScreen, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    if window.label() != "dictation-hud" {
        return Err("HUD window required".into());
    }
    let mtm = MainThreadMarker::new().ok_or("HUD must be positioned on the UI thread")?;
    let ptr = window.ns_window().map_err(|_| "Native HUD unavailable")?;
    if ptr.is_null() {
        return Err("Native HUD unavailable".into());
    }
    // SAFETY: this is Tauri's live window, accessed only on the main thread.
    let native = unsafe { &*ptr.cast::<NSWindow>() };
    let screens = NSScreen::screens(mtm);
    let mouse = NSEvent::mouseLocation();
    let screen = if choose_display {
        screens.iter().find(|s| {
            let f = s.frame();
            mouse.x >= f.origin.x
                && mouse.x <= f.origin.x + f.size.width
                && mouse.y >= f.origin.y
                && mouse.y <= f.origin.y + f.size.height
        })
    } else {
        native.screen()
    }
    .or_else(|| NSScreen::mainScreen(mtm))
    .ok_or("Display unavailable")?;
    let frame = screen.frame();
    // Move to this display before measuring its camera gutter.
    if choose_display {
        native.setFrameOrigin(NSPoint::new(
            frame.origin.x + frame.size.width / 2.0,
            frame.origin.y + frame.size.height - 100.0,
        ));
    }
    let (camera_width, camera_height) = camera_notch_size(window).unwrap_or((210, 34));
    let recording = matches!(phase, "recording" | "processing");
    let (width, height) = if recording {
        (f64::from(camera_width) + 128.0, f64::from(camera_height))
    } else if phase == "picker" {
        (540.0, 440.0)
    } else {
        (420.0, 152.0)
    };
    native.setLevel(NSMainMenuWindowLevel + 2);
    native.setFrame_display(
        NSRect::new(
            NSPoint::new(
                frame.origin.x + (frame.size.width - width) / 2.0,
                frame.origin.y + frame.size.height - height,
            ),
            NSSize::new(width, height),
        ),
        true,
    );
    Ok((camera_width, camera_height))
}
#[cfg(not(target_os = "macos"))]
pub fn position_hud(
    window: &tauri::WebviewWindow,
    _phase: &str,
    _choose_display: bool,
) -> Result<(u32, u32), String> {
    window
        .set_size(tauri::LogicalSize::new(540.0, 440.0))
        .map_err(|_| "Cannot size HUD")?;
    Ok((210, 34))
}

#[cfg(target_os = "macos")]
pub fn show_without_focus(window: &tauri::WebviewWindow) -> Result<(), String> {
    let _mtm = objc2::MainThreadMarker::new().ok_or("Window must be shown on the UI thread")?;
    let ptr = window
        .ns_window()
        .map_err(|_| "Native window unavailable")?;
    if ptr.is_null() {
        return Err("Native window unavailable".into());
    }
    // SAFETY: main-thread-only access to Tauri's live NSWindow. Never make key or activate the app.
    unsafe { &*ptr.cast::<objc2_app_kit::NSWindow>() }.orderFrontRegardless();
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn show_without_focus(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|_| "Cannot show window".into())
}
