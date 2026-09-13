//! Focus probe for draft-reply inline delivery (wd-01, wd-17).
//!
//! ## Spike outcome (Wave 1)
//!
//! Inline delivery is enabled for native `AXTextArea` / `AXTextField` with readable
//! value. Web `contenteditable` (Gmail compose) often lacks a reliable AX value —
//! those cases default to notch until AX value length > 0 (contract §5.8 / plan §9.2).
//!
//! ## Surface matrix (wd-17)
//!
//! | Surface | AX role observed | Inline when | Fallback |
//! |---------|------------------|-------------|----------|
//! | Gmail compose (Chrome) | `AXWebArea` / `AXGroup` | Non-empty AX value in compose context | Notch + Copy |
//! | Gmail thread (read) | non-editable / empty value | Never | Notch |
//! | Slack message input | `AXTextArea` or web + value | Native textarea or value present | Notch |
//! | LinkedIn DM compose | web + value in browser | Value + linkedin title heuristic | Notch |
//! | Apple Mail compose | `AXTextArea` | Always (native) | Notch if paste fails |
//! | Destroy app fields | any | Never (avoid self-paste) | Notch |
//! | Password / secure fields | `AXSecureTextField` | Never | Notch |
//!
//! **Rule:** ambiguous or empty AX value → notch (safe). Inline paste uses clipboard
//! restore per contract §5.8; failure surfaces notch fallback with Copy.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Where the draft should land after generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DraftDelivery {
    Inline,
    Notch,
}

/// Result of probing the focused UI element in the foreground app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusProbeResult {
    pub delivery: DraftDelivery,
    pub partial_draft: Option<String>,
}

/// Privacy-safe metadata for hold-to-dictate delivery. Destroy keeps this only for
/// the active key hold; it never stores the focused field value or window text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationFocusTarget {
    pub can_paste: bool,
    pub app_name: String,
    pub bundle_id: String,
    pub app_kind: String,
    /// A small, memory-only PNG of the actual foreground macOS application.
    /// It is captured from NSWorkspace at key-down and never persisted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_icon_data_url: Option<String>,
    /// Hash of transient accessibility metadata identifying the focused field.
    /// It is never serialized, logged, or persisted. Delivery, correction, and
    /// undo use it to distinguish two fields inside the same app/window.
    #[serde(skip)]
    pub focus_signature: String,
    /// Transient selection captured with the focused field. It is deliberately
    /// omitted from every serialized target and is cleared with the key hold.
    #[serde(skip)]
    pub selected_text: Option<String>,
}

const MAX_APP_ICON_PNG_BYTES: usize = 128 * 1024;

/// Returns a compact PNG of the real foreground app icon.
///
/// AppKit drawing must stay on the main thread. Hotkey capture runs there and
/// fills this bounded in-memory cache; later worker-thread probes can reuse the
/// icon without touching AppKit drawing APIs.
#[cfg(target_os = "macos")]
#[allow(deprecated, unsafe_code)]
fn foreground_app_icon_data_url(
    app: &objc2_app_kit::NSRunningApplication,
    bundle_id: &str,
) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use objc2::{runtime::AnyObject, AnyThread, MainThreadMarker};
    use objc2_app_kit::{
        NSBitmapImageFileType, NSBitmapImageRep, NSBitmapImageRepPropertyKey, NSImage, NSWorkspace,
    };
    use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize};
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static ICONS: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

    let key = bundle_id.trim();
    if key.is_empty() {
        return None;
    }
    let icons = ICONS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(cached) = icons
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(key)
        .cloned()
    {
        return cached;
    }

    // NSImage focus/drawing is main-thread-only. Returning no icon on an
    // uncached worker call is safer than moving AppKit work off the UI thread.
    let _main_thread = MainThreadMarker::new()?;
    let bundle_path = app.bundleURL()?.path()?;
    let source = NSWorkspace::sharedWorkspace().iconForFile(&bundle_path);
    let size = NSSize::new(32.0, 32.0);
    let resized = NSImage::initWithSize(NSImage::alloc(), size);
    resized.lockFocus();
    source.drawInRect(NSRect::new(NSPoint::new(0.0, 0.0), size));
    resized.unlockFocus();

    let encoded = resized.TIFFRepresentation().and_then(|tiff| {
        let bitmap = NSBitmapImageRep::imageRepWithData(&tiff)?;
        let properties = NSDictionary::<NSBitmapImageRepPropertyKey, AnyObject>::new();
        // SAFETY: an empty properties dictionary is valid for PNG encoding.
        let png = unsafe {
            bitmap.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties)?
        };
        let bytes = png.to_vec();
        (bytes.len() <= MAX_APP_ICON_PNG_BYTES)
            .then(|| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
    });

    let mut cache = icons
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if cache.len() >= 64 {
        cache.clear();
    }
    cache.insert(key.to_string(), encoded.clone());
    encoded
}

