//! Global microphone capture without a background WebKit permission request.
//! Audio is bounded, memory-only PCM; native and frontend session IDs must agree.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use std::sync::{Mutex, MutexGuard};
use tauri::{Manager, State};

const SAMPLE_RATE: u32 = 16_000;
const MAX_BYTES: usize = SAMPLE_RATE as usize * 2 * 120;
const CALLBACK_STALL: std::time::Duration = std::time::Duration::from_secs(2);

fn audio_health_error(
    valid_audio_seen: bool,
    last_valid_audio_age: std::time::Duration,
    failed: bool,
    at_capacity: bool,
) -> Option<&'static str> {
    if failed {
        Some("Microphone capture was interrupted. Check the input device and try again.")
    } else if at_capacity {
        None
    } else if !valid_audio_seen && last_valid_audio_age >= CALLBACK_STALL {
        Some("No audio is arriving from the microphone. Check macOS Microphone permission and your selected input device, then try again.")
    } else if valid_audio_seen && last_valid_audio_age >= CALLBACK_STALL {
        Some("Microphone audio stopped arriving. Check that your input device is connected, then try again.")
    } else {
        None
    }
}
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|error| error.into_inner())
}
fn matches_owner(current: Option<&str>, requested: &str) -> bool {
    current == Some(requested)
}
fn wav(pcm: &[u8]) -> Result<Vec<u8>, String> {
    if pcm.is_empty() {
        return Err(
            "No microphone audio was captured. Check System default input in macOS Sound settings."
                .into(),
        );
    }
    if pcm.len() > MAX_BYTES || pcm.len() % 2 != 0 {
        return Err("Microphone audio exceeded its safe recording limit.".into());
    }
    let mut bytes = Vec::with_capacity(44 + pcm.len());
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(pcm);
    Ok(bytes)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::{
        ffi::c_void,
        ptr,
        sync::atomic::{AtomicBool, AtomicU32, Ordering},
        time::Instant,
    };
    // Layouts/signatures from the macOS SDK AudioQueue.h and CoreAudioTypes.h.
    #[repr(C)]
    struct Format {
        rate: f64,
        id: u32,
        flags: u32,
        bytes_packet: u32,
        frames_packet: u32,
        bytes_frame: u32,
        channels: u32,
        bits: u32,
        reserved: u32,
    }
    #[repr(C)]
    struct Buffer {
        capacity: u32,
        data: *mut c_void,
        size: u32,
        user: *mut c_void,
        packet_capacity: u32,
        packets: *mut c_void,
        packet_count: u32,
    }
    type Queue = *mut c_void;
    #[link(name = "AudioToolbox", kind = "framework")]
    extern "C" {
        fn AudioQueueNewInput(
            format: *const Format,
            callback: unsafe extern "C" fn(
                *mut c_void,
                Queue,
                *mut Buffer,
                *const c_void,
                u32,
                *const c_void,
            ),
            user: *mut c_void,
            run_loop: *const c_void,
            mode: *const c_void,
            flags: u32,
            queue: *mut Queue,
        ) -> i32;
        fn AudioQueueAllocateBuffer(queue: Queue, size: u32, buffer: *mut *mut Buffer) -> i32;
        fn AudioQueueEnqueueBuffer(
            queue: Queue,
            buffer: *mut Buffer,
            packets: u32,
            descriptions: *const c_void,
        ) -> i32;
        fn AudioQueueStart(queue: Queue, time: *const c_void) -> i32;
        fn AudioQueueStop(queue: Queue, immediate: u8) -> i32;
        fn AudioQueueDispose(queue: Queue, immediate: u8) -> i32;
    }
    struct Samples {
        pcm: Mutex<Vec<u8>>,
        recording: AtomicBool,
        level: AtomicU32,
        failed: AtomicBool,
        valid_buffers: AtomicU32,
        last_valid_audio: Mutex<Instant>,
        at_capacity: AtomicBool,
    }
    impl Drop for Samples {
        fn drop(&mut self) {
            lock(&self.pcm).fill(0);
        }
    }
    pub(super) struct Recording {
        pub id: String,
        pub lease: String,
        queue: Queue,
        samples: Option<Box<Samples>>,
    }
    // AudioQueue control calls are serialized by NativeAudio's mutex. Its own
    // callback only uses a stable heap allocation, atomics and the PCM mutex.
    unsafe impl Send for Recording {}
    unsafe extern "C" fn input(
        user: *mut c_void,
        queue: Queue,
        buffer: *mut Buffer,
        _: *const c_void,
        _: u32,
        _: *const c_void,
    ) {
        // SAFETY: AudioQueue owns buffer; Samples remains allocated until synchronous disposal.
        let samples = unsafe { &*(user as *const Samples) };
        if !samples.recording.load(Ordering::Acquire) {
            return;
        }
        let buffer_ref = unsafe { &*buffer };
        if buffer_ref.data.is_null() || buffer_ref.size > buffer_ref.capacity {
            samples.failed.store(true, Ordering::Release);
            return;
        }
        let data = unsafe {
            std::slice::from_raw_parts(buffer_ref.data as *const u8, buffer_ref.size as usize)
        };
        let mut pcm = lock(&samples.pcm);
        let count = data.len().min(MAX_BYTES.saturating_sub(pcm.len())) & !1;
        pcm.extend_from_slice(&data[..count]);
        if count > 0 {
            *lock(&samples.last_valid_audio) = Instant::now();
            samples.valid_buffers.fetch_add(1, Ordering::AcqRel);
        }
        let total: f32 = data[..count]
            .chunks_exact(2)
            .map(|b| {
                let v = i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0;
                v * v
            })
            .sum();
        samples.level.store(
            (total / (count / 2).max(1) as f32).sqrt().to_bits(),
            Ordering::Release,
        );
        let full = pcm.len() == MAX_BYTES;
        if full {
            samples.at_capacity.store(true, Ordering::Release);
        }
        drop(pcm);
        if !full && samples.recording.load(Ordering::Acquire) {
            if unsafe { AudioQueueEnqueueBuffer(queue, buffer, 0, ptr::null()) } != 0 {
                samples.failed.store(true, Ordering::Release);
            }
        }
    }
    impl Recording {
        pub fn start(id: String) -> Result<Self, String> {
            let mut samples = Box::new(Samples {
                pcm: Mutex::new(Vec::with_capacity(MAX_BYTES)),
                recording: AtomicBool::new(true),
                level: AtomicU32::new(0),
                failed: AtomicBool::new(false),
                valid_buffers: AtomicU32::new(0),
                last_valid_audio: Mutex::new(Instant::now()),
                at_capacity: AtomicBool::new(false),
            });
            let format = Format {
                rate: SAMPLE_RATE as f64,
                id: u32::from_be_bytes(*b"lpcm"),
                flags: (1 << 2) | (1 << 3),
                bytes_packet: 2,
                frames_packet: 1,
                bytes_frame: 2,
                channels: 1,
                bits: 16,
                reserved: 0,
            };
            let mut queue = ptr::null_mut();
            // Null runloop uses AudioQueue's internal callback thread, not the WebView.
            let result = unsafe {
                AudioQueueNewInput(
                    &format,
                    input,
                    (&mut *samples as *mut Samples).cast(),
                    ptr::null(),
                    ptr::null(),
                    0,
                    &mut queue,
                )
            };
            if result != 0 || queue.is_null() {
                return Err(format!("Could not open the system microphone (macOS {result}). Check Sound input and Microphone permission."));
            }
            let recording = Self {
                id,
                lease: uuid::Uuid::new_v4().to_string(),
                queue,
                samples: Some(samples),
            };
            for _ in 0..3 {
                let mut buffer = ptr::null_mut();
                if unsafe { AudioQueueAllocateBuffer(queue, 2048, &mut buffer) } != 0
                    || buffer.is_null()
                {
                    return Err("Could not allocate microphone audio buffers.".into());
                }
                if unsafe { AudioQueueEnqueueBuffer(queue, buffer, 0, ptr::null()) } != 0 {
                    return Err("Could not prepare microphone audio buffers.".into());
                }
            }
            let status = unsafe { AudioQueueStart(queue, ptr::null()) };
            if status != 0 {
                return Err(format!("Could not start the system microphone (macOS {status}). Check Sound input and Microphone permission."));
            }
            Ok(recording)
        }
        pub fn level(&self) -> f32 {
            self.samples
                .as_ref()
                .map_or(0.0, |s| f32::from_bits(s.level.load(Ordering::Acquire)))
        }
        pub fn health_error(&self) -> Option<&'static str> {
            let Some(samples) = self.samples.as_ref() else {
                return Some("Microphone capture stopped. Check the input device and try again.");
            };
            audio_health_error(
                samples.valid_buffers.load(Ordering::Acquire) > 0,
                lock(&samples.last_valid_audio).elapsed(),
                samples.failed.load(Ordering::Acquire),
                samples.at_capacity.load(Ordering::Acquire),
            )
        }
        fn stop(&mut self) -> Result<(), String> {
            if self.queue.is_null() {
                return Ok(());
            }
            self.samples
                .as_ref()
                .unwrap()
                .recording
                .store(false, Ordering::Release);
            // Synchronous stop/dispose finish callbacks before Samples is freed.
            unsafe {
                AudioQueueStop(self.queue, 1);
            }
            if unsafe { AudioQueueDispose(self.queue, 1) } != 0 {
                // An unexpected disposal failure must not free callback userdata.
                // No more buffers are enqueued once recording is false.
                if let Some(samples) = self.samples.take() {
                    let mut pcm = lock(&samples.pcm);
                    pcm.fill(0);
                    pcm.clear();
                    drop(pcm);
                    let _ = Box::into_raw(samples);
                }
                self.queue = ptr::null_mut();
                return Err(
                    "Could not close microphone capture. Restart Destroy before recording again."
                        .into(),
                );
            }
            self.queue = ptr::null_mut();
            Ok(())
        }
        pub fn finish(mut self) -> Result<Vec<u8>, String> {
            self.stop()?;
            let samples = self.samples.as_ref().unwrap();
            if samples.failed.load(Ordering::Acquire) {
                return Err(
                    "Microphone capture was interrupted. Check the system input and try again."
                        .into(),
                );
            }
            wav(&lock(&samples.pcm))
        }
    }
    impl Drop for Recording {
        fn drop(&mut self) {
            let _ = self.stop();
        }
    }
}
#[cfg(not(target_os = "macos"))]
mod platform {
    pub(super) struct Recording {
        pub id: String,
        pub lease: String,
    }
    impl Recording {
        pub fn start(_: String) -> Result<Self, String> {
            Err("Native microphone capture is available on macOS only.".into())
        }
        pub fn finish(self) -> Result<Vec<u8>, String> {
            Err("Native microphone capture is available on macOS only.".into())
        }
        pub fn level(&self) -> f32 {
            0.0
        }
        pub fn health_error(&self) -> Option<&'static str> {
            None
        }
    }
}
#[derive(Default)]
pub struct NativeAudio {
    recording: Mutex<Option<platform::Recording>>,
    starting: Mutex<Option<String>>,
}
impl NativeAudio {
    pub(crate) fn health_error(&self, id: &str) -> Option<String> {
        lock(&self.recording)
            .as_ref()
            .filter(|recording| recording.id == id)
            .and_then(platform::Recording::health_error)
            .map(str::to_owned)
    }
}
impl NativeAudio {
    fn expire(&self, lease: &str) {
        let mut current = lock(&self.recording);
        if matches_owner(current.as_ref().map(|r| r.lease.as_str()), lease) {
            let recording = current.take();
            drop(current);
            drop(recording);
        }
    }
    pub(crate) fn cancel(&self, id: &str) {
        {
            let mut starting = lock(&self.starting);
            if starting.as_deref() == Some(id) {
                *starting = None;
            }
        }
        let mut current = lock(&self.recording);
        if matches_owner(current.as_ref().map(|r| r.id.as_str()), id) {
            let recording = current.take();
            drop(current);
            drop(recording);
        }
    }
    pub(crate) fn clear(&self) {
        *lock(&self.starting) = None;
        let recording = lock(&self.recording).take();
        drop(recording);
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    audio_base64: String,
    mime_type: &'static str,
}

#[tauri::command]
pub fn native_audio_start(
    app: tauri::AppHandle,
    audio: State<'_, NativeAudio>,
    session_id: String,
    device_id: Option<String>,
) -> Result<(), String> {
    if device_id
        .as_deref()
        .is_some_and(|id| !id.trim().is_empty() && id != "default")
    {
        return Err("Choose System default in Destroy’s Microphone settings for native dictation. Saved browser device IDs are not native microphone IDs.".into());
    }
    if !crate::permissions::microphone_granted() {
        return Err(
            "Allow Destroy in macOS Privacy & Security → Microphone, then try again.".into(),
        );
    }
    if !app
        .state::<crate::dictation::DictationSession>()
        .is_active(&session_id)
    {
        return Err("That dictation session was cancelled".into());
    }
    {
        let mut starting = lock(&audio.starting);
        if lock(&audio.recording).is_some() || starting.is_some() {
            return Err("A microphone recording is already active.".into());
        }
        *starting = Some(session_id.clone());
    }
    let started = platform::Recording::start(session_id.clone());
    {
        let mut starting = lock(&audio.starting);
        if starting.as_deref() == Some(&session_id) {
            *starting = None;
        }
    }
    let recording = started?;
    if !app
        .state::<crate::dictation::DictationSession>()
        .is_active(&session_id)
    {
        return Err("That dictation session was cancelled".into());
    }
    let lease = recording.lease.clone();
    let mut current = lock(&audio.recording);
    if current.is_some() {
        return Err("A microphone recording is already active.".into());
    }
    *current = Some(recording);
    drop(current);
    tauri::async_runtime::spawn(async move {
        // Keep the bounded final recording available if the webview's 119s
        // stop timer was throttled while the app was in the background.
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
        app.state::<NativeAudio>().expire(&lease);
    });
    Ok(())
}
#[tauri::command]
pub fn native_audio_finish(
    app: tauri::AppHandle,
    audio: State<'_, NativeAudio>,
    session_id: String,
) -> Result<Audio, String> {
    let mut current = lock(&audio.recording);
    if !matches_owner(current.as_ref().map(|r| r.id.as_str()), &session_id) {
        return Err("The microphone recording ended or was cancelled. Try dictating again.".into());
    }
    let recording = current.take().unwrap();
    drop(current);
    if !app
        .state::<crate::dictation::DictationSession>()
        .is_active(&session_id)
    {
        return Err("That dictation session was cancelled".into());
    }
    let mut bytes = recording.finish()?;
    if !app
        .state::<crate::dictation::DictationSession>()
        .is_active(&session_id)
    {
        bytes.fill(0);
        return Err("That dictation session was cancelled".into());
    }
    let audio_base64 = STANDARD.encode(&bytes);
    bytes.fill(0);
    Ok(Audio {
        audio_base64,
        mime_type: "audio/wav",
    })
}
#[tauri::command]
pub fn native_audio_cancel(audio: State<'_, NativeAudio>, session_id: String) {
    audio.cancel(&session_id);
}
#[tauri::command]
pub fn native_audio_level(audio: State<'_, NativeAudio>, session_id: String) -> f32 {
    lock(&audio.recording)
        .as_ref()
        .filter(|r| r.id == session_id)
        .map_or(0.0, platform::Recording::level)
}

#[tauri::command]
pub fn native_audio_health(audio: State<'_, NativeAudio>, session_id: String) -> Option<String> {
    audio.health_error(&session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wave_header_and_samples_are_correct() {
        let result = wav(&[1, 0, 255, 127]).unwrap();
        assert_eq!(&result[..4], b"RIFF");
        assert_eq!(&result[8..16], b"WAVEfmt ");
        assert_eq!(&result[24..28], &16000u32.to_le_bytes());
        assert_eq!(&result[40..44], &4u32.to_le_bytes());
        assert_eq!(&result[44..], &[1, 0, 255, 127]);
    }
    #[test]
    fn recordings_are_bounded_and_complete_samples_only() {
        assert!(wav(&[]).is_err());
        assert!(wav(&[1]).is_err());
        assert!(wav(&vec![0; MAX_BYTES + 2]).is_err());
        assert!(wav(&vec![0; MAX_BYTES]).is_ok());
    }
    #[test]
    fn stale_owner_cannot_finish_or_cancel_a_new_recording() {
        assert!(!matches_owner(Some("new"), "old"));
        assert!(!matches_owner(None, "old"));
        assert!(matches_owner(Some("new"), "new"));
    }
    #[test]
    fn empty_capture_is_rejected_before_audio_can_be_encoded() {
        assert!(wav(&[]).unwrap_err().contains("No microphone audio"));
    }
    #[test]
    fn startup_without_valid_audio_is_reported_after_grace_period() {
        assert!(audio_health_error(false, CALLBACK_STALL, false, false)
            .unwrap()
            .contains("No audio is arriving"));
    }
    #[test]
    fn active_silent_samples_are_healthy_but_a_later_stall_is_not() {
        assert_eq!(
            audio_health_error(true, std::time::Duration::ZERO, false, false),
            None
        );
        assert!(audio_health_error(true, CALLBACK_STALL, false, false)
            .unwrap()
            .contains("stopped arriving"));
    }
    #[test]
    fn empty_callbacks_do_not_count_as_valid_audio_and_capacity_is_not_a_stall() {
        assert!(audio_health_error(false, CALLBACK_STALL, false, false).is_some());
        assert_eq!(audio_health_error(true, CALLBACK_STALL, false, true), None);
    }
    #[test]
    fn cancelling_hung_startup_releases_its_reservation() {
        let audio = NativeAudio::default();
        *lock(&audio.starting) = Some("stalled-start".into());
        audio.cancel("stalled-start");
        assert!(lock(&audio.starting).is_none());
    }
}
