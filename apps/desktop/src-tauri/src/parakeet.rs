//! Native NVIDIA Parakeet TDT 0.6B v3 inference.
//!
//! The model is the reviewed ONNX int8 export, loaded by `transcribe-rs` and
//! ONNX Runtime.  The public setup layer owns model discovery/download policy;
//! this module deliberately only accepts an already-installed model directory
//! and never fetches anything at runtime.

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use transcribe_rs::{
    onnx::{
        parakeet::{ParakeetModel, ParakeetParams, TimestampGranularity},
        Quantization,
    },
    TranscribeError,
};

const SAMPLE_RATE: usize = 16_000;
const MAX_AUDIO_SAMPLES: usize = SAMPLE_RATE * 180;

const REQUIRED_FILES: &[&str] = &[
    "encoder-model.int8.onnx",
    "decoder_joint-model.int8.onnx",
    "nemo128.onnx",
    "vocab.txt",
];

struct LoadedModel {
    path: PathBuf,
    model: ParakeetModel,
}

static MODEL: OnceLock<Mutex<Option<LoadedModel>>> = OnceLock::new();

fn model_slot() -> &'static Mutex<Option<LoadedModel>> {
    MODEL.get_or_init(|| Mutex::new(None))
}

fn required_files_present(model_dir: &Path) -> Result<(), String> {
    if !model_dir.is_dir() {
        return Err(format!(
            "Parakeet model directory is missing: {}",
            model_dir.display()
        ));
    }
    for name in REQUIRED_FILES {
        let path = model_dir.join(name);
        if !path.is_file() {
            return Err(format!("Parakeet model is missing {name}"));
        }
        if std::fs::metadata(&path)
            .map_err(|_| format!("Parakeet model metadata is unavailable: {name}"))?
            .len()
            == 0
        {
            return Err(format!("Parakeet model file is empty: {name}"));
        }
    }
    Ok(())
}

fn load_model(model_dir: &Path) -> Result<ParakeetModel, String> {
    ParakeetModel::load(model_dir, &Quantization::Int8).map_err(format_load_error)
}

fn format_load_error(error: TranscribeError) -> String {
    format!("Cannot load Parakeet model: {error}")
}

/// Validate and load the immutable int8 Parakeet model once into the reusable
/// inference slot. Call this at install/selection boundaries, not from a
/// status poll. A second validation of the same canonical path is a cheap
/// cache hit; changing models replaces the one loaded slot.
pub fn validate_model(model_dir: &Path) -> Result<(), String> {
    required_files_present(model_dir)?;
    let path = model_dir
        .canonicalize()
        .map_err(|_| "Cannot resolve Parakeet model directory".to_string())?;
    let mut guard = model_slot()
        .lock()
        .map_err(|_| "Parakeet model is busy".to_string())?;
    if guard.as_ref().is_none_or(|loaded| loaded.path != path) {
        *guard = Some(LoadedModel {
            path: path.clone(),
            model: load_model(&path)?,
        });
    }
    Ok(())
}

