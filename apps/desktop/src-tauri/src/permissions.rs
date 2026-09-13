use tauri::AppHandle;
#[must_use]
pub fn microphone_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        av_microphone_status() == AvAuthStatus::Authorized
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AvAuthStatus {
    NotDetermined,
    Restricted,
    Denied,
    Authorized,
    Unknown,
}

/// In-process `AVCaptureDevice.authorizationStatus(for: .audio)`.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn av_microphone_status() -> AvAuthStatus {
    // Ensure AVFoundation is linked so `AVCaptureDevice` resolves at runtime.
    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {}

    use objc2::{class, msg_send};
    use objc2_foundation::NSString;

    // AVMediaTypeAudio == "soun" (AVMediaFormat.h).
    let media_type = NSString::from_str("soun");
    // SAFETY: Class method on AVCaptureDevice; media type is the documented audio constant.
    let status: isize = unsafe {
        msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: &*media_type
        ]
    };
    match status {
        0 => AvAuthStatus::NotDetermined,
        1 => AvAuthStatus::Restricted,
        2 => AvAuthStatus::Denied,
        3 => AvAuthStatus::Authorized,
        _ => AvAuthStatus::Unknown,
    }
}

/// Prompts for Microphone access (or returns the existing decision).
///
/// Calling this is what registers Destroy in System Settings → Privacy → Microphone.
/// Opening the Settings pane alone never adds the app to that list.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn request_microphone_access(app: AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        request_microphone_access_macos(&app).await
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Microphone permission is only available on macOS".into())
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
async fn request_microphone_access_macos(app: &AppHandle) -> Result<bool, String> {
    match av_microphone_status() {
        AvAuthStatus::Authorized => return Ok(true),
        AvAuthStatus::Denied | AvAuthStatus::Restricted => return Ok(false),
        AvAuthStatus::NotDetermined | AvAuthStatus::Unknown => {}
    }

    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {}

    use std::time::Duration;

    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;

    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    app.run_on_main_thread(move || {
        // ObjC BOOL completion handlers must use `Bool` (EncodeArgument); Rust `bool` is not.
        let block = RcBlock::new(move |granted: Bool| {
            let _ = tx.send(granted.as_bool());
        });
        let media_type = NSString::from_str("soun");
        // SAFETY: Documented AVCaptureDevice API; handler is invoked asynchronously.
        unsafe {
            let _: () = msg_send![
                class!(AVCaptureDevice),
                requestAccessForMediaType: &*media_type,
                completionHandler: &*block
            ];
        }
    })
    .map_err(|error| format!("Could not request Microphone permission: {error}"))?;

    // Never block the macOS main thread while its permission sheet is presented.
    tauri::async_runtime::spawn_blocking(move || rx.recv_timeout(Duration::from_secs(120)))
        .await
        .map_err(|error| format!("Microphone permission request failed: {error}"))?
        .map_err(|_| "Microphone permission prompt timed out".into())
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn ax_is_process_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    // SAFETY: AXIsProcessTrusted is a read-only macOS API with no preconditions.
    unsafe { AXIsProcessTrusted() }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn ax_request_process_trust() -> bool {
    use std::ffi::c_void;

    use objc2_core_foundation::{
        kCFAllocatorDefault, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
        CFBoolean, CFDictionary,
    };

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: *const c_void;
    }

    let mut keys = [unsafe { kAXTrustedCheckOptionPrompt }];
    let mut values = [CFBoolean::new(true) as *const CFBoolean as *const c_void];
    // SAFETY: The dictionary contains one valid CFString key supplied by
    // ApplicationServices and one retained global CFBoolean value.
    let options = unsafe {
        CFDictionary::new(
            kCFAllocatorDefault,
            keys.as_mut_ptr(),
            values.as_mut_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        )
    };
    let Some(options) = options else {
        return false;
    };

    // SAFETY: `options` is a valid CFDictionary containing the documented
    // kAXTrustedCheckOptionPrompt option.
    unsafe { AXIsProcessTrustedWithOptions(&*options as *const CFDictionary as *const c_void) }
}

pub fn accessibility_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        ax_is_process_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
#[tauri::command]
pub fn check_accessibility_access() -> bool {
    accessibility_granted()
}
#[tauri::command]
pub fn request_accessibility_access() -> bool {
    #[cfg(target_os = "macos")]
    {
        ax_request_process_trust()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
#[tauri::command]
pub fn get_permission_status() -> serde_json::Value {
    serde_json::json!({"microphone":microphone_granted(),"accessibility":accessibility_granted()})
}
