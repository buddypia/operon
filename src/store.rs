use crate::prelude::*;

use crate::*;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Store {
    #[serde(default = "store_schema_version")]
    pub(crate) schema_version: u32,
    pub(crate) projects: Vec<Project>,
    pub(crate) sessions: Vec<Session>,
    #[serde(default)]
    pub(crate) pending_cancellations: Vec<Uuid>,
    #[serde(default)]
    pub(crate) notifications_enabled: bool,
    #[serde(default = "default_auto_approve_workspace_prompts")]
    pub(crate) auto_approve_workspace_prompts: bool,
    #[serde(default)]
    pub(crate) theme: AppTheme,
    #[serde(default)]
    pub(crate) language: Language,
    #[serde(default)]
    pub(crate) terminal_font: TerminalFont,
    #[serde(default)]
    pub(crate) native_session_references: Vec<NativeSessionReference>,
    #[serde(default)]
    pub(crate) cli_handoffs: Vec<CliHandoff>,
    #[serde(default)]
    pub(crate) agent_launch_presets: Vec<AgentLaunchPreset>,
    #[serde(default)]
    pub(crate) search_engine_mode: SearchEngineMode,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            schema_version: STORE_SCHEMA_VERSION,
            projects: Vec::new(),
            sessions: Vec::new(),
            pending_cancellations: Vec::new(),
            notifications_enabled: false,
            auto_approve_workspace_prompts: default_auto_approve_workspace_prompts(),
            theme: AppTheme::default(),
            language: Language::default(),
            terminal_font: TerminalFont::default(),
            native_session_references: Vec::new(),
            cli_handoffs: Vec::new(),
            agent_launch_presets: Vec::new(),
            search_engine_mode: SearchEngineMode::default(),
        }
    }
}

pub(crate) fn store_schema_version() -> u32 {
    STORE_SCHEMA_VERSION
}

pub(crate) fn default_auto_approve_workspace_prompts() -> bool {
    true
}

#[derive(Debug)]
pub(crate) struct LoadedStore {
    pub(crate) store: Store,
    pub(crate) data_file: PathBuf,
    pub(crate) notice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WriteOutcome {
    Durable,
    CommittedButNotSynced(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ToolStatus {
    pub(crate) git: bool,
    pub(crate) tmux: bool,
    pub(crate) codex: bool,
    pub(crate) claude: bool,
    pub(crate) antigravity: bool,
    pub(crate) gh: bool,
}

/// Whether a tool can be launched at all, resolved the way a shell resolves a
/// command name. Availability is a filesystem question, so it is answered
/// without running anything: an earlier check spawned `--version` under a
/// fixed deadline, which raced startup time rather than installation and
/// reported Node-based CLIs that need seconds to boot as missing.
pub(crate) fn tool_available(tool: &str) -> bool {
    if tool.contains('/') {
        return is_executable_file(Path::new(tool));
    }
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|directory| is_executable_file(&directory.join(tool)))
    })
}

/// Whether a tool can be launched at all, resolved the way a shell resolves a
/// command name. Availability is a filesystem question, so it is answered
/// without running anything: an earlier check spawned `--version` under a
/// fixed deadline, which raced startup time rather than installation and
/// reported Node-based CLIs that need seconds to boot as missing.
impl WriteOutcome {
    pub(crate) fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Durable, Self::Durable) => Self::Durable,
            (Self::CommittedButNotSynced(error), Self::Durable)
            | (Self::Durable, Self::CommittedButNotSynced(error)) => {
                Self::CommittedButNotSynced(error)
            }
            (Self::CommittedButNotSynced(first), Self::CommittedButNotSynced(second)) => {
                Self::CommittedButNotSynced(format!("{first}; {second}"))
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct MigrationOutcome {
    pub(crate) notice: Option<String>,
    pub(crate) durability_retry_needed: bool,
}

pub(crate) fn app_data_directory() -> PathBuf {
    ProjectDirs::from("com", "local", "operon")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".operon"))
}