/// Probes the focused field in the foreground app per `docs/wiki-draft/00-contracts.md` §9.2.
#[must_use]
pub fn probe_focused_field() -> FocusProbeResult {
    #[cfg(target_os = "macos")]
    {
        probe_focused_field_macos().unwrap_or(FocusProbeResult {
            delivery: DraftDelivery::Notch,
            partial_draft: None,
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        FocusProbeResult {
            delivery: DraftDelivery::Notch,
            partial_draft: None,
        }
    }
}

/// The frontmost app's identity, with no accessibility round trip.
///
/// `NSWorkspace` answers immediately, while [`probe_dictation_target`] performs
/// a native accessibility round trip. Dictation needs the identity first — to
/// show the target icon and, at delivery, verify the user has not switched apps
/// — and can afford to learn `can_paste` shortly afterward.
#[must_use]
pub fn frontmost_app_identity() -> DictationFocusTarget {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWorkspace;

        let Some(app) = NSWorkspace::sharedWorkspace().frontmostApplication() else {
            return empty_dictation_target();
        };
        let bundle_id = app
            .bundleIdentifier()
            .map(|value| value.to_string())
            .unwrap_or_default();
        let app_name = app
            .localizedName()
            .map(|value| value.to_string())
            .unwrap_or_default();
        let app_icon_data_url = foreground_app_icon_data_url(&app, &bundle_id);
        let app_kind = dictation_app_kind(&FocusFields {
            bundle_id: bundle_id.clone(),
            app_name: app_name.clone(),
            window_title: String::new(),
            role: String::new(),
            subrole: String::new(),
            identifier: String::new(),
            position: String::new(),
            size: String::new(),
            editable: false,
            value: String::new(),
            selected_text: String::new(),
            value_settable: false,
        });
        DictationFocusTarget {
            // Unknown until the accessibility probe lands; never assume paste.
            can_paste: false,
            app_name,
            bundle_id,
            app_kind,
            app_icon_data_url,
            selected_text: None,
            focus_signature: String::new(),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        empty_dictation_target()
    }
}

/// Probes the current target for dictation without returning any field value.
/// Empty browser compose fields are accepted here because dictation inserts new
/// text; draft-reply keeps its stricter readable-value policy above.
///
/// Uses Destroy's native AX client, so the Accessibility grant applies to the
/// process that performs the read and no Automation subprocess can prompt.
#[must_use]
pub fn probe_dictation_target() -> DictationFocusTarget {
    #[cfg(target_os = "macos")]
    {
        read_focused_fields_macos().map_or_else(empty_dictation_target, |fields| {
            let can_paste = dictation_accepts(&fields);
            let app_kind = dictation_app_kind(&fields);
            let focus_signature = focus_signature(&fields);
            DictationFocusTarget {
                can_paste,
                app_name: fields.app_name,
                bundle_id: fields.bundle_id,
                app_kind,
                app_icon_data_url: None,
                focus_signature,
                selected_text: (can_paste && !fields.selected_text.is_empty())
                    .then_some(fields.selected_text.chars().take(16_000).collect()),
            }
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        empty_dictation_target()
    }
}

fn empty_dictation_target() -> DictationFocusTarget {
    DictationFocusTarget {
        can_paste: false,
        app_name: String::new(),
        bundle_id: String::new(),
        app_kind: "generic".into(),
        app_icon_data_url: None,
        focus_signature: String::new(),
        selected_text: None,
    }
}

#[cfg(target_os = "macos")]
fn probe_focused_field_macos() -> Option<FocusProbeResult> {
    read_focused_fields_macos().map(|fields| classify_focus_fields(&fields))
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code, clippy::too_many_lines)]
fn read_focused_fields_macos() -> Option<FocusFields> {
    use core_foundation::base::{CFHash, CFType, CFTypeRef, TCFType};
    use core_foundation::boolean::CFBoolean;
    use core_foundation::string::{CFString, CFStringRef};
    use objc2_app_kit::NSWorkspace;

    type AXUIElementRef = *const std::ffi::c_void;
    type AXError = i32;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> AXError;
        fn AXUIElementIsAttributeSettable(
            element: AXUIElementRef,
            attribute: CFStringRef,
            settable: *mut bool,
        ) -> AXError;
    }

    struct OwnedCf(std::ptr::NonNull<std::ffi::c_void>);
    impl OwnedCf {
        fn new(value: CFTypeRef) -> Option<Self> {
            std::ptr::NonNull::new(value.cast_mut()).map(Self)
        }

        fn as_type_ref(&self) -> CFTypeRef {
            self.0.as_ptr().cast_const()
        }
    }
    impl Drop for OwnedCf {
        fn drop(&mut self) {
            unsafe { core_foundation::base::CFRelease(self.as_type_ref()) };
        }
    }
    unsafe fn attribute(element: AXUIElementRef, name: CFStringRef) -> Option<OwnedCf> {
        let mut value = std::ptr::null();
        let status = AXUIElementCopyAttributeValue(element, name, &raw mut value);
        let value = OwnedCf::new(value)?;
        (status == 0).then_some(value)
    }
    unsafe fn string_attribute(element: AXUIElementRef, name: CFStringRef) -> String {
        let Some(value) = attribute(element, name) else {
            return String::new();
        };
        let wrapped = CFType::wrap_under_get_rule(value.as_type_ref());
        wrapped
            .downcast::<CFString>()
            .map(|text| text.to_string())
            .unwrap_or_default()
    }
    unsafe fn bool_attribute(element: AXUIElementRef, name: CFStringRef) -> Option<bool> {
        let value = attribute(element, name)?;
        CFType::wrap_under_get_rule(value.as_type_ref())
            .downcast::<CFBoolean>()
            .map(Into::into)
    }

    // SAFETY: every AX element comes from a successful Create/Copy call and is
    // retained by `OwnedCf` for the complete duration of each attribute read.
    unsafe {
        // kAX* attribute names are CFSTR header constants, not exported
        // ApplicationServices symbols. Use their documented string values.
        let focused_element_attr = CFString::new("AXFocusedUIElement");
        let focused_window_attr = CFString::new("AXFocusedWindow");
        let title_attr = CFString::new("AXTitle");
        let role_attr = CFString::new("AXRole");
        let subrole_attr = CFString::new("AXSubrole");
        let value_attr = CFString::new("AXValue");
        let selected_text_attr = CFString::new("AXSelectedText");
        let enabled_attr = CFString::new("AXEnabled");
        let identifier_attr = CFString::new("AXIdentifier");

        let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
        let pid = app.processIdentifier();
        let bundle_id = app
            .bundleIdentifier()
            .map(|value| value.to_string())
            .unwrap_or_default();
        let app_name = app
            .localizedName()
            .map(|value| value.to_string())
            .unwrap_or_default();
        let application = OwnedCf::new(AXUIElementCreateApplication(pid))?;
        let focused = attribute(
            application.as_type_ref(),
            focused_element_attr.as_concrete_TypeRef(),
        )?;
        let window_title = attribute(
            application.as_type_ref(),
            focused_window_attr.as_concrete_TypeRef(),
        )
        .map(|window| string_attribute(window.as_type_ref(), title_attr.as_concrete_TypeRef()))
        .unwrap_or_default();
        let role = string_attribute(focused.as_type_ref(), role_attr.as_concrete_TypeRef());
        let subrole = string_attribute(focused.as_type_ref(), subrole_attr.as_concrete_TypeRef());
        let value = string_attribute(focused.as_type_ref(), value_attr.as_concrete_TypeRef())
            .chars()
            .take(16_000)
            .collect();
        let selected_text = string_attribute(
            focused.as_type_ref(),
            selected_text_attr.as_concrete_TypeRef(),
        )
        .chars()
        .take(16_000)
        .collect();
        let editable = bool_attribute(focused.as_type_ref(), enabled_attr.as_concrete_TypeRef())
            .unwrap_or(true);
        let mut value_settable = false;
        let _ = AXUIElementIsAttributeSettable(
            focused.as_type_ref(),
            value_attr.as_concrete_TypeRef(),
            &raw mut value_settable,
        );
        let mut identifier =
            string_attribute(focused.as_type_ref(), identifier_attr.as_concrete_TypeRef());
        if identifier.is_empty() {
            // CFHash is a process-local identity for the AX proxy. It lets the
            // two probes prove focus stayed on the same element without
            // retaining or serializing any field content.
            identifier = format!("ax:{:x}", CFHash(focused.as_type_ref()));
        }

        Some(FocusFields {
            bundle_id,
            app_name,
            window_title,
            role,
            subrole,
            identifier,
            position: String::new(),
            size: String::new(),
            value,
            selected_text,
            editable,
            value_settable,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FocusFields {
    bundle_id: String,
    app_name: String,
    window_title: String,
    role: String,
    subrole: String,
    identifier: String,
    position: String,
    size: String,
    value: String,
    selected_text: String,
    editable: bool,
    /// `AXValue` is writable — the accessibility answer to "will a paste land
    /// here". True for native fields and for Chromium/Electron composers.
    value_settable: bool,
}

#[cfg(test)]
fn parse_focus_line(line: &str) -> Option<FocusFields> {
    fn flag(part: Option<&str>) -> bool {
        part.is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "true" | "yes" | "1"
            )
        })
    }

    // The focused field's value goes last so text containing "||" cannot shift
    // any of the fields Destroy makes decisions from.
    let (fields_line, selected_text) = line
        .rsplit_once('\u{1d}')
        .map_or((line, ""), |(fields, selected)| (fields, selected));
    let mut parts = fields_line.splitn(11, '\u{1e}');
    let bundle_id = parts.next()?.trim().to_string();
    let app_name = parts.next()?.trim().to_string();
    let window_title = parts.next()?.trim().to_string();
    let role = parts.next()?.trim().to_string();
    let subrole = parts.next()?.trim().to_string();
    let editable = flag(parts.next());
    let value_settable = flag(parts.next());
    let identifier = parts.next().unwrap_or_default().trim().to_string();
    let position = parts.next().unwrap_or_default().trim().to_string();
    let size = parts.next().unwrap_or_default().trim().to_string();
    let value = parts.next().unwrap_or_default().trim().to_string();
    Some(FocusFields {
        bundle_id,
        app_name,
        window_title,
        role,
        subrole,
        identifier,
        position,
        size,
        value,
        selected_text: selected_text.trim().to_string(),
        editable,
        value_settable,
    })
}

/// Privacy-safe, process-local identity for one focused accessibility element.
/// A stable accessibility identifier wins when the app provides one; geometry
/// is the fail-closed fallback. Excluding geometry from identified elements is
/// important because message boxes often grow after a paste, and correction or
/// undo must still recognize that exact element. Raw metadata never leaves this
/// module; only the digest lives for the short correction window.
fn focus_signature(fields: &FocusFields) -> String {
    if fields.identifier.is_empty() && (fields.position.is_empty() || fields.size.is_empty()) {
        return String::new();
    }
    let mut hasher = Sha256::new();
    for value in [
        &fields.bundle_id,
        &fields.window_title,
        &fields.role,
        &fields.subrole,
    ] {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    if fields.identifier.is_empty() {
        for value in [&fields.position, &fields.size] {
            hasher.update((value.len() as u64).to_be_bytes());
            hasher.update(value.as_bytes());
        }
    } else {
        hasher.update((fields.identifier.len() as u64).to_be_bytes());
        hasher.update(fields.identifier.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// True only when both probes positively identify the same focused field.
#[must_use]
pub fn same_dictation_field(left: &DictationFocusTarget, right: &DictationFocusTarget) -> bool {
    !left.bundle_id.is_empty()
        && left.bundle_id == right.bundle_id
        && !left.focus_signature.is_empty()
        && left.focus_signature == right.focus_signature
}

fn is_own_app(fields: &FocusFields) -> bool {
    crate::app_identity::is_own_bundle_id(&fields.bundle_id)
        || fields.app_name.eq_ignore_ascii_case("Destroy Dictation")
        || fields
            .app_name
            .eq_ignore_ascii_case("Destroy Dictation Dev")
}

fn is_password_field(fields: &FocusFields) -> bool {
    let role = fields.role.to_ascii_lowercase();
    let subrole = fields.subrole.to_ascii_lowercase();
    subrole.contains("secure") || role.contains("secure") || subrole.contains("password")
}

fn is_native_text_field(role: &str) -> bool {
    let role = role.to_ascii_lowercase();
    role.contains("textfield")
        || role.contains("textarea")
        || role == "axtextfield"
        || role == "axtextarea"
}

fn web_compose_accept(fields: &FocusFields) -> bool {
    let app = fields.app_name.to_ascii_lowercase();
    let title = fields.window_title.to_ascii_lowercase();
    let role = fields.role.to_ascii_lowercase();
    let is_browser = app.contains("chrome") || app.contains("safari") || app.contains("firefox");
    let compose_context = title.contains("gmail")
        || title.contains("compose")
        || title.contains("slack")
        || title.contains("linkedin");
    is_browser
        && compose_context
        && (role.contains("webarea")
            || role.contains("group")
            || is_native_text_field(&fields.role))
        && !fields.value.is_empty()
}

/// Whether dictated text will actually land in the focused element.
///
/// This is a capability test rather than an app allow-list. `AXValue` being
/// settable is the accessibility answer to "will a paste stick here", and it
/// holds for native fields, for Chromium's mapping of `contenteditable`
/// (Slack, Notion, Discord, VS Code, Cursor), and for web inputs in every
/// browser — without Destroy maintaining a list of blessed apps.
///
/// Getting this wrong in the permissive direction is the expensive mistake:
/// Destroy would report an inline paste that never happened. Anything Destroy cannot
/// positively confirm goes to the clipboard instead.
fn dictation_accepts(fields: &FocusFields) -> bool {
    if is_own_app(fields) || is_password_field(fields) || !fields.editable {
        return false;
    }
    !focus_signature(fields).is_empty()
        && (is_native_text_field(&fields.role) || fields.value_settable)
}

fn dictation_app_kind(fields: &FocusFields) -> String {
    let haystack = format!(
        "{} {} {}",
        fields.bundle_id, fields.app_name, fields.window_title
    )
    .to_ascii_lowercase();
    // Ordered most specific first: a Chrome window titled "… - Gmail" is Gmail,
    // not a generic browser.
    for (needle, kind) in [
        ("gmail", "gmail"),
        ("slack", "slack"),
        ("linkedin", "linkedin"),
        ("notion", "notion"),
        ("outlook", "outlook"),
        ("mail", "mail"),
        ("code", "editor"),
        ("cursor", "editor"),
        ("zed", "editor"),
        ("terminal", "terminal"),
        ("iterm", "terminal"),
        ("chrome", "browser"),
        ("safari", "browser"),
        ("firefox", "browser"),
        ("arc", "browser"),
        ("brave", "browser"),
        ("edge", "browser"),
    ] {
        if haystack.contains(needle) {
            return kind.into();
        }
    }
    "generic".into()
}

fn classify_focus_fields(fields: &FocusFields) -> FocusProbeResult {
    if is_own_app(fields) {
        return FocusProbeResult {
            delivery: DraftDelivery::Notch,
            partial_draft: None,
        };
    }
    if is_password_field(fields) || !fields.editable {
        return FocusProbeResult {
            delivery: DraftDelivery::Notch,
            partial_draft: None,
        };
    }

    let partial = if fields.value.is_empty() {
        None
    } else {
        Some(fields.value.clone())
    };

    if is_native_text_field(&fields.role) {
        return FocusProbeResult {
            delivery: DraftDelivery::Inline,
            partial_draft: partial,
        };
    }

    if web_compose_accept(fields) {
        return FocusProbeResult {
            delivery: DraftDelivery::Inline,
            partial_draft: partial,
        };
    }

    FocusProbeResult {
        delivery: DraftDelivery::Notch,
        partial_draft: partial.filter(|value| !value.is_empty()),
    }
}

#[cfg(test)]
fn classify_focus_line(line: &str) -> Option<FocusProbeResult> {
    let fields = parse_focus_line(line)?;
    Some(classify_focus_fields(&fields))
}

#[cfg(test)]
mod tests {
    use super::{
        classify_focus_fields, classify_focus_line, dictation_accepts, dictation_app_kind,
        focus_signature, parse_focus_line, same_dictation_field, DictationFocusTarget,
        DraftDelivery, FocusFields,
    };

    fn sample(bundle: &str, app: &str, role: &str, value: &str) -> FocusFields {
        FocusFields {
            bundle_id: bundle.into(),
            app_name: app.into(),
            window_title: "Inbox".into(),
            role: role.into(),
            subrole: String::new(),
            identifier: "focused-editor".into(),
            position: "100, 200".into(),
            size: "640, 320".into(),
            value: value.into(),
            selected_text: String::new(),
            editable: true,
            value_settable: false,
        }
    }

    #[test]
    fn rejects_own_app_focused_field() {
        let result = classify_focus_fields(&sample(
            "org.destroy.dictation.community",
            "Destroy Dictation",
            "AXTextArea",
            "notes",
        ));
        assert_eq!(result.delivery, DraftDelivery::Notch);
        assert!(result.partial_draft.is_none());

        let dev = classify_focus_fields(&sample(
            "org.destroy.dictation.community.dev",
            "Destroy Dictation Dev",
            "AXTextArea",
            "notes",
        ));
        assert_eq!(dev.delivery, DraftDelivery::Notch);
    }

    #[test]
    fn accepts_native_text_area() {
        let result =
            classify_focus_fields(&sample("com.apple.mail", "Mail", "AXTextArea", "Hi there"));
        assert_eq!(result.delivery, DraftDelivery::Inline);
        assert_eq!(result.partial_draft.as_deref(), Some("Hi there"));
    }

    #[test]
    fn web_gmail_requires_value() {
        let fields = FocusFields {
            bundle_id: "com.google.Chrome".into(),
            app_name: "Google Chrome".into(),
            window_title: "Gmail - Compose".into(),
            role: "AXWebArea".into(),
            subrole: String::new(),
            identifier: "page".into(),
            position: "0, 0".into(),
            size: "1200, 800".into(),
            value: String::new(),
            selected_text: String::new(),
            editable: true,
            value_settable: false,
        };
        assert_eq!(
            classify_focus_fields(&fields).delivery,
            DraftDelivery::Notch
        );

        let with_value = FocusFields {
            value: "Draft body".into(),
            ..fields.clone()
        };
        assert_eq!(
            classify_focus_fields(&with_value).delivery,
            DraftDelivery::Inline
        );
        assert!(!dictation_accepts(&fields));
        assert_eq!(dictation_app_kind(&fields), "gmail");
    }

    #[test]
    fn dictation_rejects_own_app_and_secure_fields() {
        let own_app = sample(
            "org.destroy.dictation.community",
            "Destroy Dictation",
            "AXTextArea",
            "",
        );
        assert!(!dictation_accepts(&own_app));

        let own_app_dev = sample(
            "org.destroy.dictation.community.dev",
            "Destroy Dictation Dev",
            "AXTextArea",
            "",
        );
        assert!(!dictation_accepts(&own_app_dev));

        let mut secure = sample("com.apple.Safari", "Safari", "AXSecureTextField", "");
        secure.subrole = "AXSecureTextField".into();
        assert!(!dictation_accepts(&secure));
    }

    #[test]
    fn rejects_secure_field() {
        let fields = FocusFields {
            subrole: "AXSecureTextField".into(),
            ..sample("com.apple.Safari", "Safari", "AXTextField", "")
        };
        assert_eq!(
            classify_focus_fields(&fields).delivery,
            DraftDelivery::Notch
        );
    }

    #[test]
    fn parses_probe_line() {
        let result =
            classify_focus_line("com.apple.mail\u{1e}Mail\u{1e}Reply\u{1e}AXTextArea\u{1e}\u{1e}true\u{1e}true\u{1e}body\u{1e}100, 200\u{1e}640, 320\u{1e}Hello")
                .expect("parse");
        assert_eq!(result.delivery, DraftDelivery::Inline);
        assert_eq!(result.partial_draft.as_deref(), Some("Hello"));
    }

    /// The field value is last in the probe line, so draft text containing the
    /// separator cannot shift the flags Destroy makes paste decisions from.
    #[test]
    fn probe_line_tolerates_separators_inside_the_value() {
        let fields =
            parse_focus_line("com.apple.mail\u{1e}Mail\u{1e}Reply\u{1e}AXTextArea\u{1e}\u{1e}true\u{1e}true\u{1e}body\u{1e}100, 200\u{1e}640, 320\u{1e}a||b||c")
                .expect("parse");
        assert!(fields.editable);
        assert!(fields.value_settable);
        assert_eq!(fields.value, "a||b||c");
    }

    #[test]
    fn captures_selected_text_without_mixing_it_into_target_metadata() {
        let fields = parse_focus_line(
            "com.apple.mail\u{1e}Mail\u{1e}Reply\u{1e}AXTextArea\u{1e}\u{1e}true\u{1e}true\u{1e}body\u{1e}100, 200\u{1e}640, 320\u{1e}Whole draft\u{1d}Tuesday",
        )
        .expect("parse");
        assert_eq!(fields.value, "Whole draft");
        assert_eq!(fields.selected_text, "Tuesday");
    }

    #[test]
    fn selected_text_is_never_serialized_with_target_metadata() {
        let target = super::DictationFocusTarget {
            can_paste: true,
            app_name: "Mail".into(),
            bundle_id: "com.apple.mail".into(),
            app_kind: "mail".into(),
            app_icon_data_url: None,
            focus_signature: "not-serialized".into(),
            selected_text: Some("private customer sentence".into()),
        };
        let json = serde_json::to_string(&target).expect("serialize target");
        assert!(!json.contains("private customer sentence"));
        assert!(!json.contains("selectedText"));
        assert!(!json.contains("not-serialized"));
    }

    /// A signed app without Accessibility receives a null AX attribute value.
    /// The 0.1.48 candidate wrapped that null in an owning CF guard and crashed
    /// in `CFRelease`. This smoke path must always return safely, regardless of
    /// whether the test runner itself has Accessibility permission.
    #[cfg(target_os = "macos")]
    #[test]
    fn unavailable_accessibility_attributes_never_crash_the_probe() {
        let _ = super::probe_dictation_target();
    }

    /// The four surfaces named in the dictation acceptance bar, plus the
    /// terminals and editors a founder-seller actually lives in. Chromium maps
    /// `contenteditable` to `AXTextArea`, so the Electron apps come for free.
    #[test]
    fn dictation_accepts_the_apps_people_actually_type_in() {
        for (bundle, app, role) in [
            ("com.tinyspeck.slackmacgap", "Slack", "AXTextArea"),
            ("notion.id", "Notion", "AXTextArea"),
            ("com.microsoft.VSCode", "Code", "AXTextArea"),
            ("com.todesktop.230313mzl4w4u92", "Cursor", "AXTextArea"),
            ("com.apple.Notes", "Notes", "AXTextArea"),
            ("com.apple.Terminal", "Terminal", "AXTextArea"),
        ] {
            assert!(
                dictation_accepts(&sample(bundle, app, role, "")),
                "{app} must take an inline paste"
            );
        }
    }

    /// A web composer with no native role still pastes when the accessibility
    /// tree says its value is writable — no app allow-list involved.
    #[test]
    fn dictation_accepts_any_settable_web_field() {
        let mut fields = sample("com.apple.Safari", "Safari", "AXWebArea", "");
        fields.window_title = "Some Internal Tool".into();
        assert!(!dictation_accepts(&fields));

        fields.value_settable = true;
        assert!(dictation_accepts(&fields));
    }

    /// Read-only content must go to the clipboard. Reporting an inline paste
    /// that never landed is the one failure that loses the user's words.
    #[test]
    fn dictation_refuses_read_only_content() {
        let mut fields = sample("com.apple.Safari", "Safari", "AXWebArea", "Article body");
        fields.window_title = "Some News Site".into();
        assert!(!dictation_accepts(&fields));

        let disabled = FocusFields {
            editable: false,
            value_settable: true,
            ..sample("com.apple.TextEdit", "TextEdit", "AXTextArea", "")
        };
        assert!(!dictation_accepts(&disabled));
    }

    #[test]
    fn exact_field_identity_distinguishes_fields_inside_one_app() {
        let first = sample("com.google.Chrome", "Chrome", "AXTextArea", "");
        let mut second = first.clone();
        second.identifier = "other-editor".into();
        second.position = "100, 600".into();
        let target = |fields: &FocusFields| DictationFocusTarget {
            can_paste: true,
            app_name: fields.app_name.clone(),
            bundle_id: fields.bundle_id.clone(),
            app_kind: "browser".into(),
            app_icon_data_url: None,
            focus_signature: focus_signature(fields),
            selected_text: None,
        };
        assert!(same_dictation_field(&target(&first), &target(&first)));
        assert!(!same_dictation_field(&target(&first), &target(&second)));
    }

    #[test]
    fn identified_field_survives_message_box_growth() {
        let first = sample("com.tinyspeck.slackmacgap", "Slack", "AXTextArea", "");
        let mut grown = first.clone();
        grown.position = "100, 120".into();
        grown.size = "640, 180".into();
        assert_eq!(focus_signature(&first), focus_signature(&grown));
    }

    #[test]
    fn anonymous_fields_remain_geometry_scoped() {
        let mut first = sample("com.google.Chrome", "Chrome", "AXTextArea", "");
        first.identifier.clear();
        let mut second = first.clone();
        second.position = "100, 600".into();
        assert_ne!(focus_signature(&first), focus_signature(&second));
    }

    #[test]
    fn dictation_app_kind_labels_editors_and_terminals() {
        assert_eq!(
            dictation_app_kind(&sample("com.todesktop.x", "Cursor", "AXTextArea", "")),
            "editor"
        );
        assert_eq!(
            dictation_app_kind(&sample("com.apple.Terminal", "Terminal", "AXTextArea", "")),
            "terminal"
        );
    }
}
