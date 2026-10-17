//! EmbeddingGemma 2 on this Mac: the `Embedder` seam, the in-process runtime,
//! and the pinned model file — where it lives, when it is ready, and how it is
//! fetched (change 157).

use crate::*;
use llama_cpp_2::context::params::{LlamaContextParams, LlamaPoolingType};
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use std::num::NonZeroU32;
use std::sync::Mutex;

/// The task prompts EmbeddingGemma 2 was trained with. A vector made without
/// its prompt is a different vector, so every input is built through these.
const QUERY_PROMPT: &str = "task: search result | query: ";
const CLASSIFICATION_PROMPT: &str = "task: classification | query: ";

pub(crate) fn query_input(text: &str) -> String {
    format!("{QUERY_PROMPT}{text}")
}

pub(crate) fn document_input(title: Option<&str>, text: &str) -> String {
    let title = title.map(str::trim).filter(|title| !title.is_empty());
    format!("title: {} | text: {text}", title.unwrap_or("none"))
}

pub(crate) fn classification_input(text: &str) -> String {
    format!("{CLASSIFICATION_PROMPT}{text}")
}

/// Cosine of two vectors the embedder already normalised.
pub(crate) fn cosine(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

pub(crate) fn normalised(vector: &[f32]) -> Vec<f32> {
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        return vector.to_vec();
    }
    vector.iter().map(|x| x / norm).collect()
}

pub(crate) fn vector_to_blob(vector: &[f32]) -> Vec<u8> {
    vector.iter().flat_map(|x| x.to_le_bytes()).collect()
}

pub(crate) fn vector_from_blob(blob: &[u8]) -> Option<Vec<f32>> {
    (blob.len() == EMBEDDING_DIMENSIONS * 4).then(|| {
        blob.chunks_exact(4)
            .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Device {
    Metal,
    Cpu,
}

impl Device {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Metal => tr("GPU (Metal)"),
            Self::Cpu => tr("CPU"),
        }
    }
}

/// What turns text into vectors. Production is `SharedEmbedder`, the model on
/// this Mac; the tests use `FixtureEmbedder`, reference vectors read from
/// `tests/fixtures/embedding-golden.json`, so ranking is tested on real
/// EmbeddingGemma 2 vectors without the 310 MB model.
pub(crate) trait Embedder: Sync {
    /// One normalised `EMBEDDING_DIMENSIONS`-float vector per text, in order.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

// ---------------------------------------------------------------------------
// The model file.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelFiles {
    pub(crate) directory: PathBuf,
}

impl ModelFiles {
    pub(crate) fn installed() -> Self {
        Self {
            directory: app_data_directory().join(EMBEDDING_MODEL_DIRECTORY_NAME),
        }
    }

    pub(crate) fn model(&self) -> PathBuf {
        self.directory.join(EMBEDDING_MODEL_FILE_NAME)
    }

    pub(crate) fn part(&self) -> PathBuf {
        self.directory
            .join(format!("{EMBEDDING_MODEL_FILE_NAME}.part"))
    }

    pub(crate) fn sidecar(&self) -> PathBuf {
        self.directory
            .join(format!("{EMBEDDING_MODEL_FILE_NAME}.verified"))
    }
}

/// What the sidecar binds readiness to: the hash that matched, and the size and
/// modification time the file had when it did. A file replaced or touched
/// since is no longer the file that was hashed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VerifiedModel {
    pub(crate) sha256: String,
    pub(crate) bytes: u64,
    pub(crate) modified_nanos: u128,
}

fn modified_nanos(metadata: &fs::Metadata) -> Option<u128> {
    metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_nanos())
}

fn verified_record(path: &Path) -> Option<VerifiedModel> {
    let metadata = fs::metadata(path).ok()?;
    Some(VerifiedModel {
        sha256: EMBEDDING_MODEL_SHA256.to_owned(),
        bytes: metadata.len(),
        modified_nanos: modified_nanos(&metadata)?,
    })
}

/// Ready means the pinned file is there and is the file whose hash matched: a
/// `stat` compared with the sidecar, never a re-hash and never the sidecar alone.
pub(crate) fn model_is_ready(files: &ModelFiles) -> bool {
    let Ok(sidecar) = fs::read(files.sidecar()) else {
        return false;
    };
    let Ok(recorded) = serde_json::from_slice::<VerifiedModel>(&sidecar) else {
        return false;
    };
    recorded.sha256 == EMBEDDING_MODEL_SHA256
        && recorded.bytes == EMBEDDING_MODEL_BYTES
        && verified_record(&files.model()).as_ref() == Some(&recorded)
}

