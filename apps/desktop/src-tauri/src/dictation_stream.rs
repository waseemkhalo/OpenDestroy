//! Authenticated hotkey-dictation PCM transport.
//!
//! The webview captures mono PCM s16le and hands short in-memory chunks to
//! these commands. Native Rust owns the authenticated websocket to mothership;
//! the Gemini credential never enters the desktop app. Audio is never written
//! to disk and every queued frame is fenced to the session that opened it.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use dictation_protocol::{
    DictationStreamClientMessage, DictationStreamServerMessage, SttPath, TranscribeResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
};
use uuid::Uuid;

const MAX_PCM_FRAME_BYTES: usize = 64 * 1024;
// Reject oversized envelopes before base64 decoding allocates their decoded
// buffer. Four input bytes encode at most three PCM bytes.
const MAX_PCM_FRAME_BASE64_BYTES: usize = ((MAX_PCM_FRAME_BYTES + 2) / 3) * 4;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(12);
const FINAL_TIMEOUT: Duration = Duration::from_secs(25);

#[derive(Debug)]
enum DictationOutbound {
    Audio(Vec<u8>),
    Finish,
    Cancel,
}

struct ActiveDictationStream {
    session_id: Uuid,
    auth: crate::auth::CallStreamSessionSnapshot,
    outbound_tx: mpsc::Sender<DictationOutbound>,
    final_rx: Option<oneshot::Receiver<Result<String, String>>>,
}

#[derive(Default)]
pub struct DictationStreamState {
    active: Option<ActiveDictationStream>,
}

pub type DictationStreamHandle = Arc<Mutex<DictationStreamState>>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationStreamStarted {
    session_id: String,
}

#[tauri::command]
pub async fn destroy_dictation_stream_start(
    _app: AppHandle,
    sample_rate_hz: u32,
    state: tauri::State<'_, DictationStreamHandle>,
    expected_user_id: Option<String>,
    expected_backend_url: Option<String>,
) -> Result<DictationStreamStarted, String> {
    if !(8_000..=48_000).contains(&sample_rate_hz) {
        return Err("Dictation audio sample rate is unsupported.".into());
    }
    crate::auth::ensure_fresh_session().await?;
    let auth = crate::auth::cached_call_stream_session_snapshot()
        .ok_or_else(|| "session is missing active team scope".to_string())?;
    crate::auth::validate_expected_account(
        &auth,
        expected_user_id.as_deref(),
        expected_backend_url.as_deref(),
    )?;
    let session_id = Uuid::new_v4();
    let (outbound_tx, outbound_rx) = mpsc::channel(64);
    let (final_tx, final_rx) = oneshot::channel();
    let (connected_tx, connected_rx) = oneshot::channel();

    {
        let mut guard = state
            .lock()
            .map_err(|_| "dictation stream state lock poisoned".to_string())?;
        if guard.active.is_some() {
            return Err("a dictation stream is already active".into());
        }
        if !crate::auth::call_stream_session_snapshot_is_current(&auth) {
            return Err("session changed during dictation start".into());
        }
        guard.active = Some(ActiveDictationStream {
            session_id,
            auth: auth.clone(),
            outbound_tx,
            final_rx: Some(final_rx),
        });
    }

    let handle = state.inner().clone();
    let auth_for_stream = auth.clone();
    tauri::async_runtime::spawn(async move {
        let result = run_stream(sample_rate_hz, auth_for_stream, outbound_rx, connected_tx).await;
        let final_result = result.map_err(|_| {
            "Live dictation was interrupted. Destroy will retry with its batch fallback."
                .to_string()
        });
        let _ = final_tx.send(final_result);
        if let Ok(mut guard) = handle.lock() {
            if guard
                .active
                .as_ref()
                .is_some_and(|active| active.session_id == session_id && active.final_rx.is_some())
            {
                guard.active = None;
            }
        }
    });

    let connected = match tokio::time::timeout(CONNECT_TIMEOUT, connected_rx).await {
        Ok(Ok(connected)) => connected,
        Ok(Err(_)) => {
            cancel_active(state.inner(), Some(session_id))?;
            return Err("Live dictation ended before connecting.".into());
        }
        Err(_) => {
            cancel_active(state.inner(), Some(session_id))?;
            return Err("Live dictation connection timed out.".into());
        }
    };
    if let Err(error) = connected {
        cancel_active(state.inner(), Some(session_id))?;
        return Err(error);
    }

    Ok(DictationStreamStarted {
        session_id: session_id.to_string(),
    })
}

