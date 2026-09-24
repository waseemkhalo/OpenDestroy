use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
};

const DEFAULT_MODEL_ID: &str = "whisper-base-en";
const MODEL_ROOT_NAME: &str = "models";
const DOWNLOAD_HEADROOM: u64 = 64 * 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 700 * 1024 * 1024;
const MAX_MODEL_BYTES: u64 = 900 * 1024 * 1024;
const WHISPER_SOURCE: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/586cef1005a81109400c1bf73c3946fa5b353458";
const PARAKEET_SOURCE: &str = "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ModelEngine {
    Whisper,
    Parakeet,
}

#[derive(Clone, Copy)]
struct Artifact {
    filename: &'static str,
    bytes: u64,
    sha256: &'static str,
    source: &'static str,
}

#[derive(Clone, Copy)]
struct ModelSpec {
    id: &'static str,
    name: &'static str,
    engine: ModelEngine,
    language: &'static str,
    artifacts: &'static [Artifact],
}

const TINY_EN: [Artifact; 1] = [Artifact {
    filename: "ggml-tiny.en.bin",
    bytes: 77_704_698,
    sha256: "8729634ed8e45db72893c34a6c671a2eef06f551eaf1056b8fa92ab45008b425",
    source: WHISPER_SOURCE,
}];
const TINY_MULTI: [Artifact; 1] = [Artifact {
    filename: "ggml-tiny.bin",
    bytes: 77_691_713,
    sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
    source: WHISPER_SOURCE,
}];
const BASE_EN: [Artifact; 1] = [Artifact {
    // This is the legacy path and digest used by the first public build.
    filename: "ggml-base.en.bin",
    bytes: 147_964_194,
    sha256: "e111a865d56afc4adf1379a2028544b3275dd25839c460953ed9a126632dcda2",
    source: WHISPER_SOURCE,
}];
const BASE_MULTI: [Artifact; 1] = [Artifact {
    filename: "ggml-base.bin",
    bytes: 147_951_465,
    sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
    source: WHISPER_SOURCE,
}];
const SMALL_EN: [Artifact; 1] = [Artifact {
    filename: "ggml-small.en.bin",
    bytes: 487_614_184,
    sha256: "cef314a500e45ace34b8f58529b983ab545b57d8d3c39b359bcabe8cb6c5a795",
    source: WHISPER_SOURCE,
}];
const SMALL_MULTI: [Artifact; 1] = [Artifact {
    filename: "ggml-small.bin",
    bytes: 487_601_967,
    sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    source: WHISPER_SOURCE,
}];
const PARAKEET: [Artifact; 5] = [
    Artifact {
        filename: "config.json",
        bytes: 97,
        sha256: "666903c76b9798caf2c210afd4f6cd60b08a8dbf9800ec8d7a3bc0d2148ac466",
        source: PARAKEET_SOURCE,
    },
    Artifact {
        filename: "decoder_joint-model.int8.onnx",
        bytes: 18_202_004,
        sha256: "eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70",
        source: PARAKEET_SOURCE,
    },
    Artifact {
        filename: "encoder-model.int8.onnx",
        bytes: 652_183_999,
        sha256: "6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09",
        source: PARAKEET_SOURCE,
    },
    Artifact {
        filename: "nemo128.onnx",
        bytes: 139_764,
        sha256: "a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f",
        source: PARAKEET_SOURCE,
    },
    Artifact {
        filename: "vocab.txt",
        bytes: 93_939,
        // Computed from the exact pinned Hugging Face revision by the runtime
        // owner; this is the artifact SHA-256, not the Git blob OID.
        sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
        source: PARAKEET_SOURCE,
    },
];

