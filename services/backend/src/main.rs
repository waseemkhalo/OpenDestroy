mod composer;
mod storage;
mod stream;
use axum::{
    extract::{ws::WebSocketUpgrade, DefaultBodyLimit, OriginalUri, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dictation_protocol::*;
use futures_util::StreamExt;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use zeroize::Zeroizing;
type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;
#[derive(Clone)]
pub struct AppState {
    http: reqwest::Client,
    store: Arc<storage::Store>,
    users: Arc<HashMap<String, String>>,
    slots: Arc<tokio::sync::Semaphore>,
}
fn error(status: StatusCode, message: impl AsRef<str>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"error":message.as_ref()})))
}
fn bad(message: impl AsRef<str>) -> (StatusCode, Json<Value>) {
    error(StatusCode::BAD_REQUEST, message)
}
fn auth(state: &AppState, headers: &HeaderMap) -> Result<String, (StatusCode, Json<Value>)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| v.len() >= 32 && v.len() <= 1024)
        .ok_or_else(|| {
            error(
                StatusCode::UNAUTHORIZED,
                "A backend access token is required",
            )
        })?;
    let hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    state.users.get(&hash).cloned().ok_or_else(|| {
        error(
            StatusCode::UNAUTHORIZED,
            "Access token is invalid or revoked",
        )
    })
}
fn profile(data: &Value) -> DictationPreferences {
    serde_json::from_value(data["preferences"].clone()).unwrap_or_default()
}
fn language(raw: &str) -> Option<&'static str> {
    match raw {
        "auto" => Some(""),
        "en" => Some("en-US"),
        "es" => Some("es-ES"),
        "fr" => Some("fr-FR"),
        "de" => Some("de-DE"),
        "nl" => Some("nl-NL"),
        "it" => Some("it-IT"),
        "pt" => Some("pt-BR"),
        "ar" => Some("ar-EG"),
        "ja" => Some("ja-JP"),
        "ko" => Some("ko-KR"),
        "zh" => Some("cmn-Hans-CN"),
        "hi" => Some("hi-IN"),
        _ => None,
    }
}
const MEDIA_APPROVALS: [&str; 3] = [
    "GIPHY_PROXY_APPROVED",
    "GIPHY_MEDIA_URL_STORAGE_APPROVED",
    "GIPHY_FAVORITES_ORDER_APPROVED",
];

/// Names the media configuration this deployment is still missing.
///
/// An absent key and an unset approval flag fail identically at the request
/// boundary, so an operator who set one of the four sees the same "unavailable"
/// as one who set none, with nothing to act on. These are variable names the
/// operator already has in `.env.example`; no value is ever read back out.
fn media_blockers_from(read: impl Fn(&str) -> Option<String>) -> Vec<&'static str> {
    let mut missing: Vec<&'static str> = MEDIA_APPROVALS
        .into_iter()
        .filter(|key| read(key).as_deref() != Some("true"))
        .collect();
    if !read("GIPHY_API_KEY").is_some_and(|v| !v.trim().is_empty()) {
        missing.push("GIPHY_API_KEY");
    }
    missing
}

fn media_blockers() -> Vec<&'static str> {
    media_blockers_from(|key| std::env::var(key).ok())
}

