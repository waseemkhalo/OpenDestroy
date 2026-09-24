# Local speech models

The public desktop app's local provider uses NVIDIA Parakeet TDT 0.6B v3 through the Rust `transcribe-rs` ONNX engine. The model is loaded in-process from an installed directory; the inference path does not install Python, start a sidecar, or fetch model/runtime files.

## Reviewed Parakeet artifact

The pinned export is [`istupakov/parakeet-tdt-0.6b-v3-onnx`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx) at commit `8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce`. Install these four files into a directory named `parakeet-tdt-0.6b-v3-int8`:

| File | Immutable URL | Size (bytes) | SHA-256 |
|---|---|---:|---|
| `encoder-model.int8.onnx` | [`resolve/8f23f0c/encoder-model.int8.onnx`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/encoder-model.int8.onnx) | 652,183,999 | `6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09` |
| `decoder_joint-model.int8.onnx` | [`resolve/8f23f0c/decoder_joint-model.int8.onnx`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/decoder_joint-model.int8.onnx) | 18,202,004 | `eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70` |
| `nemo128.onnx` | [`resolve/8f23f0c/nemo128.onnx`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/nemo128.onnx) | 139,764 | `a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f` |
| `vocab.txt` | [`resolve/8f23f0c/vocab.txt`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/vocab.txt) | 93,939 | `d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d` |

The four files total 670,619,706 bytes (about 640 MiB) before filesystem overhead. The first three hashes are the SHA-256 LFS object IDs reported by the Hugging Face model API at the pinned commit. `vocab.txt` is a small Git blob; its Git object ID is `fc43e1c723e262df60b70e1919614417162d1fe2`, while the table records its independently computed SHA-256 file digest. Do not substitute the unquantized ONNX files or a mutable `main` URL.

The catalog may additionally retain the upstream `config.json` as a 97-byte manifest marker (`666903c76b9798caf2c210afd4f6cd60b08a8dbf9800ec8d7a3bc0d2148ac466`); `transcribe-rs` does not read it. If that marker is downloaded, the managed bundle total is 670,619,803 bytes. It must use the same pinned URL: [`config.json`](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/config.json).

The model card declares CC BY 4.0 for the ONNX export and its NVIDIA base model. Preserve attribution and review the [Creative Commons terms](https://creativecommons.org/licenses/by/4.0/) before redistributing model files. The model is multilingual (25 languages in the upstream card); the app's current local setup should describe only the languages and quality actually accepted by the product.

## Runtime and offline behavior

`transcribe-rs = 0.3.11` uses `ort = 2.0.0-rc.12` with its CPU ONNX Runtime backend. The default Cargo build downloads and verifies a prebuilt ONNX Runtime archive at build time, statically links the core runtime, and copies the platform dynamic libraries needed by the native binary on macOS. It does not create a model/runtime download dependency for an installed app. The model itself remains an explicit, separately checked resource and is not checked into this repository.

For a fully offline native build, pre-populate Cargo's cache with the reviewed ONNX Runtime archive or provide an installed ONNX Runtime library directory through `ORT_LIB_LOCATION` (or `ORT_LIB_PATH`) before compiling. Do not set `ORT_PREFER_DYNAMIC_LINK` for the public bundle unless the packaging step also places and verifies the resulting `libonnxruntime.dylib` inside the app. A runtime model/network request is not an acceptable fallback for local mode.

The native wrapper's `validate_model` call loads the selected directory into
one reusable native slot after install or selection. Repeated status reads do
not load or hash every model. An in-flight transcription keeps that
slot/path snapshot, so changing a selection cannot silently redirect an active
request. The source intentionally makes no Apple GPU acceleration claim; CPU
speed, memory use, and battery impact must be measured on release hardware.

The public Parakeet path supports CPU inference on supported macOS Apple Silicon builds. This is not a claim of NVIDIA GPU or CUDA support: NVIDIA is the model publisher/family, while this manifest intentionally enables only the CPU ONNX execution provider. Measure real devices before making speed, memory, or battery claims.

## Development prerequisites

The Parakeet ONNX path itself does not require CMake. This repository may still require CMake when compiling the existing Whisper/whisper.cpp dependency. A normal native development machine must provide Rust, Xcode Command Line Tools, and CMake for that complete dependency graph; use a current toolchain compatible with the locked crates. If a developer already has an approved CMake installation, expose it through the normal `PATH` or the build system's `CMAKE` setting. Do not document a user-specific absolute path.

Do not download the Parakeet model as part of a build or automated test. The opt-in fixture smoke should run only after a reviewer approves the roughly 640 MiB resource and verifies free disk space; structural tests must remain model-free.

## Verified opt-in fixture smoke

On September 13, 2026, the four pinned files above were downloaded once into
`/private/tmp/destroy-public-parakeet-smoke` and all four SHA-256 digests
matched this document. The public `jfk.wav` fixture came from the official
[`ggerganov/whisper.cpp` samples directory](https://github.com/ggerganov/whisper.cpp/blob/master/samples/jfk.wav).
The ignored test
`parakeet::tests::transcribes_the_approved_public_fixture_with_the_native_adapter`
loaded the directory through the same `parakeet::transcribe` adapter and
returned a non-empty transcript:
“And so, my fellow Americans, ask not what your country can do for you. Ask
what you can do for your country.”

The smoke was allowed with 2,954,288 KiB free before download and 2,285,924
KiB after download; the post-test measurement was 2,201,944 KiB, remaining
above the 1.5 GiB safety floor. This is fixture evidence for loading and CPU
inference only, not a claim about production speech quality or hardware
performance.

The temporary smoke download was cleaned up after verification; the model was
not installed into the user model store.
