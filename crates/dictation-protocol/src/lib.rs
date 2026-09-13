use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationPreferences {
    pub self_correction: bool,
    pub remove_fillers: bool,
    pub app_formatting: bool,
    pub selected_text_editing: bool,
    /// BCP-47-ish language tag (`auto`, `en`, `es`, ...). The server validates
    /// the supported set before it reaches a speech provider.
    pub language: String,
    /// A short, user-approved description such as “warm, concise, no em dash”.
    pub style_note: String,
}

impl Default for DictationPreferences {
    fn default() -> Self {
        Self {
            self_correction: true,
            remove_fillers: true,
            app_formatting: true,
            selected_text_editing: true,
            language: "auto".into(),
            style_note: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateDictationPreferencesRequest {
    pub self_correction: bool,
    pub remove_fillers: bool,
    pub app_formatting: bool,
    pub selected_text_editing: bool,
    pub language: String,
    pub style_note: String,
}

/// Explicit sample used to derive a compact style note. The sample is
/// processed transiently and is never stored; only the returned summary is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnDictationStyleRequest {
    pub sample: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationStyleResponse {
    pub style_note: String,
}

/// A voice shortcut owned by one rep. `body` may contain the small set of
/// explicit user-defined variables documented by the dictation settings UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationSnippet {
    pub id: Uuid,
    pub title: String,
    pub trigger: String,
    pub body: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DictationSnippetsResponse {
    pub snippets: Vec<DictationSnippet>,
    pub max_snippets: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDictationSnippetRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub title: String,
    pub trigger: String,
    pub body: String,
}

/// How the spoken input should be composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationTransformOperation {
    Dictate,
    /// The rep explicitly ended an utterance with a rewrite command. Unlike
    /// ordinary dictation, this permits substantial rephrasing while still
    /// preserving factual literals and intent.
    Rewrite,
    EditSelected,
    CorrectPrevious,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationTransformRequest {
    pub text: String,
    pub app_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_text: Option<String>,
    pub operation: DictationTransformOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationTransformResponse {
    pub text: String,
    /// Privacy-safe names of transformations that ran; never content.
    pub applied: Vec<String>,
}

/// Which GIPHY library the dictation media picker searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationMediaKind {
    Gif,
    Sticker,
}

/// A literal, user-spoken GIPHY search request.
///
/// The server deliberately does not accept a team id or rating from the client:
/// auth scope and workplace-safe content policy are server-owned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationMediaSearchRequest {
    pub query: String,
    pub kind: DictationMediaKind,
    /// Provider result offset. The picker shows three results at a time and
    /// advances this when the rep asks for different ones. Older desktop
    /// builds omit it, so it defaults to the first page.
    #[serde(default)]
    pub offset: u32,
}

/// Minimal GIPHY result returned to the desktop. Provider response bodies are
/// not persisted and unrelated metadata is discarded at the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationMediaResult {
    pub id: String,
    pub title: String,
    pub alt_text: String,
    pub preview_url: String,
    pub content_url: String,
    pub source_url: String,
    pub width: u32,
    pub height: u32,
    pub kind: DictationMediaKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationMediaSearchResponse {
    pub query: String,
    pub kind: DictationMediaKind,
    pub provider: String,
    pub attribution: String,
    pub results: Vec<DictationMediaResult>,
}

/// A GIPHY choice the rep explicitly saved for reuse. The spoken search query
/// and provider metadata live in one encrypted payload per favorite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationMediaFavorite {
    pub id: Uuid,
    pub provider_id: String,
    pub query: String,
    pub title: String,
    pub alt_text: String,
    pub preview_url: String,
    pub content_url: String,
    pub source_url: String,
    pub width: u32,
    pub height: u32,
    pub kind: DictationMediaKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DictationMediaFavoritesResponse {
    pub favorites: Vec<DictationMediaFavorite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDictationMediaFavoriteRequest {
    pub provider_id: String,
    pub query: String,
    pub title: String,
    pub alt_text: String,
    pub preview_url: String,
    pub content_url: String,
    pub source_url: String,
    pub width: u32,
    pub height: u32,
    pub kind: DictationMediaKind,
}

/// Speech-to-text response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeResponse {
    pub text: String,
    pub path: SttPath,
}

/// Content-free controls sent by the desktop over one authenticated hotkey
/// dictation websocket. PCM audio uses binary websocket frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DictationStreamClientMessage {
    Finish,
    Cancel,
}

/// Bounded messages returned by mothership's Gemini Live relay. Interim text is
/// transient UI state; only `Final` is delivered into the focused field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DictationStreamServerMessage {
    Ready,
    Interim { text: String },
    Final { text: String },
    Error { code: String, message: String },
}

/// Whether STT ran locally or via mothership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SttPath {
    Local,
    Cloud,
}
