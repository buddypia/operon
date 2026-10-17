//! The app's side of the knowledge base (change 157): the model's state and
//! download, the schedule that runs the pipeline, the ナレッジ tab, and the
//! 「意味で一致」 group in ローカル履歴. Everything that touches a file, the
//! model, or a CLI runs as a background task; the frame reads what came back.

use crate::*;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

pub(crate) struct ModelDownloadHandle {
    pub(crate) cancel: Arc<AtomicBool>,
    pub(crate) progress: Arc<AtomicU64>,
    /// Set by the download thread once `download_model` has returned, which
    /// is after its `curl` was killed and reaped.
    pub(crate) finished: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KnowledgeSort {
    #[default]
    Freshness,
    References,
    Newest,
}

impl KnowledgeSort {
    pub(crate) const ALL: [Self; 3] = [Self::Freshness, Self::References, Self::Newest];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Freshness => tr("鮮度"),
            Self::References => tr("参照回数"),
            Self::Newest => tr("新しい順"),
        }
    }
}

/// One project's tab: the last snapshot, and the order its rows are drawn in,
/// computed when the snapshot or the sort changes rather than every frame.
#[derive(Debug, Clone, Default)]
pub(crate) struct KnowledgeView {
    pub(crate) snapshot: Option<Arc<KnowledgeSnapshot>>,
    pub(crate) error: Option<String>,
    pub(crate) order: Vec<usize>,
    pub(crate) order_key: Option<(KnowledgeSort, Option<KnowledgeKind>)>,
}

impl KnowledgeView {
    fn ordered(&mut self, sort: KnowledgeSort, kind: Option<KnowledgeKind>) -> &[usize] {
        if self.order_key != Some((sort, kind)) {
            self.order = self
                .snapshot
                .as_deref()
                .map(|snapshot| knowledge_order(&snapshot.entries, sort, kind, now()))
                .unwrap_or_default();
            self.order_key = Some((sort, kind));
        }
        &self.order
    }
}

/// The rows' order for a sort and a kind filter.
pub(crate) fn knowledge_order(
    entries: &[KnowledgeEntry],
    sort: KnowledgeSort,
    kind: Option<KnowledgeKind>,
    now: u64,
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..entries.len())
        .filter(|index| kind.is_none_or(|kind| entries[*index].kind == kind))
        .collect();
    match sort {
        KnowledgeSort::Freshness => order.sort_by(|left, right| {
            entries[*right]
                .freshness(now)
                .total_cmp(&entries[*left].freshness(now))
        }),
        KnowledgeSort::References => {
            order.sort_by_key(|index| Reverse(entries[*index].reference_count))
        }
        KnowledgeSort::Newest => order.sort_by_key(|index| Reverse(entries[*index].created_at)),
    }
    order
}

/// What a person did to a project's knowledge from the tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KnowledgeEdit {
    Enable,
    Disable,
    Clear,
    Pin(i64, bool),
    Delete(i64),
    Reference(i64),
    RetryFailed,
}

/// Apply one edit and read the project back. Only `Enable` creates a
/// database: a project that never opted in gets no file.
pub(crate) fn apply_knowledge_edit(
    root: &Path,
    project_id: Uuid,
    edit: KnowledgeEdit,
    now: u64,
) -> Result<KnowledgeSnapshot> {
    let path = knowledge_database_path(root, project_id);
    let database = match edit {
        KnowledgeEdit::Enable => Some(KnowledgeDatabase::open(&path)?),
        _ => KnowledgeDatabase::open_existing(&path)?,
    };
    if let Some(mut database) = database {
        match edit {
            KnowledgeEdit::Enable => database.set_enabled(true)?,
            KnowledgeEdit::Disable => database.set_enabled(false)?,
            KnowledgeEdit::Clear => database.clear_and_disable()?,
            KnowledgeEdit::Pin(entry, pinned) => database.set_pinned(entry, pinned)?,
            KnowledgeEdit::Delete(entry) => database.delete_entry(entry)?,
            KnowledgeEdit::Reference(entry) => database.record_references(&[entry], now)?,
            KnowledgeEdit::RetryFailed => {
                database.retry_failed()?;
            }
        }
    }
    knowledge_snapshot(root, project_id, now)
}

/// The line the tab offers an agent, built from the executable that is
/// running and the same constant `src/main.rs` dispatches on.
pub(crate) fn knowledge_command_line(executable: &Path) -> String {
    let program = executable.display().to_string();
    let program = if program.contains([' ', '\'', '"']) {
        format!("'{}'", program.replace('\'', r"'\''"))
    } else {
        program
    };
    format!("{program} {KNOWLEDGE_CLI_SUBCOMMAND} search \"…\"")
}

