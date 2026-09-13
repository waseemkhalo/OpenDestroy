//! Explicit, development-only snapshot of this app's own WebView.
//! This uses no screen capture API and exposes no production capture command.
use tauri::Manager;
pub fn schedule(app: &tauri::AppHandle) {
    let Ok(path) = std::env::var("DESTROY_DEV_SNAPSHOT") else {
        return;
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        let _ = window.with_webview(move |webview| {
            use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSImage};
            use objc2_foundation::{NSDictionary, NSError};
            // SAFETY: with_webview supplies this live WKWebView on the UI thread.
            let view = unsafe { &*(webview.inner() as *const objc2_web_kit::WKWebView) };
            let callback = block2::RcBlock::new(move |image: *mut NSImage, error: *mut NSError| {
                if !error.is_null() || image.is_null() {
                    return;
                }
                let image = unsafe { &*image };
                let Some(tiff) = image.TIFFRepresentation() else {
                    return;
                };
                let Some(bitmap) = NSBitmapImageRep::imageRepWithData(&tiff) else {
                    return;
                };
                let Some(png) = (unsafe {
                    bitmap.representationUsingType_properties(
                        NSBitmapImageFileType::PNG,
                        &NSDictionary::new(),
                    )
                }) else {
                    return;
                };
                let _ = std::fs::write(&path, png.to_vec());
            });
            unsafe {
                view.takeSnapshotWithConfiguration_completionHandler(None, &callback);
            }
        });
    });
}