/// Operator-facing explanation for a blocked media request. The reader is
/// whoever runs this backend, so it names the fix rather than the symptom.
fn media_unavailable(blockers: &[&'static str]) -> String {
    format!(
        "Media is disabled on this backend. Set {} in .env, then restart the service.",
        blockers.join(", ")
    )
}
async fn bounded(response: reqwest::Response, max: usize) -> Result<Value, String> {
    if !response.status().is_success() {
        return Err("Provider request failed; check backend provider configuration".into());
    }
    let mut out = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(bytes) = stream.next().await {
        let bytes = bytes.map_err(|_| "Provider connection interrupted")?;
        if out.len() + bytes.len() > max {
            return Err("Provider response too large".into());
        }
        out.extend_from_slice(&bytes);
    }
    serde_json::from_slice(&out).map_err(|_| "Invalid provider response".into())
}
pub async fn openai_text(
    state: &AppState,
    key: &str,
    system: &str,
    user: &str,
    max_tokens: u32,
    temperature: f32,
) -> Result<String, String> {
    let response=state.http.post("https://api.openai.com/v1/chat/completions").bearer_auth(key).json(&json!({"model":std::env::var("OPENAI_TEXT_MODEL").unwrap_or("gpt-4.1-mini".into()),"messages":[{"role":"system","content":system},{"role":"user","content":user}],"max_tokens":max_tokens,"temperature":temperature,"store":false})).send().await.map_err(|_|"Cannot reach writing provider")?;
    let result = bounded(response, 128 * 1024).await?;
    let text = result["choices"][0]["message"]["content"]
        .as_str()
        .filter(|t| !t.trim().is_empty())
        .ok_or("Writing provider returned no text")?;
    Ok(text.trim().to_owned())
}
async fn transform(state: &AppState, data: &Value, body: Value) -> Result<Value, String> {
    let request: DictationTransformRequest =
        serde_json::from_value(body).map_err(|_| "Invalid transformation")?;
    if request.text.trim().is_empty()
        || request.text.chars().count() > 12000
        || request
            .selected_text
            .as_ref()
            .is_some_and(|x| x.chars().count() > 16000)
        || request
            .previous_text
            .as_ref()
            .is_some_and(|x| x.chars().count() > 16000)
    {
        return Err("Text length exceeds transformation limits".into());
    }
    let prefs = profile(data);
    use DictationTransformOperation::*;
    if request.operation == EditSelected
        && (!prefs.selected_text_editing
            || request
                .selected_text
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty())
    {
        return Err("Selected-text editing is disabled or no selection was captured".into());
    }
    if request.operation == CorrectPrevious
        && request
            .previous_text
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err("No recent dictation to correct".into());
    }
    let mut fallback = request.text.trim().to_owned();
    let mut applied = Vec::new();
    if request.operation == Dictate {
        let snippets: Vec<DictationSnippet> =
            serde_json::from_value(data["snippets"].clone()).unwrap_or_default();
        if let Some(snippet) = composer::matching_snippet(&request.text, &snippets) {
            let mut result = snippet.body.clone();
            if let Some(vars) = data["variables"].as_object() {
                for (k, v) in vars {
                    result = result.replace(&format!("{{{{{k}}}}}"), v.as_str().unwrap_or(""));
                }
            }
            if result.contains("{{") {
                return Err("Define the missing snippet variable in Settings".into());
            }
            return Ok(json!({"text":result,"applied":["snippet"]}));
        }
        if prefs.self_correction {
            fallback = composer::apply_simple_self_correction(&fallback);
            applied.push("self_correction")
        }
        if prefs.remove_fillers {
            fallback = composer::remove_fillers_and_repetition(&fallback);
            applied.push("filler_cleanup")
        }
    }
    if request.operation != Dictate || prefs.app_formatting || !prefs.style_note.is_empty() {
        match composer::shape_with_model(state, &request, &prefs, &fallback).await {
            Ok(text) => return Ok(json!({"text":text,"applied":["model"]})),
            Err(e) if request.operation != Dictate => return Err(e),
            _ => applied.push("model_unavailable"),
        }
    }
    Ok(json!({"text":fallback,"applied":applied}))
}
async fn transcribe(state: &AppState, data: &Value, body: &Value) -> Result<Value, String> {
    let encoded = body["audio_base64"].as_str().ok_or("Audio is required")?;
    if encoded.len() > 8 * 1024 * 1024 + 4 {
        return Err("Audio exceeds size limit".into());
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "Invalid audio encoding")?;
    if bytes.is_empty() || bytes.len() > 6 * 1024 * 1024 {
        return Err("Audio is empty or too large".into());
    }
    let mime = body["mime_type"].as_str().unwrap_or("audio/wav");
    let ext = match mime.split(';').next().unwrap_or("") {
        "audio/wav" => "wav",
        "audio/mp4" => "m4a",
        "audio/webm" => "webm",
        _ => return Err("Unsupported recording type".into()),
    };
    let key = std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or("Batch transcription is not configured")?;
    let mut form = reqwest::multipart::Form::new()
        .text(
            "model",
            std::env::var("OPENAI_STT_MODEL").unwrap_or("gpt-4o-mini-transcribe".into()),
        )
        .part(
            "file",
            reqwest::multipart::Part::bytes(bytes)
                .file_name(format!("dictation.{ext}"))
                .mime_str(mime)
                .map_err(|_| "Unsupported audio")?,
        );
    let prefs = profile(data);
    if prefs.language != "auto" {
        form = form.text("language", prefs.language);
    }
    if let Some(terms) = data["terms"].as_array() {
        let prompt = terms
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        if !prompt.is_empty() {
            form = form.text("prompt", prompt);
        }
    }
    let result = bounded(
        state
            .http
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(key)
            .multipart(form)
            .send()
            .await
            .map_err(|_| "Transcription provider unavailable")?,
        256 * 1024,
    )
    .await?;
    Ok(json!({"text":result["text"].as_str().ok_or("No transcription returned")?,"path":"cloud"}))
}
fn validate_https(raw: &str) -> bool {
    reqwest::Url::parse(raw).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none()
    })
}
async fn media_search(state: &AppState, body: &Value) -> Result<Value, String> {
    let blockers = media_blockers();
    if !blockers.is_empty() {
        return Err(media_unavailable(&blockers));
    }
    let query = body["query"]
        .as_str()
        .filter(|v| !v.is_empty() && v.len() <= 200)
        .ok_or("Invalid media query")?;
    let kind = body["kind"].as_str().unwrap_or("gif");
    let path = match kind {
        "gif" => "gifs",
        "sticker" => "stickers",
        _ => return Err("Invalid media kind".into()),
    };
    let offset = body["offset"].as_u64().unwrap_or(0).min(27).to_string();
    let key = std::env::var("GIPHY_API_KEY").map_err(|_| "GIPHY is not configured")?;
    let response = state
        .http
        .get(format!("https://api.giphy.com/v1/{path}/search"))
        .query(&[
            ("api_key", key.as_str()),
            ("q", query),
            ("limit", "3"),
            ("offset", &offset),
            ("rating", "g"),
        ])
        .send()
        .await
        .map_err(|_| "Media search unavailable")?;
    let data = bounded(response, 1024 * 1024).await?;
    let results:Vec<Value>=data["data"].as_array().unwrap_or(&vec![]).iter().map(|v|{let preview=&v["images"]["fixed_height_small"];let full=&v["images"]["original"];json!({"id":v["id"],"title":v["title"],"alt_text":v["alt_text"].as_str().unwrap_or("GIF"),"preview_url":preview["url"],"content_url":full["url"],"source_url":v["url"],"width":full["width"].as_str().and_then(|v|v.parse::<u32>().ok()).unwrap_or(0),"height":full["height"].as_str().and_then(|v|v.parse::<u32>().ok()).unwrap_or(0),"kind":kind})}).collect();
    Ok(
        json!({"query":query,"kind":kind,"provider":"giphy","attribution":"Powered by GIPHY","results":results}),
    )
}
fn usage(data: &Value) -> Value {
    let rows = data["days"].as_object();
    let mut days = Vec::new();
    let (mut words, mut ms) = (0, 0);
    if let Some(rows) = rows {
        for (day, v) in rows {
            words += v["words"].as_u64().unwrap_or(0);
            ms += v["speaking_ms"].as_u64().unwrap_or(0);
            days.push(json!({"day":day,"words":v["words"],"speaking_ms":v["speaking_ms"]}));
        }
    }
    json!({"words":words,"speaking_ms":ms,"days":days})
}
async fn api(
    State(state): State<AppState>,
    headers: HeaderMap,
    method: Method,
    OriginalUri(uri): OriginalUri,
    body: Option<Json<Value>>,
) -> ApiResult {
    let user = auth(&state, &headers)?;
    let path = uri.path();
    let body = body.map(|v| v.0).unwrap_or(Value::Null);
    if matches!(path, "/v1/account" | "/v1/me") && method == Method::GET {
        return Ok(Json(json!({"user_id":user})));
    }
    // An authenticated owner can erase corrupt data without decrypting it first.
    if path == "/v1/account/data" && method == Method::DELETE {
        state.store.delete(&user).await.map_err(bad)?;
        return Ok(Json(json!({"deleted":true})));
    }
    let data = state.store.read(&user).await.map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot decrypt personal data",
        )
    })?;
    if path == "/v1/account/export" && method == Method::GET {
        return Ok(Json(data));
    }
    let _permit = state.slots.clone().try_acquire_owned().map_err(|_| {
        error(
            StatusCode::TOO_MANY_REQUESTS,
            "Backend is busy; try again shortly",
        )
    })?;
    if method == Method::POST {
        let result = match path {
            "/v1/dictation/transform" => Some(transform(&state, &data, body.clone()).await),
            "/v1/dictation/transcribe" => Some(transcribe(&state, &data, &body).await),
            "/v1/dictation/media/search" => Some(media_search(&state, &body).await),
            "/v1/dictation/style/learn" => {
                let sample = body["sample"].as_str().unwrap_or("");
                if !(40..=12000).contains(&sample.chars().count()) {
                    return Err(bad("Writing sample must have 40–12000 characters"));
                }
                let note = composer::infer_style_note(&state, sample)
                    .await
                    .map_err(|e| error(StatusCode::SERVICE_UNAVAILABLE, e))?;
                Some(
                    state
                        .store
                        .update(&user, move |v| {
                            let mut p = profile(v);
                            p.style_note = note.chars().take(400).collect();
                            v["preferences"] = json!(p);
                            Ok(json!({"style_note":p.style_note}))
                        })
                        .await,
                )
            }
            "/v1/links/search" => {
                let query = body["query"].as_str().unwrap_or("").to_lowercase();
                let terms: Vec<_> = query.split_whitespace().collect();
                let mut rows: Vec<(usize, Value)> = data["links"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| {
                        let text = format!(
                            "{} {}",
                            v["name"].as_str().unwrap_or(""),
                            v["keywords"].as_str().unwrap_or("")
                        )
                        .to_lowercase();
                        (
                            terms.iter().filter(|t| text.contains(**t)).count(),
                            v.clone(),
                        )
                    })
                    .filter(|(n, _)| *n > 0)
                    .collect();
                rows.sort_by_key(|v| std::cmp::Reverse(v.0));
                Some(Ok(
                    json!({"results":rows.into_iter().take(3).map(|v|v.1).collect::<Vec<_>>()}),
                ))
            }
            _ => None,
        };
        if let Some(result) = result {
            return result
                .map(Json)
                .map_err(|e| error(StatusCode::SERVICE_UNAVAILABLE, e));
        }
    }
    if method == Method::GET {
        let result = match path {
            "/v1/dictation/preferences" => json!(profile(&data)),
            "/v1/dictation/snippets" => {
                json!({"snippets":data["snippets"].as_array().cloned().unwrap_or_default(),"max_snippets":50})
            }
            "/v1/dictation/vocabulary" => {
                json!({"terms":data["terms"].as_array().cloned().unwrap_or_default(),"max_terms":100})
            }
            "/v1/dictation/usage" => usage(&data),
            "/v1/dictation/media/status" => {
                let blockers = media_blockers();
                json!({"enabled":blockers.is_empty(),"missing":blockers})
            }
            "/v1/dictation/media/favorites" => {
                json!({"favorites":data["favorites"].as_array().cloned().unwrap_or_default()})
            }
            "/v1/variables" => data["variables"]
                .as_object()
                .cloned()
                .map(Value::Object)
                .unwrap_or(json!({})),
            "/v1/links" => data["links"]
                .as_array()
                .cloned()
                .map(Value::Array)
                .unwrap_or(json!([])),
            _ => return Err(error(StatusCode::NOT_FOUND, "Unknown endpoint")),
        };
        return Ok(Json(result));
    }
    let owned_path = path.to_owned();
    let result=state.store.update(&user,move |data|{
 let path = owned_path.as_str();
 if method==Method::PUT{match path{
 "/v1/dictation/preferences"=>{let p:DictationPreferences=serde_json::from_value(body.clone()).map_err(|_|"Invalid preferences")?;if p.style_note.chars().count()>400||language(&p.language).is_none(){return Err("Unsupported language or style note exceeds 400 characters".into())}data["preferences"]=json!(p);return Ok(json!(p))},
 "/v1/dictation/vocabulary"=>{let terms:Vec<String>=serde_json::from_value(body["terms"].clone()).map_err(|_|"Invalid vocabulary")?;if terms.len()>100||terms.iter().any(|v|v.trim().is_empty()||v.chars().count()>80){return Err("Vocabulary allows 100 terms of 1–80 characters".into())}data["terms"]=json!(terms);return Ok(json!({"terms":terms,"max_terms":100}))},
 "/v1/variables"=>{let vars=body.as_object().ok_or("Variables must be an object")?;if vars.len()>50||vars.iter().any(|(k,v)|k.is_empty()||k.len()>40||!k.chars().all(|c|c.is_ascii_alphanumeric()||c=='_')||v.as_str().is_none_or(|v|v.len()>4000)){return Err("Invalid snippet variables".into())}data["variables"]=body.clone();return Ok(body.clone())},
 "/v1/links"=>{let rows=body.as_array().filter(|v|v.len()<=100).ok_or("Maximum 100 links")?;if rows.iter().any(|v|v["name"].as_str().is_none_or(|v|v.is_empty()||v.len()>200)||!validate_https(v["url"].as_str().unwrap_or(""))||v["keywords"].as_str().is_some_and(|v|v.len()>1000)){return Err("Every link needs a name and HTTPS URL".into())}data["links"]=body.clone();return Ok(body.clone())},
 "/v1/dictation/snippets"=>{let input:SaveDictationSnippetRequest=serde_json::from_value(body.clone()).map_err(|_|"Invalid snippet")?;if input.trigger.trim().is_empty()||input.trigger.len()>160||input.title.len()>120||input.body.is_empty()||input.body.chars().count()>4000{return Err("Invalid snippet size".into())}let id=input.id.unwrap_or_else(uuid::Uuid::new_v4);let snippet=json!(DictationSnippet{id,title:input.title,trigger:input.trigger,body:input.body});let mut rows=data["snippets"].as_array().cloned().unwrap_or_default();rows.retain(|v|v["id"]!=id.to_string());if rows.len()>=50{return Err("Maximum 50 snippets".into())}rows.push(snippet.clone());data["snippets"]=json!(rows);return Ok(snippet)},
 "/v1/dictation/media/favorites"=>{let blockers=media_blockers();if !blockers.is_empty(){return Err(media_unavailable(&blockers))}let mut favorite=body.clone();for key in ["preview_url","content_url","source_url"]{let raw=favorite[key].as_str().unwrap_or("");if !reqwest::Url::parse(raw).is_ok_and(|u|u.scheme()=="https"&&u.host_str().is_some_and(|h|h=="giphy.com"||h.ends_with(".giphy.com"))){return Err("Invalid GIPHY URL".into())}}favorite["id"]=json!(uuid::Uuid::new_v4());let mut rows=data["favorites"].as_array().cloned().unwrap_or_default();rows.retain(|v|v["provider_id"]!=favorite["provider_id"]);if rows.len()>=30{return Err("Maximum 30 favorites".into())}rows.push(favorite.clone());data["favorites"]=json!(rows);return Ok(favorite)},_=>{}}}
 if path=="/v1/dictation/usage"&&method==Method::POST{
 let day=body["day"].as_str().ok_or("Local day is required")?;let date=chrono::NaiveDate::parse_from_str(day,"%Y-%m-%d").map_err(|_|"Invalid date")?;let age=chrono::Utc::now().date_naive().signed_duration_since(date).num_days();if !(-1..=366).contains(&age){return Err("Date out of range".into())}let words=body["words"].as_u64().filter(|v|*v<=200000).ok_or("Invalid word count")?;let ms=body["speaking_ms"].as_u64().filter(|v|*v<=86400000).ok_or("Invalid speaking time")?;if !data["days"].is_object(){data["days"]=json!({})}data["days"][day]=json!({"words":words.max(data["days"][day]["words"].as_u64().unwrap_or(0)),"speaking_ms":ms.max(data["days"][day]["speaking_ms"].as_u64().unwrap_or(0))});return Ok(usage(data))}
 if method==Method::DELETE{for (prefix,key)in [("/v1/dictation/snippets/","snippets"),("/v1/dictation/media/favorites/","favorites")]{if let Some(id)=path.strip_prefix(prefix){let mut rows=data[key].as_array().cloned().unwrap_or_default();rows.retain(|v|v["id"]!=id);data[key]=json!(rows);return Ok(json!({"deleted":true}))}}}
 Err("Unsupported endpoint or method".into())}).await;
    result.map(Json).map_err(bad)
}
async fn live(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let user = auth(&state, &headers)?;
    let data = state.store.read(&user).await.map_err(bad)?;
    if std::env::var("GEMINI_PAID_PROJECT").as_deref() != Ok("true") {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Live transcription requires a configured billing-enabled Gemini project",
        ));
    }
    let key = std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            error(
                StatusCode::SERVICE_UNAVAILABLE,
                "Live transcription is not configured",
            )
        })?;
    let rate = headers
        .get("x-destroy-audio-sample-rate")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| (8000..=48000).contains(v))
        .ok_or_else(|| bad("Unsupported sample rate"))?;
    let permit = state
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| error(StatusCode::TOO_MANY_REQUESTS, "Live transcription is busy"))?;
    let model = std::env::var("GEMINI_STT_MODEL").unwrap_or("gemini-3.5-transcribe-live".into());
    let p = profile(&data);
    let language = language(&p.language)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let terms: Vec<String> = serde_json::from_value(data["terms"].clone()).unwrap_or_default();
    Ok(ws
        .max_message_size(64 * 1024)
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            stream::handle_dictation_socket(socket, key, model, rate, language, terms).await;
        }))
}
fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(json!({"ok":true})) }))
        .route("/v1/dictation/stream", get(live))
        .route("/v1/{*path}", any(api))
        .layer(DefaultBodyLimit::max(9 * 1024 * 1024))
        .with_state(state)
}
fn parse_users(raw: &str) -> Result<HashMap<String, String>, &'static str> {
    let users: HashMap<String, String> =
        serde_json::from_str(raw).map_err(|_| "Invalid DESTROY_USERS")?;
    if users.is_empty()
        || users.iter().any(|(hash, user)| {
            hash.len() != 64
                || !hash
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                || user.trim().is_empty()
                || user.len() > 256
                || user.chars().any(char::is_control)
        })
    {
        return Err("Invalid DESTROY_USERS");
    }
    Ok(users)
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = {
        let encoded = Zeroizing::new(
            std::env::var("DESTROY_DATA_KEY")
                .map_err(|_| "Set DESTROY_DATA_KEY to a base64 encoded random 32-byte key")?,
        );
        let decoded = Zeroizing::new(STANDARD.decode(encoded.as_bytes())?);
        if decoded.len() != 32 {
            return Err("DESTROY_DATA_KEY must be 32 bytes".into());
        }
        let mut key = Zeroizing::new([0u8; 32]);
        key.copy_from_slice(&decoded);
        key
    };
    let users = parse_users(
        &std::env::var("DESTROY_USERS")
            .map_err(|_| "Set DESTROY_USERS to a JSON map of token SHA-256 hashes to user IDs")?,
    )?;
    let db = std::env::var("DESTROY_DB").unwrap_or("data/dictation.sqlite".into());
    if let Some(parent) = std::path::Path::new(&db)
        .parent()
        .filter(|p| !p.as_os_str().is_empty() && !p.exists())
    {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    let state = AppState {
        http: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(35))
            .build()?,
        store: Arc::new(storage::Store::new(&db, key)?),
        users: Arc::new(users),
        slots: Arc::new(tokio::sync::Semaphore::new(8)),
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&db, std::fs::Permissions::from_mode(0o600))?;
    }
    let listener = tokio::net::TcpListener::bind(
        std::env::var("DESTROY_LISTEN").unwrap_or("127.0.0.1:8787".into()),
    )
    .await?;
    println!("Destroy backend ready (request content logging disabled)");
    axum::serve(listener, router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    fn app() -> Router {
        let users = HashMap::from([
            (
                format!("{:x}", Sha256::digest(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
                "alice".into(),
            ),
            (
                format!("{:x}", Sha256::digest(b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
                "bob".into(),
            ),
        ]);
        router(AppState {
            http: reqwest::Client::new(),
            store: Arc::new(storage::Store::new(":memory:", Zeroizing::new([3; 32])).unwrap()),
            users: Arc::new(users),
            slots: Arc::new(tokio::sync::Semaphore::new(8)),
        })
    }
    async fn call(
        app: Router,
        method: &str,
        path: &str,
        token: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        let req = axum::http::Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(axum::body::Body::from(body.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }
    #[test]
    fn media_blockers_name_each_missing_setting() {
        let configured = |key: &str| {
            Some(
                match key {
                    "GIPHY_API_KEY" => "key",
                    _ => "true",
                }
                .to_string(),
            )
        };
        assert!(media_blockers_from(configured).is_empty());

        // The reported case: a key was set, the generated flags were left alone.
        assert_eq!(
            media_blockers_from(|key| Some(
                if key == "GIPHY_API_KEY" {
                    "key"
                } else {
                    "false"
                }
                .to_string()
            )),
            MEDIA_APPROVALS.to_vec()
        );

        // A whitespace-only key is not a key, and an absent flag is not "true".
        assert_eq!(
            media_blockers_from(|key| (key != "GIPHY_API_KEY").then(|| "true".to_string())),
            vec!["GIPHY_API_KEY"]
        );
        assert_eq!(
            media_blockers_from(|key| (key == "GIPHY_API_KEY").then(|| "   ".to_string())),
            [MEDIA_APPROVALS.as_slice(), &["GIPHY_API_KEY"]].concat()
        );

        // One unset flag is still reported on its own rather than as a bare failure.
        let one_flag_missing = |key: &str| {
            Some(
                match key {
                    "GIPHY_API_KEY" => "key",
                    "GIPHY_FAVORITES_ORDER_APPROVED" => "false",
                    _ => "true",
                }
                .to_string(),
            )
        };
        assert_eq!(
            media_blockers_from(one_flag_missing),
            vec!["GIPHY_FAVORITES_ORDER_APPROVED"]
        );

        assert!(media_unavailable(&["GIPHY_API_KEY"]).contains("GIPHY_API_KEY"));
        assert!(media_unavailable(&["GIPHY_API_KEY"]).contains("restart"));
    }

    #[tokio::test]
    async fn authentication_and_isolation() {
        let a = app();
        assert_eq!(
            call(a.clone(), "GET", "/v1/account", "invalid", Value::Null)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        let token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(
            call(
                a.clone(),
                "PUT",
                "/v1/dictation/vocabulary",
                token,
                json!({"terms":["secret"]})
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            call(
                a.clone(),
                "GET",
                "/v1/dictation/vocabulary",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                Value::Null
            )
            .await
            .1["terms"],
            json!([])
        );
        assert_eq!(
            call(a.clone(), "GET", "/v1/account/export", token, Value::Null)
                .await
                .1["terms"],
            json!(["secret"])
        );
        call(a.clone(), "DELETE", "/v1/account/data", token, Value::Null).await;
        assert_eq!(
            call(a, "GET", "/v1/account/export", token, Value::Null)
                .await
                .1,
            json!({})
        );
    }
    #[tokio::test]
    async fn usage_is_monotonic() {
        let a = app();
        let t = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let day = chrono::Utc::now().date_naive().to_string();
        for words in [20, 10, 20] {
            call(
                a.clone(),
                "POST",
                "/v1/dictation/usage",
                t,
                json!({"day":day,"words":words,"speaking_ms":100}),
            )
            .await;
        }
        assert_eq!(
            call(a, "GET", "/v1/dictation/usage", t, Value::Null)
                .await
                .1["words"],
            20
        );
    }
    #[tokio::test]
    async fn selected_edit_fails_closed() {
        let a = app();
        let t = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let mut p = DictationPreferences::default();
        p.selected_text_editing = false;
        call(a.clone(), "PUT", "/v1/dictation/preferences", t, json!(p)).await;
        assert!(!call(a,"POST","/v1/dictation/transform",t,json!({"text":"rewrite","operation":"edit_selected","app_kind":"mail","selected_text":"private text"})).await.0.is_success());
    }
    #[test]
    fn community_credentials_fail_closed() {
        assert!(parse_users("{}").is_err());
        assert!(parse_users("not json").is_err());
        for (hash, user) in [
            ("a".repeat(63), "alice"),
            ("A".repeat(64), "alice"),
            ("a".repeat(64), " "),
            ("a".repeat(64), "alice\n"),
        ] {
            assert!(parse_users(&json!({hash:user}).to_string()).is_err());
        }
        assert_eq!(
            parse_users(&json!({"a".repeat(64):"alice"}).to_string())
                .unwrap()
                .len(),
            1
        );
    }
}