const CATALOG: [ModelSpec; 7] = [
    ModelSpec {
        id: "whisper-tiny-en",
        name: "Whisper Tiny · English",
        engine: ModelEngine::Whisper,
        language: "en",
        artifacts: &TINY_EN,
    },
    ModelSpec {
        id: "whisper-tiny-multilingual",
        name: "Whisper Tiny · multilingual",
        engine: ModelEngine::Whisper,
        language: "multilingual",
        artifacts: &TINY_MULTI,
    },
    ModelSpec {
        id: "whisper-base-en",
        name: "Whisper Base · English",
        engine: ModelEngine::Whisper,
        language: "en",
        artifacts: &BASE_EN,
    },
    ModelSpec {
        id: "whisper-base-multilingual",
        name: "Whisper Base · multilingual",
        engine: ModelEngine::Whisper,
        language: "multilingual",
        artifacts: &BASE_MULTI,
    },
    ModelSpec {
        id: "whisper-small-en",
        name: "Whisper Small · English",
        engine: ModelEngine::Whisper,
        language: "en",
        artifacts: &SMALL_EN,
    },
    ModelSpec {
        id: "whisper-small-multilingual",
        name: "Whisper Small · multilingual",
        engine: ModelEngine::Whisper,
        language: "multilingual",
        artifacts: &SMALL_MULTI,
    },
    ModelSpec {
        id: "parakeet-tdt-0.6b-v3",
        name: "NVIDIA Parakeet TDT 0.6B v3",
        engine: ModelEngine::Parakeet,
        language: "25 European languages",
        artifacts: &PARAKEET,
    },
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicModel {
    id: String,
    name: String,
    engine: String,
    language: String,
    download_bytes: u64,
    installed: bool,
    supported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicLocalModels {
    pub selected_model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub downloading_model_id: Option<String>,
    pub error: Option<String>,
    pub available_bytes: Option<u64>,
    pub models: Vec<PublicModel>,
}

#[derive(Clone)]
struct ActiveDownload {
    model_id: String,
    generation: u64,
    downloaded: u64,
    total: u64,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
struct DownloadState {
    generation: u64,
    active: Option<ActiveDownload>,
    error: Option<String>,
}

static DOWNLOAD_STATE: OnceLock<Mutex<DownloadState>> = OnceLock::new();
static LOADABLE_CACHE: OnceLock<Mutex<HashMap<String, (u64, u64, bool)>>> = OnceLock::new();
static IN_USE: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();
static MODEL_FS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn download_state() -> &'static Mutex<DownloadState> {
    DOWNLOAD_STATE.get_or_init(|| Mutex::new(DownloadState::default()))
}

fn loadable_cache() -> &'static Mutex<HashMap<String, (u64, u64, bool)>> {
    LOADABLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn in_use() -> &'static Mutex<HashMap<String, usize>> {
    IN_USE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn model_fs_lock() -> &'static Mutex<()> {
    MODEL_FS_LOCK.get_or_init(|| Mutex::new(()))
}

fn storage_root() -> PathBuf {
    crate::app_identity::config_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(MODEL_ROOT_NAME)
}

fn selected_model_path() -> PathBuf {
    crate::app_identity::config_path().with_file_name("public-local-model.json")
}

fn find_model(id: &str) -> Option<&'static ModelSpec> {
    CATALOG.iter().find(|model| model.id == id)
}

fn model_total(model: &ModelSpec) -> u64 {
    model.artifacts.iter().map(|artifact| artifact.bytes).sum()
}

fn model_path(model: &ModelSpec) -> PathBuf {
    if model.engine == ModelEngine::Whisper {
        storage_root().join(model.artifacts[0].filename)
    } else {
        storage_root().join(model.id)
    }
}

fn selected_model_id_unchecked() -> String {
    std::fs::read(selected_model_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice::<String>(&bytes).ok())
        .and_then(|id| find_model(&id).map(|_| id))
        .unwrap_or_else(|| DEFAULT_MODEL_ID.to_owned())
}