/// When the next scheduled run starts: 30 s after launch, then every ten
/// minutes.
pub(crate) fn next_knowledge_run(after: Instant, first: bool) -> Instant {
    after
        + Duration::from_secs(if first {
            KNOWLEDGE_RUN_FIRST_DELAY_SECONDS
        } else {
            KNOWLEDGE_RUN_INTERVAL_SECONDS
        })
}

pub(crate) struct KnowledgeState {
    pub(crate) root: PathBuf,
    pub(crate) model_files: ModelFiles,
    pub(crate) model: ModelState,
    pub(crate) download: Option<ModelDownloadHandle>,
    /// Set when 「意味検索」 or 「モデルを取得して始める」 was chosen before the
    /// model was there: what to do once it is.
    pub(crate) select_semantic_when_ready: bool,
    pub(crate) enable_when_ready: Option<Uuid>,
    pub(crate) views: HashMap<Uuid, KnowledgeView>,
    pub(crate) progress: Arc<Mutex<Option<(Uuid, usize, usize)>>>,
    pub(crate) next_run: Instant,
    pub(crate) next_idle_check: Instant,
    pub(crate) query: String,
    pub(crate) hits: Option<(Uuid, String, Vec<KnowledgeHit>)>,
    pub(crate) open_entry: Option<(Uuid, i64)>,
    pub(crate) confirm_clear: Option<Uuid>,
    pub(crate) sort: KnowledgeSort,
    pub(crate) kind_filter: Option<KnowledgeKind>,
    pub(crate) session_hits: Vec<IndexedSessionHit>,
    pub(crate) command_line: String,
}

impl KnowledgeState {
    /// Beside the store: `data_file`'s directory is the app data directory, so
    /// the app and `Operon knowledge search` read the same databases, and a
    /// test's temporary store gets a temporary knowledge base.
    pub(crate) fn beside(data_file: &Path) -> Self {
        let data_directory = data_file.parent().unwrap_or_else(|| Path::new("."));
        let executable = std::env::current_exe().unwrap_or_else(|_| PathBuf::from(APP_NAME));
        Self {
            root: data_directory.join(KNOWLEDGE_DIRECTORY_NAME),
            model_files: ModelFiles {
                directory: data_directory.join(EMBEDDING_MODEL_DIRECTORY_NAME),
            },
            model: ModelState::Missing,
            download: None,
            select_semantic_when_ready: false,
            enable_when_ready: None,
            views: HashMap::new(),
            progress: Arc::new(Mutex::new(None)),
            next_run: next_knowledge_run(Instant::now(), true),
            next_idle_check: Instant::now() + Duration::from_secs(EMBEDDING_IDLE_CHECK_SECONDS),
            query: String::new(),
            hits: None,
            open_entry: None,
            confirm_clear: None,
            sort: KnowledgeSort::default(),
            kind_filter: None,
            session_hits: Vec::new(),
            command_line: knowledge_command_line(&executable),
        }
    }

    pub(crate) fn model_ready(&self) -> bool {
        matches!(self.model, ModelState::Ready { .. })
    }