pub(crate) fn app_data_file() -> PathBuf {
    app_data_directory().join("store-v2.json")
}

/// Where builds released under the previous product name kept their data. The
/// rename moved the application-support directory with it, so a first launch
/// under the new name finds an empty directory and would otherwise present a
/// Mac with existing projects as a fresh install.
pub(crate) fn legacy_brand_data_directory() -> PathBuf {
    ProjectDirs::from("com", "local", "xirp-copy")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".xirp-copy"))
}

/// Newest snapshot first. A Mac that ran both the protected index and the
/// pre-versioned one keeps the newer of the two, and every candidate is only
/// ever read: the originals stay in place as the pre-rename rollback path, the
/// same way `store.json` survives the `store-v2.json` migration.
pub(crate) fn legacy_app_data_files() -> [PathBuf; 3] {
    let legacy_brand = legacy_brand_data_directory();
    [
        legacy_brand.join("store-v2.json"),
        app_data_directory().join("store.json"),
        legacy_brand.join("store.json"),
    ]
}

pub(crate) fn import_legacy_store_if_available(current: &Path) -> Result<MigrationOutcome> {
    import_first_legacy_store(current, &legacy_app_data_files())
}

/// The first candidate that exists decides the import; later ones are not
/// consulted, so an older snapshot can never overwrite a newer one.
pub(crate) fn import_first_legacy_store(
    current: &Path,
    candidates: &[PathBuf],
) -> Result<MigrationOutcome> {
    for legacy in candidates {
        let outcome = import_legacy_store_if_needed(current, legacy)?;
        if outcome.notice.is_some() {
            return Ok(outcome);
        }
    }
    Ok(MigrationOutcome::default())
}

pub(crate) fn instance_lock_file(store_path: &Path) -> Result<PathBuf> {
    let parent = store_path
        .parent()
        .ok_or_else(|| anyhow!(tf!("データファイルに親ディレクトリがありません")))?;
    Ok(parent.join(".operon-instance.lock"))
}

#[derive(Debug)]
pub(crate) enum InstanceLockError {
    AlreadyRunning(PathBuf),
    Unavailable(String),
}

impl std::fmt::Display for InstanceLockError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning(path) => formatter.write_str(&tf!(
                "別の Operon プロセスが {path} を使用しています。",
                path = path.display()
            )),
            Self::Unavailable(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for InstanceLockError {}

/// The lock file a build released under the previous product name holds. That
/// build keeps its own index in its own application-support directory, so
/// nothing else stops it and this one from driving the same tmux sessions from
/// two records at once — the exclusion one shared directory used to give for
/// free. `None` on a Mac that never ran it: an absent directory means no other
/// index to guard, and creating one here would invent a data location.
pub(crate) fn legacy_brand_instance_lock_file() -> Option<PathBuf> {
    let directory = legacy_brand_data_directory();
    directory
        .is_dir()
        .then(|| directory.join(".xirp-copy-instance.lock"))
}

/// Every index this launch could race against, held for the process lifetime.
pub(crate) fn acquire_instance_locks(
    store_path: &Path,
) -> std::result::Result<Vec<fs::File>, InstanceLockError> {
    let mut locks = vec![acquire_instance_lock(store_path)?];
    if let Some(path) = legacy_brand_instance_lock_file() {
        locks.push(lock_file_exclusively(&path)?);
    }
    Ok(locks)
}

pub(crate) fn acquire_instance_lock(
    store_path: &Path,
) -> std::result::Result<fs::File, InstanceLockError> {
    let path = instance_lock_file(store_path)
        .map_err(|error| InstanceLockError::Unavailable(error.to_string()))?;
    let parent = path.parent().ok_or_else(|| {
        InstanceLockError::Unavailable(
            tr("インスタンスロックファイルに親ディレクトリがありません").into(),
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        InstanceLockError::Unavailable(tf!(
            "データディレクトリ {p0} を作成できませんでした: {error}",
            error = error,
            p0 = parent.display()
        ))
    })?;
    lock_file_exclusively(&path)
}

pub(crate) fn lock_file_exclusively(
    path: &Path,
) -> std::result::Result<fs::File, InstanceLockError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|error| {
            InstanceLockError::Unavailable(tf!(
                "インスタンスロック {p0} を開けませんでした: {error}",
                error = error,
                p0 = path.display()
            ))
        })?;
    // SAFETY: `file` owns a valid descriptor for the duration of this call and
    // remains stored in `OperonApp` for the lifetime of the application.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
            return Err(InstanceLockError::AlreadyRunning(path.to_path_buf()));
        }
        return Err(InstanceLockError::Unavailable(tf!(
            "{p0} をロックできませんでした: {error}",
            error = error,
            p0 = path.display()
        )));
    }
    Ok(file)
}