fn save_selected_model(id: &str) -> Result<(), String> {
    let path = selected_model_path();
    let parent = path.parent().ok_or("Cannot find model settings folder")?;
    std::fs::create_dir_all(parent).map_err(|_| "Cannot create model settings folder")?;
    let temp = parent.join(format!(".public-local-model-{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec(id).map_err(|_| "Cannot encode selected model")?;
    let result = (|| -> Result<(), String> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "Cannot create selected model settings")?;
        file.write_all(&bytes)
            .map_err(|_| "Cannot write selected model settings")?;
        file.sync_all()
            .map_err(|_| "Cannot sync selected model settings")?;
        std::fs::rename(&temp, &path).map_err(|_| "Cannot save selected model settings".into())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}

pub(crate) fn selected_model_id() -> String {
    selected_model_id_unchecked()
}

pub(crate) fn select_model(id: &str) -> Result<(), String> {
    let model = find_model(id).ok_or("Unknown local speech model")?;
    if let Ok(state) = download_state().lock() {
        if state.active.is_some() {
            return Err("Finish or cancel the active model download first".into());
        }
    }
    save_selected_model(model.id)
}

fn metadata_stamp(path: &Path, model: &ModelSpec) -> (u64, u64) {
    let mut size = 0_u64;
    let mut stamp = 0_u64;
    for artifact in model.artifacts {
        let path = if model.engine == ModelEngine::Whisper {
            path.to_owned()
        } else {
            path.join(artifact.filename)
        };
        if let Ok(metadata) = std::fs::metadata(path) {
            size = size.saturating_add(metadata.len());
            stamp = stamp.max(
                metadata
                    .modified()
                    .ok()
                    .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|value| value.as_nanos().min(u64::MAX as u128) as u64)
                    .unwrap_or(0),
            );
        }
    }
    (size, stamp)
}

fn artifact_path(root: &Path, model: &ModelSpec, artifact: &Artifact) -> PathBuf {
    if model.engine == ModelEngine::Whisper {
        root.to_owned()
    } else {
        root.join(artifact.filename)
    }
}

fn installed_unlocked(model: &ModelSpec) -> bool {
    let root = model_path(model);
    if model.engine == ModelEngine::Parakeet && !root.join(".destroy-managed-model").is_file() {
        return false;
    }
    model.artifacts.iter().all(|artifact| {
        std::fs::metadata(artifact_path(&root, model, artifact))
            .is_ok_and(|metadata| metadata.is_file() && metadata.len() == artifact.bytes)
    })
}

fn installed(model: &ModelSpec) -> bool {
    model_fs_lock()
        .lock()
        .is_ok_and(|_| installed_unlocked(model))
}

fn hash_file(path: &Path, expected_bytes: u64, expected_sha256: &str) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut hash = Sha256::new();
    let mut bytes = [0_u8; 64 * 1024];
    let mut total = 0_u64;
    loop {
        let Ok(read) = std::io::Read::read(&mut file, &mut bytes) else {
            return false;
        };
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > expected_bytes {
            return false;
        }
        hash.update(&bytes[..read]);
    }
    total == expected_bytes && format!("{:x}", hash.finalize()) == expected_sha256
}

fn loadable_unlocked(model: &ModelSpec) -> bool {
    if !installed_unlocked(model) {
        return false;
    }
    let path = model_path(model);
    let (size, stamp) = metadata_stamp(&path, model);
    if let Ok(cache) = loadable_cache().lock() {
        if let Some((cached_size, cached_stamp, ready)) = cache.get(model.id) {
            if *cached_size == size && *cached_stamp == stamp {
                return *ready;
            }
        }
    }
    let ready = match model.engine {
        ModelEngine::Whisper => {
            let artifact = model.artifacts[0];
            if !hash_file(&path, artifact.bytes, artifact.sha256) {
                false
            } else {
                let mut params = whisper_rs::WhisperContextParameters::default();
                params.use_gpu(true);
                whisper_rs::WhisperContext::new_with_params(&path, params).is_ok()
            }
        }
        ModelEngine::Parakeet => {
            model.artifacts.iter().all(|artifact| {
                hash_file(
                    &artifact_path(&path, model, artifact),
                    artifact.bytes,
                    artifact.sha256,
                )
            }) && crate::parakeet::validate_model(&path).is_ok()
        }
    };
    if let Ok(mut cache) = loadable_cache().lock() {
        cache.insert(model.id.to_owned(), (size, stamp, ready));
    }
    ready
}

fn loadable(model: &ModelSpec) -> bool {
    model_fs_lock()
        .lock()
        .is_ok_and(|_| loadable_unlocked(model))
}