    fn running_progress(&self) -> Option<(Uuid, usize, usize)> {
        *self
            .progress
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl OperonApp {
    // -----------------------------------------------------------------------
    // The model.

    pub(crate) fn check_model_readiness(&mut self) {
        let files = self.knowledge.model_files.clone();
        self.spawn_background(BackgroundKey::ModelReadiness, move || {
            remove_change_142_placeholder();
            BackgroundResult::ModelChecked {
                key: BackgroundKey::ModelReadiness,
                state: if model_is_ready(&files) {
                    ModelState::Ready {
                        device: detect_device(),
                    }
                } else {
                    ModelState::Missing
                },
            }
        });
    }

    pub(crate) fn start_model_download(&mut self) {
        if self.knowledge.download.is_some() || self.knowledge.model_ready() {
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicBool::new(false));
        self.knowledge.download = Some(ModelDownloadHandle {
            cancel: Arc::clone(&cancel),
            progress: Arc::clone(&progress),
            finished: Arc::clone(&finished),
        });
        self.knowledge.model = ModelState::Downloading;
        let files = self.knowledge.model_files.clone();
        self.spawn_background(BackgroundKey::ModelDownload, move || {
            let outcome = download_model(&files, &cancel, &progress);
            finished.store(true, Ordering::Release);
            let state = match outcome {
                Ok(()) => ModelState::Ready {
                    device: detect_device(),
                },
                Err(ModelFailure::Cancelled) => ModelState::Missing,
                Err(failure) => ModelState::Failed(failure.reason()),
            };
            BackgroundResult::ModelChecked {
                key: BackgroundKey::ModelDownload,
                state,
            }
        });
    }

    /// 「中止」 and app exit: curl is killed; the `.part` stays to be resumed.
    pub(crate) fn cancel_model_download(&mut self) {
        if let Some(download) = &self.knowledge.download {
            download.cancel.store(true, Ordering::Release);
        }
    }

    /// At exit: cancel, then wait — up to `EMBEDDING_DOWNLOAD_EXIT_WAIT_MILLIS`
    /// — for the download thread to have killed `curl`. Returning first would
    /// let `main` exit while the child lives on for up to `--max-time`, still
    /// writing the `.part` a relaunch's resume then writes into as well.
    pub(crate) fn stop_model_download_for_exit(&mut self) {
        let Some(download) = &self.knowledge.download else {
            return;
        };
        download.cancel.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_millis(EMBEDDING_DOWNLOAD_EXIT_WAIT_MILLIS);
        while !download.finished.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub(crate) fn delete_embedding_model(&mut self) {
        let files = self.knowledge.model_files.clone();
        self.knowledge.model = ModelState::Missing;
        self.request_system_action(tr("モデルを削除"), move || Ok(delete_model(&files)?));
    }

    pub(crate) fn apply_model_state(&mut self, key: BackgroundKey, state: ModelState) {
        self.background_tasks.remove(&key);
        if key == BackgroundKey::ModelDownload {
            self.knowledge.download = None;
        }
        self.knowledge.model = state;
        if !self.knowledge.model_ready() {
            // 「モデルを取得して始める」 asked for this download only; one that was
            // cancelled or failed takes the request with it, so a later fetch
            // from Settings starts nothing the person did not ask for then.
            if key == BackgroundKey::ModelDownload {
                self.knowledge.enable_when_ready = None;
                self.knowledge.select_semantic_when_ready = false;
            }
            return;
        }
        if std::mem::take(&mut self.knowledge.select_semantic_when_ready) {
            self.store.search_engine_mode = SearchEngineMode::EmbeddedGemma2;
            self.persist();
        }
        if let Some(project_id) = self.knowledge.enable_when_ready.take() {
            self.edit_knowledge(project_id, KnowledgeEdit::Enable);
        }
    }

    // -----------------------------------------------------------------------
    // Runs.

    /// Called every frame; starts a run when one is due. Two comparisons and
    /// nothing else unless it is.
    pub(crate) fn tick_knowledge(&mut self) {
        let now = Instant::now();
        if now >= self.knowledge.next_idle_check {
            self.knowledge.next_idle_check =
                now + Duration::from_secs(EMBEDDING_IDLE_CHECK_SECONDS);
            release_idle_embedder();
        }
        if now < self.knowledge.next_run {
            return;
        }
        self.knowledge.next_run = next_knowledge_run(now, false);
        self.start_knowledge_run();
    }

    /// One job for every project: the key has no project in it, so a second
    /// run cannot start while one is going, and enabled projects are read one
    /// after another.
    pub(crate) fn start_knowledge_run(&mut self) {
        if !self.knowledge.model_ready() {
            return;
        }
        let root = self.knowledge.root.clone();
        let model = self.knowledge.model_files.model();
        let progress = Arc::clone(&self.knowledge.progress);
        let projects: Vec<(Uuid, PathBuf)> = self
            .store
            .projects
            .iter()
            .map(|project| (project.id, project.path.clone()))
            .collect();
        self.spawn_background(BackgroundKey::KnowledgeRun, move || {
            let embedder = SharedEmbedder { path: model };
            let judge = CliJudge {
                directory: knowledge_judge_directory(&root),
            };
            let set_progress = |value: Option<(Uuid, usize, usize)>| {
                *progress
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
            };
            let results = run_enabled_projects(
                &root,
                &projects,
                &embedder,
                &judge,
                &|path| scan_native_cli_sessions(path, KNOWLEDGE_SESSIONS_LISTED_PER_RUN).sessions,
                now(),
                RunLimits::default(),
                &|project, done, total| set_progress(Some((project, done, total))),
            );
            set_progress(None);
            BackgroundResult::KnowledgeRan { results }
        });
    }

    pub(crate) fn retry_knowledge_load(&mut self, project_id: Uuid) {
        if let Some(view) = self.knowledge.views.get_mut(&project_id) {
            view.error = None;
        }
        self.request_knowledge_load(project_id);
    }

    pub(crate) fn request_knowledge_load(&mut self, project_id: Uuid) {
        let root = self.knowledge.root.clone();
        self.spawn_background(BackgroundKey::KnowledgeLoad(project_id), move || {
            BackgroundResult::KnowledgeLoaded {
                key: BackgroundKey::KnowledgeLoad(project_id),
                project_id,
                result: knowledge_snapshot(&root, project_id, now())
                    .map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn edit_knowledge(&mut self, project_id: Uuid, edit: KnowledgeEdit) {
        let root = self.knowledge.root.clone();
        let key = BackgroundKey::KnowledgeEdit(project_id);
        self.spawn_background(key.clone(), move || BackgroundResult::KnowledgeLoaded {
            key,
            project_id,
            result: apply_knowledge_edit(&root, project_id, edit, now())
                .map_err(|error| error.to_string()),
        });
        if edit == KnowledgeEdit::Enable {
            // Collect now rather than at the next tick.
            self.knowledge.next_run = Instant::now();
        }
    }

    pub(crate) fn apply_knowledge_snapshot(
        &mut self,
        project_id: Uuid,
        result: UiResult<KnowledgeSnapshot>,
    ) {
        let view = self.knowledge.views.entry(project_id).or_default();
        match result {
            Ok(snapshot) => {
                view.snapshot = Some(Arc::new(snapshot));
                view.error = None;
            }
            Err(error) => view.error = Some(error),
        }
        view.order_key = None;
    }

    pub(crate) fn apply_knowledge_run(
        &mut self,
        results: Vec<(Uuid, UiResult<KnowledgeSnapshot>)>,
    ) {
        self.background_tasks.remove(&BackgroundKey::KnowledgeRun);
        for (project_id, result) in results {
            self.apply_knowledge_snapshot(project_id, result);
        }
    }

    pub(crate) fn search_knowledge(&mut self, project_id: Uuid) {
        let query = self.knowledge.query.trim().to_owned();
        if query.is_empty() || !self.knowledge.model_ready() {
            self.knowledge.hits = None;
            return;
        }
        let root = self.knowledge.root.clone();
        let model = self.knowledge.model_files.model();
        self.spawn_background(BackgroundKey::KnowledgeSearch, move || {
            let result = (|| -> Result<Vec<KnowledgeHit>> {
                let Some(database) =
                    KnowledgeDatabase::open_existing(&knowledge_database_path(&root, project_id))?
                else {
                    return Ok(Vec::new());
                };
                search_entries(&database, &SharedEmbedder { path: model }, &query, 50)
            })();
            BackgroundResult::KnowledgeSearched {
                project_id,
                query,
                result: result.map_err(|error| error.to_string()),
            }
        });
    }

    /// The second job behind 「ローカル履歴」, beside today's scan: the indexed
    /// conversations of enabled projects, by meaning.
    pub(crate) fn search_indexed_sessions_for(&mut self, query: &str) {
        self.knowledge.session_hits.clear();
        if !self.knowledge.model_ready()
            || self.store.search_engine_mode != SearchEngineMode::EmbeddedGemma2
        {
            return;
        }
        let root = self.knowledge.root.clone();
        let model = self.knowledge.model_files.model();
        let projects: Vec<(Uuid, String)> = self
            .store
            .projects
            .iter()
            .map(|project| (project.id, project.name.clone()))
            .collect();
        let query = query.to_owned();
        self.spawn_background(BackgroundKey::KnowledgeSessionSearch, move || {
            BackgroundResult::IndexedSessionsSearched {
                result: search_indexed_sessions(
                    &root,
                    &projects,
                    &SharedEmbedder { path: model },
                    &query,
                    20,
                )
                .map_err(|error| error.to_string()),
            }
        });
    }

    /// A source or an indexed session, opened the way the CLI session list
    /// opens one.
    pub(crate) fn open_knowledge_conversation(
        &mut self,
        project_id: Uuid,
        provider: CliProvider,
        session_id: &str,
        path: &Path,
    ) {
        let Some(project) = self
            .store
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .cloned()
        else {
            return;
        };
        let session = CliSession {
            provider,
            native_id: session_id.to_owned(),
            id: session_id.to_owned(),
            title: None,
            last_user_message: None,
            branch: None,
            updated_at: 0,
            path: path.to_path_buf(),
        };
        self.resume_cli_session(&project, &session);
    }

    // -----------------------------------------------------------------------
    // Settings › データ.

    pub(crate) fn ui_model_block(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        ui.label(RichText::new(tr("意味検索（EmbeddingGemma 2）")).strong());
        ui.label(
            RichText::new(tr(
                "この Mac 上で動くモデルです。会話の内容は外部に送りません。",
            ))
            .size(12.5)
            .color(palette.text_muted),
        );
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            ui.label(tr("検索"));
            let previous = self.store.search_engine_mode;
            ui.radio_value(
                &mut self.store.search_engine_mode,
                SearchEngineMode::Keyword,
                tr("キーワードのみ"),
            );
            let semantic = ui.radio(
                self.store.search_engine_mode == SearchEngineMode::EmbeddedGemma2,
                tr("意味検索"),
            );
            if semantic.clicked() {
                if self.knowledge.model_ready() {
                    self.store.search_engine_mode = SearchEngineMode::EmbeddedGemma2;
                } else {
                    self.knowledge.select_semantic_when_ready = true;
                    self.start_model_download();
                }
            }
            if self.store.search_engine_mode != previous {
                self.persist();
            }
        });
        ui.add_space(SPACE_SM);
        self.ui_model_state_row(ui);
    }

    /// The model's state and its one action, shared by Settings and the
    /// disabled ナレッジ tab.
    pub(crate) fn ui_model_state_row(&mut self, ui: &mut egui::Ui) {
        let palette = self.store.theme.palette();
        match self.knowledge.model.clone() {
            ModelState::Missing => {
                if secondary_button(ui, tr("モデルを取得 (310MB)")).clicked() {
                    self.start_model_download();
                }
            }
            ModelState::Downloading => {
                let done = self
                    .knowledge
                    .download
                    .as_ref()
                    .map_or(0, |download| download.progress.load(Ordering::Acquire));
                ui.horizontal(|ui| {
                    if done >= EMBEDDING_MODEL_BYTES {
                        ui.label(tr("確認中…"));
                    } else {
                        ui.label(tf!(
                            "取得中… {done}MB / {total}MB",
                            done = done / 1_000_000,
                            total = EMBEDDING_MODEL_BYTES / 1_000_000
                        ));
                        ui.add(
                            egui::ProgressBar::new(done as f32 / EMBEDDING_MODEL_BYTES as f32)
                                .desired_width(160.0),
                        );
                        if secondary_button(ui, tr("中止")).clicked() {
                            self.cancel_model_download();
                        }
                    }
                });
                ui.ctx().request_repaint_after(Duration::from_millis(500));
            }
            ModelState::Ready { device } => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(tf!("使用可能 ({device} / 310MB)", device = device.label()))
                            .color(palette.success),
                    );
                    if secondary_button(ui, tr("モデルを削除")).clicked() {
                        self.delete_embedding_model();
                    }
                });
            }
            ModelState::Failed(reason) => {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(tf!("取得できませんでした: {reason}", reason = reason))
                            .color(palette.danger),
                    );
                    if secondary_button(ui, tr("再試行")).clicked() {
                        self.knowledge.model = ModelState::Missing;
                        self.start_model_download();
                    }
                });
            }
        }
    }

    // -----------------------------------------------------------------------
    // Project › ナレッジ.

    pub(crate) fn ui_knowledge(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        if let std::collections::hash_map::Entry::Vacant(view) =
            self.knowledge.views.entry(project.id)
        {
            view.insert(KnowledgeView::default());
            self.request_knowledge_load(project.id);
        }
        let snapshot = self
            .knowledge
            .views
            .get(&project.id)
            .and_then(|view| view.snapshot.clone());
        let error = self
            .knowledge
            .views
            .get(&project.id)
            .and_then(|view| view.error.clone());
        if let Some(error) = &error {
            banner(ui, palette, ICON_ATTENTION, palette.danger, |ui| {
                ui.label(error);
            });
            ui.add_space(SPACE_SM);
        }
        let Some(snapshot) = snapshot else {
            if error.is_none() {
                ui.label(RichText::new(tr("読み込み中…")).color(palette.text_muted));
            } else if secondary_button(ui, tr("再試行")).clicked() {
                // A failed load leaves no snapshot to draw; this is the way back.
                self.retry_knowledge_load(project.id);
            }
            return;
        };
        if !snapshot.enabled {
            self.ui_knowledge_disabled(ui, project);
            return;
        }
        self.ui_knowledge_header(ui, project, &snapshot);
        ui.add_space(SPACE_SM);
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.knowledge.query)
                .hint_text(tr("ナレッジを検索"))
                .desired_width(420.0),
        );
        if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            self.search_knowledge(project.id);
        }
        if response.changed() && self.knowledge.query.trim().is_empty() {
            self.knowledge.hits = None;
        }
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.label(RichText::new(tr("並び:")).color(palette.text_muted));
            for sort in KnowledgeSort::ALL {
                ui.radio_value(&mut self.knowledge.sort, sort, sort.label());
            }
            ui.add_space(SPACE_LG);
            ui.label(RichText::new(tr("種類:")).color(palette.text_muted));
            egui::ComboBox::from_id_salt("knowledge-kind")
                .selected_text(
                    self.knowledge
                        .kind_filter
                        .map_or(tr("すべて"), KnowledgeKind::label),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.knowledge.kind_filter, None, tr("すべて"));
                    for kind in KnowledgeKind::ALL {
                        ui.selectable_value(
                            &mut self.knowledge.kind_filter,
                            Some(kind),
                            kind.label(),
                        );
                    }
                });
        });
        tab_bar_rule(ui, palette);
        if snapshot.entries.is_empty() {
            ui.add_space(SPACE_MD);
            ui.label(
                RichText::new(tr("まだナレッジはありません。このプロジェクトで会話を続けると、重要なやり取りがここに追加されます。"))
                    .color(palette.text_muted),
            );
        } else {
            self.ui_knowledge_rows(ui, project, &snapshot);
        }
        ui.add_space(SPACE_MD);
        hairline(ui, palette);
        ui.add_space(SPACE_SM);
        let command_line = self.knowledge.command_line.clone();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(tr("エージェントから参照:")).color(palette.text_muted));
            ui.label(RichText::new(&command_line).monospace().size(12.0));
            if small_icon_button(ui, palette, ICON_COPY, tr("コピー")).clicked() {
                ui.ctx().copy_text(command_line.clone());
            }
        });
    }

    fn ui_knowledge_disabled(&mut self, ui: &mut egui::Ui, project: &Project) {
        let palette = self.store.theme.palette();
        card_frame(palette)
            .inner_margin(egui::Margin::symmetric(18, 16))
            .show(ui, |ui| {
                ui.set_max_width(720.0);
                ui.label(
                    RichText::new(tr("このプロジェクトの会話からナレッジを集めます"))
                        .size(16.0)
                        .strong(),
                );
                ui.add_space(SPACE_SM);
                ui.label(tr("Claude Code・Codex・Antigravity との会話から、決定・原因と対処・手順・注意点を短いナレッジにまとめ、会話ごと検索できるようにします。使われないものは徐々に薄れて消え、参照されたり繰り返し出てきたりしたものは残ります。"));
                ui.add_space(SPACE_SM);
                for line in [
                    tr("・意味の判定は、この Mac 上の EmbeddingGemma 2 で行います。"),
                    tr("・残す候補だけを、その会話を行った CLI に要約させます (1日 30 回まで)。"),
                    tr("・ナレッジはこの Mac の Operon データフォルダに保存し、リポジトリには書きません。"),
                ] {
                    ui.label(RichText::new(line).color(palette.text_muted));
                }
                ui.add_space(SPACE_MD);
                if self.knowledge.model_ready() {
                    if primary_button(ui, palette, ICON_KNOWLEDGE, tr("このプロジェクトでナレッジを集める"))
                        .clicked()
                    {
                        self.edit_knowledge(project.id, KnowledgeEdit::Enable);
                    }
                } else if matches!(self.knowledge.model, ModelState::Missing) {
                    if primary_button(ui, palette, ICON_KNOWLEDGE, tr("モデルを取得して始める (310MB)"))
                        .clicked()
                    {
                        self.knowledge.enable_when_ready = Some(project.id);
                        self.start_model_download();
                    }
                } else {
                    self.ui_model_state_row(ui);
                }
            });
    }

    fn ui_knowledge_header(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        snapshot: &KnowledgeSnapshot,
    ) {
        let palette = self.store.theme.palette();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(tr("ナレッジ"))
                    .size(20.0)
                    .strong()
                    .color(palette.text_strong),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.menu_button(tr("収集を止める ▾"), |ui| {
                    if ui.button(tr("収集を止める (ナレッジは残す)")).clicked() {
                        self.edit_knowledge(project.id, KnowledgeEdit::Disable);
                        ui.close_menu();
                    }
                    if ui.button(tr("ナレッジを削除…")).clicked() {
                        self.knowledge.confirm_clear = Some(project.id);
                        ui.close_menu();
                    }
                });
                if secondary_button(ui, tr("今すぐ更新")).clicked() {
                    self.start_knowledge_run();
                }
            });
        });
        if self.knowledge.confirm_clear == Some(project.id) {
            banner(ui, palette, ICON_ATTENTION, palette.danger, |ui| {
                ui.label(tf!(
                    "{n} 件のナレッジを削除します。元に戻せません。",
                    n = snapshot.entries.len()
                ));
                if secondary_button(ui, tr("削除する")).clicked() {
                    self.edit_knowledge(project.id, KnowledgeEdit::Clear);
                    self.knowledge.confirm_clear = None;
                }
                if secondary_button(ui, tr("キャンセル")).clicked() {
                    self.knowledge.confirm_clear = None;
                }
            });
        }
        let (line, warn) = knowledge_status_line(
            snapshot,
            &self.knowledge.model,
            self.knowledge
                .running_progress()
                .filter(|(running, _, _)| *running == project.id)
                .map(|(_, done, total)| (done, total)),
        );
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(line).size(12.5).color(if warn {
                palette.warning
            } else {
                palette.text_muted
            }));
            if matches!(self.knowledge.model, ModelState::Missing)
                && secondary_button(ui, tr("モデルを取得")).clicked()
            {
                self.start_model_download();
            }
        });
        if snapshot.failed > 0 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(tf!("処理できなかった会話 {n} 件", n = snapshot.failed))
                        .size(12.5)
                        .color(palette.text_muted),
                );
                if quiet_button(ui, palette, tr("再試行")).clicked() {
                    self.edit_knowledge(project.id, KnowledgeEdit::RetryFailed);
                }
            });
        }
        if self.knowledge.running_progress().is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(500));
        }
    }

    fn ui_knowledge_rows(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        snapshot: &KnowledgeSnapshot,
    ) {
        let palette = self.store.theme.palette();
        let searched = self
            .knowledge
            .hits
            .as_ref()
            .filter(|(id, _, _)| *id == project.id)
            .map(|(_, _, hits)| hits.clone());
        let rows: Vec<(usize, Option<f32>)> = match &searched {
            Some(hits) => hits
                .iter()
                .filter_map(|hit| {
                    snapshot
                        .entries
                        .iter()
                        .position(|entry| entry.id == hit.entry.id)
                        .map(|index| (index, Some(hit.similarity)))
                })
                .collect(),
            None => {
                let (sort, kind) = (self.knowledge.sort, self.knowledge.kind_filter);
                self.knowledge
                    .views
                    .get_mut(&project.id)
                    .map(|view| {
                        view.ordered(sort, kind)
                            .iter()
                            .map(|index| (*index, None))
                            .collect()
                    })
                    .unwrap_or_default()
            }
        };
        if searched.as_ref().is_some_and(Vec::is_empty) {
            ui.add_space(SPACE_SM);
            ui.label(RichText::new(tr("一致するナレッジはありません。")).color(palette.text_muted));
            return;
        }
        let now = now();
        egui::ScrollArea::vertical()
            .id_salt("knowledge-rows")
            .max_height(ui.available_height() - 60.0)
            .show(ui, |ui| {
                for (index, similarity) in rows {
                    let entry = &snapshot.entries[index];
                    let open = self.knowledge.open_entry == Some((project.id, entry.id));
                    // Rows outside the viewport take their height and draw nothing.
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), CONTROL_HEIGHT),
                        egui::Sense::click(),
                    );
                    if ui.is_rect_visible(rect) {
                        let mut row = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect)
                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                        );
                        knowledge_row(&mut row, palette, entry, similarity, open, now);
                    }
                    if response.clicked() {
                        if open {
                            self.knowledge.open_entry = None;
                        } else {
                            self.knowledge.open_entry = Some((project.id, entry.id));
                            self.edit_knowledge(project.id, KnowledgeEdit::Reference(entry.id));
                        }
                    }
                    if open {
                        self.ui_knowledge_detail(ui, project, entry);
                    }
                }
            });
    }

    fn ui_knowledge_detail(
        &mut self,
        ui: &mut egui::Ui,
        project: &Project,
        entry: &KnowledgeEntry,
    ) {
        let palette = self.store.theme.palette();
        egui::Frame::NONE
            .inner_margin(egui::Margin {
                left: 24,
                right: 8,
                top: 4,
                bottom: 8,
            })
            .show(ui, |ui| {
                ui.label(&entry.body);
                ui.add_space(SPACE_XS);
                for source in &entry.sources {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(tf!(
                                "出典: {cli} ・ {when}",
                                cli = source.provider.label(),
                                when = relative_time(source.recorded_at)
                            ))
                            .size(12.0)
                            .color(palette.text_muted),
                        );
                        if quiet_button(ui, palette, tr("会話を開く")).clicked() {
                            self.open_knowledge_conversation(
                                project.id,
                                source.provider,
                                &source.session_id,
                                &source.transcript_path,
                            );
                        }
                    });
                }
                ui.horizontal(|ui| {
                    let pin_label = if entry.pinned {
                        tr("固定を外す")
                    } else {
                        tr("固定")
                    };
                    if secondary_button(ui, pin_label).clicked() {
                        self.edit_knowledge(
                            project.id,
                            KnowledgeEdit::Pin(entry.id, !entry.pinned),
                        );
                    }
                    if secondary_button(ui, tr("削除")).clicked() {
                        self.knowledge.open_entry = None;
                        self.edit_knowledge(project.id, KnowledgeEdit::Delete(entry.id));
                    }
                });
            });
    }

    // -----------------------------------------------------------------------
    // Sessions › ローカル履歴.

    pub(crate) fn ui_indexed_session_matches(&mut self, ui: &mut egui::Ui) {
        if self.knowledge.session_hits.is_empty() {
            return;
        }
        let palette = self.store.theme.palette();
        ui.add_space(SPACE_SM);
        ui.label(RichText::new(tr("意味で一致 (ナレッジが有効なプロジェクト)")).strong());
        let hits = self.knowledge.session_hits.clone();
        for hit in &hits {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&hit.project_name).color(palette.text_muted));
                ui.label(
                    RichText::new(hit.provider.label())
                        .size(12.0)
                        .color(palette.text_muted),
                );
                ui.label(RichText::new(hit.title.as_deref().unwrap_or(&hit.session_id)).strong());
                ui.label(
                    RichText::new(tf!("({p0}%)", p0 = (hit.similarity * 100.0).round() as u32))
                        .size(12.0)
                        .color(palette.text_muted),
                );
                if quiet_button(ui, palette, tr("会話を開く")).clicked() {
                    self.open_knowledge_conversation(
                        hit.project_id,
                        hit.provider,
                        &hit.session_id,
                        &hit.transcript_path,
                    );
                }
            });
        }
    }
}