pub(crate) fn load_store(path: &Path) -> Result<Store> {
    if !path.exists() {
        return Ok(Store::default());
    }
    let raw = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse_store_contents(&raw)
}

pub(crate) fn parse_store_contents(raw: &[u8]) -> Result<Store> {
    let store = serde_json::from_slice::<Store>(raw).context(tr("ローカルセッション記録の解析"))?;
    if store.schema_version > STORE_SCHEMA_VERSION {
        return Err(anyhow!(tf!("保存データのスキーマ {schema_version} は、対応スキーマ {STORE_SCHEMA_VERSION} より新しいです", STORE_SCHEMA_VERSION = STORE_SCHEMA_VERSION, schema_version = store.schema_version)));
    }
    Ok(store)
}

pub(crate) fn load_store_safely(path: &Path) -> LoadedStore {
    match load_store(path) {
        Ok(store) => LoadedStore {
            store,
            data_file: path.to_path_buf(),
            notice: None,
        },
        Err(error) => {
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let stamp = format!("{}-{}", now(), Uuid::new_v4().simple());
            let corrupt_file = parent.join(format!("store.corrupt-{stamp}.json"));
            match fs::rename(path, &corrupt_file) {
                Ok(()) => LoadedStore {
                    store: Store::default(),
                    data_file: path.to_path_buf(),
                    notice: Some(tf!("保存データを読み取れなかったため、{p0} に保全しました。Operon は空の記録で起動しています。プロジェクトのファイルは変更していません。エラー: {error}", error = error, p0 = corrupt_file.display())),
                },
                Err(rename_error) => {
                    let recovery_file = parent.join(format!("store.recovery-{stamp}.json"));
                    LoadedStore {
                        store: Store::default(),
                        data_file: recovery_file.clone(),
                        notice: Some(tf!("保存データを読み取れず、移動もできなかったため、Operon は上書きしません。新しい変更は {p0} を使います。元のエラー: {error}。保全時のエラー: {rename_error}", error = error, p0 = recovery_file.display(), rename_error = rename_error)),
                    }
                }
            }
        }
    }
}

pub(crate) fn cancellation_intents_file(store_path: &Path) -> PathBuf {
    let name = store_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("store.json");
    store_path.with_file_name(format!("{name}.pending-cancellations.json"))
}