pub(crate) fn embedding_model_url() -> String {
    format!(
        "https://huggingface.co/{EMBEDDING_MODEL_REPOSITORY}/resolve/{EMBEDDING_MODEL_REVISION}/{EMBEDDING_MODEL_FILE_NAME}"
    )
}

/// The one download command. `--continue-at -` resumes a `.part` a cancelled
/// or failed fetch left, and `--max-time` bounds a child a crash orphaned.
pub(crate) fn model_download_command(files: &ModelFiles) -> Command {
    let mut command = Command::new("curl");
    command
        .args([
            "--location",
            "--fail",
            "--silent",
            "--show-error",
            "--continue-at",
            "-",
            "--max-time",
        ])
        .arg(EMBEDDING_DOWNLOAD_SECONDS.to_string())
        .arg("--output")
        .arg(files.part())
        .arg(embedding_model_url());
    command
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelFailure {
    Network,
    Disk,
    Corrupt,
    Cancelled,
    Other(String),
}

impl ModelFailure {
    pub(crate) fn reason(&self) -> String {
        match self {
            Self::Network => tr("ネットワークに接続できませんでした").to_owned(),
            Self::Disk => tr("空き容量が足りません").to_owned(),
            Self::Corrupt => tr("ファイルが壊れていたため破棄しました").to_owned(),
            Self::Cancelled => tr("中止しました").to_owned(),
            Self::Other(reason) => reason.clone(),
        }
    }
}

fn sha256_of(path: &Path) -> Result<String, ModelFailure> {
    let mut command = Command::new("shasum");
    command.args(["-a", "256"]).arg(path);
    let output = run_command_with_timeout(&mut command, Duration::from_secs(120))
        .map_err(|error| ModelFailure::Other(error.to_string()))?;
    if !output.status.success() {
        return Err(ModelFailure::Other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase())
}

/// Write the sidecar for the file now at `files.model()`. Called only after its
/// hash matched.
fn record_verified(files: &ModelFiles) -> Result<(), ModelFailure> {
    let record = verified_record(&files.model()).ok_or(ModelFailure::Disk)?;
    let bytes =
        serde_json::to_vec(&record).map_err(|error| ModelFailure::Other(error.to_string()))?;
    write_file_atomically(&files.sidecar(), &bytes).map_err(|_| ModelFailure::Disk)?;
    Ok(())
}

/// Fetch, verify, and install the pinned model. `progress` receives the bytes
/// on disk about once a second; `cancel` set by 「中止」 or app exit kills curl
/// and leaves the `.part` for the next attempt to resume.
pub(crate) fn download_model(
    files: &ModelFiles,
    cancel: &AtomicBool,
    progress: &std::sync::atomic::AtomicU64,
) -> Result<(), ModelFailure> {
    fs::create_dir_all(&files.directory).map_err(|_| ModelFailure::Disk)?;
    if model_is_ready(files) {
        return Ok(());
    }
    // A final file without a sidecar is a crash between the rename and the
    // sidecar: re-hash it once rather than fetch 310 MB again.
    if files.model().is_file() {
        if sha256_of(&files.model())? == EMBEDDING_MODEL_SHA256 {
            return record_verified(files);
        }
        let _ = fs::remove_file(files.model());
    }
    let part_bytes = || fs::metadata(files.part()).map_or(0, |metadata| metadata.len());
    if part_bytes() > EMBEDDING_MODEL_BYTES {
        let _ = fs::remove_file(files.part());
    }
    // A `.part` already at full size skips curl: a range request past the end
    // returns 416, which `--fail` reports as an error.
    if part_bytes() < EMBEDDING_MODEL_BYTES {
        let finished = AtomicBool::new(false);
        let result = thread::scope(|scope| {
            scope.spawn(|| {
                while !finished.load(Ordering::Acquire) {
                    progress.store(part_bytes(), Ordering::Release);
                    thread::sleep(Duration::from_millis(250));
                }
            });
            let result = run_command_cancellable(
                &mut model_download_command(files),
                Duration::from_secs(EMBEDDING_DOWNLOAD_SECONDS + 30),
                64 * 1024,
                64 * 1024,
                cancel,
            );
            finished.store(true, Ordering::Release);
            result
        });
        progress.store(part_bytes(), Ordering::Release);
        match result {
            Err(error) if error.downcast_ref::<CommandCancelled>().is_some() => {
                return Err(ModelFailure::Cancelled);
            }
            Err(error) => return Err(ModelFailure::Other(error.to_string())),
            Ok(output) if !output.output.status.success() => {
                // curl's exit 23 is "write error" — the disk, not the network.
                return Err(match output.output.status.code() {
                    Some(23) => ModelFailure::Disk,
                    _ => ModelFailure::Network,
                });
            }
            Ok(_) => {}
        }
    }
    if part_bytes() != EMBEDDING_MODEL_BYTES || sha256_of(&files.part())? != EMBEDDING_MODEL_SHA256
    {
        let _ = fs::remove_file(files.part());
        return Err(ModelFailure::Corrupt);
    }
    fs::rename(files.part(), files.model()).map_err(|_| ModelFailure::Disk)?;
    record_verified(files)
}

/// Remove the model and its sidecar. Knowledge databases are kept: collection
/// and search pause until the model is back.
pub(crate) fn delete_model(files: &ModelFiles) -> std::io::Result<()> {
    // Unload, not release: a later 「モデルを取得」 in this process loads again.
    unload_shared_embedder();
    for path in [files.sidecar(), files.model(), files.part()] {
        match fs::remove_file(&path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

/// Change 142 wrote a marker claiming a model it never fetched, under a
/// directory no other code uses. Removed when that marker is all it holds.
pub(crate) fn remove_change_142_placeholder() {
    let Some(dirs) = ProjectDirs::from("local", "operon", "Operon") else {
        return;
    };
    let directory = dirs
        .data_local_dir()
        .join("models")
        .join("embeddinggemma-2-270m");
    let Ok(entries) = fs::read_dir(&directory) else {
        return;
    };
    let names: Vec<_> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
        .collect();
    if names.len() == 1 && names[0] == "model_ready.json" {
        let _ = fs::remove_file(directory.join("model_ready.json"));
        let _ = fs::remove_dir(&directory);
    }
}

// ---------------------------------------------------------------------------
// The runtime.

/// The model on this Mac, loaded on first use and dropped after
/// `EMBEDDING_IDLE_UNLOAD_SECONDS` unused. Field order is drop order: the
/// model goes before the backend it was loaded through.
struct LoadedModel {
    model: LlamaModel,
    backend: LlamaBackend,
    device: Device,
    path: PathBuf,
    last_used: Instant,
}

/// One per process. A caller holds it for one call of at most
/// `EMBEDDING_TEXTS_PER_CALL` texts and `EMBEDDING_CONTEXT_TOKENS` tokens, so a
/// search waits behind at most one such call of a background run.
static SHARED_MODEL: Mutex<Option<LoadedModel>> = Mutex::new(None);

/// Set by `release_shared_embedder` at exit: no model loads after it.
/// llama.cpp's Metal device asserts at process teardown (`[rsets->data count]
/// == 0`, ggml-metal-device.m) when a model is still loaded, which turns every
/// quit into an abort; a background run that embeds once more after the
/// release would bring the model back.
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

fn shared_model() -> std::sync::MutexGuard<'static, Option<LoadedModel>> {
    SHARED_MODEL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// llama.cpp and ggml log to stderr by default, and ggml's Metal device logs
/// two dozen lines while `LlamaBackend::init` runs — before `void_logs` could
/// reach them. The knowledge command's stderr is for a person, and the app has
/// no use for either, so both loggers are switched off before the first init.
fn silence_llama_logs() {
    static SILENCED: std::sync::Once = std::sync::Once::new();
    SILENCED.call_once(|| {
        llama_cpp_2::send_logs_to_tracing(
            llama_cpp_2::LogOptions::default().with_logs_enabled(false),
        );
    });
}

fn load_model(path: &Path) -> Result<LoadedModel> {
    silence_llama_logs();
    let backend = LlamaBackend::init().context(tr("EmbeddingGemma 2 を初期化できませんでした"))?;
    let device = backend_device();
    let model = LlamaModel::load_from_file(
        &backend,
        path,
        // Every layer: the model is small, and 999 is llama.cpp's own "all".
        &LlamaModelParams::default().with_n_gpu_layers(999),
    )
    .context(tr("EmbeddingGemma 2 を読み込めませんでした"))?;
    Ok(LoadedModel {
        model,
        backend,
        device,
        path: path.to_path_buf(),
        last_used: Instant::now(),
    })
}

impl LoadedModel {
    /// Embed a prefix of `texts` that fits one call, append the vectors to
    /// `out`, and return how many texts that was (at least one).
    fn embed_prefix(&self, texts: &[String], out: &mut Vec<Vec<f32>>) -> Result<usize> {
        let mut sequences = Vec::new();
        let mut total = 0;
        for text in texts.iter().take(EMBEDDING_TEXTS_PER_CALL) {
            let mut tokens = self.model.vocab().tokenize(text.as_bytes(), true, false);
            tokens.truncate(EMBEDDING_CONTEXT_TOKENS);
            if !sequences.is_empty() && total + tokens.len() > EMBEDDING_CONTEXT_TOKENS {
                break;
            }
            total += tokens.len();
            sequences.push(tokens);
        }
        let count = sequences.len();
        // Always the full eight slots, however many texts this call holds:
        // llama.cpp 2a66703 aborts the process (`GGML_ASSERT(ggml_can_mul_mat)`)
        // when `n_seq_max` is 3, 5, 6, or 7 — measured 2026-10-17 by
        // scripts/check-embedding-fidelity.sh, whose last chunk is six texts.
        // Unused slots cost nothing in a unified KV cache.
        let params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(EMBEDDING_CONTEXT_TOKENS as u32))
            .with_n_batch(EMBEDDING_CONTEXT_TOKENS as u32)
            .with_n_ubatch(EMBEDDING_CONTEXT_TOKENS as u32)
            .with_n_seq_max(EMBEDDING_TEXTS_PER_CALL as u32)
            .with_kv_unified(true)
            .with_embeddings(true)
            .with_pooling_type(LlamaPoolingType::Mean);
        let mut context = self
            .model
            .new_context(&self.backend, params)
            .context(tr("EmbeddingGemma 2 の推論を準備できませんでした"))?;
        let mut batch = LlamaBatch::new(EMBEDDING_CONTEXT_TOKENS, EMBEDDING_TEXTS_PER_CALL as i32);
        for (sequence, tokens) in (0_i32..).zip(&sequences) {
            batch.add_sequence(tokens, sequence, false)?;
        }
        context.encode(&mut batch)?;
        for sequence in 0..count as i32 {
            out.push(normalised(context.embeddings_seq_ith(sequence)?));
        }
        Ok(count)
    }
}

/// The production `Embedder`: the shared model, loaded from `path` on first use.
pub(crate) struct SharedEmbedder {
    pub(crate) path: PathBuf,
}

impl SharedEmbedder {
    pub(crate) fn installed() -> Option<Self> {
        let files = ModelFiles::installed();
        model_is_ready(&files).then(|| Self {
            path: files.model(),
        })
    }

    fn with_model<T>(&self, work: impl FnOnce(&mut LoadedModel) -> Result<T>) -> Result<T> {
        let mut guard = shared_model();
        if SHUTTING_DOWN.load(Ordering::Acquire) {
            return Err(anyhow!(tr("中止しました")));
        }
        if guard.as_ref().is_none_or(|loaded| loaded.path != self.path) {
            *guard = None;
            *guard = Some(load_model(&self.path)?);
        }
        let Some(loaded) = guard.as_mut() else {
            return Err(anyhow!(tr("EmbeddingGemma 2 を読み込めませんでした")));
        };
        let result = work(loaded);
        loaded.last_used = Instant::now();
        result
    }
}

impl Embedder for SharedEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        while out.len() < texts.len() {
            // The lock is taken per call, not per batch of texts, so a search
            // arriving mid-run waits for one call rather than the whole run.
            let start = out.len();
            self.with_model(|loaded| loaded.embed_prefix(&texts[start..], &mut out))?;
        }
        Ok(out)
    }
}

/// Drop the shared model when it has been unused for
/// `EMBEDDING_IDLE_UNLOAD_SECONDS`. Called from the frame, once a minute, so it
/// only tries the lock: an embedding in progress means the model is in use.
pub(crate) fn release_idle_embedder() {
    let Ok(mut guard) = SHARED_MODEL.try_lock() else {
        return;
    };
    if guard.as_ref().is_some_and(|loaded| {
        loaded.last_used.elapsed() >= Duration::from_secs(EMBEDDING_IDLE_UNLOAD_SECONDS)
    }) {
        *guard = None;
    }
}

/// Which device the runtime would use, without loading the 310 MB model: the
/// loaded model's when there is one, otherwise the backend's device list.
pub(crate) fn detect_device() -> Device {
    let guard = shared_model();
    if let Some(loaded) = guard.as_ref() {
        return loaded.device;
    }
    // Under the lock, so no load can start while this backend exists: llama.cpp
    // allows one backend per process.
    silence_llama_logs();
    match LlamaBackend::init() {
        Ok(_backend) => backend_device(),
        Err(_) => Device::Cpu,
    }
}

fn backend_device() -> Device {
    // ggml names its Metal backend `MTL` (measured 2026-10-17: `["MTL", "CPU"]`).
    if llama_cpp_2::list_llama_ggml_backend_devices()
        .iter()
        .any(|device| device.backend == "MTL" || device.backend.eq_ignore_ascii_case("metal"))
    {
        Device::Metal
    } else {
        Device::Cpu
    }
}

/// Drop the loaded model; the next embedding loads it again.
pub(crate) fn unload_shared_embedder() {
    *shared_model() = None;
}

/// Unload the model for good before the process exits; see `SHUTTING_DOWN`.
/// Only the exit paths call this — after it nothing in the process embeds.
pub(crate) fn release_shared_embedder() {
    SHUTTING_DOWN.store(true, Ordering::Release);
    unload_shared_embedder();
}

#[cfg(test)]
pub(crate) fn embedder_is_shut_down() -> bool {
    SHUTTING_DOWN.load(Ordering::Acquire)
}

// ---------------------------------------------------------------------------
// The golden vectors: what the tests rank on and what the shipped runtime is
// held to.

/// `(text, unit vector)` from `tests/fixtures/embedding-golden.json`.
pub(crate) fn golden_vectors(fixture: &str) -> Result<Vec<(String, Vec<f32>)>> {
    let parsed: serde_json::Value = serde_json::from_str(fixture)?;
    let rows = parsed["vectors"]
        .as_array()
        .context("the golden fixture lists no vectors")?;
    rows.iter()
        .map(|row| {
            let text = row["text"]
                .as_str()
                .context("a golden row names no text")?
                .to_owned();
            let vector = row["vector"]
                .as_array()
                .context("a golden row carries no vector")?
                .iter()
                .map(|value| value.as_f64().map(|value| value as f32))
                .collect::<Option<Vec<f32>>>()
                .context("a golden vector holds a non-number")?;
            Ok((text, normalised(&vector)))
        })
        .collect()
}

/// `Operon knowledge embed-golden <fixture>`: embeds every golden text through
/// `embedder` and prints its cosine to the reference, lowest first. Exits 1
/// when any falls below `EMBEDDING_FIDELITY_MIN_COSINE`, 3 without a model.
/// Hidden from the usage line: `scripts/check-embedding-fidelity.sh` is its
/// only caller.
pub(crate) fn embed_golden(fixture: &Path, embedder: Option<&dyn Embedder>) -> (i32, String) {
    let Some(embedder) = embedder else {
        return (
            3,
            tr("モデルが取得されていません。Operon の設定 › データ で取得してください。")
                .to_owned(),
        );
    };
    let golden = match fs::read_to_string(fixture)
        .map_err(anyhow::Error::from)
        .and_then(|text| golden_vectors(&text))
    {
        Ok(golden) => golden,
        Err(error) => return (1, format!("{}: {error}", fixture.display())),
    };
    let mut scored = Vec::with_capacity(golden.len());
    for chunk in golden.chunks(EMBEDDING_TEXTS_PER_CALL) {
        let texts: Vec<String> = chunk.iter().map(|(text, _)| text.clone()).collect();
        let vectors = match embedder.embed(&texts) {
            Ok(vectors) => vectors,
            Err(error) => return (1, error.to_string()),
        };
        for ((text, reference), vector) in chunk.iter().zip(vectors) {
            scored.push((cosine(reference, &vector), text));
        }
    }
    scored.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut report = String::new();
    for (similarity, text) in &scored {
        let preview: String = text.chars().take(60).collect();
        report.push_str(&format!(
            "{similarity:.5}\t{}\n",
            preview.replace('\n', " ")
        ));
    }
    let lowest = scored.first().map_or(1.0, |(similarity, _)| *similarity);
    let code = if lowest < EMBEDDING_FIDELITY_MIN_COSINE {
        1
    } else {
        0
    };
    (code, report)
}

/// Reference vectors for the texts in `tests/fixtures/embedding-golden.json`,
/// computed with sentence-transformers. Any other text is an error, so a test
/// cannot quietly rank on a vector nobody computed.
#[cfg(test)]
pub(crate) struct FixtureEmbedder {
    pub(crate) vectors: HashMap<String, Vec<f32>>,
}

#[cfg(test)]
impl FixtureEmbedder {
    pub(crate) fn golden() -> Self {
        let text = include_str!("../../tests/fixtures/embedding-golden.json");
        let vectors = golden_vectors(text)
            .expect("the golden fixture parses")
            .into_iter()
            .collect();
        Self { vectors }
    }
}

#[cfg(test)]
impl Embedder for FixtureEmbedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        texts
            .iter()
            .map(|text| {
                self.vectors
                    .get(text)
                    .cloned()
                    .ok_or_else(|| anyhow!("no golden vector for {text:?}"))
            })
            .collect()
    }
}