/// The line under the tab's heading: what is happening, or what stopped it,
/// and whether it is a warning.
pub(crate) fn knowledge_status_line(
    snapshot: &KnowledgeSnapshot,
    model: &ModelState,
    running: Option<(usize, usize)>,
) -> (String, bool) {
    if let Some((done, total)) = running {
        return (
            tf!(
                "会話を整理しています… ({done}/{total})",
                done = done,
                total = total
            ),
            false,
        );
    }
    if !matches!(model, ModelState::Ready { .. }) {
        return (tr("モデルがないため停止中").to_owned(), true);
    }
    if let Some(report) = &snapshot.last_report {
        if let Some((done, total)) = report.rebuilding {
            return (
                tf!(
                    "ナレッジを新しいモデルで作り直しています… ({done}/{total})",
                    done = done,
                    total = total
                ),
                true,
            );
        }
        if let Some(provider) = report.paused {
            return (
                tf!(
                    "{cli} で判定できませんでした。次回の更新で再試行します。",
                    cli = provider.label()
                ),
                true,
            );
        }
        if report.capped {
            return (
                tr("本日の判定回数の上限に達しました。明日続きを処理します。").to_owned(),
                true,
            );
        }
        if report.full_of_pinned {
            return (
                tr("固定したナレッジが上限 (2000 件) に達したため、新しいナレッジを追加していません。")
                    .to_owned(),
                true,
            );
        }
    }
    let updated = snapshot
        .updated_at
        .map_or_else(|| tr("未実行").to_owned(), relative_time);
    (
        tf!(
            "{n} 件 ・ 最終更新 {updated} ・ 本日の判定 {used}/{cap} 回",
            n = snapshot.entries.len(),
            updated = updated,
            used = snapshot.judge_calls_today,
            cap = KNOWLEDGE_DAILY_JUDGE_CALLS
        ),
        false,
    )
}