pub(crate) fn selected_model_ready() -> bool {
    find_model(&selected_model_id_unchecked()).is_some_and(loadable)
}

pub(crate) fn selected_model_engine() -> Option<ModelEngine> {
    find_model(&selected_model_id_unchecked()).map(|model| model.engine)
}

pub(crate) struct ModelLease {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) engine: ModelEngine,
}

// Compatibility view used by the existing setup and transcription boundary.
// A caller that needs to touch a model should use snapshot_selected_model so
// removal is fenced for the full duration of the operation.
pub(crate) struct SelectedModel {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) engine: ModelEngine,
}

pub(crate) fn selected() -> Option<SelectedModel> {
    let id = selected_model_id_unchecked();
    let model = find_model(&id)?;
    Some(SelectedModel {
        id,
        path: model_path(model),
        engine: model.engine,
    })
}

pub(crate) fn selected_installed() -> bool {
    selected()
        .and_then(|selected| find_model(&selected.id).map(installed))
        .unwrap_or(false)
}

pub(crate) fn last_error() -> Option<String> {
    download_state()
        .lock()
        .ok()
        .and_then(|state| state.error.clone())
}

pub(crate) fn status() -> PublicLocalModels {
    public_status()
}

pub(crate) async fn start_download(model_id: Option<String>) -> Result<(), String> {
    begin_download(model_id)
}

pub(crate) fn select(id: &str) -> Result<(), String> {
    select_model(id)
}

pub(crate) fn remove(id: &str) -> Result<PublicLocalModels, String> {
    remove_model(id)?;
    Ok(public_status())
}

impl Drop for ModelLease {
    fn drop(&mut self) {
        if let Ok(mut models) = in_use().lock() {
            if let Some(count) = models.get_mut(&self.id) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    models.remove(&self.id);
                }
            }
        }
    }
}

pub(crate) fn snapshot_selected_model() -> Result<ModelLease, String> {
    let id = selected_model_id_unchecked();
    let model = find_model(&id).ok_or("Selected local speech model is unavailable")?;
    let _filesystem = model_fs_lock().lock().map_err(|_| "Local model is busy")?;
    if !loadable_unlocked(model) {
        return Err("Download and verify the selected local speech model before dictating".into());
    }
    let mut models = in_use().lock().map_err(|_| "Local model is busy")?;
    *models.entry(id.clone()).or_insert(0) += 1;
    Ok(ModelLease {
        id,
        path: model_path(model),
        engine: model.engine,
    })
}

fn available_bytes() -> Option<u64> {
    // The model directory is created lazily; query an existing ancestor so the
    // preflight still protects a fresh installation.
    let mut path = storage_root();
    while !path.exists() {
        let Some(parent) = path.parent() else {
            return None;
        };
        if parent == path {
            return None;
        }
        path = parent.to_owned();
    }
    #[cfg(unix)]
    {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};
        let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
            return None;
        };
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: the path is NUL-free and statvfs initializes the provided struct on success.
        if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } == 0 {
            let stats = unsafe { stats.assume_init() };
            return Some((stats.f_bavail as u64).saturating_mul(stats.f_frsize as u64));
        }
    }
    None
}

fn model_status(model: &ModelSpec) -> PublicModel {
    PublicModel {
        id: model.id.to_owned(),
        name: model.name.to_owned(),
        engine: match model.engine {
            ModelEngine::Whisper => "whisper",
            ModelEngine::Parakeet => "parakeet",
        }
        .to_owned(),
        language: model.language.to_owned(),
        download_bytes: model_total(model),
        installed: installed(model),
        supported: model.engine != ModelEngine::Parakeet
            || model
                .artifacts
                .iter()
                .all(|artifact| artifact.sha256.len() == 64),
        unavailable_reason: (model.engine == ModelEngine::Parakeet
            && model
                .artifacts
                .iter()
                .any(|artifact| artifact.sha256.len() != 64))
        .then(|| "Pinned verification metadata for this model is incomplete".to_owned()),
    }
}