#[tauri::command]
pub async fn destroy_dictation_stream_send_audio(
    session_id: String,
    audio_base64: String,
    state: tauri::State<'_, DictationStreamHandle>,
) -> Result<(), String> {
    let session_id = Uuid::parse_str(&session_id).map_err(|_| "invalid dictation session")?;
    let bytes = decode_pcm_frame(&audio_base64)?;
    if bytes.is_empty() {
        return Ok(());
    }
    let sender = authorized_sender(state.inner(), session_id)?;
    sender
        .send(DictationOutbound::Audio(bytes))
        .await
        .map_err(|_| "dictation stream closed".to_string())
}

fn decode_pcm_frame(audio_base64: &str) -> Result<Vec<u8>, String> {
    let encoded = audio_base64.trim();
    if encoded.len() > MAX_PCM_FRAME_BASE64_BYTES {
        return Err("invalid dictation audio chunk".into());
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "invalid dictation audio chunk".to_string())?;
    if bytes.len() > MAX_PCM_FRAME_BYTES || bytes.len() % 2 != 0 {
        return Err("invalid dictation audio chunk".into());
    }
    Ok(bytes)
}

#[tauri::command]
pub async fn destroy_dictation_stream_finish(
    session_id: String,
    state: tauri::State<'_, DictationStreamHandle>,
) -> Result<TranscribeResponse, String> {
    let session_id = Uuid::parse_str(&session_id).map_err(|_| "invalid dictation session")?;
    let (sender, final_rx) = {
        let mut guard = state
            .lock()
            .map_err(|_| "dictation stream state lock poisoned".to_string())?;
        let active = guard
            .active
            .as_mut()
            .filter(|active| active.session_id == session_id)
            .ok_or_else(|| "dictation stream closed".to_string())?;
        if !crate::auth::call_stream_session_snapshot_is_current(&active.auth) {
            return Err("dictation session changed".into());
        }
        let final_rx = active
            .final_rx
            .take()
            .ok_or_else(|| "dictation is already finalizing".to_string())?;
        (active.outbound_tx.clone(), final_rx)
    };

    let result = async {
        sender
            .send(DictationOutbound::Finish)
            .await
            .map_err(|_| "dictation stream closed".to_string())?;
        tokio::time::timeout(FINAL_TIMEOUT, final_rx)
            .await
            .map_err(|_| "Live dictation finalization timed out.".to_string())?
            .map_err(|_| "Live dictation connection closed.".to_string())
    }
    .await;
    clear_session(state.inner(), session_id);
    result?.map(|text| TranscribeResponse {
        text,
        path: SttPath::Cloud,
    })
}

#[tauri::command]
pub fn destroy_dictation_stream_cancel(
    session_id: Option<String>,
    state: tauri::State<'_, DictationStreamHandle>,
) -> Result<(), String> {
    let requested = session_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| "invalid dictation session")?;
    cancel_active(state.inner(), requested)
}

/// Account/team boundaries synchronously revoke queue ownership before tokens
/// are changed. The best-effort cancel control contains no customer content.
pub(crate) fn prepare_account_boundary(app: &AppHandle) -> Result<(), String> {
    let Some(state) = app.try_state::<DictationStreamHandle>() else {
        return Ok(());
    };
    cancel_active(state.inner(), None)
}

