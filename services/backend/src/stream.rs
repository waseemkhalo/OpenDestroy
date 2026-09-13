use axum::extract::ws::{Message, WebSocket};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dictation_protocol::{DictationStreamClientMessage, DictationStreamServerMessage};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as ProviderMessage;
const MAX_DICTATION_STREAM_SECONDS: usize = 130;
const MAX_DICTATION_STREAM_FRAME_BYTES: usize = 64 * 1024;
const GEMINI_CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
const GEMINI_SETUP_TIMEOUT: Duration = Duration::from_secs(8);
const GEMINI_FINAL_TIMEOUT: Duration = Duration::from_secs(20);
const GEMINI_LIVE_URL: &str="wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent";
fn gemini_setup_message(model: &str, language: Option<&str>, keyterms: &[String]) -> String {
    let language_codes = language.map_or_else(Vec::new, |value| vec![value]);
    serde_json::json!({
        "setup": {
            "model": format!("models/{model}"),
            "generationConfig": {
                "responseModalities": ["TEXT"]
            },
            "realtimeInputConfig": {
                "automaticActivityDetection": {
                    "disabled": true
                }
            },
            "inputAudioTranscription": {
                "languageCodes": language_codes,
                "customVocabulary": keyterms,
                "mode": "VERBATIM"
            }
        }
    })
    .to_string()
}

pub async fn handle_dictation_socket(
    desktop: WebSocket,
    api_key: String,
    model: String,
    sample_rate: u32,
    language: Option<String>,
    keyterms: Vec<String>,
) {
    if let Err(category) = run_dictation_socket(
        desktop,
        api_key.trim(),
        &model,
        sample_rate,
        language.as_deref(),
        &keyterms,
    )
    .await
    {
        tracing::warn!(%category, %model, "Gemini live dictation relay ended");
    }
}

async fn run_dictation_socket(
    desktop: WebSocket,
    api_key: &str,
    model: &str,
    sample_rate: u32,
    language: Option<&str>,
    keyterms: &[String],
) -> Result<(), &'static str> {
    let provider_url = format!("{GEMINI_LIVE_URL}?key={}", urlencoding::encode(api_key));
    let connection = tokio::time::timeout(
        GEMINI_CONNECT_TIMEOUT,
        tokio_tungstenite::connect_async(provider_url),
    )
    .await
    .map_err(|_| "connect_timeout")?
    .map_err(|_| "connect_failed")?;
    let (mut provider, _) = connection;
    provider
        .send(ProviderMessage::Text(
            gemini_setup_message(model, language, keyterms).into(),
        ))
        .await
        .map_err(|_| "setup_send_failed")?;

    tokio::time::timeout(GEMINI_SETUP_TIMEOUT, async {
        loop {
            let message = provider
                .next()
                .await
                .ok_or("setup_closed")?
                .map_err(|_| "setup_read_failed")?;
            if provider_message_json(&message)
                .is_some_and(|json| json.get("setupComplete").is_some())
            {
                return Ok::<(), &'static str>(());
            }
            if matches!(message, ProviderMessage::Close(_)) {
                return Err("setup_closed");
            }
        }
    })
    .await
    .map_err(|_| "setup_timeout")??;

    provider
        .send(ProviderMessage::Text(
            serde_json::json!({
                "realtimeInput": { "activityStart": {} }
            })
            .to_string()
            .into(),
        ))
        .await
        .map_err(|_| "activity_start_failed")?;

    let (mut desktop_tx, mut desktop_rx) = desktop.split();
    let (mut provider_tx, mut provider_rx) = provider.split();
    send_dictation_message(&mut desktop_tx, &DictationStreamServerMessage::Ready)
        .await
        .map_err(|_| "desktop_ready_failed")?;

    let max_audio_bytes = usize::try_from(sample_rate)
        .unwrap_or(48_000)
        .saturating_mul(2)
        .saturating_mul(MAX_DICTATION_STREAM_SECONDS);
    let mut audio_bytes = 0usize;
    let mut finishing = false;
    let mut final_segments: Vec<String> = Vec::new();
    let session_deadline = tokio::time::sleep(Duration::from_secs(
        u64::try_from(MAX_DICTATION_STREAM_SECONDS).unwrap_or(130),
    ));
    tokio::pin!(session_deadline);
    let final_deadline = tokio::time::sleep(GEMINI_FINAL_TIMEOUT);
    tokio::pin!(final_deadline);

    loop {
        tokio::select! {
            _ = &mut session_deadline => {
                let _ = send_dictation_error(
                    &mut desktop_tx,
                    "dictation_too_long",
                    "Dictation is limited to two minutes.",
                ).await;
                return Err("session_timeout");
            }
            _ = &mut final_deadline, if finishing => {
                let _ = send_dictation_error(
                    &mut desktop_tx,
                    "transcribe_timeout",
                    "Gemini did not finalize the dictation in time.",
                ).await;
                return Err("final_timeout");
            }
            desktop_message = desktop_rx.next() => {
                match desktop_message {
                    Some(Ok(Message::Binary(bytes))) if !finishing => {
                        if bytes.is_empty() {
                            continue;
                        }
                        if bytes.len() > MAX_DICTATION_STREAM_FRAME_BYTES
                            || bytes.len() % 2 != 0
                            || audio_bytes.saturating_add(bytes.len()) > max_audio_bytes
                        {
                            let _ = send_dictation_error(
                                &mut desktop_tx,
                                "invalid_audio",
                                "Dictation audio was invalid or too long.",
                            ).await;
                            return Err("invalid_audio");
                        }
                        audio_bytes = audio_bytes.saturating_add(bytes.len());
                        let provider_message = serde_json::json!({
                            "realtimeInput": {
                                "audio": {
                                    "data": STANDARD.encode(&bytes),
                                    "mimeType": format!("audio/pcm;rate={sample_rate}")
                                }
                            }
                        });
                        provider_tx
                            .send(ProviderMessage::Text(provider_message.to_string().into()))
                            .await
                            .map_err(|_| "audio_forward_failed")?;
                    }
                    Some(Ok(Message::Text(text))) => {
                        let control = serde_json::from_str::<DictationStreamClientMessage>(text.as_str())
                            .map_err(|_| "invalid_control")?;
                        match control {
                            DictationStreamClientMessage::Finish if !finishing => {
                                if audio_bytes == 0 {
                                    let _ = send_dictation_error(
                                        &mut desktop_tx,
                                        "empty_audio",
                                        "No speech was captured.",
                                    ).await;
                                    return Err("empty_audio");
                                }
                                provider_tx
                                    .send(ProviderMessage::Text(
                                        serde_json::json!({
                                            "realtimeInput": { "activityEnd": {} }
                                        })
                                        .to_string()
                                        .into(),
                                    ))
                                    .await
                                    .map_err(|_| "activity_end_failed")?;
                                finishing = true;
                                final_deadline.as_mut().reset(tokio::time::Instant::now() + GEMINI_FINAL_TIMEOUT);
                            }
                            DictationStreamClientMessage::Finish => {}
                            DictationStreamClientMessage::Cancel => return Ok(()),
                        }
                    }
                    Some(Ok(Message::Ping(bytes))) => {
                        desktop_tx.send(Message::Pong(bytes)).await.map_err(|_| "desktop_pong_failed")?;
                    }
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Ok(_)) => {}
                    Some(Err(_)) => return Err("desktop_read_failed"),
                }
            }
            provider_message = provider_rx.next() => {
                match provider_message {
                    Some(Ok(message @ ProviderMessage::Text(_)))
                    | Some(Ok(message @ ProviderMessage::Binary(_))) => {
                        let Some(json) = provider_message_json(&message) else {
                            continue;
                        };
                        let (interim, final_text, turn_complete) = gemini_transcription_event(&json);
                        if let Some(interim) = interim {
                            let _ = send_dictation_message(
                                &mut desktop_tx,
                                &DictationStreamServerMessage::Interim { text: interim },
                            ).await;
                        }
                        if let Some(final_text) = final_text {
                            if final_segments.last() != Some(&final_text) {
                                final_segments.push(final_text);
                            }
                            if finishing {
                                return finish_dictation(&mut desktop_tx, &final_segments).await;
                            }
                        }
                        if finishing && turn_complete {
                            return finish_dictation(&mut desktop_tx, &final_segments).await;
                        }
                    }
                    Some(Ok(ProviderMessage::Ping(bytes))) => {
                        provider_tx.send(ProviderMessage::Pong(bytes)).await.map_err(|_| "provider_pong_failed")?;
                    }
                    Some(Ok(ProviderMessage::Close(_))) | None => {
                        let _ = send_dictation_error(
                            &mut desktop_tx,
                            "provider_closed",
                            "Gemini closed the dictation session before returning text.",
                        ).await;
                        return Err("provider_closed");
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => return Err("provider_read_failed"),
                }
            }
        }
    }
}