pub(crate) fn public_status() -> PublicLocalModels {
    let selected = selected_model_id_unchecked();
    let (downloaded, total, downloading, error) = download_state()
        .lock()
        .map(|state| {
            let active = state.active.as_ref();
            (
                active.map_or(0, |download| download.downloaded),
                active.map_or(0, |download| download.total),
                active.map(|download| download.model_id.clone()),
                state.error.clone(),
            )
        })
        .unwrap_or((0, 0, None, Some("Local model state is busy".into())));
    PublicLocalModels {
        selected_model_id: selected,
        downloaded_bytes: downloaded,
        total_bytes: total,
        downloading_model_id: downloading,
        error,
        available_bytes: available_bytes(),
        models: CATALOG.iter().map(model_status).collect(),
    }
}

pub(crate) fn remove_model(id: &str) -> Result<(), String> {
    let model = find_model(id).ok_or("Unknown local speech model")?;
    let _filesystem = model_fs_lock().lock().map_err(|_| "Local model is busy")?;
    if let Ok(state) = download_state().lock() {
        if state
            .active
            .as_ref()
            .is_some_and(|active| active.model_id == model.id)
        {
            return Err("Cannot remove a model while it is downloading".into());
        }
    }
    if in_use()
        .lock()
        .ok()
        .and_then(|models| models.get(model.id).copied())
        .is_some_and(|count| count > 0)
    {
        return Err("Cannot remove a model while it is transcribing".into());
    }
    let path = model_path(model);
    if model.engine == ModelEngine::Parakeet {
        if !path.join(".destroy-managed-model").is_file() {
            return Err("Model is not an installed managed model".into());
        }
        std::fs::remove_dir_all(&path).map_err(|_| "Cannot remove local speech model")?;
    } else if std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_file())
    {
        std::fs::remove_file(&path).map_err(|_| "Cannot remove local speech model")?;
    }
    if let Ok(mut cache) = loadable_cache().lock() {
        cache.remove(model.id);
    }
    Ok(())
}

/// Revokes every in-flight model download before a local-data reset. The
/// generation bump makes late download completions harmless, even if the
/// worker is still unwinding after the reset has started.
pub(crate) fn cancel_all_downloads() {
    if let Ok(mut state) = download_state().lock() {
        state.generation = state.generation.wrapping_add(1);
        if let Some(active) = state.active.take() {
            active.cancel.store(true, Ordering::Release);
        }
        state.error = None;
    }
}

/// Removes only catalog-owned model paths. Unknown files and directories in
/// the models folder are deliberately left untouched.
pub(crate) fn remove_all_managed_models() -> Result<(), String> {
    let _filesystem = model_fs_lock().lock().map_err(|_| "Local model is busy")?;
    if in_use()
        .lock()
        .ok()
        .is_some_and(|models| models.values().any(|count| *count > 0))
    {
        return Err("Cannot reset while a local model is transcribing".into());
    }
    for model in CATALOG {
        let path = model_path(&model);
        if model.engine == ModelEngine::Parakeet {
            if managed_model_directory_is_safe(&model, &path) {
                std::fs::remove_dir_all(&path)
                    .map_err(|_| "Cannot remove local speech model".to_string())?;
            }
        } else if std::fs::symlink_metadata(&path)
            .is_ok_and(|metadata| metadata.file_type().is_file())
        {
            std::fs::remove_file(&path)
                .map_err(|_| "Cannot remove local speech model".to_string())?;
        }
    }
    if let Ok(mut cache) = loadable_cache().lock() {
        cache.clear();
    }
    Ok(())
}

fn managed_model_directory_is_safe(model: &ModelSpec, path: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.is_dir() {
        return false;
    }
    let marker = path.join(".destroy-managed-model");
    if !std::fs::symlink_metadata(&marker).is_ok_and(|value| value.is_file()) {
        return false;
    }
    let allowed: std::collections::HashSet<&str> = model
        .artifacts
        .iter()
        .map(|artifact| artifact.filename)
        .chain(std::iter::once(".destroy-managed-model"))
        .collect();
    std::fs::read_dir(path).is_ok_and(|entries| {
        entries.flatten().all(|entry| {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                return false;
            };
            allowed.contains(name)
                && std::fs::symlink_metadata(entry.path()).is_ok_and(|value| value.is_file())
        })
    })
}

