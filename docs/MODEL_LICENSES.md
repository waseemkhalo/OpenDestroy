# Local model license and provenance inventory

This inventory describes the seven model entries currently declared in `apps/desktop/src-tauri/src/local_models.rs`. It records upstream metadata and observed download behavior; it is not a legal opinion or a release clearance.

## Distribution boundary

The repository and normal app build contain catalog metadata only; model weights and runtime artifacts are not checked into the source tree. The app downloads a selected model on demand into the user's app-support model store, then checks the pinned byte length and SHA-256 before loading it. A downloaded artifact is user-local runtime data, not a bundled project file. An installer or release that bundles model files would be a different distribution and needs a fresh owner review.

The catalog currently has six Whisper single-file artifacts and a five-file Parakeet directory. `config.json` in the Parakeet entry is a 97-byte manifest marker; the native adapter does not require it (see [LOCAL_MODELS.md](LOCAL_MODELS.md)). The model smoke test checked download, digest, and loading behavior only; it did not establish rights or permission to redistribute the artifacts.

## Whisper

The catalog pins files from the `ggerganov/whisper.cpp` Hugging Face repository at revision `586cef1005a81109400c1bf73c3946fa5b353458`. The pinned repository metadata labels the files MIT. The OpenAI Whisper project and the whisper.cpp upstream repository also publish MIT licensing information for Whisper code and model weights. Those are upstream declarations, not a clearance determination for this project.

| Catalog ID | Artifact | Pinned source | Bytes | SHA-256 | Upstream metadata |
| --- | --- | --- | ---: | --- | --- |
| `whisper-tiny-en` | `ggml-tiny.en.bin` | [pinned file](https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458/ggml-tiny.en.bin) | 77,704,698 | `8729634ed8e45db72893c34a6c671a2eef06f551eaf1056b8fa92ab45008b425` | MIT (upstream label) |
| `whisper-tiny-multilingual` | `ggml-tiny.bin` | [pinned file](https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458/ggml-tiny.bin) | 77,691,713 | `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21` | MIT (upstream label) |
| `whisper-base-en` | `ggml-base.en.bin` | [pinned file](https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458/ggml-base.en.bin) | 147,964,194 | `e111a865d56afc4adf1379a2028544b3275dd25839c460953ed9a126632dcda2` | MIT (upstream label) |
| `whisper-base-multilingual` | `ggml-base.bin` | [pinned file](https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458/ggml-base.bin) | 147,951,465 | `60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe` | MIT (upstream label) |
| `whisper-small-en` | `ggml-small.en.bin` | [pinned file](https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458/ggml-small.en.bin) | 487,614,184 | `cef314a500e45ace34b8f58529b983ab545b57d8d3c39b359bcabe8cb6c5a795` | MIT (upstream label) |
| `whisper-small-multilingual` | `ggml-small.bin` | [pinned file](https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458/ggml-small.bin) | 487,601,967 | `1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b` | MIT (upstream label) |

The MIT labels above are upstream metadata. This review did not independently establish the full chain of custody from OpenAI's original weights to each exact converted blob, or retain a permission and attribution record for this repository to redistribute these files. The hashes establish byte identity only; do not infer legal clearance from a label or hash.

Authoritative references: [pinned whisper.cpp model repository](https://huggingface.co/ggerganov/whisper.cpp/tree/586cef1005a81109400c1bf73c3946fa5b353458), [OpenAI Whisper repository](https://github.com/openai/whisper), and [whisper.cpp MIT license](https://github.com/ggml-org/whisper.cpp/blob/master/LICENSE).

## Parakeet

The catalog pins a five-file ONNX export from `istupakov/parakeet-tdt-0.6b-v3-onnx` at revision `8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce`. Its pinned repository metadata labels the export CC BY 4.0, identifies `nvidia/parakeet-tdt-0.6b-v3` as the base model, and describes the ONNX conversion for onnx-asr. The NVIDIA base model card also labels the base model CC BY 4.0. These are upstream declarations, not this project's clearance; the file-level provenance of the export, tokenizer/vocabulary, config, and any third-party components was not independently verified.

| Artifact | Pinned source | Bytes | SHA-256 | Upstream metadata |
| --- | --- | ---: | --- | --- |
| `config.json` | [pinned file](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/config.json) | 97 | `666903c76b9798caf2c210afd4f6cd60b08a8dbf9800ec8d7a3bc0d2148ac466` | CC BY 4.0 (pinned export metadata) |
| `decoder_joint-model.int8.onnx` | [pinned file](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/decoder_joint-model.int8.onnx) | 18,202,004 | `eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70` | CC BY 4.0 (pinned export metadata) |
| `encoder-model.int8.onnx` | [pinned file](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/encoder-model.int8.onnx) | 652,183,999 | `6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09` | CC BY 4.0 (pinned export metadata) |
| `nemo128.onnx` | [pinned file](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/nemo128.onnx) | 139,764 | `a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f` | CC BY 4.0 (pinned export metadata) |
| `vocab.txt` | [pinned file](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/vocab.txt) | 93,939 | `d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d` | CC BY 4.0 (pinned export metadata) |

The four required runtime files total 670,619,706 bytes; including the optional `config.json` marker totals 670,619,803 bytes. The table preserves the catalog's observed bytes and hashes, not a conclusion that every component is covered by one license.

Authoritative references: [pinned ONNX export](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/tree/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce), [pinned export README](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/blob/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce/README.md), [NVIDIA base model card](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3), and [CC BY 4.0 legal code](https://creativecommons.org/licenses/by/4.0/legalcode).

## Owner decisions before release

- Confirm exact source provenance and rights/attribution for every catalog artifact.
- Decide whether the user-download-only path is acceptable under the upstream terms; do not bundle model files until that review is complete.
- If a binary later bundles models, preserve the required license and attribution notices and re-review every component's terms.
- Keep this inventory factual and avoid describing any artifact as legally cleared.