pub(crate) fn load_cancellation_intents(store_path: &Path) -> Result<Vec<Uuid>> {
    let path = cancellation_intents_file(store_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

pub(crate) fn save_cancellation_intents(
    store_path: &Path,
    session_ids: &[Uuid],
) -> Result<WriteOutcome> {
    let path = cancellation_intents_file(store_path);
    let encoded = serde_json::to_string_pretty(session_ids)?;
    write_file_atomically(&path, format!("{encoded}\n").as_bytes())
}

pub(crate) fn save_store_then_cancellation_intents(
    store_path: &Path,
    store: &Store,
) -> Result<WriteOutcome> {
    // Do not clear or replace the recovery sidecar until the authoritative
    // store snapshot is durable. If the first write fails, the previous
    // cancellation intent remains available after a restart.
    let store_outcome =
        save_store(store_path, store).context(tr("メインのセッション記録の保存"))?;
    let cancellation_outcome = save_cancellation_intents(store_path, &store.pending_cancellations)
        .context(tr("キャンセル復旧用メタデータの保存"))?;
    Ok(store_outcome.combine(cancellation_outcome))
}

pub(crate) fn save_store(path: &Path, store: &Store) -> Result<WriteOutcome> {
    let encoded = serde_json::to_string_pretty(store)?;
    write_file_atomically(path, format!("{encoded}\n").as_bytes())
}

pub(crate) fn write_file_atomically(path: &Path, contents: &[u8]) -> Result<WriteOutcome> {
    write_file_atomically_with_sync(
        path,
        contents,
        |file| file.sync_all(),
        |directory| directory.sync_all(),
    )
}

pub(crate) fn write_file_atomically_with_sync(
    path: &Path,
    contents: &[u8],
    sync_file: impl FnOnce(&fs::File) -> std::io::Result<()>,
    sync_directory: impl FnOnce(&fs::File) -> std::io::Result<()>,
) -> Result<WriteOutcome> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!(tf!("データファイルに親ディレクトリがありません")))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("store.json"),
        Uuid::new_v4()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(contents)?;
        sync_file(&file)?;
        drop(file);
        fs::rename(&temporary, path)?;
        let directory_sync =
            fs::File::open(parent).and_then(|directory| sync_directory(&directory));
        Ok::<WriteOutcome, std::io::Error>(match directory_sync {
            Ok(()) => WriteOutcome::Durable,
            Err(error) => WriteOutcome::CommittedButNotSynced(tf!(
                "コミット後に、格納先ディレクトリ {p0} を同期できませんでした: {error}",
                error = error,
                p0 = parent.display()
            )),
        })
    })();
    match result {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(error.into())
        }
    }
}

pub(crate) fn import_legacy_store_if_needed(
    current: &Path,
    legacy: &Path,
) -> Result<MigrationOutcome> {
    import_legacy_store_if_needed_with_writer(current, legacy, write_file_atomically)
}