/// Transcribe 16 kHz mono f32 audio with the installed Parakeet model.
///
/// The model directory must contain the four reviewed int8 ONNX artifacts.
/// Inference is serialized and the loaded ONNX sessions are reused between
/// utterances.  No network access or runtime model download occurs here.
pub fn transcribe(model_dir: &Path, samples: &[f32]) -> Result<String, String> {
    if samples.is_empty() {
        return Err("Parakeet audio is empty".into());
    }
    if samples.len() > MAX_AUDIO_SAMPLES {
        return Err("Parakeet audio exceeds the local 3-minute limit".into());
    }
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err("Parakeet audio contains a non-finite sample".into());
    }

    // Cheap input guards run before model loading, so malformed input cannot
    // cause an expensive ONNX load.
    validate_model(model_dir)?;

    let path = model_dir
        .canonicalize()
        .map_err(|_| "Cannot resolve Parakeet model directory".to_string())?;
    let mut guard = model_slot()
        .lock()
        .map_err(|_| "Parakeet model is busy".to_string())?;
    if guard.as_ref().is_none_or(|loaded| loaded.path != path) {
        *guard = Some(LoadedModel {
            path: path.clone(),
            model: load_model(&path)?,
        });
    }
    let loaded = guard
        .as_mut()
        .ok_or_else(|| "Parakeet model is unavailable".to_string())?;
    let result = loaded
        .model
        .transcribe_with(
            samples,
            &ParakeetParams {
                timestamp_granularity: Some(TimestampGranularity::Segment),
                ..Default::default()
            },
        )
        .map_err(format_load_error)?;
    let text = result.text.trim().to_owned();
    if text.is_empty() {
        return Err("Parakeet returned no transcription".into());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn validates_the_reviewed_int8_directory_shape() {
        let root = tempfile_dir();
        for name in REQUIRED_FILES {
            fs::write(root.join(name), []).unwrap();
        }
        let error = validate_model(&root).unwrap_err();
        assert!(error.contains("empty"));
        fs::remove_file(root.join(REQUIRED_FILES[0])).unwrap();
        for name in &REQUIRED_FILES[1..] {
            fs::remove_file(root.join(name)).unwrap();
        }
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn rejects_empty_oversized_and_non_finite_audio_before_runtime_load() {
        let missing = Path::new("/definitely/missing/parakeet");
        assert_eq!(
            transcribe(missing, &[]).unwrap_err(),
            "Parakeet audio is empty"
        );
        let oversized = vec![0.0; MAX_AUDIO_SAMPLES + 1];
        assert_eq!(
            transcribe(missing, &oversized).unwrap_err(),
            "Parakeet audio exceeds the local 3-minute limit"
        );
        assert_eq!(
            transcribe(missing, &[f32::NAN]).unwrap_err(),
            "Parakeet audio contains a non-finite sample"
        );
    }

    #[test]
    #[ignore = "opt-in: set DESTROY_PARAKEET_SMOKE_MODEL and DESTROY_PARAKEET_SMOKE_WAV"]
    fn transcribes_the_approved_public_fixture_with_the_native_adapter() {
        let model_dir = std::env::var_os("DESTROY_PARAKEET_SMOKE_MODEL")
            .map(PathBuf::from)
            .expect("DESTROY_PARAKEET_SMOKE_MODEL must point to the verified model directory");
        let wav_path = std::env::var_os("DESTROY_PARAKEET_SMOKE_WAV")
            .map(PathBuf::from)
            .expect("DESTROY_PARAKEET_SMOKE_WAV must point to the approved public fixture");
        let wav = fs::read(&wav_path).expect("open public WAV fixture");
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        let mut cursor = 12;
        let mut audio_format = 0u16;
        let mut channels = 0u16;
        let mut sample_rate = 0u32;
        let mut data = None;
        while cursor + 8 <= wav.len() {
            let kind = &wav[cursor..cursor + 4];
            let size = u32::from_le_bytes(wav[cursor + 4..cursor + 8].try_into().unwrap()) as usize;
            let end = cursor + 8 + size;
            assert!(end <= wav.len(), "public WAV chunk exceeds file");
            if kind == b"fmt " {
                audio_format = u16::from_le_bytes(wav[cursor + 8..cursor + 10].try_into().unwrap());
                channels = u16::from_le_bytes(wav[cursor + 10..cursor + 12].try_into().unwrap());
                sample_rate = u32::from_le_bytes(wav[cursor + 12..cursor + 16].try_into().unwrap());
            } else if kind == b"data" {
                data = Some(&wav[cursor + 8..end]);
            }
            cursor = end + (size & 1);
        }
        assert_eq!(audio_format, 1);
        assert_eq!(channels, 1);
        assert_eq!(sample_rate, SAMPLE_RATE as u32);
        let bytes = data.expect("public WAV data chunk");
        let samples: Vec<f32> = bytes
            .chunks_exact(2)
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]) as f32 / i16::MAX as f32)
            .collect();
        let transcript = transcribe(&model_dir, &samples).expect("native Parakeet transcript");
        assert!(!transcript.trim().is_empty(), "native transcript was empty");
        eprintln!("Parakeet public-fixture transcript: {transcript}");
    }

    fn tempfile_dir() -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("destroy-parakeet-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        path
    }
}