fn cancel_active(state: &DictationStreamHandle, requested: Option<Uuid>) -> Result<(), String> {
    let active = {
        let mut guard = state
            .lock()
            .map_err(|_| "dictation stream state lock poisoned".to_string())?;
        if requested.is_some_and(|requested| {
            guard
                .active
                .as_ref()
                .is_none_or(|active| active.session_id != requested)
        }) {
            return Ok(());
        }
        guard.active.take()
    };
    if let Some(active) = active {
        let _ = active.outbound_tx.try_send(DictationOutbound::Cancel);
    }
    Ok(())
}

fn clear_session(state: &DictationStreamHandle, session_id: Uuid) {
    if let Ok(mut guard) = state.lock() {
        if guard
            .active
            .as_ref()
            .is_some_and(|active| active.session_id == session_id)
        {
            guard.active = None;
        }
    }
}

fn authorized_sender(
    state: &DictationStreamHandle,
    session_id: Uuid,
) -> Result<mpsc::Sender<DictationOutbound>, String> {
    let guard = state
        .lock()
        .map_err(|_| "dictation stream state lock poisoned".to_string())?;
    let active = guard
        .active
        .as_ref()
        .filter(|active| active.session_id == session_id)
        .ok_or_else(|| "dictation stream closed".to_string())?;
    if !crate::auth::call_stream_session_snapshot_is_current(&active.auth) {
        return Err("dictation session changed".into());
    }
    Ok(active.outbound_tx.clone())
}

async fn run_stream(
    sample_rate_hz: u32,
    auth: crate::auth::CallStreamSessionSnapshot,
    mut outbound_rx: mpsc::Receiver<DictationOutbound>,
    connected_tx: oneshot::Sender<Result<(), String>>,
) -> Result<String, &'static str> {
    if !crate::auth::call_stream_session_snapshot_is_current(&auth) {
        let _ = connected_tx.send(Err("session changed before dictation connected".into()));
        return Err("session_changed");
    }
    let url = stream_url_for_base(&auth.backend_url).map_err(|_| "invalid_url")?;
    let mut request = url.into_client_request().map_err(|_| "invalid_url")?;
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", auth.access_token))
            .map_err(|_| "invalid_authorization")?,
    );
    request.headers_mut().insert(
        "x-destroy-audio-sample-rate",
        HeaderValue::from_str(&sample_rate_hz.to_string()).map_err(|_| "invalid_sample_rate")?,
    );

    let (websocket, _) = match tokio::time::timeout(CONNECT_TIMEOUT, connect_async(request)).await {
        Ok(Ok(connection)) => connection,
        Ok(Err(_)) => {
            let _ = connected_tx.send(Err("Could not connect live dictation.".into()));
            return Err("connect_failed");
        }
        Err(_) => {
            let _ = connected_tx.send(Err("Live dictation connection timed out.".into()));
            return Err("connect_timeout");
        }
    };
    let (mut writer, mut reader) = websocket.split();
    let ready = tokio::time::timeout(CONNECT_TIMEOUT, async {
        loop {
            match reader.next().await {
                Some(Ok(Message::Text(text))) => {
                    match serde_json::from_str::<DictationStreamServerMessage>(text.as_str()) {
                        Ok(DictationStreamServerMessage::Ready) => return Ok(()),
                        Ok(DictationStreamServerMessage::Error { message, .. }) => {
                            return Err(message)
                        }
                        _ => {}
                    }
                }
                Some(Ok(Message::Ping(bytes))) => {
                    writer
                        .send(Message::Pong(bytes))
                        .await
                        .map_err(|_| "Live dictation handshake failed.".to_string())?;
                }
                Some(Ok(Message::Close(_))) | None => {
                    return Err("Live dictation closed during setup.".into())
                }
                Some(Ok(_)) => {}
                Some(Err(_)) => return Err("Live dictation handshake failed.".into()),
            }
        }
    })
    .await
    .map_err(|_| "connect_timeout")?;
    match ready {
        Ok(()) => {
            if !crate::auth::call_stream_session_snapshot_is_current(&auth) {
                let _ = connected_tx.send(Err("session changed during dictation setup".into()));
                return Err("session_changed");
            }
            let _ = connected_tx.send(Ok(()));
        }
        Err(error) => {
            let _ = connected_tx.send(Err(error));
            return Err("setup_failed");
        }
    }

    loop {
        tokio::select! {
            outbound = outbound_rx.recv() => {
                let Some(outbound) = outbound else {
                    return Err("outbound_closed");
                };
                if !crate::auth::call_stream_session_snapshot_is_current(&auth) {
                    let _ = writer.close().await;
                    return Err("session_changed");
                }
                match outbound {
                    DictationOutbound::Audio(bytes) => {
                        writer.send(Message::Binary(bytes.into())).await.map_err(|_| "audio_send_failed")?;
                    }
                    DictationOutbound::Finish => {
                        let control = serde_json::to_string(&DictationStreamClientMessage::Finish)
                            .map_err(|_| "control_encode_failed")?;
                        writer.send(Message::Text(control.into())).await.map_err(|_| "finish_send_failed")?;
                    }
                    DictationOutbound::Cancel => {
                        let control = serde_json::to_string(&DictationStreamClientMessage::Cancel)
                            .map_err(|_| "control_encode_failed")?;
                        let _ = writer.send(Message::Text(control.into())).await;
                        let _ = writer.close().await;
                        return Err("cancelled");
                    }
                }
            }
            inbound = reader.next() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<DictationStreamServerMessage>(text.as_str()) {
                            Ok(DictationStreamServerMessage::Final { text }) => return Ok(text),
                            Ok(DictationStreamServerMessage::Error { .. }) => return Err("server_error"),
                            Ok(DictationStreamServerMessage::Ready | DictationStreamServerMessage::Interim { .. }) => {}
                            Err(_) => return Err("invalid_server_message"),
                        }
                    }
                    Some(Ok(Message::Ping(bytes))) => {
                        writer.send(Message::Pong(bytes)).await.map_err(|_| "pong_failed")?;
                    }
                    Some(Ok(Message::Close(_))) | None => return Err("server_closed"),
                    Some(Ok(_)) => {}
                    Some(Err(_)) => return Err("socket_read_failed"),
                }
            }
        }
    }
}