pub(crate) fn ensure_no_model_in_use() -> Result<(), String> {
    if in_use()
        .lock()
        .ok()
        .is_some_and(|models| models.values().any(|count| *count > 0))
    {
        return Err("Cannot reset while a local model is transcribing".into());
    }
    Ok(())
}

fn update_progress(generation: u64, downloaded: u64) -> Result<(), String> {
    let mut state = download_state()
        .lock()
        .map_err(|_| "Local model state is busy")?;
    let active = state
        .active
        .as_mut()
        .ok_or("Model download is no longer active")?;
    if active.generation != generation || active.cancel.load(Ordering::Acquire) {
        return Err("Model download was cancelled".into());
    }
    active.downloaded = downloaded;
    Ok(())
}

async fn fetch_artifact(
    client: &reqwest::Client,
    artifact: Artifact,
    destination: &Path,
    generation: u64,
    offset: u64,
) -> Result<(), String> {
    if artifact.bytes > MAX_ARTIFACT_BYTES {
        return Err("Model artifact exceeds size limit".into());
    }
    if artifact.sha256.len() != 64 {
        return Err("Model artifact lacks an owner-approved SHA-256".into());
    }
    let response = client
        .get(format!(
            "{}/{filename}?download=true",
            artifact.source,
            filename = artifact.filename
        ))
        .send()
        .await
        .map_err(|_| "Model download unavailable")?;
    if !response.status().is_success() {
        return Err("Model download failed".into());
    }
    if response
        .content_length()
        .is_some_and(|size| size > artifact.bytes)
    {
        return Err("Model artifact exceeds its pinned size".into());
    }
    let temp = destination.with_extension(format!("download-{}.tmp", uuid::Uuid::new_v4()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|_| "Cannot create model download")?;
    let mut hash = Sha256::new();
    let mut current = 0_u64;
    let result = async {
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| "Model download interrupted")?;
            current = current.saturating_add(chunk.len() as u64);
            if current > artifact.bytes {
                return Err("Model artifact exceeds its pinned size".into());
            }
            hash.update(&chunk);
            file.write_all(&chunk).map_err(|_| "Cannot save model")?;
            update_progress(generation, offset.saturating_add(current))?;
        }
        file.sync_all().map_err(|_| "Cannot sync model")?;
        if current != artifact.bytes || format!("{:x}", hash.finalize()) != artifact.sha256 {
            return Err("Downloaded model checksum did not match".into());
        }
        Ok::<(), String>(())
    }
    .await;
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
        return result;
    }
    let _filesystem = model_fs_lock().lock().map_err(|_| "Local model is busy")?;
    if std::fs::symlink_metadata(destination).is_ok() {
        let _ = std::fs::remove_file(&temp);
        return Err("Model path already exists; remove it before retrying".into());
    }
    std::fs::rename(temp, destination).map_err(|_| "Cannot install model artifact".into())
}

async fn run_download(
    model: ModelSpec,
    generation: u64,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    let root = storage_root();
    std::fs::create_dir_all(&root).map_err(|_| "Cannot create model folder")?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(3))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|_| "Cannot create model downloader")?;
    let destination_root = root.join(format!(".{}.{}.tmp", model.id, uuid::Uuid::new_v4()));
    if model.engine == ModelEngine::Parakeet {
        std::fs::create_dir(&destination_root).map_err(|_| "Cannot stage model folder")?;
    }
    let mut offset = 0_u64;
    let result = async {
        for artifact in model.artifacts {
            if cancel.load(Ordering::Acquire) {
                return Err("Model download was cancelled".into());
            }
            let destination = artifact_path(&destination_root, &model, artifact);
            fetch_artifact(&client, *artifact, &destination, generation, offset).await?;
            offset = offset.saturating_add(artifact.bytes);
        }
        if cancel.load(Ordering::Acquire) {
            return Err("Model download was cancelled".into());
        }
        let _filesystem = model_fs_lock().lock().map_err(|_| "Local model is busy")?;
        if cancel.load(Ordering::Acquire) {
            return Err("Model download was cancelled".into());
        }
        let final_path = model_path(&model);
        if std::fs::symlink_metadata(&final_path).is_ok() {
            return Err("Model is already installed; remove it before retrying".into());
        }
        if model.engine == ModelEngine::Whisper {
            std::fs::rename(&destination_root, final_path)
                .map_err(|_| String::from("Cannot install local speech model"))?;
        } else {
            std::fs::write(
                destination_root.join(".destroy-managed-model"),
                b"destroy-public\n",
            )
            .map_err(|_| "Cannot finalize model folder")?;
            std::fs::rename(&destination_root, final_path)
                .map_err(|_| String::from("Cannot install local speech model"))?;
        }
        Ok::<(), String>(())
    }
    .await;
    if result.is_err() {
        if model.engine == ModelEngine::Parakeet {
            let _ = std::fs::remove_dir_all(&destination_root);
        } else {
            let _ = std::fs::remove_file(&destination_root);
        }
    }
    result
}