pub(crate) fn import_legacy_store_if_needed_with_writer(
    current: &Path,
    legacy: &Path,
    mut write: impl FnMut(&Path, &[u8]) -> Result<WriteOutcome>,
) -> Result<MigrationOutcome> {
    if current.exists() || !legacy.exists() {
        return Ok(MigrationOutcome::default());
    }
    let store_bytes = fs::read(legacy).with_context(|| format!("reading {}", legacy.display()))?;
    let parsed_store = match parse_store_contents(&store_bytes) {
        Ok(store) => store,
        Err(error) => {
            let fallback_store = Store::default();
            let fallback = serde_json::to_string_pretty(&fallback_store)? + "\n";
            let current_sidecar = cancellation_intents_file(current);
            let sidecar =
                serde_json::to_string_pretty(&fallback_store.pending_cancellations)? + "\n";
            let sidecar_outcome =
                write(&current_sidecar, sidecar.as_bytes()).with_context(|| {
                    tf!(
                        "{p0} に安全なキャンセル復旧用メタデータを作成",
                        p0 = current_sidecar.display()
                    )
                })?;
            let store_outcome = write(current, fallback.as_bytes())
                .with_context(|| tf!("{p0} に安全な現在の記録を作成", p0 = current.display()))?;
            let write_outcome = sidecar_outcome.combine(store_outcome);
            let durability_retry_needed =
                matches!(write_outcome, WriteOutcome::CommittedButNotSynced(_));
            let durability_notice = match write_outcome {
                WriteOutcome::Durable => String::new(),
                WriteOutcome::CommittedButNotSynced(sync_error) => tf!(" 新しい空の記録は書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。エラー: {sync_error}", sync_error = sync_error),
            };
            return Ok(MigrationOutcome {
                notice: Some(tf!("既存のメタデータを取り込めなかったため、{p0} は変更していません。Operon は有効な復旧メタデータを備えた空の記録を作成しました。プロジェクトのファイルは変更していません。エラー: {error}。{durability_notice}", durability_notice = durability_notice, error = error, p0 = legacy.display())),
                durability_retry_needed,
            });
        }
    };

    let legacy_sidecar = cancellation_intents_file(legacy);
    let current_sidecar = cancellation_intents_file(current);
    let mut pending_cancellations = parsed_store.pending_cancellations.clone();
    let mut sidecar_notice = String::new();
    if legacy_sidecar.exists() {
        let sidecar = fs::read(&legacy_sidecar)
            .with_context(|| format!("reading {}", legacy_sidecar.display()))?;
        match serde_json::from_slice::<Vec<Uuid>>(&sidecar) {
            Ok(recovered) => pending_cancellations.extend(recovered),
            Err(error) => {
                sidecar_notice = tf!(" 旧形式のキャンセル用サイドカーが壊れていたため、{p0} は変更していません。Operon はセッション記録から有効な復旧メタデータを再構築しました。エラー: {error}。", error = error, p0 = legacy_sidecar.display());
            }
        }
    }
    let mut seen = HashSet::new();
    pending_cancellations.retain(|session_id| seen.insert(*session_id));
    let encoded_sidecar = serde_json::to_string_pretty(&pending_cancellations)? + "\n";
    let sidecar_outcome =
        write(&current_sidecar, encoded_sidecar.as_bytes()).with_context(|| {
            tf!(
                "{p0} へのキャンセル復旧の取り込み",
                p0 = current_sidecar.display()
            )
        })?;
    let store_outcome =
        write(current, &store_bytes).with_context(|| format!("importing {}", legacy.display()))?;
    let write_outcome = sidecar_outcome.combine(store_outcome);
    let durability_retry_needed = matches!(write_outcome, WriteOutcome::CommittedButNotSynced(_));
    let durability_notice = match write_outcome {
        WriteOutcome::Durable => String::new(),
        WriteOutcome::CommittedButNotSynced(error) => tf!(" 取り込みは書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。エラー: {error}", error = error),
    };
    Ok(MigrationOutcome {
        notice: Some(tf!("既存のセッションメタデータを保護された現在の記録に取り込みました。旧形式のスナップショットは {p0} のまま変更していません。{sidecar_notice}{durability_notice}", durability_notice = durability_notice, p0 = legacy.display(), sidecar_notice = sidecar_notice)),
        durability_retry_needed,
    })
}

impl ToolStatus {
    /// Resolving names on PATH answers this in microseconds, so detection runs
    /// inline and cannot report an installed tool as missing.
    pub(crate) fn detect() -> Self {
        Self {
            git: tool_available("git"),
            tmux: tool_available("tmux"),
            codex: tool_available("codex"),
            claude: tool_available("claude"),
            antigravity: tool_available(ANTIGRAVITY_COMMAND),
            gh: tool_available("gh"),
        }
    }

    pub(crate) fn agent_available(&self, agent: &str) -> bool {
        match agent {
            "codex" => self.codex,
            "claude" => self.claude,
            // Keep the old persisted ID while using Antigravity's `agy` CLI.
            "gemini" => self.antigravity,
            "custom" => true,
            _ => false,
        }
    }

    /// Names only the tools that are actually absent. Starting a session needs
    /// both tmux and the agent, and a notice that always lists both cannot say
    /// which one to install.
    pub(crate) fn missing_session_tools(&self, provider: CliProvider) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.tmux {
            missing.push("tmux");
        }
        if !self.agent_available(provider.agent()) {
            missing.push(provider.executable());
        }
        missing
    }

    pub(crate) fn available_agent_count(&self) -> usize {
        [self.codex, self.claude, self.antigravity]
            .into_iter()
            .filter(|available| *available)
            .count()
    }
}

/// A combination of launch options a person named and kept. It stores the
/// pieces — model, mode, effort, flag IDs — rather than the command line those
/// pieces build, so a preset saved today still launches correctly after a CLI
/// renames a flag and `agent_flag_options` is updated to match.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentLaunchPreset {
    pub(crate) id: Uuid,
    pub(crate) agent: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) mode: String,
    #[serde(default)]
    pub(crate) effort: String,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
}