fn stream_url() -> Result<String, String> {
    stream_url_for_base(&crate::auth::backend_url())
}

fn stream_url_for_base(base: &str) -> Result<String, String> {
    let ws_base = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        return Err("unsupported mothership URL".into());
    };
    Ok(format!("{ws_base}/v1/dictation/stream"))
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;

    use super::{decode_pcm_frame, stream_url_for_base, MAX_PCM_FRAME_BASE64_BYTES};

    #[test]
    fn dictation_stream_url_uses_wss_for_production() {
        assert_eq!(
            stream_url_for_base("https://dictation.example").expect("url"),
            "wss://dictation.example/v1/dictation/stream"
        );
    }

    #[test]
    fn dictation_stream_url_uses_ws_for_local_mothership() {
        assert_eq!(
            stream_url_for_base("http://127.0.0.1:8080").expect("url"),
            "ws://127.0.0.1:8080/v1/dictation/stream"
        );
    }

    #[test]
    fn pcm_base64_limit_covers_the_largest_valid_frame() {
        let valid = vec![0u8; 64 * 1024];
        let encoded = base64::engine::general_purpose::STANDARD.encode(valid);
        assert_eq!(encoded.len(), MAX_PCM_FRAME_BASE64_BYTES);
        assert_eq!(
            decode_pcm_frame(&encoded).expect("valid frame").len(),
            64 * 1024
        );
    }

    #[test]
    fn oversized_pcm_envelope_is_rejected_before_decode() {
        let oversized = "A".repeat(MAX_PCM_FRAME_BASE64_BYTES + 1);
        assert!(decode_pcm_frame(&oversized).is_err());
    }
}
