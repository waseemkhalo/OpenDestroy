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
    camera_notch_size_for_screen(&screen)
}

#[cfg(target_os = "macos")]
#[must_use]
fn camera_notch_size_for_screen(screen: &objc2_app_kit::NSScreen) -> Option<(u32, u32)> {
    let left = screen.auxiliaryTopLeftArea();
    let right = screen.auxiliaryTopRightArea();
    notch_geometry_from_auxiliary_areas(
        left.origin.x,
        left.size.width,
        left.size.height,
        right.origin.x,
        right.size.width,
        right.size.height,
    )
}

/// Converts AppKit's safe menu-bar rectangles into the physical camera gap.
/// Kept independent of AppKit so malformed/overlapping display data is tested
/// without requiring a real notched Mac in CI.
#[must_use]
fn notch_geometry_from_auxiliary_areas(
    left_origin_x: f64,
    left_width: f64,
    left_height: f64,
    right_origin_x: f64,
    right_width: f64,
    right_height: f64,
) -> Option<(u32, u32)> {
    if [
        left_origin_x,
        left_width,
        left_height,
        right_origin_x,
        right_width,
        right_height,
    ]
    .iter()
    .any(|value| !value.is_finite())
        || left_width <= 0.0
        || right_width <= 0.0
    {
        return None;
    }

    let width = right_origin_x - (left_origin_x + left_width);
    let height = left_height.max(right_height);

    // Reject zero/implausible rectangles (including non-notched external
    // displays) rather than letting bad AppKit data swallow the hit area.
    if !(120.0..=360.0).contains(&width) || !(20.0..=64.0).contains(&height) {
        return None;
    }

    Some((width.round() as u32, height.round() as u32))
}

const CAMERA_FALLBACK_WIDTH: u32 = 200;
const CAMERA_FALLBACK_HEIGHT: u32 = 34;
const RECORDING_FLANK_WIDTH: u32 = 128;
const RECORDING_HUD_HEIGHT: u32 = 46;

#[must_use]
const fn recording_hud_width(camera_width: u32) -> u32 {
    camera_width.saturating_add(RECORDING_FLANK_WIDTH * 2)
}

#[must_use]
const fn recording_hud_height(camera_height: u32) -> u32 {
    if camera_height > RECORDING_HUD_HEIGHT {
        camera_height
    } else {
        RECORDING_HUD_HEIGHT
    }
}

#[must_use]
const fn recording_warning_hud_height(camera_height: u32) -> u32 {
    recording_hud_height(camera_height).saturating_add(84)
}

#[must_use]
fn recording_warning_hud_origin_y(screen_bottom: f64, camera_height: u32) -> f64 {
    screen_bottom - f64::from(recording_hud_height(camera_height))
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
    let (width, height) = size.unwrap_or((CAMERA_FALLBACK_WIDTH, CAMERA_FALLBACK_HEIGHT));
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
    // Resolve the selected display directly. Measuring through the window here
    // can read the previous display while a multi-monitor HUD is moving.
    let (camera_width, camera_height) = camera_notch_size_for_screen(&screen)
        .unwrap_or((CAMERA_FALLBACK_WIDTH, CAMERA_FALLBACK_HEIGHT));
    let recording = matches!(phase, "recording" | "recording-warning" | "processing");
    let (width, height) = if phase == "recording-warning" {
        (
            f64::from(recording_hud_width(camera_width)),
            f64::from(recording_warning_hud_height(camera_height)),
        )
    } else if recording {
        (
            f64::from(recording_hud_width(camera_width)),
            f64::from(recording_hud_height(camera_height)),
        )
    } else if phase == "picker" {
        (540.0, 440.0)
    } else {
        // Room for a readable recovery message, paste shortcut and actions
        // below the camera gutter. Recording/processing keep their slim notch.
        (440.0, 240.0)
    };
    let origin_y = if phase == "recording-warning" {
        recording_warning_hud_origin_y(frame.origin.y + frame.size.height, camera_height)
    } else {
        frame.origin.y + frame.size.height - height
    };
    native.setLevel(NSMainMenuWindowLevel + 2);
    native.setFrame_display(
        NSRect::new(
            NSPoint::new(frame.origin.x + (frame.size.width - width) / 2.0, origin_y),
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
    Ok((CAMERA_FALLBACK_WIDTH, CAMERA_FALLBACK_HEIGHT))
}

#[cfg(test)]
mod tests {
    use super::{
        notch_geometry_from_auxiliary_areas, recording_hud_height, recording_hud_width,
        recording_warning_hud_height, recording_warning_hud_origin_y, CAMERA_FALLBACK_HEIGHT,
        CAMERA_FALLBACK_WIDTH, RECORDING_FLANK_WIDTH, RECORDING_HUD_HEIGHT,
    };

    #[test]
    fn derives_camera_gap_from_display_safe_areas() {
        assert_eq!(
            notch_geometry_from_auxiliary_areas(0.0, 500.0, 34.0, 700.0, 580.0, 32.0),
            Some((200, 34))
        );
    }

    #[test]
    fn rejects_invalid_or_non_notched_safe_areas() {
        assert_eq!(
            notch_geometry_from_auxiliary_areas(0.0, 500.0, 34.0, 500.0, 580.0, 32.0),
            None
        );
        assert_eq!(
            notch_geometry_from_auxiliary_areas(0.0, 500.0, 34.0, 100.0, 580.0, 32.0),
            None
        );
        assert_eq!(
            notch_geometry_from_auxiliary_areas(0.0, 500.0, f64::NAN, 700.0, 580.0, 32.0),
            None
        );
    }

    #[test]
    fn recording_footprint_keeps_equal_source_derived_flanks() {
        assert_eq!(RECORDING_FLANK_WIDTH, 128);
        assert_eq!(recording_hud_width(CAMERA_FALLBACK_WIDTH), 456);
        assert_eq!(
            recording_hud_height(CAMERA_FALLBACK_HEIGHT),
            RECORDING_HUD_HEIGHT
        );
        assert_eq!(CAMERA_FALLBACK_HEIGHT, 34);
    }

    #[test]
    fn recording_warning_footprint_has_room_for_readable_copy() {
        assert_eq!(recording_warning_hud_height(CAMERA_FALLBACK_HEIGHT), 130);
        assert!(recording_warning_hud_height(CAMERA_FALLBACK_HEIGHT) > RECORDING_HUD_HEIGHT);
        assert_eq!(
            recording_warning_hud_origin_y(1000.0, CAMERA_FALLBACK_HEIGHT),
            954.0
        );
    }
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