fn provider_message_json(message: &ProviderMessage) -> Option<serde_json::Value> {
    match message {
        ProviderMessage::Text(text) => serde_json::from_str(text.as_str()).ok(),
        ProviderMessage::Binary(bytes) => serde_json::from_slice(bytes).ok(),
        _ => None,
    }
}

fn gemini_transcription_event(json: &serde_json::Value) -> (Option<String>, Option<String>, bool) {
    let Some(content) = json.get("serverContent") else {
        return (None, None, false);
    };
    let text = |field: &str| {
        content
            .get(field)
            .and_then(|value| value.get("text"))
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    (
        text("interimInputTranscription"),
        text("inputTranscription"),
        content
            .get("turnComplete")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    )
}

async fn finish_dictation(
    desktop_tx: &mut SplitSink<WebSocket, Message>,
    segments: &[String],
) -> Result<(), &'static str> {
    let text = segments.join(" ").trim().to_string();
    if text.is_empty() {
        send_dictation_error(desktop_tx, "empty_transcript", "No speech was recognized.")
            .await
            .map_err(|_| "desktop_final_failed")?;
        return Err("empty_transcript");
    }
    send_dictation_message(desktop_tx, &DictationStreamServerMessage::Final { text })
        .await
        .map_err(|_| "desktop_final_failed")
}

async fn send_dictation_error(
    desktop_tx: &mut SplitSink<WebSocket, Message>,
    code: &str,
    message: &str,
) -> Result<(), axum::Error> {
    send_dictation_message(
        desktop_tx,
        &DictationStreamServerMessage::Error {
            code: code.to_string(),
            message: message.to_string(),
        },
    )
    .await
}

async fn send_dictation_message(
    desktop_tx: &mut SplitSink<WebSocket, Message>,
    message: &DictationStreamServerMessage,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message).expect("dictation server message serializes");
    desktop_tx.send(Message::Text(text.into())).await
}
