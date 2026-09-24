use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ActError {
    #[error("{0}")]
    InvalidParams(String),
    #[error("{0}")]
    Exec(String),
}

use serde_json::Value;

// Rich browser editors consume the pasteboard asynchronously. Restoring at
// 150ms races Gmail, Notion, Slack, and similar contenteditable surfaces and
// can paste the user's previous clipboard instead of the dictated text.
const CLIPBOARD_RESTORE_DELAY_MS: u64 = 450;
/// A pasteboard can expose lazy file/image providers of arbitrary size. Keep
/// the preservation snapshot bounded so dictation cannot duplicate an
/// unbounded clipboard payload into Destroy's process memory.
const MAX_CLIPBOARD_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;
const KEY_Z: u16 = 0x06;
const KEY_V: u16 = 0x09;

pub fn type_text(params: &Value) -> Result<String, ActError> {
    let text = params
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| ActError::InvalidParams("type_text requires 'text' param".into()))?;

    paste_preserving_clipboard(text)?;
    Ok(format!("Typed {} chars into focused field", text.len()))
}

pub fn paste(params: &Value) -> Result<String, ActError> {
    if let Some(text) = params.get("text").and_then(Value::as_str) {
        paste_text(text)?;
        Ok(format!("Pasted {} chars", text.len()))
    } else {
        paste_from_clipboard()?;
        Ok("Pasted from clipboard".into())
    }
}

/// Sends the standard application-local undo command. Callers must first
/// verify that the intended application is still frontmost; this helper does
/// not activate or choose a target on its own.
pub fn undo_last_edit() -> Result<(), ActError> {
    post_command_shortcut(KEY_Z)
}

/// Places a single local file on the macOS clipboard and pastes it into the
/// focused app. Messaging and mail composers generally interpret this as an
/// attachment. The caller owns file lifetime and target validation.
pub fn paste_file(path: &Path) -> Result<(), ActError> {
    paste_prepared_preserving_clipboard(|| write_file_to_clipboard(path))
}

/// Sets clipboard to `text`, synthesizes ⌘V, then restores the prior clipboard.
///
/// Returns [`ActError`] when paste or clipboard restore fails (caller should fall back to notch Copy).
pub fn paste_preserving_clipboard(text: &str) -> Result<(), ActError> {
    paste_prepared_preserving_clipboard(|| restore_plain_text(text.as_bytes()))
}

/// Snapshots every pasteboard item/type, lets the caller prepare a new
/// pasteboard payload, synthesizes Command-V, then restores the complete prior
/// pasteboard. A restore failure is logged but does not report paste failure —
/// the keystroke already succeeded and a fallback paste would duplicate data.
pub fn paste_prepared_preserving_clipboard<F>(prepare: F) -> Result<(), ActError>
where
    F: FnOnce() -> Result<(), ActError>,
{
    // Do not overwrite a clipboard update that happened while the rich
    // snapshot was being read. This is the first point at which the old code
    // could silently clobber a user's newer clipboard contents.
    let prior_change_count = clipboard_change_count();
    let prior = read_clipboard()?;
    if clipboard_change_count() != prior_change_count {
        return Err(ActError::Exec(
            "clipboard changed before inline paste could start".into(),
        ));
    }
    if let Err(error) = prepare() {
        restore_clipboard_if_unchanged(&prior, prior_change_count);
        return Err(error);
    }
    let prepared_change_count = clipboard_change_count();
    if let Err(error) = paste_from_clipboard() {
        restore_clipboard_if_unchanged(&prior, prepared_change_count);
        return Err(error);
    }
    thread::sleep(Duration::from_millis(CLIPBOARD_RESTORE_DELAY_MS));
    if clipboard_change_count() != prepared_change_count {
        return Ok(());
    }
    if let Err(error) = restore_clipboard(&prior) {
        tracing::warn!(%error, "paste succeeded but prior clipboard could not be restored");
    }
    Ok(())
}

