//! Honest runtime boundary for builds without the ARM-only ONNX dependency.
use std::path::Path;
const UNAVAILABLE: &str =
    "Parakeet requires Apple Silicon in this release. Choose Whisper or cloud speech.";
pub fn validate_model(_model_dir: &Path) -> Result<(), String> {
    Err(UNAVAILABLE.into())
}
pub fn transcribe(_model_dir: &Path, _samples: &[f32]) -> Result<String, String> {
    Err(UNAVAILABLE.into())
}