fn knowledge_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    entry: &KnowledgeEntry,
    similarity: Option<f32>,
    open: bool,
    now: u64,
) {
    ui.spacing_mut().item_spacing.x = SPACE_SM;
    ui.label(
        RichText::new(if open {
            ICON_DISCLOSURE_OPEN
        } else {
            ICON_DISCLOSURE_CLOSED
        })
        .size(12.0)
        .color(palette.text_faint),
    );
    ui.label(
        RichText::new(entry.kind.label())
            .size(12.0)
            .strong()
            .color(palette.accent_text),
    );
    let right_width = 260.0;
    let title_width = (ui.available_width() - right_width).max(80.0);
    ui.add_sized(
        egui::vec2(title_width, CONTROL_HEIGHT),
        egui::Label::new(match similarity {
            Some(similarity) => tf!(
                "{title} ({p0}%)",
                title = entry.title,
                p0 = (similarity * 100.0).round() as u32
            ),
            None => entry.title.clone(),
        })
        .truncate(),
    );
    freshness_bar(ui, palette, entry.freshness(now));
    ui.label(
        RichText::new(tf!("参照 {n}", n = entry.reference_count))
            .size(12.0)
            .color(palette.text_muted),
    );
    if entry.pinned {
        ui.label(RichText::new(ICON_PIN).size(12.0).color(palette.text_muted));
    }
    if let Some(source) = entry.sources.last() {
        ui.label(
            RichText::new(source.provider.label())
                .size(11.5)
                .color(palette.text_faint),
        );
    }
}

/// Ten cells: filled share is the freshness; under 0.3 — about to fade — the
/// fill turns to `warning`.
fn freshness_bar(ui: &mut egui::Ui, palette: &Palette, freshness: f64) {
    let cell = egui::vec2(6.0, 10.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(cell.x * 10.0 + 9.0, cell.y),
        egui::Sense::hover(),
    );
    let filled = (freshness * 10.0).round().clamp(0.0, 10.0) as usize;
    let fill = if freshness < 0.3 {
        palette.warning
    } else {
        palette.accent
    };
    for index in 0..10 {
        let min = rect.min + egui::vec2(index as f32 * (cell.x + 1.0), 0.0);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(min, cell),
            1.0,
            if index < filled {
                fill
            } else {
                palette.border_subtle
            },
        );
    }
}