/// Returns the pasteboard generation when macOS exposes one. A missing value
/// on non-macOS keeps the helper testable while preserving the existing
/// platform fallback behavior.
fn clipboard_change_count() -> Option<isize> {
    #[cfg(target_os = "macos")]
    {
        Some(objc2_app_kit::NSPasteboard::generalPasteboard().changeCount())
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn restore_clipboard_if_unchanged(snapshot: &ClipboardSnapshot, expected: Option<isize>) {
    if should_restore_clipboard(expected, clipboard_change_count()) {
        let _ = restore_clipboard(snapshot);
    } else {
        tracing::debug!("clipboard changed; preserving the newer user contents");
    }
}

fn should_restore_clipboard(expected: Option<isize>, current: Option<isize>) -> bool {
    expected == current
}

#[cfg(test)]
mod tests {
    use super::should_restore_clipboard;

    #[test]
    fn restore_is_skipped_after_a_new_clipboard_generation() {
        assert!(should_restore_clipboard(Some(17), Some(17)));
        assert!(!should_restore_clipboard(Some(17), Some(18)));
    }

    #[test]
    fn non_macos_fallback_has_no_generation_to_conflict() {
        assert!(should_restore_clipboard(None, None));
    }
}

/// Leaves `text` on the clipboard for a user-controlled paste.
///
/// Dictation uses this when the original focused field is no longer available.
/// Unlike [`paste_preserving_clipboard`], this intentionally replaces the prior
/// clipboard because the recovery contract is "copied — press Command-V".
pub fn copy_to_clipboard(text: &str) -> Result<(), ActError> {
    restore_plain_text(text.as_bytes())
}

fn paste_text(text: &str) -> Result<(), ActError> {
    // Feed pbcopy over stdin so Unicode, paragraphs, quotes, and backslashes
    // reach the target byte-for-byte. Interpolating dictation into AppleScript
    // previously changed newlines and made delivery fragile.
    restore_plain_text(text.as_bytes())?;
    paste_from_clipboard()
}

fn paste_from_clipboard() -> Result<(), ActError> {
    post_command_shortcut(KEY_V)
}

/// Posts a shortcut from the signed Destroy process itself. Using System Events via
/// `osascript` made macOS evaluate a second UI-scripting client and could show
/// privacy prompts even though Destroy was already enabled under Accessibility.
#[cfg(target_os = "macos")]
fn post_command_shortcut(keycode: u16) -> Result<(), ActError> {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|()| ActError::Exec("could not create keyboard event source".into()))?;
    let down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
        .map_err(|()| ActError::Exec("could not create key-down event".into()))?;
    let up = CGEvent::new_keyboard_event(source, keycode, false)
        .map_err(|()| ActError::Exec("could not create key-up event".into()))?;
    down.set_flags(CGEventFlags::CGEventFlagCommand);
    up.set_flags(CGEventFlags::CGEventFlagCommand);
    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn post_command_shortcut(_keycode: u16) -> Result<(), ActError> {
    Err(ActError::Exec(
        "keyboard shortcuts are only available on macOS".into(),
    ))
}

#[cfg(target_os = "macos")]
fn write_file_to_clipboard(path: &Path) -> Result<(), ActError> {
    use objc2::runtime::ProtocolObject;
    use objc2_app_kit::{NSPasteboard, NSPasteboardWriting};
    use objc2_foundation::{NSArray, NSString, NSURL};

    let path = path
        .to_str()
        .ok_or_else(|| ActError::InvalidParams("attachment path is not valid UTF-8".into()))?;
    let path = NSString::from_str(path);
    let url = NSURL::fileURLWithPath(&path);
    let object: &ProtocolObject<dyn NSPasteboardWriting> = ProtocolObject::from_ref(&*url);
    let objects = NSArray::from_slice(&[object]);
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    if pasteboard.writeObjects(&objects) {
        Ok(())
    } else {
        Err(ActError::Exec(
            "could not place attachment on clipboard".into(),
        ))
    }
}

#[cfg(not(target_os = "macos"))]
fn write_file_to_clipboard(_path: &Path) -> Result<(), ActError> {
    Err(ActError::Exec(
        "file clipboard is only available on macOS".into(),
    ))
}

#[cfg(target_os = "macos")]
struct ClipboardSnapshot {
    items: Vec<objc2::rc::Retained<objc2_app_kit::NSPasteboardItem>>,
}

#[cfg(not(target_os = "macos"))]
type ClipboardSnapshot = Vec<u8>;

#[cfg(target_os = "macos")]
fn read_clipboard() -> Result<ClipboardSnapshot, ActError> {
    use objc2_app_kit::{NSPasteboard, NSPasteboardItem};

    let pasteboard = NSPasteboard::generalPasteboard();
    let mut copies = Vec::new();
    let mut total_bytes = 0usize;
    if let Some(items) = pasteboard.pasteboardItems() {
        for source in &items {
            let copy = NSPasteboardItem::new();
            let data_types = source.types();
            for data_type in &data_types {
                if let Some(data) = source.dataForType(&data_type) {
                    total_bytes = total_bytes.saturating_add(data.length());
                    if total_bytes > MAX_CLIPBOARD_SNAPSHOT_BYTES {
                        return Err(ActError::Exec(
                            "clipboard is too large to preserve for inline paste".into(),
                        ));
                    }
                    let _ = copy.setData_forType(&data, &data_type);
                }
            }
            copies.push(copy);
        }
    }
    Ok(ClipboardSnapshot { items: copies })
}

#[cfg(not(target_os = "macos"))]
fn read_clipboard() -> Result<Vec<u8>, ActError> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|error| ActError::Exec(format!("pbpaste: {error}")))?;
    if !output.status.success() {
        return Err(ActError::Exec("pbpaste failed".into()));
    }
    Ok(output.stdout)
}

#[cfg(target_os = "macos")]
fn restore_clipboard(snapshot: &ClipboardSnapshot) -> Result<(), ActError> {
    use objc2::runtime::ProtocolObject;
    use objc2_app_kit::{NSPasteboard, NSPasteboardWriting};
    use objc2_foundation::NSArray;

    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.clearContents();
    if snapshot.items.is_empty() {
        return Ok(());
    }
    let objects: Vec<&ProtocolObject<dyn NSPasteboardWriting>> = snapshot
        .items
        .iter()
        .map(|item| ProtocolObject::from_ref(&**item))
        .collect();
    let array = NSArray::from_slice(&objects);
    if pasteboard.writeObjects(&array) {
        Ok(())
    } else {
        Err(ActError::Exec("restore rich clipboard failed".into()))
    }
}

#[cfg(not(target_os = "macos"))]
fn restore_clipboard(bytes: &[u8]) -> Result<(), ActError> {
    restore_plain_text(bytes)
}

fn restore_plain_text(bytes: &[u8]) -> Result<(), ActError> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| ActError::Exec(format!("pbcopy: {error}")))?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(bytes)
            .map_err(|error| ActError::Exec(format!("pbcopy write: {error}")))?;
    }
    let status = child
        .wait()
        .map_err(|error| ActError::Exec(format!("pbcopy wait: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(ActError::Exec("pbcopy failed".into()))
    }
}