fn finish_download(generation: u64, result: Result<(), String>) {
    if let Ok(mut state) = download_state().lock() {
        if state
            .active
            .as_ref()
            .is_some_and(|active| active.generation == generation)
        {
            state.active = None;
            state.error = result.err();
        }
    }
}

pub(crate) fn begin_download(model_id: Option<String>) -> Result<(), String> {
    let id = model_id.unwrap_or_else(selected_model_id_unchecked);
    let model = *find_model(&id).ok_or("Unknown local speech model")?;
    if model.engine == ModelEngine::Parakeet
        && model
            .artifacts
            .iter()
            .any(|artifact| artifact.sha256.len() != 64)
    {
        return Err("Model artifact lacks an owner-approved SHA-256".into());
    }
    let total = model_total(&model);
    if total > MAX_MODEL_BYTES {
        return Err("Model exceeds the download size limit".into());
    }
    // Never replace an existing path from an async task. Explicit removal is
    // the repair flow for a same-size corrupt or stale install.
    if std::fs::symlink_metadata(model_path(&model)).is_ok() {
        return Ok(());
    }
    let mut state = download_state()
        .lock()
        .map_err(|_| "Local model state is busy")?;
    if let Some(active) = state.active.as_ref() {
        return if active.model_id == model.id {
            Ok(())
        } else {
            active.cancel.store(true, Ordering::Release);
            Err("Another local model is downloading".into())
        };
    }
    let free = available_bytes().ok_or("Cannot determine free space for this model download")?;
    if free < total.saturating_add(DOWNLOAD_HEADROOM) {
        return Err("Not enough free space for this model download".into());
    }
    state.generation = state.generation.wrapping_add(1);
    let generation = state.generation;
    let cancel = Arc::new(AtomicBool::new(false));
    state.error = None;
    state.active = Some(ActiveDownload {
        model_id: model.id.to_owned(),
        generation,
        downloaded: 0,
        total,
        cancel: cancel.clone(),
    });
    drop(state);
    tauri::async_runtime::spawn(async move {
        let result = run_download(model, generation, cancel).await;
        finish_download(generation, result);
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_immutable_verified_artifacts() {
        assert_eq!(CATALOG.len(), 7);
        for model in CATALOG {
            assert!(model_total(&model) > 0 && model_total(&model) <= MAX_MODEL_BYTES);
            for artifact in model.artifacts {
                assert!(artifact.source.contains("/resolve/") && artifact.source.contains("0"));
                assert_eq!(artifact.sha256.len(), 64);
                assert!(artifact.bytes > 0 && artifact.bytes <= MAX_ARTIFACT_BYTES);
            }
        }
    }

    #[test]
    fn model_ids_are_catalog_only_and_cannot_be_paths() {
        for id in ["../models", "whisper-base-en/../../x", "", "unknown"] {
            assert!(find_model(id).is_none());
        }
        assert!(find_model(DEFAULT_MODEL_ID).is_some());
    }

    #[test]
    fn parakeet_total_is_bounded_without_downloading_it() {
        let model = find_model("parakeet-tdt-0.6b-v3").unwrap();
        assert_eq!(model_total(model), 670_619_803);
        assert!(!installed(model));
    }
}
