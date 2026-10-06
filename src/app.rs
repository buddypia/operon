use crate::*;

mod keymap;
pub(crate) mod screens;

// The keyboard is data this module reads and `screens.rs` draws, so it is
// reachable by symbol name from anywhere the way every other module's is.
pub(crate) use keymap::*;

pub(crate) type UiResult<T> = std::result::Result<T, String>;

/// One session's find bar and what it has found.
#[derive(Debug, Clone, Default)]
pub(crate) struct TerminalSearch {
    pub(crate) open: bool,
    pub(crate) query: String,
    pub(crate) matches: Vec<TerminalMatch>,
    pub(crate) current: usize,
    pub(crate) needs_focus: bool,
    /// Set once when the pane should jump, and cleared by the frame that jumps,
    /// so scrolling by hand afterwards is not undone on the next frame.
    pub(crate) scroll_to: Option<usize>,
}

/// One user prompt turn recorded for session navigation.
#[derive(Debug, Clone)]
pub(crate) struct PromptTurn {
    pub(crate) turn: usize,
    pub(crate) prompt: String,
    pub(crate) timestamp: Instant,
    pub(crate) line: usize,
    pub(crate) completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BackgroundKey {
    WorkspaceScan,
    GitChanges(Uuid),
    GitDiff(Uuid, Option<String>),
    GitHistory(Uuid),
    Worktrees(Uuid),
    PullRequests(Uuid),
    Files(Uuid),
    WorktreeFiles(PathBuf),
    FileOpen(DocumentId),
    FileSave(DocumentId),
    FileWatch(DocumentId),
    Skills(Uuid),
    Rules(Uuid),
    SessionPoll,
    SessionStart(Uuid),
    EmptySessionStart(Uuid),
    SessionStop(Uuid),
    SessionOutput(Uuid),
    TerminalInput(Uuid),
    TerminalResize(Uuid),
    TerminalClose(Uuid),
    FullHistoryImport(Uuid),
    NativeSessionResolution(Uuid),
    WorktreeMutation(Uuid),
    LandWorktree(Uuid),
    CreatePullRequest(Uuid),
    FetchIssues(Uuid),
    GitMutation(Uuid),
    CommitMessage(Uuid),
    AiDiffReview(Uuid),
    Upstream(Uuid),
    PortScan,
    ToolStatus,
    RecoveryScan,
    RecoveryAdopt(String),
    CliSessions(Uuid),
    TranscriptSearch,
    SystemAction(Uuid),
    Notification(Uuid),
    /// Registering or removing the managed status hooks in the other CLIs'
    /// settings files. One key: the three CLIs are done in one pass so that a
    /// toggle cannot leave two of them half-registered.
    HookApply,
}

#[derive(Debug)]
pub(crate) enum BackgroundResult {
    HooksApplied {
        installs: Vec<HookInstall>,
    },
    WorkspaceScan {
        workspace: PathBuf,
        repositories: Vec<PathBuf>,
        truncated: bool,
    },
    GitChanges {
        project_id: Uuid,
        result: UiResult<GitChangesSnapshot>,
    },
    GitDiff {
        project_id: Uuid,
        file: Option<String>,
        generation: u64,
        result: UiResult<String>,
    },
    GitHistory {
        project_id: Uuid,
        result: UiResult<String>,
    },
    Worktrees {
        project_id: Uuid,
        result: UiResult<Vec<Worktree>>,
    },
    PullRequests {
        project_id: Uuid,
        result: UiResult<Vec<PullRequest>>,
    },
    Files {
        project_id: Uuid,
        scan: FileScanSnapshot,
    },
    WorktreeFiles {
        root: PathBuf,
        scan: FileScanSnapshot,
    },
    FileOpened {
        document: DocumentId,
        result: UiResult<FileOpenOutcome>,
    },
    FileSaved {
        document: DocumentId,
        /// The text now on disk, which is what the document's `disk` becomes.
        result: UiResult<String>,
    },
    FileWatched {
        document: DocumentId,
        /// The document's `disk` at the moment this read was issued, carried
        /// back so the answer can be checked against the question. Without it a
        /// read in flight is a fact about a document that may no longer exist:
        /// a reload or a save landing first moves `disk`, and applying an older
        /// read on top of a newer one rolls the file back. The same shape as
        /// `BackgroundResult::GitDiff`'s `generation`, for the same reason.
        known: String,
        /// What is on disk now, and `None` where that is still what the
        /// document was holding — which is the answer almost every time.
        result: UiResult<Option<FileOpenOutcome>>,
    },
    Skills {
        project_id: Uuid,
        scan: FileScanSnapshot,
    },
    Rules {
        project_id: Uuid,
        scan: FileScanSnapshot,
    },
    SessionPoll {
        states: Vec<(Uuid, TmuxObservation)>,
        activities: Vec<(Uuid, AgentActivity)>,
        previews: Vec<(Uuid, ConversationPreview)>,
        output: Option<(Uuid, UiResult<TerminalCapture>)>,
    },
    CliSessions {
        project_id: Uuid,
        scan: CliSessionScan,
        announce: bool,
    },
    TranscriptSearch {
        query: String,
        scan: TranscriptSearchSnapshot,
    },
    SessionStarted {
        session_id: Uuid,
        result: UiResult<Option<String>>,
    },
    EmptySessionStarted {
        session_id: Uuid,
        result: UiResult<Option<String>>,
    },
    SessionStopped {
        session_id: Uuid,
        result: UiResult<()>,
    },
    SessionOutput {
        session_id: Uuid,
        result: UiResult<TerminalCapture>,
    },
    TerminalInputSent {
        session_id: Uuid,
        result: UiResult<()>,
    },
    TerminalResized {
        session_id: Uuid,
        size: (u16, u16),
        result: UiResult<()>,
    },
    TerminalClosed {
        session_id: Uuid,
        result: UiResult<()>,
    },
    FullHistoryImported(Box<FullHistoryImportResult>),
    NativeSessionResolved {
        session_id: Uuid,
        result: UiResult<CliSession>,
    },
    WorktreeCreated {
        project_id: Uuid,
        result: UiResult<CreatedWorktree>,
    },
    StagedChangeRecorded {
        project_id: Uuid,
        result: UiResult<String>,
    },
    PushFinished {
        project_id: Uuid,
        result: UiResult<String>,
    },
    CommitMessageDrafted {
        project_id: Uuid,
        result: UiResult<String>,
    },
    AiDiffReview {
        project_id: Uuid,
        result: UiResult<String>,
    },
    UpstreamRead {
        project_id: Uuid,
        result: UiResult<UpstreamState>,
    },
    ListeningPorts(Vec<(ListeningPort, Option<PathBuf>)>),
    WorktreeRemoved {
        project_id: Uuid,
        path: PathBuf,
        result: UiResult<()>,
    },
    WorktreeLanded {
        project_id: Uuid,
        result: UiResult<LandResult>,
    },
    PullRequestCreated {
        project_id: Uuid,
        result: UiResult<String>,
    },
    IssuesFetched {
        project_id: Uuid,
        result: UiResult<Vec<GitHubIssue>>,
    },
    ToolStatus(ToolStatus),
    OrphanedTmuxSessions(UiResult<Vec<OrphanedTmuxSession>>),
    OrphanAdopted {
        orphan: OrphanedTmuxSession,
        result: UiResult<(Uuid, Option<String>)>,
    },
    SystemAction {
        id: Uuid,
        action: String,
        result: UiResult<()>,
    },
    Notification {
        id: Uuid,
        result: UiResult<()>,
    },
}

#[derive(Debug)]
pub(crate) struct FullHistoryImportResult {
    pub(crate) import_id: Uuid,
    pub(crate) project: Project,
    pub(crate) source: CliSession,
    pub(crate) target: String,
    pub(crate) note: String,
    pub(crate) result: UiResult<ImportedNativeSession>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectTab {
    Overview,
    Git,
    Worktrees,
    PullRequests,
    Files,
    /// The skills and the instruction files the agents read, one above the
    /// other (change 093: they were two tabs).
    AgentSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitView {
    Changes,
    Diff,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionView {
    List,
    Grid,
}

pub(crate) struct OperonApp {
    // Holding these files keeps the process-wide advisory locks alive: this
    // build's index, and the pre-rename index when that directory still
    // exists. Tests hold none; the real app acquires them before reading the
    // shared store.
    pub(crate) _instance_locks: Vec<fs::File>,
    pub(crate) store: Store,
    pub(crate) data_file: PathBuf,
    pub(crate) page: Page,
    pub(crate) selected_project: Option<Uuid>,
    pub(crate) project_tab: ProjectTab,
    /// The settings page's chosen section (change 094). In memory: the page is
    /// opened on purpose, and a missing tool opens the one it needs.
    pub(crate) settings_section: SettingsSection,
    pub(crate) git_view: GitView,
    pub(crate) session_view: SessionView,
    pub(crate) session_library_open: bool,
    /// Which status groups the session list is narrowed to, indexed by
    /// `SessionStatusGroup::index`. All false means no narrowing at all, which
    /// is why it is a set of independent toggles and not a selected index.
    ///
    /// Deliberately not in the store: a filter that survived a restart would
    /// open the app on a list with sessions missing and no memory of why.
    pub(crate) status_filter: [bool; 4],
    pub(crate) git_selected_file: Option<String>,
    pub(crate) selected_file: Option<PathBuf>,
    pub(crate) project_path_input: String,
    pub(crate) workspace_scan_input: String,
    pub(crate) show_manual_project_entry: bool,
    pub(crate) goal_input: String,
    pub(crate) session_name_input: String,
    pub(crate) selected_agent: String,
    pub(crate) custom_command: String,
    /// Which registered login the next session runs under, or `None` for the
    /// machine's own — which is what every launch before change 031 did.
    pub(crate) agent_account_input: Option<Uuid>,
    /// The two fields of the "add an account" row in settings.
    pub(crate) account_agent_input: String,
    pub(crate) account_name_input: String,
    pub(crate) agent_model_input: String,
    pub(crate) agent_mode_input: String,
    pub(crate) agent_effort_input: String,
    /// The flag IDs switched on for the next launch, in the order they were
    /// picked. `build_agent_launch_command` orders them by the catalogue, so
    /// this holds the person's clicks and never the command's word order.
    pub(crate) agent_flag_inputs: Vec<String>,
    pub(crate) recent_agent_settings: RecentAgentSettings,
    pub(crate) acknowledged_launch: Option<LaunchAcknowledgement>,
    pub(crate) session_path_input: String,
    pub(crate) session_path_project: Option<Uuid>,
    pub(crate) worktree_branch_input: String,
    pub(crate) pending_worktree_removal: Option<PathBuf>,
    pub(crate) pending_project_removal: Option<Uuid>,
    pub(crate) pending_session_close: Option<Uuid>,
    /// The session whose sidebar title is being edited, with the text as typed.
    /// Renaming writes to the store, so it is committed on Enter and discarded
    /// on Escape rather than on every keystroke.
    pub(crate) renaming_session: Option<Uuid>,
    pub(crate) rename_input: String,
    pub(crate) rename_needs_focus: bool,
    pub(crate) remove_after_close: HashSet<Uuid>,
    pub(crate) selected_session: Option<Uuid>,
    pub(crate) session_output: HashMap<Uuid, String>,
    pub(crate) session_activity: HashMap<Uuid, ActivityTracker>,
    /// The last exchange each managed terminal showed, as its row quotes it.
    /// In memory only: it is a reading of a live screen, so a restart re-reads
    /// it rather than restoring a quote whose terminal may be long gone.
    pub(crate) session_previews: HashMap<Uuid, ConversationPreview>,
    pub(crate) terminal_layouts: HashMap<Uuid, Vec<LayoutJob>>,
    pub(crate) terminal_cursors: HashMap<Uuid, (u16, u16)>,
    pub(crate) terminal_preedits: HashMap<Uuid, String>,
    pub(crate) terminal_input_queue: HashMap<Uuid, VecDeque<TerminalInput>>,
    pub(crate) terminal_resize_requested: HashMap<Uuid, (u16, u16)>,
    pub(crate) terminal_resize_applied: HashMap<Uuid, (u16, u16)>,
    pub(crate) depends_on_input: Option<Uuid>,
    pub(crate) window_maximized: bool,
    pub(crate) last_session_refresh: Instant,
    /// The Files view is a collaboration surface: an agent can write its
    /// working tree while a person holds the Diff tab open. Refresh status and
    /// the open Diff at a modest cadence so a cached comparison is never
    /// presented as current.
    pub(crate) last_editor_git_refresh: HashMap<Uuid, Instant>,
    pub(crate) store_retry_pending: bool,
    pub(crate) search: String,
    pub(crate) file_search: String,
    pub(crate) transcript_search_input: String,
    pub(crate) transcript_matches: Vec<TranscriptMatch>,
    pub(crate) cli_sessions: HashMap<Uuid, Vec<CliSession>>,
    pub(crate) handoff_note_input: String,
    pub(crate) handoff_target_inputs: HashMap<String, String>,
    pub(crate) managed_handoff_targets: HashMap<Uuid, String>,
    pub(crate) git_changes_cache: HashMap<Uuid, UiResult<GitChangesSnapshot>>,
    pub(crate) git_diff_cache: HashMap<(Uuid, Option<String>), UiResult<String>>,
    /// The current request generation per Diff. An older background result is
    /// not allowed to repopulate a cache an agent write just invalidated.
    pub(crate) git_diff_generations: HashMap<(Uuid, Option<String>), u64>,
    pub(crate) git_history_cache: HashMap<Uuid, UiResult<String>>,
    pub(crate) worktree_cache: HashMap<Uuid, UiResult<Vec<Worktree>>>,
    pub(crate) pull_request_cache: HashMap<Uuid, UiResult<Vec<PullRequest>>>,
    pub(crate) file_cache: HashMap<Uuid, FileScanSnapshot>,
    /// Every file the editor has open, in the order their tabs sit in.
    pub(crate) open_documents: Vec<OpenDocument>,
    /// Which tab is in front. A pair rather than an index into the vector
    /// above, so that closing a tab cannot leave this pointing at whichever
    /// file happened to slide into that slot.
    pub(crate) active_document: Option<DocumentId>,
    pub(crate) editor_view: EditorView,
    /// The directories a person has opened in the project tree. Keyed by
    /// project, so two projects do not share one expansion.
    pub(crate) expanded_directories: HashSet<(Uuid, PathBuf)>,
    /// Files whose diff is folded shut in the review pane, by path.
    pub(crate) collapsed_diff_files: HashSet<String>,
    /// The project tree as last built, with the project and the filter text it
    /// was built for. One entry, because one tree is on screen: folding twelve
    /// thousand paths back into a tree is cheap once and wasteful sixty times
    /// a second.
    pub(crate) file_tree_cache: Option<(Uuid, String, Vec<FileTreeNode>)>,
    /// Parsed diffs, keyed the same way `git_diff_cache` is. Parsing 180 KiB of
    /// diff on every frame would be the whole frame; this is the same bargain
    /// `terminal_layouts` makes with its captured output.
    pub(crate) diff_file_cache: HashMap<(Uuid, Option<String>), Vec<DiffFile>>,
    /// One bounded layout cache for the one document visible in the editor.
    pub(crate) editor_text_layout_cache: EditorTextLayoutCache,
    /// The same bargain for the rendered view of that document: the parse and
    /// the fence highlighting are derived state, and Markdown opens rendered.
    pub(crate) markdown_preview_cache: MarkdownPreviewCache,
    /// Which document was last compared against the bytes on disk, and when.
    /// Agents write into the tree this editor is reading, so "what I loaded"
    /// going stale is the normal case here.
    ///
    /// One slot rather than a map, because one document is watched: the front
    /// tab. A map would keep an entry per file opened all session and would
    /// need pruning on every path a tab can close by, and it would make a tab
    /// brought to the front wait out the previous tab's two seconds instead of
    /// being read at once.
    pub(crate) last_editor_file_watch: Option<(DocumentId, Instant)>,
    pub(crate) skill_cache: HashMap<Uuid, FileScanSnapshot>,
    pub(crate) rule_cache: HashMap<Uuid, FileScanSnapshot>,
    pub(crate) background_tasks: HashSet<BackgroundKey>,
    pub(crate) background_sender: Sender<BackgroundResult>,
    pub(crate) background_receiver: Receiver<BackgroundResult>,
    pub(crate) command_palette_open: bool,
    pub(crate) command_selection: usize,
    pub(crate) command_palette_needs_focus: bool,
    pub(crate) command_search: String,
    pub(crate) orphaned_tmux_sessions: Vec<OrphanedTmuxSession>,
    pub(crate) tools: ToolStatus,
    pub(crate) notice: Option<String>,
    /// A success that only needs to be seen, and when it was set. While
    /// `notice` still holds this text it is drawn as a toast and cleared after
    /// `NOTICE_TOAST_SECONDS`; a notice that replaced it is a different text and
    /// is never cleared by this timer.
    pub(crate) brief_notice: Option<(String, Instant)>,
    /// The keys this session answers to, read once at startup. Every shortcut
    /// below is a lookup in here rather than a literal, so the palette can draw
    /// the chord the frame loop is actually listening for.
    pub(crate) keymap: Keymap,
    /// Which logins are registered, and which session ran under which. Beside
    /// the store rather than in it, because the store is a paused surface.
    pub(crate) accounts: AgentAccounts,
    /// The restore currently on screen. `Some` puts the modal up; sending it to
    /// the background clears this and leaves the import thread alone, so the
    /// result still arrives in the ordinary notice.
    pub(crate) restore_progress: Option<RestoreProgress>,
    /// The active CLI or terminal launch on screen. `Some` displays the launch
    /// progress modal; dismissing it clears visibility while allowing background
    /// session setup to continue.
    pub(crate) launch_progress: Option<LaunchProgress>,
    /// The launch sheet over whatever page is showing (change 092).
    pub(crate) launch_sheet_open: bool,
    /// Set when the sheet opens, cleared once its request field has taken
    /// keyboard focus: a terminal pane forwards keys while it holds focus.
    pub(crate) launch_sheet_needs_focus: bool,
    /// Whether the sheet's project is a git repository, asked when the sheet
    /// opens or its project changes: the footer is drawn every frame, and a
    /// file-system stat there would stall it on a slow volume.
    pub(crate) launch_sheet_is_git: bool,
    /// The project 履歴's CLI section is showing, when not the selected one.
    pub(crate) history_project: Option<Uuid>,
    pub(crate) durability_warning: Option<String>,
    pub(crate) applied_theme: Option<AppTheme>,
    pub(crate) applied_terminal_font: Option<TerminalFont>,
    /// The socket the CLIs' own status hooks post to, and what has arrived on
    /// it. Both are `None` until the first frame binds them, and stay `None`
    /// when the mechanism is switched off or the socket cannot be opened.
    pub(crate) hook_listener: Option<HookListener>,
    pub(crate) hook_events: Option<Receiver<HookEvent>>,
    /// What each managed session's hooks last reported. In memory only: a
    /// state nobody has confirmed since the restart is a state the screen is
    /// better placed to answer for.
    pub(crate) hook_status: HashMap<Uuid, HookStatus>,
    /// The token this process minted for each session's current launch. An
    /// event carrying an older one belongs to a process that has been replaced.
    pub(crate) hook_launch_tokens: HashMap<Uuid, String>,
    pub(crate) hook_installs: Vec<HookInstall>,
    pub(crate) hooks_enabled: bool,
    pub(crate) hooks_started: bool,
    /// Sessions that said something while nobody was reading them. In memory:
    /// persisting it would put a field on the store, which
    /// `docs/sdlc/risk.yaml` holds at `paused`, and a restart already
    /// re-derives every other live reading in this app.
    pub(crate) unread_sessions: HashSet<Uuid>,
    /// The one session a person marked unread by hand. Exempt from being
    /// cleared by the view it is already in, until the selection moves — so the
    /// click that marks it is not the click that unmarks it.
    pub(crate) unread_hold: Option<Uuid>,
    /// A worktree that has just been made, whose repository carries a setup
    /// script the person has not yet agreed to run. Drawn as the block in the
    /// Worktrees tab; `None` is the ordinary case and draws nothing.
    pub(crate) pending_setup: Option<PendingSetup>,
    /// Which sessions are setup runs, so that the one notice about how a setup
    /// ended can be worded for a setup. In memory for the same reason
    /// `unread_sessions` is: the alternative is a store field, and the store is
    /// a paused surface. A restart costs one notice nobody is waiting for.
    pub(crate) setup_sessions: HashSet<Uuid>,
    /// Which script contents this machine has agreed to run, per project.
    pub(crate) setup_trust: SetupTrust,
    /// Every project's notes on a diff, and the one being written. Loaded from
    /// a sidecar beside the store, because a review interrupted is a review
    /// finished later and the store is a paused surface.
    pub(crate) diff_annotations: DiffAnnotations,
    /// Set while the review pane is being drawn, drained at the top of the next
    /// frame. The pane holds a borrow of the parsed diff while it runs, so it
    /// cannot reach the whole application to write the file; a flag on a
    /// disjoint field can be set from inside that borrow and acted on outside
    /// it.
    pub(crate) diff_comments_dirty: bool,
    /// Which changed files go into the next commit, per project. A selection
    /// rather than work: it is re-derived from `git status` whenever the list
    /// changes, so nothing is lost by keeping it out of the store.
    pub(crate) git_staged_files: HashMap<Uuid, HashSet<String>>,
    pub(crate) commit_message_input: String,
    /// The upstream reading behind the push line, per project.
    pub(crate) upstream_cache: HashMap<Uuid, UpstreamState>,
    /// What the machine is listening on, and which worktree each belongs to.
    /// One reading, shared by every session's header; nothing is persisted,
    /// because a port reading is true for thirty seconds.
    pub(crate) listening_ports: Vec<(ListeningPort, Option<PathBuf>)>,
    pub(crate) ports_scanned_at: Option<Instant>,
    /// Looking through one session's scrollback. Per session, and not
    /// persisted: a search is a question being asked right now.
    pub(crate) terminal_search: HashMap<Uuid, TerminalSearch>,
    /// What Claude Code last said about the account's limits, and when it said
    /// it. Not persisted: a percentage is true for minutes, and one read from a
    /// previous run would be drawn as current.
    pub(crate) rate_limits: Option<(RateLimits, Instant)>,
    /// Where a path seen in output actually is, once. `None` means it was
    /// looked for and is not a file, which is as worth remembering as a hit:
    /// output repeats itself, and the answer does not change while a session
    /// is running.
    pub(crate) resolved_paths: HashMap<String, Option<PathBuf>>,
    /// Bumped by every insert into, prune of, and clear of `resolved_paths`,
    /// so the list of files a terminal mentioned is rebuilt when the map
    /// changes and not on every frame. A length would miss a prune followed by
    /// an insert.
    pub(crate) resolved_paths_generation: u64,
    /// The files the selected session's terminal has mentioned, as
    /// `(session, generation, root, paths relative to root)`.
    pub(crate) touched_files_cache: Option<(Uuid, u64, PathBuf, Vec<PathBuf>)>,
    /// Candidates the pane found while drawing and the cache has not heard of.
    /// Filled inside the draw and drained outside it, because a `fs::metadata`
    /// call in a draw path is what the Rust rule forbids.
    pub(crate) unresolved_paths: Vec<String>,
    /// Which session the current resolved_paths cache belongs to.
    pub(crate) resolved_paths_session: Option<Uuid>,
    /// The line an editor document should be scrolled to when it next draws.
    /// Spent by the frame that uses it, so scrolling by hand afterwards stands.
    pub(crate) editor_jump_line: Option<usize>,
    /// A right-click on a resolved path, and which session it came from.
    pub(crate) terminal_path_menu: Option<(Uuid, PathBuf)>,
    /// User prompt turns recorded per session for navigation.
    pub(crate) session_prompt_turns: HashMap<Uuid, Vec<PromptTurn>>,
    /// Whether the one side panel beside the terminal is open. Defaults to
    /// true; folded from its own tab row and reopened from the header.
    pub(crate) show_session_inspector: bool,
    /// Which tab of that panel is showing.
    pub(crate) session_inspector_tab: InspectorTab,
    /// Which side of the terminal the inspector panel is docked to.
    pub(crate) session_inspector_side: SidebarSide,
    /// Custom width for the session inspector when resized by dragging.
    pub(crate) session_inspector_w: Option<f32>,
    /// Active drag of a sidebar tab, if any.
    pub(crate) dragging_sidebar_tab: Option<(InspectorTab, SidebarSide)>,
    /// Custom width for the workspace session list sidebar.
    pub(crate) session_list_w: f32,
    /// The session list's share of its column while the side panel is
    /// stacked under it.
    pub(crate) session_list_split: f32,
    /// What the files tab is narrowed to. Empty draws the tree.
    pub(crate) session_file_filter: String,
    /// The paths that filter matched, as `(root, query, matches)`, so a scan of
    /// thousands of paths is searched when the query changes and not per frame.
    pub(crate) session_filter_results: Option<(PathBuf, String, Vec<PathBuf>)>,
    /// Whether the folded 「隠しフォルダ」 row of the files tab is open.
    pub(crate) session_hidden_expanded: bool,
    /// The session row the pointer was over last frame. A row learns it is
    /// hovered only after it is drawn, and its `···` and `✕` are drawn in
    /// place of the time while it is, so the answer is one frame old.
    pub(crate) hovered_session_row: Option<Uuid>,
    /// Cache of file scans for worktree roots in session view, keyed by worktree
    /// path. Prevents collision with `file_cache` (which holds project roots
    /// keyed by `project_id`).
    pub(crate) session_file_cache: HashMap<PathBuf, FileScanSnapshot>,
    /// Cached file tree structure for the active root in session view. Avoids
    /// re-running `build_file_tree` on thousands of paths every frame (60 FPS).
    /// Held as `(root, visible, hidden)`: the top-level entries whose name
    /// starts with a dot are split off when the tree is built, not per frame.
    pub(crate) session_file_tree_cache: Option<(PathBuf, Vec<FileTreeNode>, Vec<FileTreeNode>)>,
    /// Latest automated AI review findings for a project diff.
    pub(crate) diff_ai_review: HashMap<Uuid, String>,
    /// Last git commit failure details per project (message, staged files, error trace) for AI recovery.
    pub(crate) last_commit_failure: HashMap<Uuid, (String, Vec<String>, String)>,
    pub(crate) pending_land_worktree: Option<(Uuid, PathBuf, String)>,
    pub(crate) land_auto_cleanup: bool,
    pub(crate) pending_pr_modal: Option<(Uuid, PathBuf, String, String, bool)>,
    pub(crate) cached_github_issues: HashMap<Uuid, Vec<GitHubIssue>>,
    pub(crate) session_checkpoints: HashMap<Uuid, Vec<Checkpoint>>,
    pub(crate) pending_run_script: Option<PendingRunScript>,
}

impl OperonApp {
    pub(crate) fn new() -> Self {
        let data_file = app_data_file();
        // A first launch has no record here and no legacy snapshot to import.
        // That is the one moment the system's own language decides anything:
        // an existing store keeps whatever its owner already chose, and serde
        // fills the field as Japanese for stores written before it existed.
        let fresh_install =
            !data_file.exists() && !legacy_app_data_files().iter().any(|path| path.exists());
        let loaded = load_store_safely(&data_file);
        let mut store = loaded.store;
        if fresh_install {
            store.language = detected_language();
        }
        Self::from_state(loaded.data_file, store, ToolStatus::detect(), loaded.notice)
    }

    pub(crate) fn new_with_instance_locks(
        instance_locks: Vec<fs::File>,
        migration: MigrationOutcome,
    ) -> Self {
        let mut app = Self::new();
        app._instance_locks = instance_locks;
        app.store_retry_pending = migration.durability_retry_needed;
        if migration.durability_retry_needed {
            app.durability_warning = Some(
                tr("取り込んだメタデータのディスク永続化はまだ確認できていません。再試行が終わるまで Operon を開いたままにしてください。")
                    .into(),
            );
        }
        if let Some(migration_notice) = migration.notice {
            app.notice = Some(match app.notice {
                Some(existing) => format!("{migration_notice} {existing}"),
                None => migration_notice,
            });
        }
        app
    }

    pub(crate) fn from_state(
        data_file: PathBuf,
        mut store: Store,
        tools: ToolStatus,
        mut notice: Option<String>,
    ) -> Self {
        set_active_language(store.language);
        if store.notifications_enabled {
            request_notification_authorization();
        }
        let accounts = load_agent_accounts(&data_file);
        // A notes file that stopped parsing is moved aside before anything can
        // replace it, and where it went is said out loud — an empty set that
        // overwrites twelve notes is the failure, not the empty set itself.
        let (rescued_comments, rescued_comments_path) = load_diff_comments_reporting(&data_file);
        if let Some(rescued) = rescued_comments_path {
            let warning = tf!(
                "レビューのコメントファイルを読めなかったため、{path} に退避しました。コメントは空から始まります。",
                path = rescued.display()
            );
            notice = Some(match notice {
                Some(existing) => format!("{existing} {warning}"),
                None => warning,
            });
        }
        // Before anything is drawn, because the keys have to be settled by the
        // first frame. A file that cannot be read leaves the defaults running
        // and says so beside every other startup notice.
        let (keymap, keymap_notice) = read_keymap();
        if let Some(warning) = keymap_notice {
            notice = Some(match notice {
                Some(existing) => format!("{existing} {warning}"),
                None => warning,
            });
        }
        match load_cancellation_intents(&data_file) {
            Ok(recovered) => store.pending_cancellations.extend(recovered),
            Err(error) => {
                let warning = tf!(
                    "キャンセル復旧用メタデータを読み取れなかったため、上書きしていません: {error}",
                    error = error
                );
                notice = Some(match notice {
                    Some(existing) => format!("{existing} {warning}"),
                    None => warning,
                });
            }
        }
        // A record older than this schema carries failures that were never read
        // off an exit status: the build that wrote it called a terminal it could
        // not find a failure, so a machine that restarted overnight took the
        // tmux server with it and turned every session on the list red. Put
        // those verdicts back to the one state that means "ask tmux", and let
        // the ordinary poll re-judge each against whatever evidence survives. A
        // pane still sitting there with a non-zero status is a failure again on
        // the next poll; one that is gone is recorded as unresolved, which is
        // all this app ever knew about it. Skipped without tmux, because then
        // nothing would ever answer and the question is worse than the wrong
        // answer it replaces.
        if store.schema_version < EVIDENCE_BASED_FAILURE_SCHEMA_VERSION && tools.tmux {
            for session in &mut store.sessions {
                if session.status == SessionStatus::Failed {
                    session.status = SessionStatus::Unknown;
                }
            }
        }
        let pending_cancellations = std::mem::take(&mut store.pending_cancellations);
        let mut seen_cancellations = HashSet::new();
        for session_id in pending_cancellations {
            let Some(session) = store
                .sessions
                .iter_mut()
                .find(|session| session.id == session_id)
            else {
                continue;
            };
            if matches!(
                session.status,
                SessionStatus::Exited | SessionStatus::Cancelled | SessionStatus::Queued
            ) {
                continue;
            }
            // An older retained app can observe a completed tmux stop as
            // Failed because it does not understand the sidecar. Restore the
            // pollable state so the current app can reconcile it as Cancelled.
            session.status = SessionStatus::Active;
            if seen_cancellations.insert(session_id) {
                store.pending_cancellations.push(session_id);
            }
        }
        let page = initial_page(&store);
        let recent_agent_settings = load_recent_agent_settings(&data_file);
        let selected_agent = recent_agent_settings
            .last_selected_agent
            .as_deref()
            .filter(|agent| tools.agent_available(agent))
            .map(|agent| agent.to_owned())
            .unwrap_or_else(|| default_agent(&tools).to_owned());
        let (
            agent_model_input,
            agent_mode_input,
            agent_effort_input,
            agent_flag_inputs,
            custom_command,
        ) = if let Some(settings) = recent_agent_settings.agents.get(&selected_agent) {
            (
                settings.model.clone(),
                settings.mode.clone(),
                settings.effort.clone(),
                settings.flags.clone(),
                settings.custom_command.clone(),
            )
        } else {
            (
                String::new(),
                String::new(),
                String::new(),
                Vec::new(),
                String::new(),
            )
        };
        // The toggle lives beside the scripts rather than in the store: the
        // store is where a session's own history is kept, and this is a fact
        // about the machine's CLI configuration.
        let hooks_enabled = load_hook_settings(&data_file).enabled;
        let setup_trust = load_setup_trust(&data_file);
        let diff_annotations = DiffAnnotations {
            comments: rescued_comments.comments,
            ..DiffAnnotations::default()
        };
        let (background_sender, background_receiver) = mpsc::channel();
        Self {
            _instance_locks: Vec::new(),
            store,
            data_file,
            page,
            selected_project: None,
            project_tab: ProjectTab::Overview,
            settings_section: SettingsSection::Appearance,
            git_view: GitView::Changes,
            session_view: SessionView::List,
            session_library_open: false,
            status_filter: [false; 4],
            git_selected_file: None,
            selected_file: None,
            project_path_input: String::new(),
            workspace_scan_input: String::new(),
            show_manual_project_entry: false,
            goal_input: String::new(),
            session_name_input: String::new(),
            selected_agent,
            custom_command,
            agent_account_input: None,
            account_agent_input: "claude".to_owned(),
            account_name_input: String::new(),
            agent_model_input,
            agent_mode_input,
            agent_effort_input,
            agent_flag_inputs,
            recent_agent_settings,
            acknowledged_launch: None,
            session_path_input: String::new(),
            session_path_project: None,
            worktree_branch_input: String::new(),
            pending_worktree_removal: None,
            pending_project_removal: None,
            pending_session_close: None,
            renaming_session: None,
            rename_input: String::new(),
            rename_needs_focus: false,
            remove_after_close: HashSet::new(),
            selected_session: None,
            session_output: HashMap::new(),
            session_activity: HashMap::new(),
            session_previews: HashMap::new(),
            terminal_layouts: HashMap::new(),
            terminal_cursors: HashMap::new(),
            terminal_preedits: HashMap::new(),
            terminal_input_queue: HashMap::new(),
            terminal_resize_requested: HashMap::new(),
            terminal_resize_applied: HashMap::new(),
            depends_on_input: None,
            window_maximized: false,
            last_session_refresh: Instant::now(),
            last_editor_git_refresh: HashMap::new(),
            store_retry_pending: false,
            search: String::new(),
            file_search: String::new(),
            transcript_search_input: String::new(),
            transcript_matches: Vec::new(),
            cli_sessions: HashMap::new(),
            handoff_note_input: String::new(),
            handoff_target_inputs: HashMap::new(),
            managed_handoff_targets: HashMap::new(),
            git_changes_cache: HashMap::new(),
            git_diff_cache: HashMap::new(),
            git_diff_generations: HashMap::new(),
            git_history_cache: HashMap::new(),
            worktree_cache: HashMap::new(),
            pull_request_cache: HashMap::new(),
            file_cache: HashMap::new(),
            open_documents: Vec::new(),
            active_document: None,
            editor_view: EditorView::Edit,
            expanded_directories: HashSet::new(),
            collapsed_diff_files: HashSet::new(),
            file_tree_cache: None,
            diff_file_cache: HashMap::new(),
            editor_text_layout_cache: EditorTextLayoutCache::default(),
            markdown_preview_cache: MarkdownPreviewCache::default(),
            last_editor_file_watch: None,
            skill_cache: HashMap::new(),
            rule_cache: HashMap::new(),
            background_tasks: HashSet::new(),
            background_sender,
            background_receiver,
            command_palette_open: false,
            command_selection: 0,
            command_palette_needs_focus: false,
            command_search: String::new(),
            orphaned_tmux_sessions: Vec::new(),
            tools,
            notice,
            brief_notice: None,
            keymap,
            accounts,
            restore_progress: None,
            launch_progress: None,
            launch_sheet_open: false,
            launch_sheet_needs_focus: false,
            launch_sheet_is_git: false,
            history_project: None,
            durability_warning: None,
            applied_theme: None,
            applied_terminal_font: None,
            hook_listener: None,
            hook_events: None,
            hook_status: HashMap::new(),
            hook_launch_tokens: HashMap::new(),
            hook_installs: Vec::new(),
            hooks_enabled,
            hooks_started: false,
            unread_sessions: HashSet::new(),
            pending_setup: None,
            setup_sessions: HashSet::new(),
            setup_trust,
            diff_annotations,
            diff_comments_dirty: false,
            git_staged_files: HashMap::new(),
            commit_message_input: String::new(),
            upstream_cache: HashMap::new(),
            listening_ports: Vec::new(),
            ports_scanned_at: None,
            terminal_search: HashMap::new(),
            rate_limits: None,
            resolved_paths: HashMap::new(),
            resolved_paths_generation: 0,
            touched_files_cache: None,
            unresolved_paths: Vec::new(),
            resolved_paths_session: None,
            editor_jump_line: None,
            terminal_path_menu: None,
            session_prompt_turns: HashMap::new(),
            show_session_inspector: true,
            session_inspector_tab: InspectorTab::Files,
            session_inspector_side: SidebarSide::Left,
            session_inspector_w: None,
            dragging_sidebar_tab: None,
            session_list_w: SIDEBAR_DEFAULT_W,
            session_list_split: SIDEBAR_SPLIT_DEFAULT,
            session_file_filter: String::new(),
            session_filter_results: None,
            session_hidden_expanded: false,
            hovered_session_row: None,
            session_file_cache: HashMap::new(),
            session_file_tree_cache: None,
            unread_hold: None,
            diff_ai_review: HashMap::new(),
            last_commit_failure: HashMap::new(),
            pending_land_worktree: None,
            land_auto_cleanup: true,
            pending_pr_modal: None,
            cached_github_issues: HashMap::new(),
            session_checkpoints: HashMap::new(),
            pending_run_script: None,
        }
    }

    pub(crate) fn record_current_agent_settings(&mut self) {
        let agent = self.selected_agent.clone();
        let is_custom = agent == "custom";
        let settings = AgentLaunchSettings {
            model: if is_custom {
                String::new()
            } else {
                self.agent_model_input.trim().to_owned()
            },
            mode: if is_custom {
                String::new()
            } else {
                self.agent_mode_input.trim().to_owned()
            },
            effort: if is_custom {
                String::new()
            } else {
                self.agent_effort_input.trim().to_owned()
            },
            flags: if is_custom {
                Vec::new()
            } else {
                self.agent_flag_inputs.clone()
            },
            custom_command: if is_custom {
                self.custom_command.trim().to_owned()
            } else {
                String::new()
            },
        };
        self.recent_agent_settings.last_selected_agent = Some(agent.clone());
        self.recent_agent_settings.agents.insert(agent, settings);
    }

    pub(crate) fn persist_recent_agent_settings(&mut self) {
        let prev = self.recent_agent_settings.clone();
        self.record_current_agent_settings();
        if prev == self.recent_agent_settings {
            return;
        }
        if let Err(error) = save_recent_agent_settings(&self.data_file, &self.recent_agent_settings)
        {
            eprintln!("Operon: failed to save recent agent settings: {error}");
        }
    }

    pub(crate) fn apply_agent_launch_settings(&mut self, agent: &str) {
        if let Some(settings) = self.recent_agent_settings.agents.get(agent).cloned() {
            self.agent_model_input = settings.model;
            self.agent_mode_input = settings.mode;
            self.agent_effort_input = settings.effort;
            self.agent_flag_inputs = settings.flags;
            if agent == "custom" {
                self.custom_command = settings.custom_command;
            } else {
                self.custom_command.clear();
            }
        } else {
            self.agent_model_input.clear();
            self.agent_mode_input.clear();
            self.agent_effort_input.clear();
            self.agent_flag_inputs.clear();
            self.custom_command.clear();
        }
    }

    pub(crate) fn selected_project(&self) -> Option<&Project> {
        self.selected_project
            .and_then(|id| self.store.projects.iter().find(|project| project.id == id))
    }

    pub(crate) fn select_project(&mut self, project_id: Option<Uuid>) {
        if self.selected_project != project_id {
            self.goal_input.clear();
            self.session_name_input.clear();
            self.acknowledged_launch = None;
            self.session_path_input.clear();
            self.session_path_project = None;
            self.worktree_branch_input.clear();
            self.depends_on_input = None;
            self.pending_worktree_removal = None;
            self.pending_project_removal = None;
            self.pending_session_close = None;
            self.selected_session = None;
            self.session_library_open = false;
            self.selected_file = None;
            self.git_selected_file = None;
            self.active_document = None;
            self.file_search.clear();
            self.handoff_note_input.clear();
            self.handoff_target_inputs.clear();
            self.managed_handoff_targets.clear();
        }
        self.selected_project = project_id;
    }

    pub(crate) fn request_system_action(
        &mut self,
        action: impl Into<String>,
        job: impl FnOnce() -> Result<()> + Send + 'static,
    ) {
        let id = Uuid::new_v4();
        let action = action.into();
        let result_action = action.clone();
        self.spawn_background(BackgroundKey::SystemAction(id), move || {
            BackgroundResult::SystemAction {
                id,
                action: result_action,
                result: job().map_err(|error| error.to_string()),
            }
        });
        self.notice = Some(format!("{action}…"));
    }

    /// A success that only needs to be seen: drawn as a toast and gone after
    /// `NOTICE_TOAST_SECONDS`. A failure goes through `notice` and stays.
    pub(crate) fn notice_briefly(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.brief_notice = Some((text.clone(), Instant::now()));
        self.notice = Some(text);
    }

    /// Clears the notice once its brief time is up, and says how long is left
    /// otherwise so the frame loop can ask for a repaint then.
    pub(crate) fn expire_brief_notice(&mut self, now: Instant) -> Option<Duration> {
        if !notice_is_brief(&self.notice, &self.brief_notice) {
            self.brief_notice = None;
            return None;
        }
        let (_, set_at) = self.brief_notice.as_ref()?;
        let lifetime = Duration::from_secs(NOTICE_TOAST_SECONDS);
        let elapsed = now.saturating_duration_since(*set_at);
        if elapsed >= lifetime {
            self.notice = None;
            self.brief_notice = None;
            None
        } else {
            Some(lifetime - elapsed)
        }
    }

    pub(crate) fn persist(&mut self) -> bool {
        self.store.schema_version = STORE_SCHEMA_VERSION;
        let result = save_store(&self.data_file, &self.store);
        self.apply_persist_result(result)
    }

    pub(crate) fn apply_appearance(&mut self, ctx: &egui::Context) {
        if self.applied_theme != Some(self.store.theme) {
            apply_interface_metrics(ctx);
            ctx.set_visuals(self.store.theme.visuals());
            self.applied_theme = Some(self.store.theme);
            // Terminal output is coloured once, when tmux hands it over, and
            // then kept as a laid-out job. Those jobs hold the previous
            // theme's ANSI table, so a theme switch has to throw them away or
            // every open terminal keeps the colours of the theme it was
            // captured under.
            self.terminal_layouts.clear();
        }
        if self.applied_terminal_font != Some(self.store.terminal_font) {
            configure_macos_fonts(ctx, self.store.terminal_font);
            self.applied_terminal_font = Some(self.store.terminal_font);
        }
    }

    pub(crate) fn apply_persist_result(&mut self, result: Result<WriteOutcome>) -> bool {
        match result {
            Ok(WriteOutcome::Durable) => true,
            Ok(WriteOutcome::CommittedButNotSynced(error)) => {
                self.store_retry_pending = true;
                let warning = tf!("変更は書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。エラー: {error}", error = error);
                self.durability_warning = Some(warning);
                true
            }
            Err(error) => {
                self.notice = Some(tf!(
                    "データを保存できませんでした。直前の変更は適用されていません: {error}",
                    error = error
                ));
                false
            }
        }
    }

    pub(crate) fn persist_external_transition(&mut self, success: &str, external_effect: &str) {
        match save_store_then_cancellation_intents(&self.data_file, &self.store) {
            Ok(WriteOutcome::Durable) => {
                self.store_retry_pending = false;
                self.durability_warning = None;
                self.notice = Some(success.into());
            }
            Ok(WriteOutcome::CommittedButNotSynced(error)) => {
                self.store_retry_pending = true;
                let warning = tf!("{external_effect}。メタデータは書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。エラー: {error}", error = error, external_effect = external_effect);
                self.durability_warning = Some(warning);
            }
            Err(error) => {
                self.store_retry_pending = true;
                self.notice = Some(tf!("{external_effect}が、ローカルのメタデータを保存できませんでした。Operon が自動的に再試行します。アプリを開いたままにするか、再起動後にターミナル復旧を使ってください。エラー: {error}", error = error, external_effect = external_effect));
            }
        }
    }

    pub(crate) fn retry_pending_store(&mut self) {
        if !self.store_retry_pending {
            return;
        }
        match save_store_then_cancellation_intents(&self.data_file, &self.store) {
            Ok(WriteOutcome::Durable) => {
                self.store_retry_pending = false;
                self.durability_warning = None;
                self.notice = Some(tr("保留していたセッションのメタデータを保存しました。").into());
            }
            Ok(WriteOutcome::CommittedButNotSynced(error)) => {
                let warning = tf!("セッションのメタデータを再度書き込みましたが、ディスクへの永続化はまだ確認できていません。Operon が再試行します。エラー: {error}", error = error);
                self.durability_warning = Some(warning);
            }
            Err(error) => {
                self.notice = Some(tf!("セッションのメタデータはまだ保存待ちです。Operon が再試行します。エラー: {error}", error = error));
            }
        }
    }

    pub(crate) fn worktree_mutation_running(&self, project_id: Uuid) -> bool {
        self.background_tasks
            .contains(&BackgroundKey::WorktreeMutation(project_id))
    }

    pub(crate) fn cancellation_pending(&self, session_id: Uuid) -> bool {
        self.store.pending_cancellations.contains(&session_id)
    }

    pub(crate) fn spawn_background(
        &mut self,
        key: BackgroundKey,
        job: impl FnOnce() -> BackgroundResult + Send + 'static,
    ) {
        if !self.background_tasks.insert(key) {
            return;
        }
        let sender = self.background_sender.clone();
        thread::spawn(move || {
            let _ = sender.send(job());
        });
    }

    pub(crate) fn process_background_results(&mut self) {
        while let Ok(result) = self.background_receiver.try_recv() {
            match result {
                BackgroundResult::HooksApplied { installs } => {
                    self.background_tasks.remove(&BackgroundKey::HookApply);
                    // Only a failure is worth interrupting for: a CLI that is
                    // simply not installed is a row in Settings, not a notice.
                    if let Some(failure) =
                        installs.iter().find_map(|install| match &install.state {
                            HookInstallState::Failed(error) => {
                                Some((install.provider.label(), error.clone()))
                            }
                            _ => None,
                        })
                    {
                        self.notice = Some(tf!(
                            "{p0} のフックを登録できませんでした: {error}",
                            p0 = failure.0,
                            error = failure.1
                        ));
                    }
                    self.hook_installs = installs;
                }
                BackgroundResult::WorkspaceScan {
                    workspace,
                    repositories,
                    truncated,
                } => {
                    self.background_tasks.remove(&BackgroundKey::WorkspaceScan);
                    self.finish_workspace_import(workspace, repositories, truncated);
                }
                BackgroundResult::GitChanges { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::GitChanges(project_id));
                    self.apply_git_changes(project_id, result);
                }
                BackgroundResult::GitDiff {
                    project_id,
                    file,
                    generation,
                    result,
                } => {
                    self.background_tasks
                        .remove(&BackgroundKey::GitDiff(project_id, file.clone()));
                    let key = (project_id, file.clone());
                    if self.git_diff_generations.get(&key).copied().unwrap_or(0) != generation {
                        self.request_latest_git_diff(project_id, file);
                        continue;
                    }
                    // The parse is derived from this text, so it has to go
                    // before the text it was derived from is replaced.
                    self.diff_file_cache.remove(&key);
                    self.git_diff_cache.insert(key, result);
                }
                BackgroundResult::GitHistory { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::GitHistory(project_id));
                    self.git_history_cache.insert(project_id, result);
                }
                BackgroundResult::Worktrees { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::Worktrees(project_id));
                    self.worktree_cache.insert(project_id, result);
                }
                BackgroundResult::PullRequests { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::PullRequests(project_id));
                    self.pull_request_cache.insert(project_id, result);
                }
                BackgroundResult::Files { project_id, scan } => {
                    self.background_tasks
                        .remove(&BackgroundKey::Files(project_id));
                    self.file_cache.insert(project_id, scan);
                    self.file_tree_cache = None;
                    self.session_file_tree_cache = None;
                    self.session_filter_results = None;
                }
                BackgroundResult::WorktreeFiles { root, scan } => {
                    self.background_tasks
                        .remove(&BackgroundKey::WorktreeFiles(root.clone()));
                    self.session_file_cache.insert(root, scan);
                    self.session_file_tree_cache = None;
                    self.session_filter_results = None;
                }
                BackgroundResult::FileOpened { document, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::FileOpen(document.clone()));
                    self.apply_opened_file(&document, result);
                }
                BackgroundResult::FileSaved { document, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::FileSave(document.clone()));
                    self.apply_saved_file(&document, result);
                }
                BackgroundResult::FileWatched {
                    document,
                    known,
                    result,
                } => {
                    self.background_tasks
                        .remove(&BackgroundKey::FileWatch(document.clone()));
                    self.apply_watched_file(&document, &known, result);
                }
                BackgroundResult::Skills { project_id, scan } => {
                    self.background_tasks
                        .remove(&BackgroundKey::Skills(project_id));
                    self.skill_cache.insert(project_id, scan);
                }
                BackgroundResult::Rules { project_id, scan } => {
                    self.background_tasks
                        .remove(&BackgroundKey::Rules(project_id));
                    self.rule_cache.insert(project_id, scan);
                }
                BackgroundResult::SessionPoll {
                    states,
                    activities,
                    previews,
                    output,
                } => {
                    self.background_tasks.remove(&BackgroundKey::SessionPoll);
                    self.apply_session_poll(states, activities, previews, output);
                }
                BackgroundResult::CliSessions {
                    project_id,
                    scan,
                    announce,
                } => {
                    self.background_tasks
                        .remove(&BackgroundKey::CliSessions(project_id));
                    let count = scan.sessions.len();
                    self.remember_cli_sessions(project_id, &scan.sessions);
                    self.cli_sessions.insert(project_id, scan.sessions);
                    if announce || scan.truncated || scan.unreadable_entries > 0 {
                        self.notice = Some(if scan.truncated || scan.unreadable_entries > 0 {
                            tf!(
                                "上限付きスキャンで、このマシンの Codex・Claude・Antigravity のセッションを {count} 件見つけました。一部の履歴は省略されています{unreadable}。",
                                count = count,
                                unreadable = if scan.unreadable_entries > 0 {
                                    tf!("（{unreadable_entries} 件のエントリを読み取れませんでした）", unreadable_entries = scan.unreadable_entries)
                                } else {
                                    String::new()
                                }
                            )
                        } else {
                            tf!("このプロジェクトのローカル CLI セッションを {count} 件見つけました。", count = count)
                        });
                    }
                }
                BackgroundResult::TranscriptSearch { query, scan } => {
                    self.background_tasks
                        .remove(&BackgroundKey::TranscriptSearch);
                    let count = scan.matches.len();
                    self.transcript_matches = scan.matches;
                    self.notice = Some(if scan.truncated || scan.unreadable_entries > 0 {
                        tf!(
                            "上限付き検索で “{query}” に一致するローカル履歴を {count} 件見つけました（{files} ファイル、{mebibytes} MiB、{lines} 行を走査）。一部の履歴は省略されています{unreadable}。",
                            query = query,
                            count = count,
                            files = scan.scanned_files,
                            mebibytes = scan.scanned_bytes / (1024 * 1024),
                            lines = scan.scanned_lines,
                            unreadable = if scan.unreadable_entries > 0 {
                                tf!("（{unreadable_entries} 件のエントリを読み取れませんでした）", unreadable_entries = scan.unreadable_entries)
                            } else {
                                String::new()
                            }
                        )
                    } else {
                        tf!(
                            "“{query}” に一致するローカル履歴を {count} 件見つけました。",
                            count = count,
                            query = query
                        )
                    });
                }
                BackgroundResult::SessionStarted { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::SessionStart(session_id));
                    let previous_store = self.store.clone();
                    match result {
                        Ok(branch) => {
                            if let Some(session) = self
                                .store
                                .sessions
                                .iter_mut()
                                .find(|session| session.id == session_id)
                            {
                                session.status = SessionStatus::Active;
                                session.branch = branch;
                            }
                            // Make every successful launch visible in Operon itself.
                            // The inspector captures tmux output and sends user input back
                            // to the same pane, so Claude's first-run prompt and later
                            // conversation remain usable without a separate Terminal window.
                            let was_dismissed = self
                                .launch_progress
                                .as_ref()
                                .is_some_and(|p| p.session_id == session_id && p.dismissed);
                            if self
                                .launch_progress
                                .as_ref()
                                .is_some_and(|p| p.session_id == session_id)
                            {
                                self.launch_progress = None;
                            }
                            if !was_dismissed {
                                self.select_session(session_id);
                                self.page = Page::Sessions;
                                self.session_library_open = false;
                                self.request_session_output(session_id);
                            }
                            self.persist_external_transition(
                                tr("エージェントセッションを起動し、アプリ内ターミナルで開きました。"),
                                tr("エージェントセッションは tmux で起動しました"),
                            );
                            self.request_native_session_resolution(session_id);
                        }
                        Err(error) => {
                            if let Some(session) = self
                                .store
                                .sessions
                                .iter_mut()
                                .find(|session| session.id == session_id)
                            {
                                session.status = SessionStatus::Failed;
                            }
                            if let Some(progress) = self
                                .launch_progress
                                .as_mut()
                                .filter(|p| p.session_id == session_id)
                            {
                                progress.failure = Some(error.clone());
                            }
                            if self.persist() {
                                self.notice = Some(tf!(
                                    "セッションを開始できませんでした: {error}",
                                    error = error
                                ));
                            } else {
                                self.store = previous_store;
                                self.notice = Some(tf!("セッションを開始できませんでした: {error}。失敗状態を保存できなかったため、改めて確認します。", error = error));
                            }
                        }
                    }
                }
                BackgroundResult::EmptySessionStarted { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::EmptySessionStart(session_id));
                    let previous_store = self.store.clone();
                    match result {
                        Ok(branch) => {
                            if let Some(session) = self
                                .store
                                .sessions
                                .iter_mut()
                                .find(|session| session.id == session_id)
                            {
                                session.status = SessionStatus::Active;
                                session.branch = branch;
                            }
                            let was_dismissed = self
                                .launch_progress
                                .as_ref()
                                .is_some_and(|p| p.session_id == session_id && p.dismissed);
                            if self
                                .launch_progress
                                .as_ref()
                                .is_some_and(|p| p.session_id == session_id)
                            {
                                self.launch_progress = None;
                            }
                            if !was_dismissed {
                                self.select_session(session_id);
                                self.page = Page::Sessions;
                                self.session_library_open = false;
                                self.request_session_output(session_id);
                            }
                            self.persist_external_transition(
                                tr("空のターミナルセッションを tmux で起動しました。"),
                                tr("空のターミナルセッションは tmux で起動しました"),
                            );
                        }
                        Err(error) => {
                            if let Some(session) = self
                                .store
                                .sessions
                                .iter_mut()
                                .find(|session| session.id == session_id)
                            {
                                session.status = SessionStatus::Failed;
                            }
                            if let Some(progress) = self
                                .launch_progress
                                .as_mut()
                                .filter(|p| p.session_id == session_id)
                            {
                                progress.failure = Some(error.clone());
                            }
                            if self.persist() {
                                self.notice = Some(tf!(
                                    "空のターミナルを開始できませんでした: {error}",
                                    error = error
                                ));
                            } else {
                                self.store = previous_store;
                                self.notice = Some(tf!("空のターミナルを開始できませんでした: {error}。失敗状態を保存できなかったため、改めて確認します。", error = error));
                            }
                        }
                    }
                }
                BackgroundResult::SessionStopped { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::SessionStop(session_id));
                    let remove_requested = self.remove_after_close.remove(&session_id);
                    match result {
                        Ok(()) => {
                            self.store
                                .pending_cancellations
                                .retain(|pending| *pending != session_id);
                            if let Some(session) = self
                                .store
                                .sessions
                                .iter_mut()
                                .find(|session| session.id == session_id)
                            {
                                session.status = SessionStatus::Cancelled;
                            }
                            self.persist_external_transition(
                                tr("セッションをキャンセルしました。"),
                                tr("tmux セッションはキャンセルされました"),
                            );
                            if remove_requested {
                                self.remove_session_record(session_id);
                            }
                        }
                        Err(error) => {
                            let already_cancelled = self
                                .store
                                .sessions
                                .iter()
                                .find(|session| session.id == session_id)
                                .is_some_and(|session| session.status == SessionStatus::Cancelled);
                            let gone = tmux_error_state(&error) == TmuxState::Gone;
                            if already_cancelled || gone {
                                if let Some(session) = self
                                    .store
                                    .sessions
                                    .iter_mut()
                                    .find(|session| session.id == session_id)
                                {
                                    session.status = SessionStatus::Cancelled;
                                }
                                self.store
                                    .pending_cancellations
                                    .retain(|pending| *pending != session_id);
                                self.persist_external_transition(
                                    tr("セッションをキャンセルしました。"),
                                    tr("tmux セッションはキャンセルされました"),
                                );
                                if remove_requested {
                                    self.remove_session_record(session_id);
                                }
                                continue;
                            }
                            // The 「停止」 this asks for must still finish a
                            // delete the person already confirmed, and the
                            // notice says so: a stop that deletes without
                            // saying it would is a surprise.
                            if remove_requested {
                                self.remove_after_close.insert(session_id);
                                self.notice = Some(tf!("セッションが停止したか確認できませんでした: {error}。削除は保留中です。もう一度「停止」を実行すると、停止を確かめてから記録を削除します。", error = error));
                            } else {
                                self.notice = Some(tf!("セッションが停止したか確認できませんでした: {error}。永続化されたキャンセル指示は保留のままです。もう一度「停止」を実行してください。", error = error));
                            }
                        }
                    }
                }
                BackgroundResult::SessionOutput { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::SessionOutput(session_id));
                    match result {
                        Ok(capture) => {
                            self.set_session_output(session_id, capture.output, capture.cursor);
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "ターミナル出力を読み取れませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                }
                BackgroundResult::FullHistoryImported(imported_result) => {
                    let FullHistoryImportResult {
                        import_id,
                        project,
                        source,
                        target,
                        note,
                        result,
                    } = *imported_result;
                    self.background_tasks
                        .remove(&BackgroundKey::FullHistoryImport(import_id));
                    // Only this restore's own modal closes. A second restore
                    // started from the project page would otherwise lose its
                    // window to whichever import happened to land first.
                    if self
                        .restore_progress
                        .as_ref()
                        .is_some_and(|progress| progress.import_id == import_id)
                    {
                        self.restore_progress = None;
                    }
                    match result {
                        Ok(imported) => self.finish_full_history_import(
                            &project, &source, &target, &note, imported,
                        ),
                        Err(error) => {
                            self.notice = Some(tf!("会話全履歴を復元できませんでした。元の CLI セッションは変更されていません: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::NativeSessionResolved { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::NativeSessionResolution(session_id));
                    match result {
                        Ok(native) => {
                            let previous_store = self.store.clone();
                            if let Some(session) = self
                                .store
                                .sessions
                                .iter_mut()
                                .find(|session| session.id == session_id)
                            {
                                session.native_session_id = Some(native.native_id.clone());
                                session.native_session_path = Some(native.path.clone());
                                if native.provider == CliProvider::Gemini {
                                    session.agent_command =
                                        native.provider.native_resume_command(&native.id);
                                }
                            } else {
                                continue;
                            }
                            if self.persist() {
                                self.notice = Some(tf!(
                                    "ネイティブ再開の準備ができました: {p0}",
                                    p0 = native_resume_display(native.provider, &native.native_id)
                                ));
                            } else {
                                self.store = previous_store;
                                self.notice = Some(
                                    tr("ネイティブ再開 ID は見つかりましたが、まだ保存できていません。Operon がローカル履歴から再検出します。")
                                        .into(),
                                );
                            }
                        }
                        Err(error) => {
                            self.notice = Some(tf!("新しいネイティブセッションをまだ特定できません: {error}。「ローカル CLI セッションを検出」から ID を指定して再開してください。", error = error));
                        }
                    }
                }
                BackgroundResult::TerminalInputSent { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::TerminalInput(session_id));
                    match result {
                        Ok(()) => {
                            self.request_session_output(session_id);
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "ターミナルに入力を送れませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                    // Input that arrived while tmux was handling the previous
                    // frame remains ordered in this queue.
                    self.flush_terminal_input(session_id);
                }
                BackgroundResult::TerminalResized {
                    session_id,
                    size,
                    result,
                } => {
                    self.background_tasks
                        .remove(&BackgroundKey::TerminalResize(session_id));
                    match result {
                        Ok(()) => {
                            self.terminal_resize_applied.insert(session_id, size);
                            self.request_session_output(session_id);
                        }
                        Err(error) => {
                            if self.terminal_resize_requested.get(&session_id) == Some(&size) {
                                self.terminal_resize_requested.remove(&session_id);
                            }
                            if tmux_error_state(&error) == TmuxState::Gone {
                                let cancellation_was_pending =
                                    self.cancellation_pending(session_id);
                                if cancellation_was_pending {
                                    self.store
                                        .pending_cancellations
                                        .retain(|pending| *pending != session_id);
                                    if let Some(session) =
                                        self.store.sessions.iter_mut().find(|session| {
                                            session.id == session_id
                                                && matches!(
                                                    session.status,
                                                    SessionStatus::Active | SessionStatus::Starting
                                                )
                                        })
                                    {
                                        session.status = SessionStatus::Cancelled;
                                    }
                                    self.persist_external_transition(
                                        tr("セッションをキャンセルしました。"),
                                        tr("tmux セッションはキャンセルされました"),
                                    );
                                } else if let Some(session) =
                                    self.store.sessions.iter_mut().find(|session| {
                                        session.id == session_id
                                            && matches!(
                                                session.status,
                                                SessionStatus::Active | SessionStatus::Starting
                                            )
                                    })
                                {
                                    // The server is gone whether or not the store
                                    // can be written. Rolling back to `Active`
                                    // would ask for a resize, and so a tmux child
                                    // and a store write, every frame; stay `Lost`
                                    // and retry the save.
                                    session.status = SessionStatus::Lost;
                                    if !self.persist() {
                                        self.store_retry_pending = true;
                                        self.notice = Some(tr("tmux が見つからないためセッションを停止扱いにしましたが、保存できませんでした。Operon が自動的に再試行します。").into());
                                    }
                                }
                            } else {
                                self.notice = Some(tf!(
                                    "ターミナルのサイズを変更できませんでした: {error}",
                                    error = error
                                ));
                            }
                        }
                    }
                    self.flush_terminal_resize(session_id);
                }
                BackgroundResult::TerminalClosed { session_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::TerminalClose(session_id));
                    let remove_requested = self.remove_after_close.remove(&session_id);
                    let failed_session = self
                        .store
                        .sessions
                        .iter()
                        .find(|session| session.id == session_id)
                        .is_some_and(|session| session.status == SessionStatus::Failed);
                    let is_gone = match &result {
                        Ok(()) => true,
                        Err(error) => tmux_error_state(error) == TmuxState::Gone,
                    };
                    if is_gone {
                        if remove_requested {
                            self.remove_session_record(session_id);
                        } else if failed_session {
                            self.notice = Some(
                                tr("古いターミナルを停止しました。失敗したセッションを再試行できます。")
                                    .into(),
                            );
                        } else {
                            self.notice =
                                Some(tr("ターミナルを閉じました。記録は残しています。").into());
                        }
                    } else if let Err(error) = result {
                        if remove_requested {
                            self.remove_after_close.insert(session_id);
                        }
                        self.notice = Some(tf!(
                            "ターミナルを閉じられませんでした: {error}",
                            error = error
                        ));
                    }
                }
                BackgroundResult::ListeningPorts(ports) => {
                    self.background_tasks.remove(&BackgroundKey::PortScan);
                    self.ports_scanned_at = Some(Instant::now());
                    // A reading of nothing replaces the previous one: a server
                    // that stopped should stop being named.
                    self.listening_ports = ports;
                }
                BackgroundResult::UpstreamRead { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::Upstream(project_id));
                    // A failure is a reading of "nothing known", not a notice:
                    // this runs on its own without anybody asking for it, and a
                    // banner nobody caused is a banner nobody trusts.
                    self.upstream_cache
                        .insert(project_id, result.unwrap_or_default());
                }
                BackgroundResult::StagedChangeRecorded { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::GitMutation(project_id));
                    match result {
                        Ok(_) => {
                            self.last_commit_failure.remove(&project_id);
                            self.commit_message_input.clear();
                            self.git_staged_files.remove(&project_id);
                            self.git_changes_cache.remove(&project_id);
                            self.upstream_cache.remove(&project_id);
                            self.invalidate_project_diff_views(project_id);
                            self.notice_briefly(tr("コミットしました。"));
                        }
                        Err(error) => {
                            let files = self.staged_selection(project_id);
                            self.last_commit_failure.insert(
                                project_id,
                                (self.commit_message_input.clone(), files, error.clone()),
                            );
                            self.notice =
                                Some(tf!("コミットできませんでした: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::PushFinished { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::GitMutation(project_id));
                    self.upstream_cache.remove(&project_id);
                    match result {
                        Ok(_) => {
                            self.notice_briefly(tr("push しました。"));
                        }
                        Err(error) => {
                            self.notice =
                                Some(tf!("push できませんでした: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::CommitMessageDrafted { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::CommitMessage(project_id));
                    match result {
                        Ok(drafted) => {
                            // Into the field, never past it. The person edits
                            // it and presses the other button.
                            self.commit_message_input = drafted;
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "コミットメッセージを書けませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                }
                BackgroundResult::AiDiffReview { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::AiDiffReview(project_id));
                    match result {
                        Ok(reviewed) => {
                            self.diff_ai_review.insert(project_id, reviewed);
                            self.notice_briefly(tr("AIレビューが完了しました。"));
                        }
                        Err(error) => {
                            self.notice =
                                Some(tf!("AIレビューに失敗しました: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::WorktreeCreated { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::WorktreeMutation(project_id));
                    match result {
                        Ok(created) => {
                            if self.selected_project == Some(project_id) {
                                self.session_path_input = created.destination.display().to_string();
                                self.session_path_project = Some(project_id);
                                self.worktree_branch_input.clear();
                            }
                            self.worktree_cache.remove(&project_id);
                            // Before the notice, because a repository that says
                            // how to set itself up replaces the notice with
                            // something more useful: either the block that asks,
                            // or the sentence saying the setup is running.
                            let destination = created.destination.clone();
                            self.notice = Some(if self.selected_project == Some(project_id) {
                                tf!(
                                    "worktree {p0} を {p1} から作成し、次のセッション用に選択しました。",
                                    p0 = created.branch,
                                    p1 = created.base
                                )
                            } else {
                                tf!(
                                    "worktree {p0} を {p1} から作成しました。使うにはプロジェクトを開き直してください。",
                                    p0 = created.branch,
                                    p1 = created.base
                                )
                            });
                            self.offer_setup(project_id, &destination);
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "worktree を作成できませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                }
                BackgroundResult::WorktreeRemoved {
                    project_id,
                    path,
                    result,
                } => {
                    self.background_tasks
                        .remove(&BackgroundKey::WorktreeMutation(project_id));
                    match result {
                        Ok(()) => {
                            if self.selected_project == Some(project_id)
                                && self.session_path_input == path.display().to_string()
                            {
                                self.session_path_input.clear();
                                self.session_path_project = None;
                            }
                            self.pending_worktree_removal = None;
                            self.worktree_cache.remove(&project_id);
                            self.notice = Some(
                                tr("worktree を削除しました。Git ブランチは残しています。").into(),
                            );
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "worktree を削除できませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                }
                BackgroundResult::WorktreeLanded { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::LandWorktree(project_id));
                    self.pending_land_worktree = None;
                    match result {
                        Ok(land) => {
                            self.worktree_cache.remove(&project_id);
                            self.invalidate_project_file_views(project_id);
                            self.invalidate_project_diff_views(project_id);
                            let msg = if land.cleaned_up_worktree {
                                tf!(
                                    "{branch} を {mainline} にマージし、worktree とブランチを削除しました（{count} コミット）。",
                                    branch = land.worktree_branch,
                                    mainline = land.mainline_branch,
                                    count = land.commits_merged
                                )
                            } else {
                                tf!(
                                    "{branch} を {mainline} にマージしました（{count} コミット）。",
                                    branch = land.worktree_branch,
                                    mainline = land.mainline_branch,
                                    count = land.commits_merged
                                )
                            };
                            self.notice = Some(msg);
                        }
                        Err(error) => {
                            self.notice = Some(tf!("マージに失敗しました: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::PullRequestCreated { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::CreatePullRequest(project_id));
                    self.pending_pr_modal = None;
                    match result {
                        Ok(url) => {
                            self.pull_request_cache.remove(&project_id);
                            self.notice =
                                Some(tf!("プルリクエストを作成しました: {url}", url = url));
                        }
                        Err(error) => {
                            self.notice =
                                Some(tf!("PRの作成に失敗しました: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::IssuesFetched { project_id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::FetchIssues(project_id));
                    match result {
                        Ok(issues) => {
                            self.cached_github_issues.insert(project_id, issues);
                        }
                        Err(error) => {
                            self.notice =
                                Some(tf!("Issue の取得に失敗しました: {error}", error = error));
                        }
                    }
                }
                BackgroundResult::ToolStatus(status) => {
                    self.background_tasks.remove(&BackgroundKey::ToolStatus);
                    self.tools = status;
                    self.notice_briefly(tr("ローカルツールの検出結果を更新しました。"));
                }
                BackgroundResult::OrphanedTmuxSessions(result) => {
                    self.background_tasks.remove(&BackgroundKey::RecoveryScan);
                    match result {
                        Ok(sessions) => {
                            let count = sessions.len();
                            self.orphaned_tmux_sessions = sessions;
                            self.notice = Some(tf!(
                                "未登録の Operon ターミナルセッションを {count} 件見つけました。",
                                count = count
                            ));
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "復旧可能なターミナルを検索できませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                }
                BackgroundResult::OrphanAdopted { orphan, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::RecoveryAdopt(orphan.name.clone()));
                    match result {
                        Ok((project_id, branch)) => {
                            if self
                                .store
                                .sessions
                                .iter()
                                .any(|session| session.tmux_name == orphan.name)
                            {
                                self.notice = Some(
                                    tr("そのターミナルセッションは既に登録されています。").into(),
                                );
                                continue;
                            }
                            if !self
                                .store
                                .projects
                                .iter()
                                .any(|project| project.id == project_id)
                            {
                                self.notice = Some(
                                    tr("復旧が完了する前に、対象のプロジェクトが削除されました。")
                                        .into(),
                                );
                                continue;
                            }
                            let previous_store = self.store.clone();
                            self.store.sessions.push(Session {
                                id: Uuid::new_v4(),
                                project_id,
                                name: plain_terminal_name(&orphan.cwd),
                                goal: tr("実行中の Operon tmux セッションから復旧しました").into(),
                                agent: "existing terminal".into(),
                                tmux_name: orphan.name.clone(),
                                created_at: orphan.created_at,
                                // An adopted terminal was launched outside
                                // Operon, so its start is only known as the
                                // tmux session's own creation time.
                                launched_at: Some(orphan.created_at),
                                status: SessionStatus::Active,
                                worktree_path: Some(orphan.cwd.clone()),
                                branch,
                                agent_command: String::new(),
                                native_session_id: None,
                                native_session_path: None,
                                origin: Some(SessionOrigin::Adopted),
                                depends_on: Vec::new(),
                            });
                            if !self.persist() {
                                self.store = previous_store;
                                continue;
                            }
                            self.orphaned_tmux_sessions
                                .retain(|candidate| candidate.name != orphan.name);
                            self.notice = Some(
                                tr("ターミナルセッションをこのプロジェクトに復旧しました。").into(),
                            );
                        }
                        Err(error) => {
                            self.notice = Some(tf!(
                                "ターミナルを復旧できませんでした: {error}",
                                error = error
                            ));
                        }
                    }
                }
                BackgroundResult::SystemAction { id, action, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::SystemAction(id));
                    match result {
                        Ok(()) => self.notice_briefly(tf!("{action}: 完了", action = action)),
                        Err(error) => {
                            self.notice = Some(tf!(
                                "{action}: 失敗 {error}",
                                action = action,
                                error = error
                            ))
                        }
                    }
                }
                BackgroundResult::Notification { id, result } => {
                    self.background_tasks
                        .remove(&BackgroundKey::Notification(id));
                    // Success is silent; a delivery failure is worth one line
                    // because it usually means notifications are switched off
                    // for scripting in System Settings.
                    if let Err(error) = result {
                        self.notice =
                            Some(tf!("通知を送信できませんでした: {error}", error = error));
                    }
                }
            }
        }
    }

    pub(crate) fn request_git_changes(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::GitChanges(project_id), move || {
            let result = (|| {
                let status = git_output(&path, &["status", "--short", "--branch"])
                    .map_err(|error| error.to_string())?;
                let porcelain = git_output(
                    &path,
                    &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
                )
                .map_err(|error| error.to_string())?;
                Ok(GitChangesSnapshot {
                    status,
                    files: parse_porcelain_changed_files(&porcelain),
                })
            })();
            BackgroundResult::GitChanges { project_id, result }
        });
    }

    /// Accept a Git status snapshot and drop only derived Diff data when it
    /// differs from the last working-tree fingerprint. Keeping the fresh status
    /// means the file tree does not flash empty while the replacement Diff is
    /// fetched in the background.
    pub(crate) fn apply_git_changes(
        &mut self,
        project_id: Uuid,
        result: UiResult<GitChangesSnapshot>,
    ) {
        let changed = matches!(
            (self.git_changes_cache.get(&project_id), &result),
            (Some(Ok(previous)), Ok(next)) if previous.status != next.status
        );
        self.git_changes_cache.insert(project_id, result);
        if changed {
            self.invalidate_project_diff_views(project_id);
        }
    }

    /// Start a status refresh only while the Files view is visible. Status keeps
    /// the tree badges current, but it is not a Diff fingerprint: a second
    /// write to an already-modified path leaves `git status` text unchanged.
    /// Therefore an open Diff is discarded and fetched again on every due poll.
    pub(crate) fn refresh_editor_git_views(&mut self, project: &Project) {
        let due = self
            .last_editor_git_refresh
            .get(&project.id)
            .is_none_or(|last| last.elapsed() >= Duration::from_secs(2));
        if due || !self.git_changes_cache.contains_key(&project.id) {
            self.request_git_changes(project);
            if let Some(file) = self.invalidate_open_editor_diff(project.id) {
                self.request_git_diff(project, Some(file));
            }
            self.last_editor_git_refresh
                .insert(project.id, Instant::now());
        }
    }

    /// Compare the front tab against the bytes on disk, at most every two
    /// seconds, and never on the frame's own thread.
    ///
    /// This app runs agents against the working tree a person is reading, so a
    /// file changing underneath an open editor is the ordinary case rather than
    /// the exceptional one. Before this, the editor went on showing what it had
    /// loaded and said nothing, and the way out was a button somebody had to
    /// already know was there.
    ///
    /// The front tab only. The others are not being read, and the one that
    /// comes to the front is compared two seconds later — the same delay the
    /// front tab already lives with.
    pub(crate) fn watch_active_document(&mut self, project: &Project) {
        let Some(document) = self.active_document.clone() else {
            return;
        };
        if document.project != project.id {
            return;
        }
        let Some(index) = self.document_index(&document) else {
            return;
        };
        // Two states share this guard. A document still loading has nothing to
        // compare against — its `disk` is the empty string it was created with
        // — and a document that cannot be edited at all, because it is binary
        // or past the size ceiling, has an empty `disk` for good. Watching
        // either would compare the file against `""` on every tick, find a
        // difference every time, and say so every two seconds. The cost is that
        // a file which becomes editable again needs the reload button; the
        // alternative is a notice that never stops.
        if self.open_documents[index].read_only.is_some() {
            return;
        }
        // Asked without allocating: on all but one frame in a hundred the
        // answer is "not yet", and a `PathBuf` allocated to be told that is an
        // allocation in the draw path.
        let due = self
            .last_editor_file_watch
            .as_ref()
            .is_none_or(|(watched, last)| {
                *watched != document || last.elapsed() >= Duration::from_secs(2)
            });
        // A save is about to rewrite the file itself and a read is about to
        // replace what the document holds, so a watch issued beside either
        // would be asking about a document that is mid-change. This only
        // covers the instant the watch is issued; `apply_watched_file` checks
        // the same three things again when the answer comes back, which is
        // where the race actually lives.
        if !due || self.editor_file_work_in_flight(&document) {
            return;
        }
        self.last_editor_file_watch = Some((document.clone(), Instant::now()));
        let known = self.open_documents[index].disk.clone();
        let asked = known.clone();
        // The document's own root, not the project's: a worktree document is
        // watched where it lives.
        let root = document.root.clone();
        let path = document.path.clone();
        self.spawn_background(BackgroundKey::FileWatch(document.clone()), move || {
            BackgroundResult::FileWatched {
                document,
                known: asked,
                result: read_file_for_editing_if_changed(&root, &path, &known)
                    .map_err(|error| error.to_string()),
            }
        });
    }

    /// Whether this app is itself in the middle of reading or writing the file,
    /// which is the one case where a difference on disk is not somebody else's
    /// doing. Read at both ends of a watch: once before asking, and once when
    /// the answer arrives, because the interesting window is between them.
    fn editor_file_work_in_flight(&self, document: &DocumentId) -> bool {
        self.background_tasks
            .contains(&BackgroundKey::FileSave(document.clone()))
            || self
                .background_tasks
                .contains(&BackgroundKey::FileOpen(document.clone()))
    }

    /// Take what the watch found. The buffer decides which of the two things
    /// happens, and it is the only thing that decides.
    pub(crate) fn apply_watched_file(
        &mut self,
        document: &DocumentId,
        known: &str,
        result: UiResult<Option<FileOpenOutcome>>,
    ) {
        let Some(index) = self.document_index(document) else {
            // The tab was closed while the read was in flight.
            return;
        };
        // The answer is only about the document that asked. A reload or a save
        // landing first moves `disk`, and this read compared the file against
        // the bytes from before that — applying it would roll the document back
        // to a version nobody asked for and announce it as an agent's write.
        // Another read still in flight is the same fact one moment earlier.
        if known != self.open_documents[index].disk || self.editor_file_work_in_flight(document) {
            return;
        }
        // A read that failed says nothing on screen. The file was deleted, or
        // became unreadable, and the document a person is reading is still the
        // document they were reading; a banner appearing every two seconds for
        // a file that is gone is worse than the save error they will get if
        // they try to write it.
        let Ok(Some(outcome)) = result else {
            return;
        };
        if self.open_documents[index].modified() {
            self.open_documents[index].external_change = true;
            return;
        }
        // A failed save is a sentence addressed to the person, and nobody asked
        // for this read — so it survives the adoption. `apply_opened_file`
        // clears it because a reload is somebody answering the error; a watch
        // is not.
        let unanswered_save_error = self.open_documents[index].save_error.clone();
        self.apply_opened_file(document, Ok(outcome));
        self.open_documents[index].save_error = unanswered_save_error;
        self.notice = Some(tf!(
            "{p0} はエディタの外で変更されたため、読み直しました。",
            p0 = self.open_documents[index].named()
        ));
        // What changed on disk is a change git can show, so the badges and the
        // editor's own diff stop agreeing the moment this lands.
        self.invalidate_project_file_views(document.project);
    }

    /// Remove the one comparison currently presented in the editor, if any.
    /// The caller immediately schedules a replacement, so the UI says loading
    /// rather than showing bytes it can no longer call current.
    pub(crate) fn invalidate_open_editor_diff(&mut self, project_id: Uuid) -> Option<String> {
        let active = self.active_document.as_ref()?;
        if self.editor_view != EditorView::Diff || active.project != project_id {
            return None;
        }
        if !self
            .document_index(active)
            .is_some_and(|index| self.document_may_have_a_diff(index))
        {
            return None;
        }
        let file = active.path.to_string_lossy().into_owned();
        let key = (project_id, Some(file.clone()));
        self.advance_git_diff_generation(key.clone());
        self.git_diff_cache.remove(&key);
        self.diff_file_cache.remove(&key);
        Some(file)
    }

    pub(crate) fn request_git_diff(&mut self, project: &Project, file: Option<String>) {
        let project_id = project.id;
        let path = project.path.clone();
        let key_file = file.clone();
        let generation = self
            .git_diff_generations
            .get(&(project_id, file.clone()))
            .copied()
            .unwrap_or(0);
        self.spawn_background(
            BackgroundKey::GitDiff(project_id, key_file.clone()),
            move || {
                let result = git_working_tree_diff(&path, file.as_deref())
                    .map_err(|error| error.to_string());
                BackgroundResult::GitDiff {
                    project_id,
                    file: key_file,
                    generation,
                    result,
                }
            },
        );
    }

    /// Reissue a Diff after an older in-flight request has completed. A project
    /// can disappear between request and result, in which case there is no view
    /// left that could use the replacement.
    pub(crate) fn request_latest_git_diff(&mut self, project_id: Uuid, file: Option<String>) {
        let project = self
            .store
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .cloned();
        if let Some(project) = project {
            self.request_git_diff(&project, file);
        }
    }

    pub(crate) fn request_git_history(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::GitHistory(project_id), move || {
            let result = git_output(
                &path,
                &["log", "--graph", "--decorate", "--oneline", "-n", "50"],
            )
            .map_err(|error| error.to_string());
            BackgroundResult::GitHistory { project_id, result }
        });
    }

    pub(crate) fn request_worktrees(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::Worktrees(project_id), move || {
            BackgroundResult::Worktrees {
                project_id,
                result: list_worktrees(&path).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn request_pull_requests(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::PullRequests(project_id), move || {
            BackgroundResult::PullRequests {
                project_id,
                result: list_pull_requests(&path).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn request_files(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::Files(project_id), move || {
            BackgroundResult::Files {
                project_id,
                scan: list_files(&path, FILE_TREE_LIMIT),
            }
        });
    }

    pub(crate) fn request_session_files(&mut self, root: &Path) {
        let path = root.to_path_buf();
        self.spawn_background(BackgroundKey::WorktreeFiles(path.clone()), move || {
            BackgroundResult::WorktreeFiles {
                root: path.clone(),
                scan: list_files(&path, FILE_TREE_LIMIT),
            }
        });
    }

    /// Whether the open document at this index is one the Diff view can draw
    /// correctly.
    ///
    /// The Diff pane reads the *project's* working tree and keys its cache by
    /// `(project, relative path)`, so a document rooted at a worktree would be
    /// shown the project's diff for the same relative path — a different file,
    /// under a key that cannot tell the two apart. Refused rather than widened:
    /// widening the key would also re-key change 020's notes sidecar.
    ///
    /// Named, and called by all three places that act on it — the view switcher,
    /// the 差分を見る button, and the refresh that discards the cache. The third
    /// one used to ask nothing, so a worktree document made the project's cache
    /// be thrown away and `git diff` run one frame before the view refused to
    /// draw. Three copies of a question is how the third one goes quietly wrong.
    ///
    /// Takes the index rather than the id: the drawing site already has one, and
    /// a `DocumentId` there would rebuild two `PathBuf`s and walk the open list
    /// again, every frame — which is the cost `document_index` below exists to
    /// have removed.
    pub(crate) fn document_may_have_a_diff(&self, index: usize) -> bool {
        self.open_documents
            .get(index)
            .is_some_and(|document| document.project_rooted)
    }

    /// The document the project's file tree should mark, if any.
    ///
    /// Only the project's own file: the tree shows the project, and a document
    /// opened against a worktree has the same relative path as the project's
    /// copy, so marking that row would point at a file which is not the one on
    /// screen. Named rather than written into the tree's argument list, so the
    /// test and the drawing ask the same function.
    pub(crate) fn tree_highlighted_document(&self) -> Option<&DocumentId> {
        self.active_document.as_ref().filter(|document| {
            self.document_index(document)
                .is_some_and(|index| self.open_documents[index].project_rooted)
        })
    }

    pub(crate) fn document_index(&self, id: &DocumentId) -> Option<usize> {
        // Field by field rather than `document.id() == *id`, which allocated
        // two `PathBuf`s per candidate. This runs several times a frame.
        self.open_documents.iter().position(|document| {
            document.project == id.project && document.root == id.root && document.path == id.path
        })
    }
    /// Open a file in the editor, or bring it to the front where it is already
    /// open. Re-opening deliberately keeps the buffer as typed: clicking a file
    /// in the tree is navigation, and navigation that discarded edits would be
    /// a trap.
    pub(crate) fn open_document(&mut self, project: &Project, file: PathBuf) {
        self.open_document_at(project, project.path.clone(), file);
    }

    /// The same, for a file that is not under the project folder — one followed
    /// from a link in a session running in a worktree. The root travels with
    /// the document because a worktree sits beside the project, not inside it.
    pub(crate) fn open_document_at(&mut self, project: &Project, root: PathBuf, file: PathBuf) {
        // One spelling of a root, or the same file is two documents: two tabs,
        // two background keys, and a buffer that can reach the wrong copy. On
        // macOS `/var/x` and `/private/var/x` are the same directory and not
        // the same `PathBuf`.
        //
        // Both callers already hand a canonical root — the tree passes
        // `Project::path`, which `add_project_path` canonicalises and
        // `a_projects_stored_path_is_canonical` holds, and a click passes what
        // `document_root_for` resolved. So this is idempotent rather than
        // load-bearing, and the `unwrap_or` fires only for a root that has been
        // deleted, where the read and the save that follow fail anyway.
        let root = root.canonicalize().unwrap_or(root);
        // Both sides, or the comparison is between a canonical root and
        // whatever spelling the project happens to be stored under — and every
        // one of the project's own documents would be labelled as if it came
        // from somewhere else. `add_project_path` canonicalises today, so this
        // only bites a store written before it did or a path added by hand;
        // that is exactly the case nobody would think to try.
        let project_root = project
            .path
            .canonicalize()
            .unwrap_or_else(|_| project.path.clone());
        let id = DocumentId {
            project: project.id,
            root: root.clone(),
            path: file.clone(),
        };
        self.project_tab = ProjectTab::Files;
        self.active_document = Some(id.clone());
        if self.document_index(&id).is_some() {
            return;
        }
        let project_rooted = root == project_root;
        let branch = (!project_rooted)
            .then(|| self.worktree_label(&root))
            .flatten();
        self.open_documents.push(OpenDocument {
            project: project.id,
            root,
            path: file.clone(),
            branch,
            project_rooted,
            disk: String::new(),
            buffer: String::new(),
            // Until the read lands, the file cannot be edited — an empty buffer
            // saved over a file that simply had not loaded yet would empty it.
            read_only: Some(tr("読み込んでいます…").into()),
            save_error: None,
            external_change: false,
            image_data: None,
        });
        // Markdown and images open rendered in Preview mode.
        // Everything else opens as text, which is the only way it can be read.
        self.editor_view = if is_markdown(&file) || is_image(&file) {
            EditorView::Preview
        } else {
            EditorView::Edit
        };
        self.request_file_open(&id);
    }

    pub(crate) fn request_file_open(&mut self, document: &DocumentId) {
        let document = document.clone();
        // The document's own root, so a worktree file is read where it lives.
        let root = document.root.clone();
        let path = document.path.clone();
        self.spawn_background(BackgroundKey::FileOpen(document.clone()), move || {
            BackgroundResult::FileOpened {
                document,
                result: read_file_for_editing(&root, &path).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn apply_opened_file(
        &mut self,
        document: &DocumentId,
        result: UiResult<FileOpenOutcome>,
    ) {
        let Some(index) = self.document_index(document) else {
            // The tab was closed while the read was in flight.
            return;
        };
        let document = &mut self.open_documents[index];
        document.save_error = None;
        // Whatever the read says is now what the editor knows, so the warning
        // that the editor's copy was out of date has been answered.
        document.external_change = false;
        match result {
            Ok(FileOpenOutcome::Text(text)) => {
                document.disk = text.clone();
                document.buffer = text;
                document.read_only = None;
                document.image_data = None;
            }
            Ok(FileOpenOutcome::Image {
                bytes,
                width,
                height,
            }) => {
                document.disk = String::new();
                document.buffer = String::new();
                document.image_data = Some(ImageDocumentData {
                    bytes,
                    width,
                    height,
                });
                document.read_only = None;
            }
            Ok(FileOpenOutcome::Unopenable(reason)) => {
                document.disk = String::new();
                document.buffer = String::new();
                document.read_only = Some(reason);
                document.image_data = None;
            }
            Err(error) => {
                document.disk = String::new();
                document.buffer = String::new();
                document.read_only =
                    Some(tf!("ファイルを開けませんでした: {error}", error = error));
                document.image_data = None;
            }
        }
    }

    /// Write the front tab back to disk. A document with nothing to write, or
    /// one already being written, is left alone rather than queued.
    pub(crate) fn save_active_document(&mut self) {
        let Some(document) = self.active_document.clone() else {
            return;
        };
        let Some(index) = self.document_index(&document) else {
            return;
        };
        if !self.open_documents[index].modified() {
            return;
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::FileSave(document.clone()))
        {
            return;
        }
        self.open_documents[index].save_error = None;
        let expected = self.open_documents[index].disk.clone();
        let contents = self.open_documents[index].buffer.clone();
        // The root the document was opened from. Writing to the project's copy
        // of a worktree file is the one failure this change could introduce.
        let root = document.root.clone();
        let path = document.path.clone();
        self.spawn_background(BackgroundKey::FileSave(document.clone()), move || {
            BackgroundResult::FileSaved {
                document,
                result: save_project_file(&root, &path, &expected, &contents)
                    .map(|()| contents)
                    .map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn apply_saved_file(&mut self, document: &DocumentId, result: UiResult<String>) {
        let Some(index) = self.document_index(document) else {
            return;
        };
        match result {
            Ok(written) => {
                // Which copy, when there is more than one. The format string is
                // the message id, so saying more here costs no translation row.
                let named = self.open_documents[index].named();
                let open = &mut self.open_documents[index];
                open.disk = written;
                open.save_error = None;
                // The save only got here by proving the file still held what
                // this editor read, so nothing outside it is unaccounted for.
                open.external_change = false;
                self.notice = Some(tf!("{p0} を保存しました。", p0 = named));
                // What was just written is a change git can show, so the two
                // views of it stop agreeing the moment the save lands.
                self.invalidate_project_file_views(document.project);
            }
            Err(error) => {
                self.open_documents[index].save_error = Some(error);
            }
        }
    }

    /// Drop what this project's git views were showing. Called when something
    /// this app did changed the working tree, so that the status badges, the
    /// diffs, and their parses are all re-read together rather than one of
    /// them lagging a save behind the others.
    pub(crate) fn invalidate_project_file_views(&mut self, project_id: Uuid) {
        self.git_changes_cache.remove(&project_id);
        self.invalidate_project_diff_views(project_id);
    }

    /// Diff text and parsed hunks have one source of truth: the latest Git
    /// output. This smaller invalidation preserves a newly refreshed status.
    pub(crate) fn invalidate_project_diff_views(&mut self, project_id: Uuid) {
        let mut keys = self
            .git_diff_generations
            .keys()
            .filter(|(id, _)| *id == project_id)
            .cloned()
            .collect::<HashSet<_>>();
        keys.extend(
            self.git_diff_cache
                .keys()
                .filter(|(id, _)| *id == project_id)
                .cloned(),
        );
        keys.extend(
            self.diff_file_cache
                .keys()
                .filter(|(id, _)| *id == project_id)
                .cloned(),
        );
        keys.extend(self.background_tasks.iter().filter_map(|key| match key {
            BackgroundKey::GitDiff(id, file) if *id == project_id => Some((*id, file.clone())),
            _ => None,
        }));
        for key in keys {
            self.advance_git_diff_generation(key);
        }
        self.git_diff_cache.retain(|(id, _), _| *id != project_id);
        self.diff_file_cache.retain(|(id, _), _| *id != project_id);
    }

    fn advance_git_diff_generation(&mut self, key: (Uuid, Option<String>)) {
        let generation = self.git_diff_generations.entry(key).or_insert(0);
        *generation = generation.saturating_add(1);
    }
    /// Close a tab, and hand the front to its neighbour the way every editor
    /// does — the tab to the right, or the one to the left where it was last.
    pub(crate) fn close_document(&mut self, document: &DocumentId) {
        let Some(index) = self.document_index(document) else {
            return;
        };
        self.open_documents.remove(index);
        if self.active_document.as_ref() != Some(document) {
            return;
        }
        let neighbour = self.open_documents.get(index).or_else(|| {
            index
                .checked_sub(1)
                .and_then(|left| self.open_documents.get(left))
        });
        self.active_document = neighbour.map(OpenDocument::id);
    }

    /// The branch a worktree is on, for the badge that says which copy of a
    /// file a tab holds. Falls back to the folder's own name, which this
    /// application derives from the branch when it makes one.
    ///
    /// Called once, when the document is opened, and the answer is kept on it.
    /// The first cut called this from the tab row, which draws every frame, and
    /// compared canonically — so every session that was not the match reached
    /// `canonicalize`, and fifteen sessions with two worktree tabs open was
    /// about ten thousand `stat` calls a second from `fn update`. A path stored
    /// by this application is already canonical (`add_project_path` and the
    /// orphan scan both canonicalise on the way in, and a worktree path is
    /// derived from a project's), so `==` is the whole comparison.
    pub(crate) fn worktree_label(&self, root: &Path) -> Option<String> {
        self.store
            .sessions
            .iter()
            .find(|session| session.worktree_path.as_deref() == Some(root))
            .and_then(|session| session.branch.clone())
            .or_else(|| {
                root.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
    }

    /// Read the file back off disk, discarding what is in the buffer. This is
    /// the way out of a save the working tree refused: an agent wrote the file
    /// while it was open, and the person has decided the agent's copy wins.
    pub(crate) fn reload_document(&mut self, document: &DocumentId) {
        if let Some(index) = self.document_index(document) {
            self.open_documents[index].save_error = None;
        }
        self.invalidate_project_file_views(document.project);
        self.request_file_open(document);
    }

    /// Make sure `diff_file_cache` holds the parse of this diff, fetching the
    /// text behind it where it has not been asked for yet.
    ///
    /// Returns nothing on purpose. Handing back a borrow would tie the parsed
    /// diff to a `&mut self` for as long as it is being drawn, and drawing it
    /// also needs the fold state beside it; the caller reads the two fields
    /// directly instead, which the borrow checker can see are different fields.
    pub(crate) fn ensure_diff_parsed(&mut self, project: &Project, file: Option<String>) {
        let key = (project.id, file.clone());
        if !self.git_diff_cache.contains_key(&key) {
            self.request_git_diff(project, file);
            return;
        }
        if self.diff_file_cache.contains_key(&key) {
            return;
        }
        let parsed = match self.git_diff_cache.get(&key) {
            Some(Ok(diff)) => parse_unified_diff(diff),
            _ => Vec::new(),
        };
        self.diff_file_cache.insert(key, parsed);
    }

    /// The working tree's status for each changed path, for the badges in the
    /// tree. Absent until the Git view has been opened once, which is the point
    /// at which this app has a reason to have run `git status`.
    pub(crate) fn git_status_badges(&self, project_id: Uuid) -> HashMap<String, String> {
        match self.git_changes_cache.get(&project_id) {
            Some(Ok(snapshot)) => snapshot
                .files
                .iter()
                .map(|file| (file.path.clone(), file.status.clone()))
                .collect(),
            _ => HashMap::new(),
        }
    }

    pub(crate) fn request_skills(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::Skills(project_id), move || {
            BackgroundResult::Skills {
                project_id,
                scan: find_named_files(&path, "SKILL.md", 80),
            }
        });
    }

    pub(crate) fn request_rules(&mut self, project: &Project) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::Rules(project_id), move || {
            let scan = find_named_files_any(
                &path,
                &[
                    "AGENTS.md",
                    "CLAUDE.md",
                    "GEMINI.md",
                    "CODEX.md",
                    "INSTRUCTIONS.md",
                ],
                200,
            );
            BackgroundResult::Rules { project_id, scan }
        });
    }

    pub(crate) fn request_session_poll(&mut self) {
        if !self.tools.tmux {
            return;
        }
        let active = self
            .store
            .sessions
            .iter()
            .filter(|session| {
                session.status == SessionStatus::Active
                    || session.status == SessionStatus::Unknown
                    || (session.status == SessionStatus::Starting
                        && !self
                            .background_tasks
                            .contains(&BackgroundKey::SessionStart(session.id))
                        && !self
                            .background_tasks
                            .contains(&BackgroundKey::EmptySessionStart(session.id)))
            })
            .map(|session| (session.id, session.tmux_name.clone(), session.agent.clone()))
            .collect::<Vec<_>>();
        if active.is_empty() {
            return;
        }
        let selected = self
            .selected_session
            .and_then(|selected| active.iter().find(|(id, ..)| *id == selected).cloned());
        self.spawn_background(BackgroundKey::SessionPoll, move || {
            let mut states = Vec::with_capacity(active.len());
            let mut activities = Vec::new();
            let mut previews = Vec::new();
            for batch in active.chunks(16) {
                thread::scope(|scope| {
                    let workers = batch
                        .iter()
                        .map(|(id, name, agent)| {
                            // A plain terminal has no agent turn to finish and
                            // no conversation to quote, so its screen is never
                            // read. Every managed agent's is, whether or not
                            // notifications are on: the session list shows both
                            // readings, and a status nobody can see is a status
                            // that cannot be trusted.
                            let read_screen = agent != "terminal";
                            scope.spawn(move || {
                                let observation = tmux_observation(name.as_str());
                                let screen = (read_screen && observation.state == TmuxState::Alive)
                                    .then(|| tmux_capture_screen(name.as_str()).ok())
                                    .flatten();
                                let activity = screen.as_deref().map(detect_agent_activity);
                                let preview = screen
                                    .as_deref()
                                    .map(|screen| read_conversation_preview(agent, screen));
                                (*id, observation, activity, preview)
                            })
                        })
                        .collect::<Vec<_>>();
                    for (id, observation, activity, preview) in
                        workers.into_iter().filter_map(|worker| worker.join().ok())
                    {
                        states.push((id, observation));
                        if let Some(activity) = activity {
                            activities.push((id, activity));
                        }
                        if let Some(preview) = preview {
                            previews.push((id, preview));
                        }
                    }
                });
            }
            let output = selected.map(|(id, name, _)| {
                (
                    id,
                    tmux_capture_with_cursor(&name).map_err(|error| error.to_string()),
                )
            });
            BackgroundResult::SessionPoll {
                states,
                activities,
                previews,
                output,
            }
        });
    }

    pub(crate) fn apply_session_poll(
        &mut self,
        states: Vec<(Uuid, TmuxObservation)>,
        activities: Vec<(Uuid, AgentActivity)>,
        previews: Vec<(Uuid, ConversationPreview)>,
        output: Option<(Uuid, UiResult<TerminalCapture>)>,
    ) {
        self.apply_session_previews(previews);
        let previous_store = self.store.clone();
        let mut changed = false;
        let mut completed = Vec::new();
        let mut failed = Vec::new();
        let mut completed_cancellations = Vec::new();
        let mut finished_setups: Vec<(Uuid, Option<i32>)> = Vec::new();
        for (id, observation) in states {
            let cancellation_pending = self.cancellation_pending(id);
            if let Some(session) = self.store.sessions.iter_mut().find(|session| {
                session.id == id
                    && matches!(
                        session.status,
                        SessionStatus::Starting | SessionStatus::Active | SessionStatus::Unknown
                    )
            }) {
                let next = if cancellation_pending
                    && matches!(observation.state, TmuxState::Dead | TmuxState::Gone)
                {
                    Some(SessionStatus::Cancelled)
                } else {
                    session_status_after_observation(&session.status, observation)
                };
                if let Some(next) = next {
                    if next == SessionStatus::Exited {
                        completed.push(session.name.clone());
                    } else if next == SessionStatus::Failed {
                        failed.push(session.name.clone());
                    } else if next == SessionStatus::Cancelled {
                        completed_cancellations.push(id);
                    }
                    // A setup run is a session like any other, and it ends like
                    // one. The only difference is which sentence says so, and
                    // the status alone cannot say it: `Failed` folds every
                    // non-zero code together, and the code is the thing worth
                    // reading.
                    if self.setup_sessions.contains(&id)
                        && matches!(next, SessionStatus::Exited | SessionStatus::Failed)
                    {
                        finished_setups.push((id, observation.exit_status));
                    }
                    session.status = next;
                    changed = true;
                }
            }
        }
        if !completed_cancellations.is_empty() {
            self.store
                .pending_cancellations
                .retain(|pending| !completed_cancellations.contains(pending));
        }
        if let Some((id, Ok(capture))) = output {
            self.set_session_output(id, capture.output, capture.cursor);
        }
        if changed && !self.persist() {
            self.store = previous_store;
            return;
        }
        if changed {
            match save_cancellation_intents(&self.data_file, &self.store.pending_cancellations) {
                Ok(WriteOutcome::Durable) => {}
                Ok(WriteOutcome::CommittedButNotSynced(error)) => {
                    self.store_retry_pending = true;
                    let warning = tf!("キャンセル復旧用メタデータは書き込まれましたが、ディスクへの永続化を確認できませんでした。Operon が自動的に再試行します。エラー: {error}", error = error);
                    self.durability_warning = Some(warning);
                }
                Err(error) => {
                    self.store_retry_pending = true;
                    self.notice = Some(tf!("セッション状態は保存しましたが、キャンセル復旧用メタデータが保留中です。Operon が自動的に再試行します。エラー: {error}", error = error));
                }
            }
        }
        if changed && self.store.notifications_enabled {
            // These stay on the quiet path: a notification is ambient, and
            // reporting each one in the status line would bury whatever the
            // user was actually reading there.
            for name in completed {
                self.send_notification(tf!("セッションが完了しました: {name}", name = name));
            }
            for name in failed {
                self.send_notification(tf!("セッションが失敗しました: {name}", name = name));
            }
        }
        for (id, status) in finished_setups {
            self.setup_sessions.remove(&id);
            self.notice = Some(match status {
                Some(0) => tr("セットアップが完了しました。").into(),
                Some(code) => tf!(
                    "セットアップが終了コード {p0} で終わりました。セッションを開いて確認してください。",
                    p0 = code
                ),
                None => tr(
                    "セットアップがどう終わったか読み取れませんでした。セッションを開いて確認してください。",
                )
                .into(),
            });
        }
        self.apply_session_activities(activities);
    }

    /// Keep the quote each session row draws in step with what its terminal is
    /// showing.
    ///
    /// A screen with nothing quotable on it leaves the previous quote alone.
    /// Mid-turn output routinely fills a pane edge to edge, and a row that
    /// blanked every time an agent ran `ls` would be flickering rather than
    /// reporting. The cost is bounded and visible: after a conversation is
    /// cleared, the row keeps quoting the exchange before it until the next one
    /// arrives. A session that stops being polled loses its quote outright.
    pub(crate) fn apply_session_previews(&mut self, previews: Vec<(Uuid, ConversationPreview)>) {
        self.session_previews
            .retain(|id, _| previews.iter().any(|(polled, _)| polled == id));
        for (id, preview) in previews {
            if !preview.is_empty() {
                self.session_previews.insert(id, preview);
            }
        }
    }

    /// Turn each polled run state into the notification its change deserves.
    ///
    /// Trackers live only in memory: after a restart the first reading is a
    /// baseline, so restarting Operon beside an idle session cannot claim
    /// that session just finished something.
    pub(crate) fn apply_session_activities(&mut self, activities: Vec<(Uuid, AgentActivity)>) {
        // Trackers are folded in either way — they are what the session list
        // reads — but only an enabled notification setting may leave the app.
        let notifications_enabled = self.store.notifications_enabled;
        for body in self.session_activity_notifications(activities) {
            if notifications_enabled {
                self.send_notification(body);
            }
        }
    }

    pub(crate) fn session_activity_notifications(
        &mut self,
        activities: Vec<(Uuid, AgentActivity)>,
    ) -> Vec<String> {
        self.session_activity
            .retain(|id, _| activities.iter().any(|(polled, _)| polled == id));
        // A reading nobody has confirmed for half an hour stops speaking for
        // the session. Dropping it here, rather than letting it age forward,
        // is what lets the screen take the question back — and dropping the
        // tracker with it makes the next screen reading a baseline, so the
        // handover cannot announce a turn that ended while nobody was looking.
        let now = Instant::now();
        let stale: Vec<Uuid> = self
            .hook_status
            .iter()
            .filter(|(_, status)| !hook_status_is_fresh(status, now))
            .map(|(id, _)| *id)
            .collect();
        for id in stale {
            self.hook_status.remove(&id);
            self.session_activity.remove(&id);
        }
        let mut bodies = Vec::new();
        for (id, activity) in activities {
            // While the CLI is reporting for itself, the screen is a second
            // opinion nobody asked for: it would settle two polls late and
            // notify a second time for the same turn.
            if self.hook_status.contains_key(&id) {
                continue;
            }
            let notice = {
                let tracker = self.session_activity.entry(id).or_default();
                observe_agent_activity(tracker, activity)
            };
            let Some(notice) = notice else {
                continue;
            };
            let Some(session) = self.store.sessions.iter().find(|session| session.id == id) else {
                continue;
            };
            let body = activity_notification_body(notice, &session.name, &session.agent);
            self.note_session_activity_change(id);
            bodies.push(body);
        }
        bodies
    }

    // ── the CLIs' own status hooks ──────────────────────────────────────────

    /// Binds the receiving socket and registers the managed hooks, once per
    /// run. Called from the first frame rather than from construction: binding
    /// needs the egui context to wake the window when an event lands, and a
    /// failure here has to reach the person as an ordinary notice.
    pub(crate) fn ensure_hooks_started(&mut self, ctx: &egui::Context) {
        if self.hooks_started {
            return;
        }
        self.hooks_started = true;
        if self.hooks_enabled {
            self.bind_hook_listener(ctx);
        }
        self.request_hook_apply(self.hooks_enabled);
    }

    pub(crate) fn bind_hook_listener(&mut self, ctx: &egui::Context) {
        if self.hook_listener.is_some() {
            return;
        }
        let socket = hook_socket_path(&self.data_file);
        match HookListener::bind(&socket, Some(ctx.clone())) {
            Ok((listener, events)) => {
                self.hook_listener = Some(listener);
                self.hook_events = Some(events);
            }
            Err(error) => {
                self.notice = Some(tf!(
                    "フック受信ソケットを開けませんでした: {error}",
                    error = error
                ));
            }
        }
    }

    /// Registers or removes the managed hooks. Off the UI thread: it reads and
    /// rewrites three settings files that belong to other products.
    pub(crate) fn request_hook_apply(&mut self, enabled: bool) {
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
            return;
        };
        let data_file = self.data_file.clone();
        let socket = hook_socket_path(&self.data_file);
        let tools = self.tools.clone();
        // Every registered account's directory gets the hooks too, so an agent
        // running under a second login still reports what it is doing.
        let accounts = self.accounts.clone();
        self.spawn_background(BackgroundKey::HookApply, move || {
            let (installs, _) = apply_hooks(&data_file, &home, &socket, &tools, enabled, &accounts);
            BackgroundResult::HooksApplied { installs }
        });
    }

    /// The identity the next launch of this session carries into its pane.
    /// `None` while the mechanism is off, which is what stops the scripts from
    /// posting at all: they exit as soon as the session variable is missing.
    pub(crate) fn mint_hook_environment(&mut self, session_id: Uuid) -> Option<HookEnvironment> {
        if !self.hooks_enabled {
            return None;
        }
        let session = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)?;
        let launch_token = Uuid::new_v4().simple().to_string();
        self.hook_launch_tokens
            .insert(session_id, launch_token.clone());
        // A new launch is a new turn's worth of evidence; whatever the previous
        // process last said about this session is no longer about this process.
        self.hook_status.remove(&session_id);
        Some(HookEnvironment {
            session: session.tmux_name.clone(),
            launch_token,
            endpoint: hook_endpoint_path(&self.data_file),
            socket: hook_socket_path(&self.data_file),
        })
    }

    pub(crate) fn drain_hook_events(&mut self) {
        let events: Vec<HookEvent> = match &self.hook_events {
            Some(receiver) => receiver.try_iter().collect(),
            None => return,
        };
        for event in events {
            self.apply_hook_event(event);
        }
    }

    pub(crate) fn apply_hook_event(&mut self, event: HookEvent) {
        // A usage reading is about the account, not about a pane: it is not an
        // agent state and never reaches the state machine. It also arrives from
        // a script whose only job is to forward what the CLI already said, so
        // it is not gated on a launch token — a pane that outlived this process
        // still knows the true numbers.
        if event.event == STATUSLINE_EVENT {
            if let Some(limits) = parse_rate_limits(&event.payload) {
                self.rate_limits = Some((limits, Instant::now()));
            }
            return;
        }
        let Some(session_id) = self
            .store
            .sessions
            .iter()
            .find(|session| session.tmux_name == event.session)
            .map(|session| session.id)
        else {
            return;
        };
        // A token this process minted and does not recognise belongs to a
        // process that has since been replaced. No token recorded at all means
        // the pane outlived the Operon that launched it, and its word is the
        // best evidence there is.
        if self
            .hook_launch_tokens
            .get(&session_id)
            .is_some_and(|token| *token != event.launch_token)
        {
            return;
        }
        let Some(reading) = normalise_hook_event(&event) else {
            return;
        };
        self.adopt_native_identity(session_id, &reading);

        // Record user prompt turns for session timeline navigation
        if event.event == "UserPromptSubmit"
            || (event.source == CliProvider::Gemini && event.event == "PreInvocation")
        {
            let prompt = event
                .payload
                .get("prompt")
                .and_then(|p| p.as_str())
                .or_else(|| event.payload.get("user_prompt").and_then(|p| p.as_str()))
                .or_else(|| event.payload.get("query").and_then(|p| p.as_str()));
            if let Some(prompt) = prompt {
                self.record_prompt_turn(session_id, prompt.to_string());
            }
        } else if reading.activity == AgentActivity::Idle {
            self.mark_prompt_turn_completed(session_id);
        }
        let (status, notice) =
            apply_hook_reading(self.hook_status.get(&session_id), reading, Instant::now());
        self.hook_status.insert(session_id, status);
        let Some(notice) = notice else {
            return;
        };
        // The mark is not the notification: it is what is left when the
        // notification was missed, so it does not ask whether notifications
        // are on.
        self.note_session_activity_change(session_id);
        if !self.store.notifications_enabled {
            return;
        }
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        let body = activity_notification_body(notice, &session.name, &session.agent);
        self.send_notification(body);
    }

    /// The conversation the CLI says this terminal is on.
    ///
    /// Worth writing down the moment it arrives: the alternative this replaces
    /// matches a transcript's timestamp against the launch and refuses a tie,
    /// and Claude rotates the id on `/clear` without the file names saying so.
    pub(crate) fn adopt_native_identity(&mut self, session_id: Uuid, reading: &HookReading) {
        let Some(native_id) = reading.session_id.clone() else {
            return;
        };
        if !is_safe_cli_session_id(&native_id) {
            return;
        }
        let path = reading.transcript_path.as_ref().map(PathBuf::from);
        let previous_store = self.store.clone();
        let Some(session) = self
            .store
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if session.native_session_id.as_deref() == Some(native_id.as_str())
            && (path.is_none() || session.native_session_path == path)
        {
            return;
        }
        session.native_session_id = Some(native_id);
        if path.is_some() {
            session.native_session_path = path;
        }
        if !self.persist() {
            self.store = previous_store;
        }
    }

    /// Records a user prompt turn and its starting line in terminal output for timeline navigation.
    pub(crate) fn record_prompt_turn(&mut self, session_id: Uuid, prompt: String) {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return;
        }
        let line = self
            .terminal_layouts
            .get(&session_id)
            .map(|lines| lines.len())
            .unwrap_or(0);
        let turns = self.session_prompt_turns.entry(session_id).or_default();
        if let Some(last) = turns.last_mut() {
            if last.prompt == prompt && last.line == line {
                return;
            }
            last.completed = true;
        }
        let turn_number = turns.last().map(|t| t.turn + 1).unwrap_or(1);
        if turns.len() >= 500 {
            turns.remove(0);
        }
        turns.push(PromptTurn {
            turn: turn_number,
            prompt: prompt.chars().take(300).collect(),
            timestamp: Instant::now(),
            line,
            completed: false,
        });
        self.capture_checkpoint(session_id, prompt);
    }

    /// Marks the latest prompt turn for a session as completed when the agent finishes working.
    pub(crate) fn mark_prompt_turn_completed(&mut self, session_id: Uuid) {
        if let Some(turns) = self.session_prompt_turns.get_mut(&session_id) {
            if let Some(last) = turns.last_mut() {
                last.completed = true;
            }
        }
    }

    /// Seeds an initial prompt turn from the session's recorded goal if no turns have been captured yet.
    pub(crate) fn ensure_initial_prompt_turn(&mut self, session_id: Uuid) {
        if self
            .session_prompt_turns
            .get(&session_id)
            .is_some_and(|turns| !turns.is_empty())
        {
            return;
        }
        let Some((goal, is_active)) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .map(|session| {
                (
                    session.goal.clone(),
                    session.status == SessionStatus::Active,
                )
            })
        else {
            return;
        };
        let goal = goal.trim();
        if !goal.is_empty() {
            self.record_prompt_turn(session_id, goal.to_string());
            if !is_active {
                self.mark_prompt_turn_completed(session_id);
            }
        }
    }

    /// What a session's hooks last said, while that is still recent enough to
    /// outrank the screen. A stale reading is dropped here rather than aged
    /// forward: it is the poll that then re-baselines from what is visible.
    pub(crate) fn fresh_hook_status(&self, session_id: Uuid) -> Option<&HookStatus> {
        self.hook_status
            .get(&session_id)
            .filter(|status| hook_status_is_fresh(status, Instant::now()))
    }

    /// The activity a row draws: what the hooks reported while that is fresh,
    /// otherwise what the screen settled on.
    pub(crate) fn session_activity_for(&self, session_id: Uuid) -> Option<AgentActivity> {
        self.fresh_hook_status(session_id)
            .map(|status| status.activity)
            .or_else(|| {
                self.session_activity
                    .get(&session_id)
                    .and_then(|tracker| tracker.settled)
            })
    }

    pub(crate) fn send_notification(&mut self, body: String) {
        let id = Uuid::new_v4();
        self.spawn_background(BackgroundKey::Notification(id), move || {
            BackgroundResult::Notification {
                id,
                result: send_macos_notification(APP_NAME, &body).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn start_ready_queued_sessions(&mut self) {
        if !self.tools.tmux {
            return;
        }
        let ready = ready_queued_session_ids(&self.store.sessions);
        for session_id in ready {
            let launchable = self
                .store
                .sessions
                .iter()
                .find(|session| session.id == session_id)
                .is_some_and(|session| {
                    self.tools.agent_available(&session.agent)
                        && !self.worktree_mutation_running(session.project_id)
                });
            if launchable {
                self.start_session(session_id);
            }
        }
    }

    pub(crate) fn cancel_queued_session(&mut self, session_id: Uuid) {
        let previous_store = self.store.clone();
        let Some(session) = self
            .store
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if session.status != SessionStatus::Queued {
            self.notice = Some(tr("キャンセルできるのは待機中のセッションだけです。").into());
            return;
        }
        session.status = SessionStatus::Cancelled;
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        self.notice = Some(tr("キュー内のセッションをキャンセルしました。").into());
    }

    pub(crate) fn request_cli_sessions(&mut self, project: &Project, announce: bool) {
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::CliSessions(project_id), move || {
            BackgroundResult::CliSessions {
                project_id,
                scan: scan_native_cli_sessions(&path, 80),
                announce,
            }
        });
    }

    pub(crate) fn request_tool_status(&mut self) {
        self.spawn_background(BackgroundKey::ToolStatus, || {
            BackgroundResult::ToolStatus(ToolStatus::detect())
        });
        self.notice = Some(tr("ローカルツールを確認しています…").into());
    }

    pub(crate) fn add_project_path(&mut self, path: PathBuf) -> bool {
        if !path.is_dir() {
            self.notice = Some(tr("既存のフォルダを選んでください。").into());
            return false;
        }
        let path = path.canonicalize().unwrap_or(path);
        if let Some(existing) = self
            .store
            .projects
            .iter()
            .find(|project| project.path == path)
        {
            let existing_id = existing.id;
            self.select_project(Some(existing_id));
            self.page = Page::Projects;
            self.notice = Some(tr("そのプロジェクトは追加済みのため、開きました。").into());
            return false;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Project")
            .to_owned();
        let project = Project {
            id: Uuid::new_v4(),
            path,
            name,
            added_at: now(),
        };
        let previous_store = self.store.clone();
        let project_id = project.id;
        self.store.projects.push(project);
        if !self.persist() {
            self.store = previous_store;
            return false;
        }
        self.select_project(Some(project_id));
        self.project_path_input.clear();
        self.page = Page::Projects;
        self.project_tab = ProjectTab::Overview;
        self.notice = Some(tr("プロジェクトを追加しました。").into());
        true
    }

    pub(crate) fn add_project(&mut self) {
        let path = PathBuf::from(self.project_path_input.trim());
        self.add_project_path(path);
    }

    pub(crate) fn choose_project_folder(&mut self) {
        #[cfg(not(test))]
        {
            let selection = rfd::FileDialog::new()
                .set_title(tr("プロジェクトフォルダを選択"))
                .pick_folder();
            self.apply_project_folder_choice(selection);
        }
        #[cfg(test)]
        {
            self.notice = Some("choose_project_folder_invoked".into());
        }
    }

    pub(crate) fn apply_project_folder_choice(&mut self, selection: Option<PathBuf>) {
        if let Some(path) = selection {
            self.add_project_path(path);
        } else {
            self.notice = Some(tr("プロジェクトの選択をキャンセルしました。").into());
        }
    }

    pub(crate) fn choose_workspace_folder(&mut self) {
        let selection = rfd::FileDialog::new()
            .set_title(tr("スキャンするワークスペースを選択"))
            .pick_folder();
        self.apply_workspace_folder_choice(selection);
    }

    pub(crate) fn apply_workspace_folder_choice(&mut self, selection: Option<PathBuf>) {
        if let Some(path) = selection {
            self.workspace_scan_input = path.display().to_string();
            self.import_workspace();
        } else {
            self.notice = Some(tr("ワークスペースの選択をキャンセルしました。").into());
        }
    }

    pub(crate) fn add_dropped_folders(&mut self, dropped: Vec<egui::DroppedFile>) {
        let (folders, rejected_count) = dropped_project_folders(&dropped);
        if folders.is_empty() {
            self.notice = Some(tr("Finder からフォルダをドロップすると追加できます。").into());
            return;
        }
        let folder_count = folders.len();
        let mut added = 0;
        let mut save_error = None;
        for path in folders {
            if self.add_project_path(path) {
                added += 1;
            } else if self
                .notice
                .as_deref()
                .is_some_and(|notice| notice.starts_with("データを保存できませんでした"))
            {
                save_error = self.notice.clone();
            }
        }
        let duplicates_or_failed = folder_count.saturating_sub(added);
        if folder_count > 1 || rejected_count > 0 || duplicates_or_failed > 0 {
            self.select_project(None);
            self.page = Page::Projects;
            self.notice = Some(if let Some(error) = save_error {
                tf!(
                    "{added} 件のプロジェクトフォルダを追加しました。{error}",
                    added = added,
                    error = error
                )
            } else {
                tf!(
                    "{added} 件のプロジェクトフォルダを追加しました。{duplicates}{rejected}",
                    added = added,
                    duplicates = if duplicates_or_failed > 0 {
                        tf!(" {duplicates_or_failed} 件のフォルダは登録済みか、保存できませんでした。", duplicates_or_failed = duplicates_or_failed)
                    } else {
                        String::new()
                    },
                    rejected = if rejected_count > 0 {
                        tf!(
                            " フォルダではない {rejected_count} 件を無視しました。",
                            rejected_count = rejected_count
                        )
                    } else {
                        String::new()
                    }
                )
            });
        }
    }

    pub(crate) fn import_workspace(&mut self) {
        let workspace = PathBuf::from(self.workspace_scan_input.trim());
        if !workspace.is_dir() {
            self.notice = Some(tr("ワークスペースフォルダを選んでください。").into());
            return;
        }
        let workspace = workspace.canonicalize().unwrap_or(workspace);
        if self
            .background_tasks
            .contains(&BackgroundKey::WorkspaceScan)
        {
            self.notice = Some(tr("ワークスペースのスキャンは実行中です。").into());
            return;
        }
        let scan_root = workspace.clone();
        self.spawn_background(BackgroundKey::WorkspaceScan, move || {
            let (repositories, truncated) = scan_git_repositories(&scan_root, 3);
            BackgroundResult::WorkspaceScan {
                workspace: scan_root.clone(),
                repositories,
                truncated,
            }
        });
        self.notice = Some(tr("ワークスペースをスキャンしています…").into());
    }

    pub(crate) fn finish_workspace_import(
        &mut self,
        workspace: PathBuf,
        repositories: Vec<PathBuf>,
        truncated: bool,
    ) {
        let previous_store = self.store.clone();
        let was_empty = previous_store.projects.is_empty();
        let mut imported = 0;
        for path in repositories {
            let path = path.canonicalize().unwrap_or(path);
            if self
                .store
                .projects
                .iter()
                .any(|project| project.path == path)
            {
                continue;
            }
            let name = path
                .strip_prefix(&workspace)
                .ok()
                .filter(|relative| !relative.as_os_str().is_empty())
                .map(|relative| relative.display().to_string())
                .or_else(|| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(ToOwned::to_owned)
                })
                .unwrap_or_else(|| "Project".into());
            self.store.projects.push(Project {
                id: Uuid::new_v4(),
                path,
                name,
                added_at: now(),
            });
            imported += 1;
        }
        if imported > 0 && !self.persist() {
            self.store = previous_store;
            return;
        }
        self.workspace_scan_input.clear();
        if was_empty {
            self.page = Page::Projects;
            self.select_project(None);
        }
        self.notice = Some(if imported == 0 {
            if truncated {
                tr("ワークスペースのスキャン上限に達しましたが、新しい Git リポジトリは見つかりませんでした。対象フォルダを絞って再試行してください。")
                    .into()
            } else {
                tr("3 階層以内に新しい Git リポジトリは見つかりませんでした。").into()
            }
        } else if truncated {
            tf!("{imported} 件の Git プロジェクトを取り込みました。上限付きスキャンの限度に達したため、一部のフォルダは省略されている可能性があります。", imported = imported)
        } else {
            tf!(
                "ワークスペースから {imported} 件の Git プロジェクトを取り込みました。",
                imported = imported
            )
        });
    }

    pub(crate) fn remove_project(&mut self, project_id: Uuid) {
        if self
            .background_tasks
            .contains(&BackgroundKey::WorktreeMutation(project_id))
        {
            self.notice = Some(tr("worktree 操作の完了を待ってください。").into());
            return;
        }
        let active = self.store.sessions.iter().any(|session| {
            session.project_id == project_id
                && matches!(
                    session.status,
                    SessionStatus::Starting
                        | SessionStatus::Active
                        | SessionStatus::Queued
                        | SessionStatus::Unknown
                )
        });
        if active {
            self.notice = Some(tr("実行中のセッションを停止または完了してください。").into());
            return;
        }
        let previous_store = self.store.clone();
        self.store
            .projects
            .retain(|project| project.id != project_id);
        self.store
            .sessions
            .retain(|session| session.project_id != project_id);
        self.store
            .native_session_references
            .retain(|reference| reference.project_id != project_id);
        self.store
            .cli_handoffs
            .retain(|handoff| handoff.project_id != project_id);
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        self.select_project(None);
        if self
            .selected_session
            .is_some_and(|id| self.store.sessions.iter().all(|session| session.id != id))
        {
            self.selected_session = None;
            self.resolved_paths.clear();
            self.resolved_paths_generation += 1;
            self.unresolved_paths.clear();
            self.resolved_paths_session = None;
            self.terminal_path_menu = None;
        }
        self.pending_project_removal = None;
        self.notice = Some(tr("プロジェクトの記録を削除しました。ファイルは残っています。").into());
    }

    pub(crate) fn launch_session(&mut self) {
        let Some(project) = self.selected_project().cloned() else {
            self.notice = Some(tr("先にプロジェクトを選んでください。").into());
            return;
        };
        if self.worktree_mutation_running(project.id) {
            self.notice = Some(tr("worktree 操作の完了を待ってください。").into());
            return;
        }
        if !self.tools.tmux {
            self.notice = Some(tr("tmux が必要です。「設定」を開いてください。").into());
            return;
        }
        if !self.tools.agent_available(&self.selected_agent) {
            self.notice = Some(tf!("{selected_agent} が PATH 上に見つかりません。インストールしてから、「設定」で「ツールを再確認」を押してください。", selected_agent = self.selected_agent));
            return;
        }
        let account = self
            .agent_account_input
            .filter(|_| agent_supports_accounts(&self.selected_agent));
        if let Err(reason) = account_ready_to_launch(&self.accounts, account) {
            self.notice = Some(reason);
            return;
        }
        let user_goal = self.goal_input.trim().to_owned();
        let agent_command = match build_agent_launch_command(
            &self.selected_agent,
            &self.custom_command,
            &self.agent_model_input,
            &self.agent_mode_input,
            &self.agent_effort_input,
            &self.agent_flag_inputs,
        ) {
            Ok(command) => command,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        let worktree_path = match resolve_selected_session_path(
            project.id,
            &project.path,
            self.session_path_project,
            &self.session_path_input,
        ) {
            Ok(path) => path,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        let id = Uuid::new_v4();
        let (agent_command, native_session_id) =
            command_with_managed_native_session_id(&self.selected_agent, agent_command, id);
        let goal = goal_with_native_tracking_token(&self.selected_agent, user_goal.clone(), id);
        let tmux_name = managed_tmux_name(id);
        let raw_name = if self.session_name_input.trim().is_empty() {
            // A request written in Japanese has no spaces to cut on, so the
            // title comes from its opening line rather than its first words.
            conversation_topic_name(
                nonempty_transcript_text(&user_goal).as_deref(),
                None,
                || {
                    format!(
                        "{} · {}",
                        workspace_display_name(&worktree_path),
                        agent_choice_copy(&self.selected_agent).0
                    )
                },
            )
        } else {
            self.session_name_input.trim().to_owned()
        };
        let name = truncate_chars(&raw_name, 80);
        let depends_on = self.depends_on_input.into_iter().collect::<Vec<_>>();
        if let Some(dependency_id) = depends_on.first() {
            let Some(dependency) = self
                .store
                .sessions
                .iter()
                .find(|session| session.id == *dependency_id)
            else {
                self.notice = Some(
                    tr("選択した依存セッションは存在しません。別のセッションを選んでください。")
                        .into(),
                );
                self.depends_on_input = None;
                return;
            };
            if dependency.project_id != project.id {
                self.notice = Some(
                    tr("選択した依存セッションは別のプロジェクトのものです。選び直してください。")
                        .into(),
                );
                self.depends_on_input = None;
                return;
            }
            if matches!(
                dependency.status,
                SessionStatus::Failed | SessionStatus::Cancelled | SessionStatus::Lost
            ) || self.cancellation_pending(dependency.id)
            {
                self.notice =
                    Some(tr("完了しなかったセッションは、新しい依存先にできません。").into());
                return;
            }
        }
        let should_queue = !depends_on.is_empty() && !self.dependencies_finished(&depends_on);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id: project.id,
            name,
            goal,
            agent: self.selected_agent.clone(),
            tmux_name,
            created_at: now(),
            launched_at: None,
            status: if should_queue {
                SessionStatus::Queued
            } else {
                SessionStatus::Starting
            },
            worktree_path: Some(worktree_path.clone()),
            branch: None,
            agent_command: agent_command.clone(),
            native_session_id,
            native_session_path: None,
            origin: None,
            depends_on,
        });
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        // Which login this conversation belongs to, so that reopening it later
        // reopens it under the same one rather than under whatever is default.
        self.remember_session_account(id, account);
        self.persist_recent_agent_settings();
        self.goal_input.clear();
        self.session_name_input.clear();
        self.depends_on_input = None;
        if should_queue {
            self.notice = Some(tr("依存セッションの完了までキューに入れました。").into());
        } else {
            self.launch_progress = Some(LaunchProgress {
                session_id: id,
                project_name: project.name.clone(),
                worktree_path: worktree_path.clone(),
                agent: self.selected_agent.clone(),
                agent_command: agent_command.clone(),
                goal: user_goal.clone(),
                started_at: Instant::now(),
                failure: None,
                dismissed: false,
            });
            self.start_session(id);
        }
    }

    pub(crate) fn launch_empty_session(&mut self) {
        let Some(project) = self.selected_project().cloned() else {
            self.notice = Some(tr("先にプロジェクトを選んでください。").into());
            return;
        };
        if self.worktree_mutation_running(project.id) {
            self.notice = Some(tr("worktree 操作の完了を待ってください。").into());
            return;
        }
        if !self.tools.tmux {
            self.notice = Some(tr("tmux が必要です。「設定」を開いてください。").into());
            return;
        }
        let worktree_path = match resolve_selected_session_path(
            project.id,
            &project.path,
            self.session_path_project,
            &self.session_path_input,
        ) {
            Ok(path) => path,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        let id = Uuid::new_v4();
        let tmux_name = managed_tmux_name(id);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id: project.id,
            name: plain_terminal_name(&worktree_path),
            goal: tr("AI を起動しない素のターミナルです。").into(),
            agent: "terminal".into(),
            tmux_name: tmux_name.clone(),
            created_at: now(),
            launched_at: None,
            status: SessionStatus::Starting,
            worktree_path: Some(worktree_path.clone()),
            branch: None,
            agent_command: String::new(),
            native_session_id: None,
            native_session_path: None,
            origin: Some(SessionOrigin::PlainTerminal),
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        let project_path = project.path;
        // A plain terminal has no agent to report for, but a person who types
        // one in by hand gets the same reporting as a launched session.
        let hook_environment = self.mint_hook_environment(id);
        self.launch_progress = Some(LaunchProgress {
            session_id: id,
            project_name: project.name.clone(),
            worktree_path: worktree_path.clone(),
            agent: "terminal".into(),
            agent_command: String::new(),
            goal: String::new(),
            started_at: Instant::now(),
            failure: None,
            dismissed: false,
        });
        self.spawn_background(BackgroundKey::EmptySessionStart(id), move || {
            let result = (|| {
                if !is_allowed_session_path(&project_path, &worktree_path) {
                    return Err(
                        tr("作業ディレクトリが、このプロジェクトのメインフォルダでも Git worktree でもありません。")
                            .into(),
                    );
                }
                if !tool_available("tmux") {
                    return Err(tr("tmux が必要です。次のコマンドでインストールしてください: brew install tmux").into());
                }
                start_empty_tmux_session(
                    &tmux_name,
                    &worktree_path,
                    &hook_variables(hook_environment.as_ref()),
                )
                    .map_err(|error| error.to_string())?;
                Ok(git_branch(&worktree_path).ok())
            })();
            BackgroundResult::EmptySessionStarted {
                session_id: id,
                result,
            }
        });
        self.notice = Some(tr("ターミナルを起動しています…").into());
    }

    pub(crate) fn launch_custom_script_session(
        &mut self,
        project_id: Uuid,
        worktree_path: &Path,
        name: &str,
        command: &str,
    ) {
        if !self.tools.tmux {
            self.notice = Some(tr("tmux が必要です。「設定」を開いてください。").into());
            return;
        }
        let id = Uuid::new_v4();
        let tmux_name = managed_tmux_name(id);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id,
            name: name.to_owned(),
            goal: format!("Run: {command}"),
            agent: "script".into(),
            tmux_name: tmux_name.clone(),
            created_at: now(),
            launched_at: Some(now()),
            status: SessionStatus::Starting,
            worktree_path: Some(worktree_path.to_path_buf()),
            branch: None,
            agent_command: command.to_owned(),
            native_session_id: None,
            native_session_path: None,
            origin: Some(SessionOrigin::PlainTerminal),
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        let wt = worktree_path.to_path_buf();
        let cmd = command.to_owned();
        let hook_environment = self.mint_hook_environment(id);
        self.spawn_background(BackgroundKey::SessionStart(id), move || {
            let result = start_tmux_agent_session(
                &tmux_name,
                &wt,
                &cmd,
                &hook_variables(hook_environment.as_ref()),
            )
            .map(|_| None)
            .map_err(|e| e.to_string());
            BackgroundResult::SessionStarted {
                session_id: id,
                result,
            }
        });
        self.select_session(id);
    }

    pub(crate) fn create_worktree(&mut self) {
        let Some(project) = self.selected_project().cloned() else {
            self.notice = Some(tr("先にプロジェクトを選んでください。").into());
            return;
        };
        let branch = self.worktree_branch_input.trim().to_owned();
        if branch.is_empty() {
            self.notice = Some(tr("worktree のブランチ名を入力してください。").into());
            return;
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::WorktreeMutation(project.id))
        {
            self.notice = Some(tr("worktree の操作が既に実行中です。").into());
            return;
        }
        let project_id = project.id;
        let project_path = project.path;
        self.spawn_background(BackgroundKey::WorktreeMutation(project_id), move || {
            BackgroundResult::WorktreeCreated {
                project_id,
                result: create_worktree_from_mainline(&project_path, &branch),
            }
        });
        self.notice = Some(tr("worktree を作成しています…").into());
    }

    pub(crate) fn remove_worktree(&mut self, project: &Project, path: &Path) {
        if worktree_has_reserved_session(&self.store.sessions, path) {
            self.notice = Some(
                tr("この worktree を削除する前に、使用中および待機中のセッションをすべて停止・完了・キャンセルしてください。")
                    .into(),
            );
            return;
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::WorktreeMutation(project.id))
        {
            self.notice = Some(tr("worktree の操作が既に実行中です。").into());
            return;
        }
        let project_id = project.id;
        let project_path = project.path.clone();
        let path = path.to_path_buf();
        let result_path = path.clone();
        self.spawn_background(BackgroundKey::WorktreeMutation(project_id), move || {
            let result = (|| {
                let worktrees = list_worktrees(&project_path).map_err(|error| error.to_string())?;
                let Some(worktree) = worktrees.iter().find(|worktree| worktree.path == path) else {
                    return Err(
                        tr("その worktree はこのプロジェクトのものではなくなっています。").into(),
                    );
                };
                if worktree.is_main {
                    return Err(tr("メインの worktree は削除できません。").into());
                }
                let mut command = git_command(&project_path);
                command
                    .args(["-C"])
                    .arg(&project_path)
                    .args(["worktree", "remove"])
                    .arg(&path);
                let output = run_command_with_timeout(&mut command, Duration::from_secs(30))
                    .map_err(|error| tf!("Git を実行できませんでした: {error}", error = error))?;
                if output.status.success() {
                    Ok(())
                } else {
                    Err(tf!(
                        "Git が削除を拒否しました: {p0}",
                        p0 = String::from_utf8_lossy(&output.stderr).trim()
                    ))
                }
            })();
            BackgroundResult::WorktreeRemoved {
                project_id,
                path: result_path,
                result,
            }
        });
        self.notice = Some(tr("worktree を削除しています…").into());
    }

    pub(crate) fn land_worktree(
        &mut self,
        project: &Project,
        worktree_path: &Path,
        branch: &str,
        auto_cleanup: bool,
    ) {
        if landing_blocked_by_session(&self.store.sessions, project.id, worktree_path) {
            self.notice = Some(
                tr("この worktree またはメイン作業ツリーで動作中・待機中のセッションがあります。停止・完了・キャンセルしてからマージしてください。")
                    .into(),
            );
            return;
        }
        // The merge writes the project checkout's index and `HEAD`, so it waits
        // for, and is waited for by, every other writer of that checkout.
        if Self::git_worktree_busy(&self.background_tasks, project.id) {
            self.notice = Some(tr("Git の操作が既に実行中です。").into());
            return;
        }

        let project_id = project.id;
        let project_path = project.path.clone();
        let wt_path = worktree_path.to_path_buf();
        let branch_name = branch.to_owned();

        self.spawn_background(BackgroundKey::LandWorktree(project_id), move || {
            let result = git_land_worktree(&project_path, &wt_path, &branch_name, auto_cleanup)
                .map_err(|e| e.to_string());
            BackgroundResult::WorktreeLanded { project_id, result }
        });
        self.notice = Some(tr("worktree を main にマージしています…").into());
    }

    pub(crate) fn create_pull_request(
        &mut self,
        project: &Project,
        worktree_path: &Path,
        title: &str,
        body: &str,
        draft: bool,
    ) {
        if self
            .background_tasks
            .contains(&BackgroundKey::CreatePullRequest(project.id))
        {
            self.notice = Some(tr("PR の作成が既に実行中です。").into());
            return;
        }

        let project_id = project.id;
        let wt_path = worktree_path.to_path_buf();
        let title_str = title.to_owned();
        let body_str = body.to_owned();

        self.spawn_background(BackgroundKey::CreatePullRequest(project_id), move || {
            let result = git_create_pull_request(&wt_path, &title_str, &body_str, draft)
                .map_err(|e| e.to_string());
            BackgroundResult::PullRequestCreated { project_id, result }
        });
        self.notice = Some(tr("プルリクエストを作成しています…").into());
    }

    pub(crate) fn fetch_github_issues(&mut self, project: &Project) {
        if self
            .background_tasks
            .contains(&BackgroundKey::FetchIssues(project.id))
        {
            return;
        }
        let project_id = project.id;
        let project_path = project.path.clone();
        self.spawn_background(BackgroundKey::FetchIssues(project_id), move || {
            let result = list_github_issues(&project_path).map_err(|e| e.to_string());
            BackgroundResult::IssuesFetched { project_id, result }
        });
    }

    pub(crate) fn capture_checkpoint(&mut self, session_id: Uuid, summary: &str) {
        let Some(session) = self.store.sessions.iter().find(|s| s.id == session_id) else {
            return;
        };
        let Some(ref wt_path) = session.worktree_path else {
            return;
        };
        let turn_index = self
            .session_prompt_turns
            .get(&session_id)
            .map(|turns| turns.len())
            .unwrap_or(0);
        if let Ok(checkpoint) = git_create_checkpoint(wt_path, session_id, turn_index, summary) {
            self.session_checkpoints
                .entry(session_id)
                .or_default()
                .push(checkpoint);
        }
    }

    pub(crate) fn restore_checkpoint(&mut self, session_id: Uuid, commit_sha: &str) {
        let Some(session) = self.store.sessions.iter().find(|s| s.id == session_id) else {
            return;
        };
        let Some(wt_path) = session.worktree_path.clone() else {
            return;
        };
        let project_id = session.project_id;
        match git_restore_checkpoint(&wt_path, commit_sha) {
            Ok(()) => {
                self.invalidate_project_file_views(project_id);
                self.invalidate_project_diff_views(project_id);
                self.notice = Some(tr("チェックポイントを復元しました。").into());
            }
            Err(e) => {
                self.notice = Some(tf!(
                    "チェックポイントの復元に失敗しました: {error}",
                    error = e
                ));
            }
        }
    }

    pub(crate) fn refresh_checkpoints(&mut self, session_id: Uuid) {
        let Some(session) = self.store.sessions.iter().find(|s| s.id == session_id) else {
            return;
        };
        let Some(ref wt_path) = session.worktree_path else {
            return;
        };
        if let Ok(checkpoints) = git_list_checkpoints(wt_path, session_id) {
            self.session_checkpoints.insert(session_id, checkpoints);
        }
    }

    pub(crate) fn scan_orphaned_tmux_sessions(&mut self) {
        let known = self
            .store
            .sessions
            .iter()
            .map(|session| session.tmux_name.clone())
            .collect::<HashSet<_>>();
        self.spawn_background(BackgroundKey::RecoveryScan, move || {
            BackgroundResult::OrphanedTmuxSessions(
                list_operon_tmux_sessions()
                    .map(|sessions| {
                        sessions
                            .into_iter()
                            .filter(|session| !known.contains(&session.name))
                            .collect()
                    })
                    .map_err(|error| error.to_string()),
            )
        });
        self.notice = Some(tr("復旧可能な tmux セッションを検索しています…").into());
    }

    pub(crate) fn adopt_orphaned_tmux_session(&mut self, orphan: &OrphanedTmuxSession) {
        if self
            .store
            .sessions
            .iter()
            .any(|session| session.tmux_name == orphan.name)
        {
            self.notice = Some(tr("そのターミナルセッションは既に登録されています。").into());
            return;
        }
        let key = BackgroundKey::RecoveryAdopt(orphan.name.clone());
        if self.background_tasks.contains(&key) {
            return;
        }
        let orphan = orphan.clone();
        let projects = self.store.projects.clone();
        self.spawn_background(key, move || {
            let result = projects
                .iter()
                .find(|project| is_allowed_session_path(&project.path, &orphan.cwd))
                .map(|project| (project.id, git_branch(&orphan.cwd).ok()))
                .ok_or_else(|| {
                    tr("先に、このターミナルの作業ディレクトリを含むプロジェクトを登録してください。")
                        .to_owned()
                });
            BackgroundResult::OrphanAdopted { orphan, result }
        });
        self.notice = Some(tr("ターミナルのプロジェクトを確認しています…").into());
    }

    pub(crate) fn search_local_transcripts(&mut self) {
        let query = self.transcript_search_input.trim().to_owned();
        if query.is_empty() {
            self.transcript_matches.clear();
            return;
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::TranscriptSearch)
        {
            self.notice = Some(tr("ローカル履歴の検索が既に実行中です。").into());
            return;
        }
        let search_query = query.clone();
        self.spawn_background(BackgroundKey::TranscriptSearch, move || {
            BackgroundResult::TranscriptSearch {
                query: search_query.clone(),
                scan: search_local_transcripts(&search_query, 50),
            }
        });
        self.notice = Some(tf!(
            "“{query}” をローカル履歴から検索しています…",
            query = query
        ));
    }

    pub(crate) fn scan_cli_sessions(&mut self, project: &Project) {
        self.request_cli_sessions(project, true);
        self.notice = Some(tr("ローカル CLI セッションを検索しています…").into());
    }

    pub(crate) fn resume_cli_session(&mut self, project: &Project, cli_session: &CliSession) {
        if self.worktree_mutation_running(project.id) {
            self.notice = Some(tr("worktree 操作の完了を待ってください。").into());
            return;
        }
        if !is_safe_cli_session_id(&cli_session.id) {
            self.notice = Some(tr("この CLI セッション ID は安全に再開できません。").into());
            return;
        }
        let missing = self.tools.missing_session_tools(cli_session.provider);
        if !missing.is_empty() {
            self.notice = Some(tf!(
                "この会話を再開するには {tools} が PATH 上に必要です。",
                tools = missing.join(tr(" と "))
            ));
            return;
        }
        let id = Uuid::new_v4();
        let tmux_name = managed_tmux_name(id);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id: project.id,
            name: restored_session_name(cli_session),
            goal: cli_session.last_user_message.clone().unwrap_or_else(|| {
                tf!(
                    "既存の {p0} セッションを再開します。",
                    p0 = cli_session.provider.label()
                )
            }),
            agent: cli_session.provider.agent().into(),
            tmux_name,
            created_at: now(),
            launched_at: None,
            status: SessionStatus::Starting,
            worktree_path: Some(project.path.clone()),
            branch: cli_session.branch.clone(),
            agent_command: cli_session.provider.native_resume_command(&cli_session.id),
            native_session_id: Some(cli_session.native_id.clone()),
            native_session_path: Some(cli_session.path.clone()),
            origin: Some(SessionOrigin::Resumed {
                from: cli_session.provider,
            }),
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        self.start_session(id);
    }

    /// Resolves the provider-owned ID created by a managed launch. Restored
    /// conversations receive a new destination-provider ID; every later
    /// launch reuses that ID and therefore keeps the restored full context.
    pub(crate) fn request_native_session_resolution(&mut self, session_id: Uuid) {
        if self
            .background_tasks
            .contains(&BackgroundKey::NativeSessionResolution(session_id))
        {
            return;
        }
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return;
        };
        if session.native_session_id.is_some() {
            return;
        }
        let Some(provider) = cli_provider_for_agent(&session.agent) else {
            return;
        };
        let Some(project) = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .cloned()
        else {
            return;
        };
        let tracking_token = native_tracking_token(session_id);
        // Only a Codex launch whose first prompt carries the tracking token can
        // be identified from what the launch itself leaves behind. Claude
        // reserves its ID before starting. An untracked Codex launch leaves
        // nothing a resolver could match, and an Antigravity launch has no
        // conversation to name until the user types into it, which can be hours
        // later, so waiting on either would only report a failure the user
        // cannot act on. Their conversations are recovered when a restore or
        // resume actually needs the ID.
        if provider != CliProvider::Codex
            || !codex_launch_can_be_resolved(&session.goal, session_id)
        {
            return;
        }
        self.spawn_background(
            BackgroundKey::NativeSessionResolution(session_id),
            move || BackgroundResult::NativeSessionResolved {
                session_id,
                result: resolve_codex_session_after_launch(&project.path, &tracking_token),
            },
        );
    }

    pub(crate) fn resume_managed_native_session(&mut self, source_id: Uuid) {
        let Some(source) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == source_id)
            .cloned()
        else {
            return;
        };
        let Some(provider) = cli_provider_for_agent(&source.agent) else {
            return;
        };
        let Some(native_session_id) = source.native_session_id.clone() else {
            self.notice = Some(
                tr("このターミナルにはネイティブ再開 ID がまだ保存されていません。初回起動の完了を待つか、ローカル CLI セッションを検出してください。")
                    .into(),
            );
            return;
        };
        if !is_safe_cli_session_id(&native_session_id) {
            self.notice =
                Some(tr("保存されているネイティブ再開 ID は安全に実行できません。").into());
            return;
        }
        let missing = self.tools.missing_session_tools(provider);
        if !missing.is_empty() {
            self.notice = Some(tf!(
                "この会話を再開するには {tools} が PATH 上に必要です。",
                tools = missing.join(tr(" と "))
            ));
            return;
        }
        let project_path = self
            .store
            .projects
            .iter()
            .find(|project| project.id == source.project_id)
            .map(|project| project.path.clone())
            .unwrap_or_default();
        let workspace = source.worktree_path.clone().unwrap_or(project_path.clone());
        let recorded_transcript = source
            .native_session_path
            .clone()
            .or_else(|| native_session_path_for_project(provider, &workspace, &native_session_id));
        // Reopening the wrong conversation is exactly as wrong as restoring it
        // elsewhere, so the stored identity is re-checked here too.
        let (native_session_id, mut native_session_path) =
            match corrected_managed_claude_conversation(
                &source,
                &workspace,
                recorded_transcript.as_deref(),
            ) {
                Ok(Some(corrected)) => (corrected.native_id, Some(corrected.path)),
                Ok(None) => (native_session_id, source.native_session_path.clone()),
                Err(error) => {
                    self.notice = Some(tf!(
                        "このターミナルの実際の会話を特定できませんでした: {error}",
                        error = error
                    ));
                    return;
                }
            };
        if !is_safe_cli_session_id(&native_session_id) {
            self.notice =
                Some(tr("保存されているネイティブ再開 ID は安全に実行できません。").into());
            return;
        }
        match validate_session_transcript_for_resume(
            provider,
            &workspace,
            &native_session_id,
            native_session_path
                .as_deref()
                .or(recorded_transcript.as_deref()),
        ) {
            TranscriptResumeReadiness::Ready(path) => {
                native_session_path = Some(path);
            }
            TranscriptResumeReadiness::Sanitized {
                path,
                removed_bytes,
            } => {
                native_session_path = Some(path);
                self.notice = Some(tf!(
                    "強制終了で破損したトランスクリプトの末尾（{removed_bytes} バイト）を安全に修復して再開します。",
                    removed_bytes = removed_bytes
                ));
            }
            TranscriptResumeReadiness::EmptyOrMissing => {
                self.notice = Some(
                    tr("会話ログが記録される前に終了したため、会話 ID からの再開はできません。新しいセッションとして開始します。")
                        .into(),
                );
                self.start_session(source_id);
                return;
            }
        }
        let id = Uuid::new_v4();
        let tmux_name = managed_tmux_name(id);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id: source.project_id,
            // Reopening a conversation keeps its title: naming the mechanism
            // would only push the subject of the work out of view, and every
            // resumed session would read the same way.
            name: session_title(&source),
            goal: source.goal,
            agent: provider.agent().into(),
            tmux_name,
            created_at: now(),
            launched_at: None,
            status: SessionStatus::Starting,
            worktree_path: source.worktree_path,
            branch: source.branch,
            // Reopening the conversation reopens the agent that was having it:
            // the model, the mode, and the switches this terminal was launched
            // with travel with the resume, minus the ones that would contradict
            // it.
            agent_command: resume_command_with_launch_options(
                provider,
                &source.agent,
                &source.agent_command,
                &native_session_id,
            ),
            native_session_id: Some(native_session_id),
            native_session_path,
            origin: Some(SessionOrigin::Resumed { from: provider }),
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        // Change 027 made a resume carry the options its launch was started
        // with. The login is one of those: reopening a work conversation under
        // a personal account is a refusal at best, and a second conversation
        // nobody meant to start at worst.
        self.remember_session_account(id, self.accounts.account_of_session(source_id));
        self.start_session(id);
    }

    pub(crate) fn remember_cli_sessions(&mut self, project_id: Uuid, cli_sessions: &[CliSession]) {
        let previous_store = self.store.clone();
        self.store
            .native_session_references
            .retain(|reference| reference.project_id != project_id);
        self.store
            .native_session_references
            .extend(cli_sessions.iter().map(|session| NativeSessionReference {
                project_id,
                provider: session.provider,
                session_id: session.native_id.clone(),
                source_path: session.path.clone(),
                last_seen_at: session.updated_at,
            }));
        if !self.persist() {
            self.store = previous_store;
        }
    }

    pub(crate) fn handoff_cli_session(
        &mut self,
        project: &Project,
        source: &CliSession,
        target: &str,
    ) {
        let transcript = match full_history_restore_transcript(source) {
            Ok(transcript) => transcript,
            Err(error) => {
                self.notice = Some(tf!(
                    "この会話のローカル履歴ファイルを読めないため復元できません: {error}",
                    error = error
                ));
                return;
            }
        };
        let Ok(destination) = full_history_restore_destination(target) else {
            self.notice = Some(
                tr("会話全履歴の復元先として選べるのは Codex CLI・Claude Code・Antigravity CLI です。")
                    .into(),
            );
            return;
        };
        if self.worktree_mutation_running(project.id) {
            self.notice = Some(tr("worktree 操作の完了を待ってください。").into());
            return;
        }
        let missing = self.tools.missing_session_tools(destination);
        if !missing.is_empty() {
            self.notice = Some(tf!(
                "{destination} へ会話全履歴を復元するには {tools} が PATH 上に必要です。",
                destination = destination.label(),
                tools = missing.join(tr(" と "))
            ));
            return;
        }
        let import_id = Uuid::new_v4();
        let note = self.handoff_note_input.trim().to_owned();
        let source_provider = source.provider;
        let source_conversation = source.native_id.clone();
        let mut source = source.clone();
        source.last_user_message = restore_source_last_user_message(&source, &transcript);
        let source_last_message = source
            .last_user_message
            .clone()
            .unwrap_or_else(|| tr("（最後のユーザーメッセージを取得できませんでした）").to_owned());
        let project = project.clone();
        let source = source;
        let target = target.to_owned();
        let archive_root = shared_session_archive_directory(&self.data_file);
        self.spawn_background(BackgroundKey::FullHistoryImport(import_id), move || {
            BackgroundResult::FullHistoryImported(Box::new(FullHistoryImportResult {
                import_id,
                project: project.clone(),
                source: source.clone(),
                target: target.clone(),
                note: note.clone(),
                result: import_full_cli_history(
                    &project,
                    &source,
                    &target,
                    &archive_root,
                    import_id,
                ),
            }))
        });
        // A restore takes long enough to be waited on, so it says so in the one
        // place a person is already looking — the middle of the window — and
        // the banner is left for whatever it turns out to be.
        self.notice = None;
        self.restore_progress = Some(RestoreProgress {
            import_id,
            source: source_provider,
            source_conversation,
            destination,
            last_user_message: source_last_message,
            started_at: Instant::now(),
        });
    }

    pub(crate) fn finish_full_history_import(
        &mut self,
        project: &Project,
        source: &CliSession,
        target: &str,
        note: &str,
        imported: ImportedNativeSession,
    ) {
        let id = imported.managed_session_id;
        let tmux_name = managed_tmux_name(id);
        let previous_store = self.store.clone();
        self.store.cli_handoffs.push(CliHandoff {
            id: Uuid::new_v4(),
            project_id: project.id,
            source_provider: source.provider,
            source_session_id: source.native_id.clone(),
            source_path: source.path.clone(),
            target_agent: target.to_owned(),
            note: note.to_owned(),
            created_at: now(),
            full_history: true,
            archive_path: Some(imported.archive_path.clone()),
            destination_session_id: imported.native_session_id.clone(),
        });
        self.store.sessions.push(Session {
            id,
            project_id: project.id,
            name: restored_session_name(source),
            goal: source.last_user_message.clone().unwrap_or_else(|| {
                tf!(
                    "{p0} の会話全履歴を復元しました。",
                    p0 = source.provider.label()
                )
            }),
            agent: target.to_owned(),
            tmux_name,
            created_at: now(),
            launched_at: None,
            status: SessionStatus::Starting,
            worktree_path: Some(project.path.clone()),
            branch: source.branch.clone(),
            agent_command: imported.agent_command.clone(),
            native_session_id: imported.native_session_id.clone(),
            native_session_path: Some(imported.native_session_path.clone()),
            origin: Some(SessionOrigin::Restored {
                from: source.provider,
            }),
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            self.notice = Some(match imported.native_session_id.as_deref() {
                Some(native_session_id) => tf!("復元先 {p0} は作成されましたが、Operon の記録に保存できませんでした。手動で {p1} を実行できます。", p0 = imported.provider.label(), p1 = imported.provider.native_resume_command(native_session_id)),
                None => tf!("復元用 {p0} セッションは準備されましたが、Operon の記録に保存できませんでした。", p0 = imported.provider.label()),
            });
            return;
        }
        self.handoff_note_input.clear();
        // "そのまま" is what this used to claim. A restore projects the source
        // — it carries conversation and reasoning and leaves the CLI's own
        // bookkeeping behind — so the notice reports what it carried instead of
        // asserting a fidelity nothing checks.
        self.notice = Some(tf!(
            "{source} の会話 {source_conversation} から {records} 件の会話ターンを復元し、{destination} の会話 {destination_conversation} を作成しました。{breakdown}復元先セッションを起動しています…",
            source = source.provider.label(),
            source_conversation = source.native_id,
            records = imported.imported_records,
            destination = imported.provider.label(),
            destination_conversation = imported
                .native_session_id
                .as_deref()
                .unwrap_or(tr("（不明）")),
            breakdown = restore_classification_notice(&imported.classification),
        ));
        self.start_session(id);
    }

    /// Restores a managed Codex or Claude session into another CLI without
    /// forging or reusing provider-specific conversation IDs.
    pub(crate) fn handoff_managed_session(&mut self, source_id: Uuid, target: &str) {
        let Some(source) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == source_id)
            .cloned()
        else {
            return;
        };
        let Some(source_provider) = cli_provider_for_agent(&source.agent) else {
            self.notice =
                Some(tr("復元できるのは Codex・Claude・Antigravity のセッションだけです。").into());
            return;
        };
        let Some(project) = self
            .store
            .projects
            .iter()
            .find(|project| project.id == source.project_id)
            .cloned()
        else {
            self.notice =
                Some(tr("このセッションのプロジェクトは利用できなくなっています。").into());
            return;
        };
        // Current managed Claude launches reserve this ID before the terminal
        // starts. Older records can be missing the duplicated metadata even
        // though their stored launch command still carries the durable ID.
        let command_native_session_id =
            native_session_id_from_agent_command(source_provider, &source.agent_command);
        let recovered_source = if source.native_session_id.is_none()
            && command_native_session_id.is_none()
        {
            let workspace = source.worktree_path.as_deref().unwrap_or(&project.path);
            let recovered = match source_provider {
                CliProvider::Claude => {
                    recover_managed_claude_session(workspace, source.created_at, &source.goal)
                }
                CliProvider::Codex => recover_managed_codex_session(workspace, source.created_at),
                // Antigravity's conversation is identified by which `agy`
                // process this terminal is, so it needs the launch rather than
                // the record: a queued terminal starts once its dependencies
                // finish, and a retried one starts again.
                CliProvider::Gemini => recover_managed_antigravity_session(
                    workspace,
                    source.launched_at.unwrap_or(source.created_at),
                ),
            };
            match recovered {
                Ok(session) => session,
                Err(error) => {
                    self.notice = Some(tf!("{p0} 会話履歴の自動復旧に失敗しました。元の会話は変更されていません: {error}", error = error, p0 = source_provider.label()));
                    return;
                }
            }
        } else {
            None
        };
        let Some(native_session_id) = source
            .native_session_id
            .clone()
            .or(command_native_session_id)
            .or_else(|| {
                recovered_source
                    .as_ref()
                    .map(|session| session.native_id.clone())
            })
        else {
            self.notice = Some(tf!("このセッションの {label} 会話IDを復元できませんでした。Discover local CLI sessions から該当する {label} 会話を選んで復元してください。", label = source_provider.label()));
            return;
        };
        let native_session_path = source
            .native_session_path
            .clone()
            .or_else(|| {
                recovered_source
                    .as_ref()
                    .map(|session| session.path.clone())
            })
            .or_else(|| {
                let workspace = source.worktree_path.as_deref().unwrap_or(&project.path);
                native_session_path_for_project(source_provider, workspace, &native_session_id)
            });
        let Some(native_session_path) = native_session_path else {
            self.notice = Some(
                tr("元のネイティブ会話ログが見つかりません。完全復元ではログの全記録が必要です。")
                    .into(),
            );
            return;
        };
        // Restoring the wrong conversation is indistinguishable from a restore
        // that did nothing, so re-read the terminal's own conversation whenever
        // the stored identity cannot be its.
        let workspace = source.worktree_path.clone().unwrap_or(project.path.clone());
        let (native_session_id, native_session_path) = match corrected_managed_claude_conversation(
            &source,
            &workspace,
            Some(&native_session_path),
        ) {
            Ok(Some(corrected)) => (corrected.native_id, corrected.path),
            Ok(None) => (native_session_id, native_session_path),
            Err(error) => {
                self.notice = Some(tf!("このセッションの実際の Claude 会話を特定できませんでした。元の会話は変更されていません: {error}", error = error));
                return;
            }
        };
        if source.native_session_id.as_deref() != Some(&native_session_id)
            || source.native_session_path.as_deref() != Some(native_session_path.as_path())
        {
            let previous_store = self.store.clone();
            if let Some(stored) = self
                .store
                .sessions
                .iter_mut()
                .find(|session| session.id == source_id)
            {
                stored.native_session_id = Some(native_session_id.clone());
                stored.native_session_path = Some(native_session_path.clone());
            }
            if !self.persist() {
                self.store = previous_store;
                return;
            }
        }
        self.handoff_cli_session(
            &project,
            &CliSession {
                provider: source_provider,
                native_id: native_session_id.clone(),
                id: native_session_id,
                title: Some(session_title(&source)),
                // A terminal launched without a goal has no message of its own.
                // Leave it unset so the restore reads the conversation's own
                // transcript instead of carrying an empty string through.
                last_user_message: nonempty_transcript_text(&session_display_goal(&source.goal)),
                branch: source.branch.clone(),
                updated_at: source.created_at,
                path: native_session_path,
            },
            target,
        );
    }

    /// Open the launch sheet for a project over the current page. Every way
    /// to start a session comes here, so none of them navigates away.
    pub(crate) fn open_project_session_setup(&mut self, project_id: Uuid) {
        self.select_project(Some(project_id));
        self.launch_sheet_open = true;
        self.launch_sheet_needs_focus = true;
        self.launch_sheet_is_git = self
            .selected_project()
            .is_some_and(|project| has_git_metadata(&project.path));
    }

    /// Whether the sheet's launch button is enabled — and so whether ⌘↩
    /// launches. One answer for both, because the danger acknowledgement is
    /// enforced here and nowhere in `launch_session` itself.
    pub(crate) fn launch_ready(&self, project: &Project) -> bool {
        let agent_ready = self.tools.agent_available(&self.selected_agent);
        let custom_command_ready =
            self.selected_agent != "custom" || !self.custom_command.trim().is_empty();
        let dangerous_mode = launch_needs_acknowledgement(
            &self.selected_agent,
            &self.agent_mode_input,
            &self.agent_flag_inputs,
            &self.custom_command,
        );
        self.tools.tmux
            && agent_ready
            && custom_command_ready
            && !self.worktree_mutation_running(project.id)
            && (!dangerous_mode || self.launch_acknowledged())
    }

    pub(crate) fn open_project_worktrees(&mut self, project_id: Uuid) {
        self.select_project(Some(project_id));
        self.project_tab = ProjectTab::Worktrees;
        self.page = Page::Projects;
    }

    pub(crate) fn open_new_session(&mut self) {
        let project_id = self
            .selected_session
            .and_then(|session_id| {
                self.store
                    .sessions
                    .iter()
                    .find(|session| session.id == session_id)
                    .map(|session| session.project_id)
            })
            .or(self.selected_project)
            .or_else(|| self.store.projects.first().map(|project| project.id));
        let Some(project_id) = project_id else {
            self.notice = Some(tr("先にプロジェクトを追加してください。").into());
            self.page = Page::Projects;
            return;
        };
        self.open_project_session_setup(project_id);
    }

    /// Remove a session from a list. A live terminal is stopped first; files
    /// and worktrees are deliberately left alone.
    pub(crate) fn request_session_removal(&mut self, session_id: Uuid) {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return;
        };
        match session.status {
            SessionStatus::Active => self.pending_session_close = Some(session_id),
            SessionStatus::Queued => {
                self.cancel_queued_session(session_id);
                self.remove_session_record(session_id);
            }
            SessionStatus::Exited
            | SessionStatus::Failed
            | SessionStatus::Cancelled
            | SessionStatus::Lost => {
                self.remove_after_close.insert(session_id);
                self.close_completed_terminal(session_id)
            }
            SessionStatus::Starting | SessionStatus::Unknown => {
                self.notice = Some(tr("起動中です。実行状態になってから閉じてください。").into());
            }
        }
    }

    fn session_removal_confirmation(&mut self, ui: &mut egui::Ui, session_id: Uuid) {
        if self.pending_session_close != Some(session_id) {
            return;
        }
        ui.label(
            RichText::new(tr(
                "実行中のセッションを削除しますか？ 停止後に一覧から取り除きます。",
            ))
            .small()
            .color(self.store.theme.palette().accent_soft),
        );
        ui.horizontal(|ui| {
            if ui.small_button(tr("キャンセル")).clicked() {
                self.pending_session_close = None;
            }
            if ui.small_button(tr("削除")).clicked() {
                self.pending_session_close = None;
                self.remove_after_close.insert(session_id);
                self.stop_session(session_id);
            }
        });
    }

    pub(crate) fn dependencies_finished(&self, dependencies: &[Uuid]) -> bool {
        dependencies_finished(&self.store.sessions, dependencies)
    }

    /// How many sessions want the person: the ones stopped on a question, the
    /// ones stopped on a prerequisite that will never arrive, and the ones
    /// that said something while nobody was reading them. One number, because
    /// the question a reader is asking the header is "how many need me", and a
    /// session that is two of those is still one session.
    ///
    /// It is the 要対応 filter group, counted — the same predicate, so the
    /// home screen's attention banner and the chip on the session list cannot
    /// drift into meaning two different things. Two consequences of saying it
    /// that way: a
    /// queued session whose prerequisite failed now counts, and one that was
    /// waiting on an answer when a stop was asked for no longer does, because
    /// a terminal on its way out is not waiting on anybody.
    pub(crate) fn attention_count(&self) -> usize {
        self.store
            .sessions
            .iter()
            .filter(|session| self.session_in_status_group(session, SessionStatusGroup::NeedsYou))
            .count()
    }

    /// Which state a session is in, without the words. This is what the
    /// session list's filter chips count with — once per session in the
    /// counting pass, and once more per session in the filtering pass — so it
    /// must not allocate. `status_view` builds two `String`s and is the wrong
    /// thing to count with.
    pub(crate) fn session_status_kind_for(&self, session: &Session) -> SessionStatusKind {
        session_status_kind(
            &self.store.sessions,
            session,
            self.cancellation_pending(session.id),
            self.session_activity_for(session.id),
        )
    }

    /// Whether a session belongs in a group, for the purpose of the filter.
    ///
    /// This is where the groups stop being a partition: a session that spoke
    /// while nobody was reading it wants a person whatever state it is in, so
    /// an unread finished session is in 要対応 *and* in 終了. The chips are
    /// independent filters rather than a segmented control, so that is a
    /// property and not a contradiction — but it does mean the four counts can
    /// sum to more than the number of sessions, and nothing claims otherwise.
    fn session_in_status_group(&self, session: &Session, group: SessionStatusGroup) -> bool {
        self.kind_in_status_group(self.session_status_kind_for(session), session, group)
    }

    /// The same question asked of a kind already in hand. Deciding the kind
    /// means walking the dependency list of a queued session, so a caller that
    /// asks about several groups computes it once and comes here.
    fn kind_in_status_group(
        &self,
        kind: SessionStatusKind,
        session: &Session,
        group: SessionStatusGroup,
    ) -> bool {
        if group == SessionStatusGroup::NeedsYou && self.is_session_unread(session.id) {
            return true;
        }
        kind.group() == group
    }

    /// How many sessions are in each group right now, indexed by
    /// `SessionStatusGroup::index`. One pass, no allocation.
    pub(crate) fn status_group_counts(&self) -> [usize; 4] {
        let mut counts = [0usize; 4];
        for session in &self.store.sessions {
            let group = self.session_status_kind_for(session).group();
            counts[group.index()] += 1;
            // The unread overlay, kept in step with `attention_count`: a
            // session already counted under 要対応 by its state is not counted
            // twice.
            if group != SessionStatusGroup::NeedsYou && self.is_session_unread(session.id) {
                counts[SessionStatusGroup::NeedsYou.index()] += 1;
            }
        }
        counts
    }

    /// Whether the session list should show this session. With nothing
    /// selected everything passes, which is the list as it was before there
    /// was a filter at all.
    pub(crate) fn session_matches_status_filter(&self, session: &Session) -> bool {
        if !self.status_filter.iter().any(|selected| *selected) {
            return true;
        }
        // Once per session, not once per selected chip: for a queued session,
        // deciding the kind walks its dependencies against every other
        // session, and this runs for every row on every frame.
        let kind = self.session_status_kind_for(session);
        SessionStatusGroup::GROUPS.iter().any(|group| {
            self.status_filter[group.index()] && self.kind_in_status_group(kind, session, *group)
        })
    }

    /// Turn one chip on or off.
    pub(crate) fn toggle_status_filter(&mut self, group: SessionStatusGroup) {
        let index = group.index();
        self.status_filter[index] = !self.status_filter[index];
        self.forget_pending_session_close();
    }

    pub(crate) fn clear_status_filter(&mut self) {
        self.status_filter = [false; 4];
        self.forget_pending_session_close();
    }

    /// The one move the home screen's attention banner offers: the session
    /// list, already narrowed to the sessions it was counting.
    pub(crate) fn show_sessions_needing_you(&mut self) {
        self.show_status_group(SessionStatusGroup::NeedsYou);
    }

    /// The session list narrowed to exactly one group — what a Home tile
    /// opens, so the tile and the chip it lands on count the same thing.
    pub(crate) fn show_status_group(&mut self, group: SessionStatusGroup) {
        self.page = Page::Sessions;
        self.status_filter = [false; 4];
        self.status_filter[group.index()] = true;
        self.forget_pending_session_close();
    }

    /// Open ⌘K from its chord or from the toolbar, one path so the two cannot
    /// disagree about focus or where the selection starts.
    pub(crate) fn open_command_palette(&mut self) {
        self.command_palette_open = true;
        self.command_palette_needs_focus = true;
        self.command_selection = 0;
    }

    /// Every change to what the list shows drops a removal confirmation that
    /// has not been answered. The confirmation is drawn inside the card it
    /// belongs to, so re-narrowing can take the question off screen with the
    /// answer still pending, and widening later brings back a 削除しますか that
    /// nobody remembers asking for. Nothing is lost by forgetting it: the
    /// removal only happens on the confirm click.
    fn forget_pending_session_close(&mut self) {
        self.pending_session_close = None;
    }

    /// The state to show for a session, including the live agent activity the
    /// poller has settled on.
    pub(crate) fn status_view(&self, session: &Session) -> SessionStatusView {
        let mut view = session_status_view(
            &self.store.sessions,
            session,
            self.cancellation_pending(session.id),
            self.session_activity_for(session.id),
        );
        // The CLIs name the tool they are running, and "Bash を実行中" answers
        // the question a person actually has about a working session, which
        // "応答を生成中" cannot.
        if let Some(tool) = self
            .fresh_hook_status(session.id)
            .filter(|status| status.activity == AgentActivity::Working)
            .and_then(|status| status.tool.as_deref())
        {
            view.hint = tf!("{p0} を実行中", p0 = tool);
        }
        view
    }

    pub(crate) fn begin_session_rename(&mut self, session: &Session) {
        self.renaming_session = Some(session.id);
        // Seed the editor with what the list shows, not the raw stored name:
        // a legacy record still carries the old internal wording.
        self.rename_input = session_title(session);
        self.rename_needs_focus = true;
    }

    pub(crate) fn cancel_session_rename(&mut self) {
        self.renaming_session = None;
        self.rename_input.clear();
        self.rename_needs_focus = false;
    }

    /// Commit an inline rename. An empty name would leave a session that cannot
    /// be told apart from any other, so it keeps the previous one instead.
    pub(crate) fn commit_session_rename(&mut self) {
        let Some(session_id) = self.renaming_session.take() else {
            return;
        };
        let name = self.rename_input.trim().to_owned();
        self.rename_input.clear();
        self.rename_needs_focus = false;
        if name.is_empty() {
            self.notice =
                Some(tr("セッション名は空にできません。前の名前のままにしました。").into());
            return;
        }
        let previous_store = self.store.clone();
        let Some(session) = self
            .store
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if session.name == name {
            return;
        }
        session.name = name;
        if !self.persist() {
            self.store = previous_store;
        }
    }

    pub(crate) fn start_session(&mut self, session_id: Uuid) {
        if self
            .background_tasks
            .contains(&BackgroundKey::SessionStart(session_id))
            || self
                .background_tasks
                .contains(&BackgroundKey::EmptySessionStart(session_id))
        {
            return;
        }
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return;
        };
        if can_retry_session(&session.status) && session.native_session_id.is_some() {
            if tmux_observation(&session.tmux_name).state == TmuxState::Alive {
                self.notice = Some(
                    tr("古いターミナルがまだ動作しています。ネイティブ会話を再開する前に、停止してください。")
                        .into(),
                );
                return;
            }
            self.resume_managed_native_session(session_id);
            return;
        }
        if self.worktree_mutation_running(session.project_id) {
            self.notice = Some(if session.status == SessionStatus::Queued {
                tr("実行中の worktree 操作が完了するまで、準備できたセッションはキューに残ります。")
                    .into()
            } else {
                tr("worktree 操作の完了を待ってください。").into()
            });
            return;
        }
        let project_path = self
            .store
            .projects
            .iter()
            .find(|project| project.id == session.project_id)
            .map(|project| project.path.clone())
            .unwrap_or_default();
        let previous_status = session.status.clone();
        let previous_launched_at = session.launched_at;
        let retrying = can_retry_session(&previous_status);
        // Recorded before the launch rather than after it, so that a launch
        // this process never sees finish still leaves the moment its CLI
        // started. That moment is what identifies the conversation the CLI
        // creates for this terminal.
        let launched_at = now();
        if let Some(stored) = self
            .store
            .sessions
            .iter_mut()
            .find(|stored| stored.id == session_id)
        {
            stored.status = SessionStatus::Starting;
            stored.launched_at = Some(launched_at);
        }
        if !self.persist() {
            if let Some(stored) = self
                .store
                .sessions
                .iter_mut()
                .find(|stored| stored.id == session_id)
            {
                stored.status = previous_status;
                stored.launched_at = previous_launched_at;
            }
            return;
        }
        let auto_approve_workspace_prompts = self.store.auto_approve_workspace_prompts;
        // A launch token per launch, not per session: a retried pane replaces
        // the process that held the previous one, and the events still in
        // flight from that process must not land on the new turn.
        let hooks = self.mint_hook_environment(session_id);
        // Which login this session runs under. Checked before tmux is asked for
        // a window: a window that opens, fails on its first line, and dies
        // leaves a session record behind for a run that never started.
        let account = self.accounts.account_of_session(session_id);
        if let Err(reason) = account_ready_to_launch(&self.accounts, account) {
            if let Some(stored) = self
                .store
                .sessions
                .iter_mut()
                .find(|stored| stored.id == session_id)
            {
                stored.status = previous_status;
                stored.launched_at = previous_launched_at;
            }
            self.notice = Some(reason);
            return;
        }
        let account = account_variables(&self.accounts, account);
        if self
            .launch_progress
            .as_ref()
            .is_none_or(|p| p.session_id != session_id)
        {
            let project_name = self
                .store
                .projects
                .iter()
                .find(|p| p.id == session.project_id)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            self.launch_progress = Some(LaunchProgress {
                session_id,
                project_name,
                worktree_path: session
                    .worktree_path
                    .clone()
                    .unwrap_or_else(|| project_path.clone()),
                agent: session.agent.clone(),
                agent_command: session.agent_command.clone(),
                goal: session.goal.clone(),
                started_at: Instant::now(),
                failure: None,
                dismissed: false,
            });
        }
        self.spawn_background(BackgroundKey::SessionStart(session_id), move || {
            let result = launch_stored_session(
                &session,
                &project_path,
                retrying,
                auto_approve_workspace_prompts,
                hooks.as_ref(),
                &account,
            );
            BackgroundResult::SessionStarted { session_id, result }
        });
        self.notice = Some(if retrying {
            tr("セッションを再試行しています…").into()
        } else {
            tr("セッションを起動しています…").into()
        });
    }

    pub(crate) fn stop_session(&mut self, session_id: Uuid) {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return;
        };
        if session.status != SessionStatus::Active {
            self.notice = Some(
                tr("停止できるのは、実行中またはキャンセル保留中のセッションだけです。").into(),
            );
            return;
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::SessionStop(session_id))
        {
            return;
        }
        if !self.persist_cancellation_intent(session_id) {
            return;
        }
        let tmux_name = session.tmux_name;
        let observation_name = tmux_name.clone();
        self.spawn_background(BackgroundKey::SessionStop(session_id), move || {
            let mut command = tmux_command();
            command.args(["kill-session", "-t", &tmux_name]);
            let result = run_command_with_timeout(&mut command, Duration::from_secs(5))
                .map_err(|error| tf!("tmux を実行できませんでした: {error}", error = error))
                .and_then(|output| {
                    if output.status.success() {
                        Ok(())
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if tmux_error_state(&stderr) == TmuxState::Gone {
                            Ok(())
                        } else {
                            Err(stderr.trim().to_owned())
                        }
                    }
                });
            let result = match result {
                Err(_error)
                    if matches!(
                        tmux_observation(&observation_name).state,
                        TmuxState::Dead | TmuxState::Gone
                    ) =>
                {
                    Ok(())
                }
                result => result,
            };
            BackgroundResult::SessionStopped { session_id, result }
        });
        self.notice = Some(tr("セッションを停止しています…").into());
    }

    pub(crate) fn persist_cancellation_intent(&mut self, session_id: Uuid) -> bool {
        if self.cancellation_pending(session_id) {
            // A pending intent may have been reconstructed from directory
            // entries whose sync result was lost with a prior process. Never
            // trust the in-memory marker alone across restart: re-confirm one
            // durable copy on every Stop attempt before changing tmux.
            let previous_store = self.store.clone();
            return self.persist_cancellation_intent_copies(previous_store);
        }
        if !self
            .store
            .sessions
            .iter()
            .any(|session| session.id == session_id && session.status == SessionStatus::Active)
        {
            return false;
        }
        let previous_store = self.store.clone();
        self.store.pending_cancellations.push(session_id);
        self.persist_cancellation_intent_copies(previous_store)
    }

    pub(crate) fn persist_cancellation_intent_copies(&mut self, previous_store: Store) -> bool {
        // Write both independent recovery copies before touching tmux. A
        // confirmed directory sync for either copy is sufficient to recover
        // the intent after a crash; two unconfirmed renames are not.
        let sidecar_result =
            save_cancellation_intents(&self.data_file, &self.store.pending_cancellations);
        let store_result = save_store(&self.data_file, &self.store);
        self.apply_cancellation_intent_write_results(previous_store, sidecar_result, store_result)
    }

    pub(crate) fn apply_cancellation_intent_write_results(
        &mut self,
        previous_store: Store,
        sidecar_result: Result<WriteOutcome>,
        store_result: Result<WriteOutcome>,
    ) -> bool {
        let recovery_is_durable = matches!(&sidecar_result, Ok(WriteOutcome::Durable))
            || matches!(&store_result, Ok(WriteOutcome::Durable));
        let recovery_was_committed = matches!(
            &sidecar_result,
            Ok(WriteOutcome::Durable | WriteOutcome::CommittedButNotSynced(_))
        ) || matches!(
            &store_result,
            Ok(WriteOutcome::Durable | WriteOutcome::CommittedButNotSynced(_))
        );
        let write_details = [
            ("recovery sidecar", &sidecar_result),
            ("main index", &store_result),
        ]
        .into_iter()
        .filter_map(|(label, result)| match result {
            Ok(WriteOutcome::Durable) => None,
            Ok(WriteOutcome::CommittedButNotSynced(error)) => Some(tf!(
                "{label}: 永続化未確認（{error}）",
                error = error,
                label = label
            )),
            Err(error) => Some(format!("{label}: {error}")),
        })
        .collect::<Vec<_>>()
        .join("; ");

        if recovery_is_durable {
            if write_details.is_empty() {
                // Both current snapshots now have confirmed directory
                // entries, so a warning from an earlier uncertain attempt is
                // no longer actionable and must not linger during Stop.
                self.store_retry_pending = false;
                self.durability_warning = None;
            } else {
                self.store_retry_pending = true;
                self.durability_warning = Some(tf!("キャンセル指示の復旧コピーは 1 つ永続化されましたが、複製の修復が必要です。停止しても安全です。Operon が自動的に再試行します。{write_details}", write_details = write_details));
            }
            return true;
        }

        if recovery_was_committed {
            self.store_retry_pending = true;
            self.durability_warning = Some(tf!("キャンセル指示は書き込まれましたが、どちらの復旧コピーも永続化を確認できていません。Operon が自動的に再試行し、tmux は動作したままになります。{write_details}", write_details = write_details));
            self.notice = Some(
                tr("キャンセル復旧の永続化が未確認のため、tmux は変更していません。Operon を開いたままにし、警告が消えてからもう一度「停止」を選んでください。")
                    .into(),
            );
            return false;
        }

        self.store = previous_store;
        self.notice = Some(tf!(
            "キャンセル指示を保存できなかったため、tmux は変更していません: {write_details}",
            write_details = write_details
        ));
        false
    }

    pub(crate) fn close_completed_terminal(&mut self, session_id: Uuid) {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        else {
            return;
        };
        if !matches!(
            session.status,
            SessionStatus::Exited
                | SessionStatus::Failed
                | SessionStatus::Cancelled
                | SessionStatus::Lost
        ) {
            self.notice = Some(tr("閉じられるのは完了したセッションだけです。").into());
            return;
        }
        if self
            .background_tasks
            .contains(&BackgroundKey::TerminalClose(session_id))
        {
            return;
        }
        let stop_running_terminal = session.status == SessionStatus::Failed;
        let tmux_name = session.tmux_name;
        self.spawn_background(BackgroundKey::TerminalClose(session_id), move || {
            let result = match tmux_observation(&tmux_name).state {
                TmuxState::Alive if !stop_running_terminal => Err(tr(
                    "そのターミナルはまだ実行中です。代わりに「停止」を使ってください。",
                )
                .into()),
                TmuxState::Alive | TmuxState::Dead => {
                    close_tmux_session_confirming_gone(&tmux_name)
                }
                TmuxState::Gone => Ok(()),
                TmuxState::Unknown => {
                    Err(tr("ターミナルがまだ開いているか判定できませんでした。").into())
                }
            };
            BackgroundResult::TerminalClosed { session_id, result }
        });
        self.notice = Some(if stop_running_terminal {
            tr("古いターミナルを停止しています…").into()
        } else {
            tr("ターミナルを閉じています…").into()
        });
    }

    pub(crate) fn remove_session_record(&mut self, session_id: Uuid) {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if matches!(
            session.status,
            SessionStatus::Starting | SessionStatus::Active | SessionStatus::Queued
        ) {
            self.notice = Some(tr("先にこのセッションを停止または完了してください。").into());
            return;
        }
        let dependent_count = queued_dependency_count(&self.store.sessions, session_id);
        if dependent_count > 0 {
            self.notice = Some(tf!("このセッションは待機中の {dependent_count} 件から依存されています。先にそれらを開始または削除してください。", dependent_count = dependent_count));
            return;
        }
        let previous_store = self.store.clone();
        self.store
            .sessions
            .retain(|session| session.id != session_id);
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        self.session_output.remove(&session_id);
        self.terminal_layouts.remove(&session_id);
        self.terminal_cursors.remove(&session_id);
        self.terminal_preedits.remove(&session_id);
        self.session_prompt_turns.remove(&session_id);
        self.remove_after_close.remove(&session_id);
        self.unread_sessions.remove(&session_id);
        self.hook_status.remove(&session_id);
        self.hook_launch_tokens.remove(&session_id);
        if self.unread_hold == Some(session_id) {
            self.unread_hold = None;
        }
        if self.selected_session == Some(session_id) {
            self.selected_session = None;
            self.resolved_paths.clear();
            self.resolved_paths_generation += 1;
            self.unresolved_paths.clear();
            self.resolved_paths_session = None;
        }
        if self.terminal_path_menu.as_ref().map(|(id, _)| *id) == Some(session_id) {
            self.terminal_path_menu = None;
        }
        self.notice =
            Some(tr("セッションの記録を削除しました。ファイルと worktree は残っています。").into());
    }

    pub(crate) fn refresh_session_output(&mut self, session_id: Uuid) {
        self.request_session_output(session_id);
    }

    pub(crate) fn open_session_in_terminal(&mut self, session_id: Uuid) {
        if !self
            .store
            .sessions
            .iter()
            .any(|session| session.id == session_id)
        {
            return;
        }
        self.select_session(session_id);
        self.page = Page::Sessions;
        self.session_library_open = false;
        // Fetch immediately instead of making the newly selected terminal wait
        // for the periodic two-second session poll.
        self.refresh_session_output(session_id);
    }

    /// The one place the selected session changes.
    ///
    /// Reading a session is what clears its mark, and "reading" is exactly this
    /// call — so a second site assigning the field directly would leave a
    /// session bold after it had been read, which is the shape of bug this
    /// entry point exists to prevent.
    pub(crate) fn select_session(&mut self, session_id: Uuid) {
        if self.unread_hold.is_some_and(|held| held != session_id) {
            self.unread_hold = None;
        }
        if self.selected_session != Some(session_id) {
            self.resolved_paths.clear();
            self.resolved_paths_generation += 1;
            self.unresolved_paths.clear();
            self.resolved_paths_session = Some(session_id);
        }
        self.selected_session = Some(session_id);
        self.refresh_checkpoints(session_id);
        if self.unread_hold != Some(session_id) {
            self.unread_sessions.remove(&session_id);
        }
    }

    /// Marks a session unread by hand, and holds it there until the person
    /// looks at something else.
    pub(crate) fn mark_session_unread(&mut self, session_id: Uuid) {
        self.unread_sessions.insert(session_id);
        self.unread_hold = Some(session_id);
    }

    pub(crate) fn is_session_unread(&self, session_id: Uuid) -> bool {
        self.unread_sessions.contains(&session_id)
    }

    /// Whether a session is the one a person is actually looking at. Anything
    /// else — another page, the session library over the top, another session
    /// selected — is not reading it.
    pub(crate) fn is_session_on_screen(&self, session_id: Uuid) -> bool {
        self.page == Page::Sessions
            && !self.session_library_open
            && self.selected_session == Some(session_id)
    }

    /// Both paths that decide a session changed state call this, at the point
    /// each already decided the change was worth a notification. A change a
    /// person watched happen is not unread.
    pub(crate) fn note_session_activity_change(&mut self, session_id: Uuid) {
        if !self.is_session_on_screen(session_id) {
            self.unread_sessions.insert(session_id);
        }
    }

    pub(crate) fn request_session_output(&mut self, session_id: Uuid) {
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if self
            .background_tasks
            .contains(&BackgroundKey::SessionOutput(session_id))
        {
            return;
        }
        let tmux_name = session.tmux_name.clone();
        self.spawn_background(BackgroundKey::SessionOutput(session_id), move || {
            BackgroundResult::SessionOutput {
                session_id,
                result: tmux_capture_with_cursor(&tmux_name).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn set_session_output(
        &mut self,
        session_id: Uuid,
        output: String,
        cursor: Option<(u16, u16)>,
    ) {
        // ANSI parsing is noticeably more expensive than drawing a button.
        // Parse only when tmux has supplied fresh output, rather than once per
        // egui repaint while the user interacts with the terminal.
        let palette = self.store.theme.palette();
        self.terminal_layouts
            .insert(session_id, terminal_line_layouts(&output, palette));
        self.session_output.insert(session_id, output);
        if let Some(cursor) = cursor {
            self.terminal_cursors.insert(session_id, cursor);
        }
        // The buffer moved, so where the query occurs moved with it. Here
        // rather than per frame: this walks every line, and the lines only
        // change when tmux has said something new.
        self.refresh_terminal_search(session_id);
        if self.resolved_paths_session == Some(session_id) {
            self.resolved_paths.retain(|_, resolved| resolved.is_some());
            self.resolved_paths_generation += 1;
        }
    }

    pub(crate) fn queue_terminal_input(&mut self, session_id: Uuid, input: Vec<TerminalInput>) {
        if input.is_empty() {
            return;
        }
        let Some(session) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        if session.status != SessionStatus::Active {
            return;
        }
        self.terminal_input_queue
            .entry(session_id)
            .or_default()
            .extend(input);
        self.flush_terminal_input(session_id);
    }

    pub(crate) fn flush_terminal_input(&mut self, session_id: Uuid) {
        if self
            .background_tasks
            .contains(&BackgroundKey::TerminalInput(session_id))
        {
            return;
        }
        let input = self
            .terminal_input_queue
            .get_mut(&session_id)
            .map(|queue| queue.drain(..).collect::<Vec<_>>())
            .unwrap_or_default();
        if input.is_empty() {
            return;
        }
        let Some(tmux_name) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id && session.status == SessionStatus::Active)
            .map(|session| session.tmux_name.clone())
        else {
            return;
        };
        self.spawn_background(BackgroundKey::TerminalInput(session_id), move || {
            BackgroundResult::TerminalInputSent {
                session_id,
                result: tmux_send_input(&tmux_name, &input).map_err(|error| error.to_string()),
            }
        });
    }

    pub(crate) fn request_terminal_resize(&mut self, session_id: Uuid, size: (u16, u16)) {
        if self.terminal_resize_requested.get(&session_id) == Some(&size)
            && self.terminal_resize_applied.get(&session_id) == Some(&size)
        {
            return;
        }
        self.terminal_resize_requested.insert(session_id, size);
        self.flush_terminal_resize(session_id);
    }

    pub(crate) fn flush_terminal_resize(&mut self, session_id: Uuid) {
        if self
            .background_tasks
            .contains(&BackgroundKey::TerminalResize(session_id))
        {
            return;
        }
        let Some(size) = self.terminal_resize_requested.get(&session_id).copied() else {
            return;
        };
        if self.terminal_resize_applied.get(&session_id) == Some(&size) {
            return;
        }
        let Some(tmux_name) = self
            .store
            .sessions
            .iter()
            .find(|session| session.id == session_id && session.status == SessionStatus::Active)
            .map(|session| session.tmux_name.clone())
        else {
            return;
        };
        self.spawn_background(BackgroundKey::TerminalResize(session_id), move || {
            BackgroundResult::TerminalResized {
                session_id,
                size,
                result: tmux_resize(&tmux_name, size).map_err(|error| error.to_string()),
            }
        });
    }
}

impl eframe::App for OperonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_appearance(ctx);
        let palette = self.store.theme.palette();
        self.ensure_hooks_started(ctx);
        self.drain_hook_events();
        self.persist_diff_comments();
        self.request_port_scan();
        self.handle_terminal_search_keys(ctx);
        self.resolve_terminal_paths();
        self.process_background_results();
        self.start_ready_queued_sessions();
        self.handle_app_shortcuts(ctx);
        if self.command_palette_open && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.close_command_palette();
        }
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        if !dropped.is_empty() {
            self.add_dropped_folders(dropped);
        }
        if self.last_session_refresh.elapsed() >= Duration::from_secs(2) {
            self.retry_pending_store();
            self.request_session_poll();
            self.last_session_refresh = Instant::now();
        }
        ctx.request_repaint_after(if self.background_tasks.is_empty() {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(100)
        });
        // A toast leaves on time even when nothing else would wake the frame.
        if let Some(remaining) = self.expire_brief_notice(Instant::now()) {
            ctx.request_repaint_after(remaining);
        }
        self.ui_topbar(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 10.0);
            // Two screens hold something that is as wide as it is and take the
            // whole window edge to edge: the session workspace, which is a
            // terminal, and the editor, which is a file tree beside a page of
            // code. Every other page is inset by a gutter and takes the rest.
            let full_width = self.page == Page::Sessions
                || (self.page == Page::Projects
                    && self.project_tab == ProjectTab::Files
                    // The tab a person left the last project on is remembered,
                    // so this has to check that there is a project open —
                    // otherwise the project list itself would be drawn at full
                    // width and, worse, outside the scroll area it needs.
                    && self.selected_project.is_some());
            let content_width = if full_width {
                ui.available_width()
            } else {
                page_column_width(ui.available_width())
            };
            let content_height = ui.available_height();
            let side_space = (ui.available_width() - content_width) / 2.0;
            ui.horizontal(|ui| {
                ui.add_space(side_space);
                ui.allocate_ui_with_layout(
                    egui::vec2(content_width, content_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        // Two banners, and they answer to different rules. The
                        // durability warning is about data on disk and stays
                        // until the retry clears it, so it has no close button
                        // and wears the warning edge. An ordinary notice is the
                        // result of something the person just did, and closes.
                        if let Some(warning) = self.durability_warning.clone() {
                            banner(ui, palette, ICON_ATTENTION, palette.warning, |ui| {
                                ui.label(RichText::new(warning).size(13.0).color(palette.text));
                            });
                            ui.add_space(SPACE_SM);
                        }
                        // A brief notice is a toast, drawn over the window's
                        // corner after the page, and takes no row here.
                        let brief = notice_is_brief(&self.notice, &self.brief_notice);
                        if let Some(notice) = self.notice.clone().filter(|_| !brief) {
                            let mut dismissed = false;
                            banner(ui, palette, ICON_NOTICE, palette.info, |ui| {
                                ui.label(RichText::new(notice).size(13.0).color(palette.text));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if small_icon_button(ui, ICON_CLOSE, tr("この通知を閉じる"))
                                            .clicked()
                                        {
                                            dismissed = true;
                                        }
                                    },
                                );
                            });
                            if dismissed {
                                self.notice = None;
                            }
                            ui.add_space(SPACE_SM);
                        }
                        // The two full-width screens also scroll themselves —
                        // the terminal sticks to the bottom of its own buffer,
                        // and the editor scrolls its tree and its text
                        // separately. Wrapping either in the page's scroll
                        // area would give it an unbounded height and take its
                        // scrolling away.
                        if full_width {
                            match self.page {
                                Page::Sessions => self.ui_terminal_workspace(ui),
                                _ => self.ui_projects(ui),
                            }
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    match self.page {
                                        Page::Home => self.ui_home(ui),
                                        Page::Projects => self.ui_projects(ui),
                                        Page::Sessions => self.ui_terminal_workspace(ui),
                                        Page::Settings => self.ui_settings(ui),
                                    }
                                });
                        }
                    },
                );
            });
        });
        if ctx.input(|input| !input.raw.hovered_files.is_empty()) {
            let rect = ctx.screen_rect().shrink(14.0);
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("folder-drop-overlay"),
            ));
            painter.rect_filled(rect, 8.0, palette.scrim);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                tr("フォルダをドロップしてプロジェクトを追加"),
                egui::FontId::proportional(24.0),
                palette.accent_soft,
            );
        }
        self.ui_brief_notice_toast(ctx, palette);
        self.ui_command_palette(ctx);
        self.ui_launch_sheet(ctx);
        self.ui_restore_modal(ctx);
        self.ui_launch_modal(ctx);
        self.ui_land_worktree_modal(ctx, palette);
        self.ui_create_pr_modal(ctx, palette);
        self.ui_run_script_modal(ctx, palette);
    }
}

impl OperonApp {
    /// The brief notice, in the window's bottom-right corner. Its ✕ clears the
    /// notice and the record that made it brief together, so neither outlives
    /// the other.
    pub(crate) fn ui_brief_notice_toast(&mut self, ctx: &egui::Context, palette: &Palette) {
        if !notice_is_brief(&self.notice, &self.brief_notice) {
            return;
        }
        let Some(text) = self.notice.clone() else {
            return;
        };
        let mut dismissed = false;
        egui::Area::new(egui::Id::new("brief-notice-toast"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-SPACE_LG, -SPACE_LG))
            .interactable(true)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(palette.raised)
                    .stroke(egui::Stroke::new(1.0, palette.border_subtle))
                    .corner_radius(egui::CornerRadius::same(RADIUS_CARD))
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .shadow(ctx.style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_max_width(420.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(ICON_STATUS_DONE)
                                    .size(14.0)
                                    .color(palette.success),
                            );
                            ui.label(RichText::new(text).size(13.0).color(palette.text));
                            if small_icon_button(ui, ICON_CLOSE, tr("この通知を閉じる")).clicked()
                            {
                                dismissed = true;
                            }
                        });
                    });
            });
        if dismissed {
            self.notice = None;
            self.brief_notice = None;
        }
    }
}

pub(crate) fn initial_page(store: &Store) -> Page {
    if store.projects.is_empty() {
        Page::Projects
    } else {
        Page::Home
    }
}

/// What a session's one state verb does when its button is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionVerbAction {
    Stop,
    Start,
    ResumeNative,
}

/// The one verb a session's state offers, shared by every surface that acts on
/// a session. The verb sits at the same end of its row everywhere and carries
/// its word outright, so the action that changes a session's state is found by
/// place and read by name instead of decoded from a glyph per surface.
pub(crate) struct SessionVerb {
    pub(crate) icon: &'static str,
    pub(crate) label: &'static str,
    pub(crate) hint: String,
    pub(crate) danger: bool,
    pub(crate) action: SessionVerbAction,
}

/// Whether the notice on screen is the brief one. Compared by text, so a
/// notice that replaced it — a failure, most importantly — is not.
pub(crate) fn notice_is_brief(notice: &Option<String>, brief: &Option<(String, Instant)>) -> bool {
    matches!((notice, brief), (Some(shown), Some((text, _))) if shown == text)
}

/// Whether the session header's verb keeps its red hover wash. Only the stop of
/// a running session whose agent is idle gives it up: nothing is being
/// interrupted. 「停止中…」 and a resume that carries a dangerous launch flag
/// keep theirs, because their red is the warning.
pub(crate) fn verb_is_loud(
    verb: &SessionVerb,
    status: &SessionStatus,
    cancellation_pending: bool,
    activity: Option<AgentActivity>,
) -> bool {
    let idle_stop = verb.action == SessionVerbAction::Stop
        && *status == SessionStatus::Active
        && !cancellation_pending
        && activity == Some(AgentActivity::Idle);
    verb.danger && !idle_stop
}

pub(crate) fn session_verb(session: &Session, cancellation_pending: bool) -> Option<SessionVerb> {
    // A stop that has not landed yet stays clickable: the pending cancellation
    // is retried by pressing it again, so the verb keeps its word and says what
    // a second press does instead of going dark while the terminal winds down.
    if cancellation_pending {
        return Some(SessionVerb {
            icon: ICON_STOP,
            label: tr("停止中…"),
            hint: tr("ターミナルの終了待ち。もう一度選ぶと停止を再試行します。").into(),
            danger: true,
            action: SessionVerbAction::Stop,
        });
    }
    let native = session.native_session_id.is_some();
    match session.status {
        SessionStatus::Active => Some(SessionVerb {
            icon: ICON_STOP,
            label: tr("停止"),
            hint: tr("実行中の CLI を停止します（記録は残ります）").into(),
            danger: true,
            action: SessionVerbAction::Stop,
        }),
        // A queued session keeps its force-start: the chip above says whether
        // it is READY, QUEUED, or BLOCKED, and the verb stays the same either
        // way, because the slot never moves.
        SessionStatus::Queued => Some(SessionVerb {
            icon: ICON_START,
            label: tr("今すぐ開始"),
            hint: tr("待機中のセッションを今開始します").into(),
            danger: false,
            action: SessionVerbAction::Start,
        }),
        SessionStatus::Failed | SessionStatus::Lost => Some(if native {
            resume_verb(session)
        } else {
            SessionVerb {
                icon: ICON_START,
                label: tr("再試行"),
                hint: tr("このセッションをもう一度実行します").into(),
                danger: false,
                action: SessionVerbAction::Start,
            }
        }),
        SessionStatus::Exited | SessionStatus::Cancelled if native => Some(resume_verb(session)),
        _ => None,
    }
}

/// The verb that reopens a conversation, and the sentence that says what it
/// will carry with it.
///
/// The options are named rather than left silent because one of them can be a
/// choice the launch screen made the person acknowledge in writing: an agent
/// that was allowed to work without asking goes on doing that, and a person who
/// cannot see that from the button they are about to press has not agreed to it
/// a second time so much as not been told.
///
/// This runs in a draw path, once per session row per frame. Both calls below
/// walk one short validated line: `carried_launch_option_labels` builds the
/// labels it returns, and `carried_launch_is_dangerous` puts the carried tokens
/// through the same gate the launch screen uses, which allocates a handful of
/// small `Vec`s of borrowed strings — the command's tokens, and one per
/// dangerous table entry it compares against.
///
/// Measured against what this function already does rather than against zero:
/// it builds a `String` for the hint and a `Vec<String>` of labels every frame
/// regardless. No `Command::new`, no `fs`, no parse. The earlier version of this
/// comment said "no allocation beyond that hint", which stopped being true when
/// the danger question started reading the command instead of one flag table.
fn resume_verb(session: &Session) -> SessionVerb {
    let mut hint = tr("保存された会話 ID からこのセッションを再開します").to_owned();
    let carried = carried_launch_option_labels(&session.agent, &session.agent_command);
    if !carried.is_empty() {
        hint.push('\n');
        hint.push_str(&tf!(
            "引き継ぐ設定: {options}",
            options = carried.join(tr("、"))
        ));
    }
    SessionVerb {
        icon: ICON_RESUME,
        label: tr("会話 ID から再開"),
        hint,
        danger: carried_launch_is_dangerous(&session.agent, &session.agent_command),
        action: SessionVerbAction::ResumeNative,
    }
}
pub(crate) fn default_agent(tools: &ToolStatus) -> &'static str {
    if tools.codex {
        "codex"
    } else if tools.claude {
        "claude"
    } else if tools.antigravity {
        "gemini"
    } else {
        "custom"
    }
}
pub(crate) fn resolve_session_path(project_path: &Path, input: &str) -> UiResult<PathBuf> {
    let candidate = if input.trim().is_empty() {
        project_path.to_path_buf()
    } else {
        PathBuf::from(input.trim())
    };
    if !candidate.is_dir() {
        return Err(tr("選択した作業ディレクトリが存在しません。").into());
    }
    Ok(candidate.canonicalize().unwrap_or(candidate))
}
pub(crate) fn resolve_selected_session_path(
    project_id: Uuid,
    project_path: &Path,
    selected_for_project: Option<Uuid>,
    input: &str,
) -> UiResult<PathBuf> {
    if !input.trim().is_empty() && selected_for_project != Some(project_id) {
        return Err(
            tr("その作業ディレクトリは別のプロジェクト用に選択されています。worktree を選び直してください。")
                .into(),
        );
    }
    resolve_session_path(project_path, input)
}
/// Whether a tool can be launched at all, resolved the way a shell resolves a
/// command name. Availability is a filesystem question, so it is answered
/// without running anything: an earlier check spawned `--version` under a
/// fixed deadline, which raced startup time rather than installation and
/// reported Node-based CLIs that need seconds to boot as missing.
/// Follows symlinks, so a link left behind by an uninstalled tool is not
/// mistaken for the tool itself.
pub(crate) fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

/// A file open in the editor.
///
/// `disk` is the whole point of this struct. Agents in this app write to the
/// same working tree a person is reading, so "what I loaded" and "what is there
/// now" are routinely different, and a save that did not check would delete an
/// agent's work without saying so. It is also what the unsaved-changes mark and
/// the editor's own diff are computed from.
/// Image preview data held by an open image document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageDocumentData {
    pub(crate) bytes: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct OpenDocument {
    pub(crate) project: Uuid,
    /// What `path` is relative to.
    ///
    /// The project folder for a file browsed from the tree, and the session's
    /// worktree for one followed from a terminal link. A worktree sits beside
    /// the project rather than inside it, so before this existed a link into
    /// one could be underlined and never opened.
    pub(crate) root: PathBuf,
    /// Relative to `root`, never to the project.
    pub(crate) path: PathBuf,
    /// Which branch this copy is from, for a document that is not the
    /// project's own. Decided once, when the document is opened: the tab row
    /// draws every frame and had no business walking the session list there.
    pub(crate) branch: Option<String>,
    /// Whether this document is the project's own file. Decided at the same
    /// moment and from the same canonical pair as `branch`, so that no drawing
    /// code re-derives it against a spelling of `project.path` that may not
    /// match the root.
    ///
    /// It cannot go stale. Nothing rewrites `Project::path` in place — adding a
    /// project at a different location pushes a new `Project` with a new id, so
    /// a document opened under the old one is filtered out by
    /// `document.project == project.id` before this is ever read. If somebody
    /// later writes a "move this project" action that edits the path, this is
    /// the field it breaks.
    pub(crate) project_rooted: bool,
    /// What the file held when it was opened or last saved.
    pub(crate) disk: String,
    /// What the person has typed. Equal to `disk` until they type.
    pub(crate) buffer: String,
    /// Why this file cannot be edited, where it cannot.
    pub(crate) read_only: Option<String>,
    /// A failed save, kept next to the document rather than in the one-line
    /// notice at the top of the window, where the next unrelated message would
    /// wipe it out before it had been read.
    pub(crate) save_error: Option<String>,
    /// Something outside this editor wrote the file while there were unsaved
    /// edits in the buffer. Only ever set in that case: with nothing to lose,
    /// the document adopts the new bytes instead of raising this, because a
    /// question whose only sensible answer is yes teaches people to dismiss
    /// questions.
    pub(crate) external_change: bool,
    /// Decoded image data when the open document is a viewable image format.
    pub(crate) image_data: Option<ImageDocumentData>,
}

/// What an agent CLI or terminal launch is doing while it prepares in the background.
/// Present on screen as `ui_launch_modal` until the launch finishes, errors, or is
/// dismissed to run in the background.
#[derive(Debug, Clone)]
pub(crate) struct LaunchProgress {
    pub(crate) session_id: Uuid,
    pub(crate) project_name: String,
    pub(crate) worktree_path: PathBuf,
    pub(crate) agent: String,
    pub(crate) agent_command: String,
    pub(crate) goal: String,
    pub(crate) started_at: Instant,
    pub(crate) failure: Option<String>,
    pub(crate) dismissed: bool,
}

pub(crate) fn launch_progress_title(progress: &LaunchProgress) -> String {
    if progress.failure.is_some() {
        tr("セッションの起動に失敗しました").into()
    } else if progress.agent == "terminal" {
        tr("ターミナルを起動しています…").into()
    } else {
        tf!(
            "{agent} セッションを起動しています…",
            agent = agent_choice_copy(&progress.agent).0
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    Home,
    Projects,
    Sessions,
    Settings,
}

impl OpenDocument {
    fn modified(&self) -> bool {
        self.read_only.is_none() && self.buffer != self.disk
    }

    /// Where a root meets a path.
    ///
    /// Written once because getting it wrong writes a buffer into the wrong
    /// copy of a file — the project's `src/app.rs` instead of the worktree's,
    /// which have equal relative paths and different contents.
    ///
    /// `no_source_file_joins_a_project_path_to_a_document_path` catches a
    /// project root `join`ed to a document path, on one line or split across
    /// several by rustfmt. It does **not** catch a root taken into a variable
    /// first, and it does not see a root and a path handed to the same function
    /// without a `join` — which is how the diff view came to read the project's
    /// file for a worktree document, found in review rather than by this test.
    /// The claim is what it checks, not "the only place": a guard described as
    /// wider than it is, is a guard people stop reading.
    pub(crate) fn absolute(&self) -> PathBuf {
        self.root.join(&self.path)
    }

    /// How this document is named in a sentence a person reads. The relative
    /// path for the project's own file, and the branch beside it for a copy
    /// from a worktree — otherwise 「src/app.rs を保存しました。」 is the same
    /// sentence for two different files, which is the confusion this whole
    /// change exists to remove.
    pub(crate) fn named(&self) -> String {
        // `branch` alone, not `branch` and `!project_rooted`: the two are
        // decided together in `open_document_at` and a project-rooted document
        // never carries one. Asking both would be the same question written
        // twice, and the second copy is the one that goes wrong later.
        match &self.branch {
            Some(branch) => tf!("{p0} · {p1}", p0 = branch.clone(), p1 = self.path.display()),
            None => self.path.display().to_string(),
        }
    }

    pub(crate) fn id(&self) -> DocumentId {
        DocumentId {
            project: self.project,
            root: self.root.clone(),
            path: self.path.clone(),
        }
    }
}

/// Which root a followed path belongs to, and its path relative to that root.
///
/// The project folder when the file is under it, and the session's worktree
/// when it is under that. A worktree sits *beside* the project rather than
/// inside it, so before this existed the underline condition and the open
/// condition named two sets that could never both hold, and a link in a
/// worktree session underlined and never opened.
///
/// `None` for a file under neither. That is not a refusal on principle — there
/// is simply no root to address it from, which is why the answer stayed a
/// refusal rather than becoming a forced open at a name the editor would
/// resolve somewhere else.
pub(crate) fn document_root_for(
    resolved: &Path,
    project: &Path,
    worktree: Option<&Path>,
) -> Option<(PathBuf, PathBuf)> {
    // The innermost root that holds the file, not the first one tried. A
    // worktree made under the project folder — which is what `.worktrees/`
    // means — is inside the project, so asking the project first attributed the
    // worktree's own files to the project: no branch on the tab, a Diff view
    // offered that reads the wrong tree, and a file the agent had just written
    // reporting itself unchanged because the project's `git diff` says nothing
    // about a path it ignores.
    //
    // Deepest match wins is the rule `src/git/ports.rs` already uses to decide
    // which worktree a listening socket belongs to, for the same reason: nesting
    // is what makes the shallow answer wrong rather than merely different.
    [Some(project.to_path_buf()), worktree.map(Path::to_path_buf)]
        .into_iter()
        .flatten()
        .filter_map(|candidate| {
            let root = candidate
                .canonicalize()
                .unwrap_or_else(|_| candidate.clone());
            let relative = resolved.strip_prefix(&root).ok()?;
            Some((root, relative.to_path_buf()))
        })
        .max_by_key(|(root, _)| root.components().count())
}

/// Which document. The root is part of the identity because two files with the
/// same relative path under different roots are two files: a worktree's
/// `src/app.rs` is not the project's, and a key that could not tell them apart
/// would show one tab for both and save one buffer over whichever the path
/// happened to resolve to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct DocumentId {
    pub(crate) project: Uuid,
    pub(crate) root: PathBuf,
    pub(crate) path: PathBuf,
}

impl DocumentId {
    /// The full path, for the hover that says exactly which file a tab holds.
    /// The same join as `OpenDocument::absolute`, and the only other one — both
    /// live here so the guard can name where a root may meet a path.
    pub(crate) fn absolute_display(&self) -> String {
        self.root.join(&self.path).display().to_string()
    }
}

/// The exact combination somebody ticked 「理解したうえで起動する」 for. Keying
/// the consent to what was consented to is what keeps it from going stale: any
/// edit to the mode or the switches produces a different combination, and the
/// tick is simply no longer for this one. The bool this replaced had to be
/// cleared by hand at every edit site, and the next edit path that forgot would
/// have carried one launch's consent into another's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LaunchAcknowledgement {
    pub(crate) agent: String,
    pub(crate) mode: String,
    /// Sorted, so that ticking two switches in the other order is still the
    /// combination that was agreed to.
    pub(crate) flags: Vec<String>,
    /// The typed line, for the one agent whose whole launch is one. Without it,
    /// consent given for `claude --dangerously-skip-permissions` would still be
    /// held after the line was edited to run something else entirely: the other
    /// three fields do not move when a custom command is retyped.
    pub(crate) custom_command: String,
}

fn git_branch(path: &Path) -> Result<String> {
    git_output(path, &["rev-parse", "--abbrev-ref", "HEAD"]).map(|branch| branch.trim().to_owned())
}

impl GitView {
    pub(crate) fn all() -> [Self; 3] {
        [Self::Changes, Self::Diff, Self::History]
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Changes => "Changes",
            Self::Diff => "Diff",
            Self::History => "History",
        }
    }
}

/// The settings page's five sections, in the order its left list draws them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsSection {
    Appearance,
    Agents,
    Notifications,
    Data,
    Keys,
}

impl SettingsSection {
    pub(crate) fn all() -> [Self; 5] {
        [
            Self::Appearance,
            Self::Agents,
            Self::Notifications,
            Self::Data,
            Self::Keys,
        ]
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Appearance => tr("外観"),
            Self::Agents => tr("エージェント"),
            Self::Notifications => tr("通知"),
            Self::Data => tr("データと復旧"),
            Self::Keys => tr("キー割り当て"),
        }
    }
    /// The one line under the section's title: what a person will find there.
    pub(crate) fn summary(self) -> &'static str {
        match self {
            Self::Appearance => tr("テーマ、フォント、表示言語"),
            Self::Agents => tr("起動に必要なツールと、状態の検知方法"),
            Self::Notifications => tr("セッションの状態が変わったときの通知"),
            Self::Data => tr("この Mac に保存されるデータと、失われた端末の復旧"),
            Self::Keys => tr("ショートカットの一覧。変更はファイルで行います"),
        }
    }
}

impl OperonApp {
    /// Settings, at one section. The places that send a person here because a
    /// tool is missing name エージェント, so they land on the fix rather than on
    /// whichever section was open last.
    pub(crate) fn open_settings(&mut self, section: SettingsSection) {
        self.page = Page::Settings;
        self.settings_section = section;
    }
}

impl ProjectTab {
    pub(crate) fn all() -> [Self; 6] {
        [
            Self::Overview,
            Self::Git,
            Self::Worktrees,
            Self::PullRequests,
            Self::Files,
            Self::AgentSettings,
        ]
    }
    /// What the tab holds, not the tool behind it: 「変更」 rather than
    /// "Git", 「ファイル」 rather than the editor that is one of its modes.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Overview => tr("概要"),
            Self::Git => tr("変更"),
            Self::Worktrees => "worktree",
            Self::PullRequests => "PR",
            Self::Files => tr("ファイル"),
            Self::AgentSettings => tr("エージェント設定"),
        }
    }
}

impl OperonApp {
    /// The number a project tab carries, from what its tab last loaded — or
    /// `None` when nothing has loaded yet or the load failed. Read every frame
    /// by the tab row, so it only looks things up: it asks for nothing, and a
    /// count appears once 概要 or the tab itself has loaded it.
    pub(crate) fn project_tab_count(&self, tab: ProjectTab, project_id: Uuid) -> Option<usize> {
        match tab {
            ProjectTab::Git => match self.git_changes_cache.get(&project_id) {
                Some(Ok(snapshot)) => Some(snapshot.files.len()),
                _ => None,
            },
            ProjectTab::Worktrees => match self.worktree_cache.get(&project_id) {
                Some(Ok(worktrees)) => Some(worktrees.len()),
                _ => None,
            },
            ProjectTab::PullRequests => match self.pull_request_cache.get(&project_id) {
                Some(Ok(requests)) => Some(requests.len()),
                _ => None,
            },
            ProjectTab::Overview | ProjectTab::Files | ProjectTab::AgentSettings => None,
        }
    }
}

impl OperonApp {
    /// The choices made inside 「3. 詳しい設定（任意）」, in the order that
    /// section draws them, each already translated.
    ///
    /// Only what differs from the default. A header that listed its defaults
    /// would say the same thing on every screen and stop being read, and the
    /// question it answers is "what did I change", not "what is there".
    ///
    /// One function because the header and the guard have to be the same
    /// answer. A summary the header builds and a test rebuilds is two answers
    /// to one question, and the second copy is the one that goes wrong later —
    /// lesson 005.
    ///
    /// Called once per frame on the launch screen. Every branch is a `is_empty`
    /// on a field already in memory, and the strings it builds are the ones
    /// being drawn.
    pub(crate) fn launch_detail_summary(&self) -> Vec<String> {
        let named = |label: &str, value: &str| format!("{label} {value}");
        let mut chosen = Vec::new();
        if !self.session_name_input.trim().is_empty() {
            chosen.push(named(tr("セッション名"), self.session_name_input.trim()));
        }
        if !self.agent_model_input.trim().is_empty() {
            chosen.push(named(tr("モデル"), self.agent_model_input.trim()));
        }
        if !self.agent_mode_input.is_empty() {
            chosen.push(named(
                agent_mode_label(&self.selected_agent),
                &self.agent_mode_input,
            ));
        }
        if !self.agent_effort_input.is_empty() {
            chosen.push(named(tr("推論の深さ"), &self.agent_effort_input));
        }
        // Catalogue order rather than click order, the same reason
        // `resolve_agent_flags` walks the table: a summary that reorders itself
        // between frames is one nobody can compare against what ran.
        for flag in agent_flag_options(&self.selected_agent) {
            if self.agent_flag_inputs.iter().any(|id| id == flag.id) {
                chosen.push(tr(flag.label).to_owned());
            }
        }
        if !self.session_path_input.trim().is_empty() {
            chosen.push(named(
                tr("作業フォルダ"),
                &workspace_display_name(Path::new(self.session_path_input.trim())),
            ));
        }
        if self.depends_on_input.is_some() {
            chosen.push(tr("ほかのセッションの後に開始").to_owned());
        }
        chosen
    }

    /// The folded section's own heading, carrying what is set inside it.
    ///
    /// Composed here rather than at the call site so that the header and its
    /// guard are the same answer: the cap and the trailing count are what
    /// "a folded section never hides a decision" actually claims, and a claim
    /// checked against a second copy of itself is not checked.
    pub(crate) fn launch_detail_title(&self) -> String {
        let heading = tr("詳しい設定（任意）");
        let summary = self.launch_detail_summary();
        if summary.is_empty() {
            return heading.to_owned();
        }
        let shown = summary
            .iter()
            .take(LAUNCH_DETAIL_SUMMARY_MAX)
            .cloned()
            .collect::<Vec<_>>()
            .join(" · ");
        let rest = summary.len().saturating_sub(LAUNCH_DETAIL_SUMMARY_MAX);
        if rest == 0 {
            format!("{heading} · {shown}")
        } else {
            format!("{heading} · {shown} · {}", tf!("ほか {p0} 件", p0 = rest))
        }
    }

    /// Whether this frame saw the chord bound to an action. An action with no
    /// chord is never pressed, which is what `null` in the keymap file buys.
    /// The app-wide chords, read once per frame before anything draws.
    pub(crate) fn handle_app_shortcuts(&mut self, ctx: &egui::Context) {
        // The launch sheet holds a request being typed. ⌘K opens a palette
        // that can switch project, and ⌘2 clears the project outright: either
        // would throw the request away, so neither is live while it is open.
        if !self.launch_sheet_open && self.chord_pressed(ctx, "palette.open") {
            self.open_command_palette();
        }
        // Before the window draws, so the selection it paints is the one the
        // arrows just moved — and so an arrow never reaches the text field and
        // moves the caret instead.
        self.handle_palette_keys(ctx);
        // Adding a project selects it, which clears the sheet's request too.
        if !self.launch_sheet_open && self.chord_pressed(ctx, "project.add") {
            self.choose_project_folder();
        }
        // The shortcuts a person brings with them from every other Mac app of
        // this shape: ⌘1–3 for the three places, ⌘, for settings, unless a
        // keymap file says otherwise. They are deliberately not live while a
        // terminal has focus — a TUI owns its own keys, and `⌘` is the one
        // modifier macOS keeps for the app.
        if !self.command_palette_open && !self.launch_sheet_open {
            if self.chord_pressed(ctx, "session.new") {
                self.open_new_session();
            }
            for (action, page) in [
                ("page.home", Page::Home),
                ("page.projects", Page::Projects),
                ("page.sessions", Page::Sessions),
                ("page.settings", Page::Settings),
            ] {
                if self.chord_pressed(ctx, action) {
                    if page == Page::Projects {
                        self.select_project(None);
                    }
                    self.page = page;
                }
            }
        }
    }

    pub(crate) fn chord_pressed(&self, ctx: &egui::Context, action: &str) -> bool {
        let Some(chord) = self.keymap.chord_for(action) else {
            return false;
        };
        ctx.input(|input| input.modifiers.matches_logically(chord.modifiers))
            && ctx.input(|input| input.key_pressed(chord.key))
    }

    /// The same question, for a chord that must not also reach whatever has
    /// focus. `consume_key` removes the event, which is the whole of "the agent
    /// never sees it".
    pub(crate) fn chord_consumed(&self, ctx: &egui::Context, action: &str) -> bool {
        let Some(chord) = self.keymap.chord_for(action) else {
            return false;
        };
        ctx.input_mut(|input| input.consume_key(chord.modifiers, chord.key))
    }

    /// Put the keymap where a person can edit it: write the defaults if there
    /// is nothing there, then open the folder. Never overwrites a file — the
    /// keymap is theirs once it exists.
    ///
    /// The write happens on a click rather than in the frame's own work, and it
    /// is one small file. It is still the only `fs` call reachable from a
    /// drawing path in this module, which is why it is named here.
    pub(crate) fn open_keymap_file(&mut self) {
        let path = keymap_path();
        match write_default_keymap() {
            Ok(created) => {
                let folder = path
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| path.clone());
                self.request_system_action(tr("Finder で表示"), move || reveal_path(&folder));
                self.notice = Some(if created {
                    tr("キー割り当てファイルを作成しました。").into()
                } else {
                    tr("キー割り当てファイルの場所を開きました。").into()
                });
            }
            Err(error) => {
                self.notice = Some(tf!(
                    "キー割り当てファイルを作成できませんでした: {error}",
                    error = error
                ));
            }
        }
    }

    /// Register a login: a name the person chose and a folder they pick.
    ///
    /// The folder picker is the only way in, because an account is a directory
    /// that already exists and a typed path is a directory that might not.
    /// Adding one re-runs the hook install, which is what puts the managed
    /// entries into the new folder.
    pub(crate) fn choose_agent_account_folder(&mut self) {
        let selection = rfd::FileDialog::new()
            .set_title(tr("アカウントのフォルダを選択"))
            .pick_folder();
        self.add_agent_account(selection);
    }

    /// The choice, separated from the dialog that made it, so that every
    /// refusal is testable without a window — the same shape
    /// `apply_project_folder_choice` already has.
    pub(crate) fn add_agent_account(&mut self, selection: Option<PathBuf>) {
        let Some(path) = selection else {
            self.notice = Some(tr("アカウントの選択をキャンセルしました。").into());
            return;
        };
        let agent = self.account_agent_input.clone();
        let name = self.account_name_input.clone();
        match register_account(&mut self.accounts, &agent, &name, &path) {
            Ok(_) => {
                if let Err(error) = save_agent_accounts(&self.data_file, &self.accounts) {
                    self.notice = Some(tf!(
                        "アカウントの記録を保存できませんでした: {error}",
                        error = error
                    ));
                    return;
                }
                self.account_name_input.clear();
                self.notice = Some(tf!("アカウント「{name}」を追加しました。", name = name));
                // The hooks have to reach the new folder, or a session under it
                // would run without any of them.
                self.request_hook_apply(self.hooks_enabled);
            }
            Err(reason) => self.notice = Some(reason),
        }
    }

    /// Remove a login. The folder is left exactly as it is — including the
    /// managed hook entries, which the hook setting removes when it is turned
    /// off and which are not this button's to take out of a directory the
    /// person may still be using.
    pub(crate) fn forget_agent_account(&mut self, id: Uuid) {
        self.accounts.accounts.retain(|account| account.id != id);
        self.accounts.sessions.retain(|record| record.account != id);
        if self.agent_account_input == Some(id) {
            self.agent_account_input = None;
        }
        if let Err(error) = save_agent_accounts(&self.data_file, &self.accounts) {
            self.notice = Some(tf!(
                "アカウントの記録を保存できませんでした: {error}",
                error = error
            ));
        }
    }

    /// Whether something is already writing this project's git index.
    ///
    /// One question, in one place, read by both actions and both buttons.
    /// `spawn_background` fences per key, and two keys reached
    /// `git_stage_exactly` — so the draft button stayed live while a commit sat
    /// inside its pre-commit hook, and its `git reset` emptied the index under
    /// it. A guard that restated the key list at each call site is the shape
    /// `.claude/rules/identifiers.md` forbids, so the list lives here.
    pub(crate) fn git_worktree_busy(tasks: &HashSet<BackgroundKey>, project: Uuid) -> bool {
        [
            BackgroundKey::GitMutation(project),
            BackgroundKey::CommitMessage(project),
            BackgroundKey::LandWorktree(project),
        ]
        .iter()
        .any(|key| tasks.contains(key))
    }

    /// Record which login a session ran under, and forget what nothing points
    /// at any more.
    ///
    /// Pruned on every write rather than on a schedule: the file sits beside
    /// the store and is read at startup, and one record per session ever
    /// launched would make it grow for the life of the machine.
    pub(crate) fn remember_session_account(&mut self, session: Uuid, account: Option<Uuid>) {
        self.accounts.remember_session(session, account);
        let live = self
            .store
            .sessions
            .iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();
        self.accounts.prune(&live);
        if let Err(error) = save_agent_accounts(&self.data_file, &self.accounts) {
            self.notice = Some(tf!(
                "アカウントの記録を保存できませんでした: {error}",
                error = error
            ));
        }
    }

    pub(crate) fn close_command_palette(&mut self) {
        self.command_palette_open = false;
        self.command_search.clear();
        self.command_selection = 0;
    }
}

impl OperonApp {
    /// The restore window: the only job in this app a person waits on.
    ///
    /// Every other background task finishes inside a frame or two and reports
    /// into the notice banner. A full-history restore does not — it reads a
    /// whole conversation and writes it back out in another CLI's format — and
    /// a one-line banner asking somebody to wait is a line they will scroll
    /// past. So this takes the window the way ⌘K does, and answers the three
    /// things people ask of a restore: which conversation is being read, where
    /// it is going, and whether it is costing anything.
    ///
    /// Dismissing it closes the window and nothing else. The import is a
    /// background thread either way, and its result still arrives in the
    /// banner — so the button says that rather than pretending to cancel.
    pub(crate) fn ui_restore_modal(&mut self, ctx: &egui::Context) {
        let Some(progress) = self.restore_progress.clone() else {
            return;
        };
        let palette = self.store.theme.palette();
        let screen = ctx.screen_rect();
        let mut dismissed = false;
        // The veil, sensing clicks so the page behind it is genuinely out of
        // reach. Unlike the palette's, a click on it does not close anything:
        // a stray click should not end a wait a person meant to sit through.
        egui::Area::new(egui::Id::new("restore-scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, palette.scrim);
            });
        if !self.command_palette_open && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            dismissed = true;
        }
        egui::Area::new(egui::Id::new("restore-modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(
                screen.center().x - 260.0,
                screen.top() + screen.height() * 0.22,
            ))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(palette.raised)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .corner_radius(egui::CornerRadius::same(RADIUS_WINDOW))
                    .inner_margin(egui::Margin::same(18))
                    .shadow(ui.style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_width(520.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = SPACE_SM;
                            ui.add(egui::Spinner::new().size(15.0).color(palette.accent));
                            ui.label(
                                RichText::new(restore_progress_title(&progress))
                                    .size(15.0)
                                    .strong()
                                    .color(palette.text_strong),
                            );
                        });
                        ui.add_space(SPACE_SM);
                        // The conversation by name, because a project can hold
                        // several that opened with the same request and the ID
                        // is the only thing that tells them apart.
                        plumbing_row(
                            ui,
                            palette,
                            tr("復元元の会話 ID"),
                            &progress.source_conversation,
                        );
                        plumbing_row(ui, palette, tr("復元先"), progress.destination.label());
                        ui.add_space(SPACE_SM);
                        ui.label(
                            RichText::new(tr("復元元の最後の依頼"))
                                .size(11.5)
                                .color(palette.text_faint),
                        );
                        ui.label(
                            RichText::new(truncate_chars(&progress.last_user_message, 220))
                                .size(12.5)
                                .color(palette.text),
                        );
                        ui.add_space(SPACE_MD);
                        hairline(ui, palette);
                        ui.add_space(SPACE_SM);
                        // Two rows, not one: the sentence and the button used to
                        // share a horizontal layout and overlapped at this
                        // width. `restore_progress_footer` owns that geometry
                        // and is the only place it is written.
                        let (_status, action) = restore_progress_footer(
                            ui,
                            palette,
                            progress.started_at.elapsed().as_secs(),
                        );
                        if action
                            .on_hover_text(tr(
                                "この表示だけ閉じます。復元は続き、結果は通知に出ます。",
                            ))
                            .clicked()
                        {
                            dismissed = true;
                        }
                    });
            });
        if dismissed {
            self.restore_progress = None;
        }
    }

    /// The modal tracking active CLI agent or terminal launch progress.
    ///
    /// Presents the target folder path, the executing CLI command and model,
    /// an animated spinner, and real-time elapsed seconds. When the launch fails,
    /// it transitions to a clear diagnostic error card with recovery options.
    /// Dismissing the modal allows the launch to continue in the background.
    pub(crate) fn ui_launch_modal(&mut self, ctx: &egui::Context) {
        let Some(progress) = self.launch_progress.clone() else {
            return;
        };
        if progress.dismissed {
            return;
        }
        let palette = self.store.theme.palette();
        let screen = ctx.screen_rect();
        let mut close_modal = false;
        let mut dismiss_to_background = false;
        let mut retry_session = false;
        let mut open_settings = false;

        egui::Area::new(egui::Id::new("launch-scrim"))
            .order(egui::Order::Middle)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, palette.scrim);
            });

        if !self.command_palette_open && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            if progress.failure.is_some() {
                close_modal = true;
            } else {
                dismiss_to_background = true;
            }
        }

        egui::Area::new(egui::Id::new("launch-modal"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(
                screen.center().x - 260.0,
                screen.top() + screen.height() * 0.20,
            ))
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(palette.raised)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .corner_radius(egui::CornerRadius::same(RADIUS_WINDOW))
                    .inner_margin(egui::Margin::same(18))
                    .shadow(ui.style().visuals.popup_shadow)
                    .show(ui, |ui| {
                        ui.set_width(520.0);
                        let is_failure = progress.failure.is_some();
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = SPACE_SM;
                            if is_failure {
                                ui.label(
                                    RichText::new(ICON_ATTENTION)
                                        .size(17.0)
                                        .color(palette.danger),
                                );
                            } else {
                                ui.add(egui::Spinner::new().size(15.0).color(palette.accent));
                            }
                            ui.label(
                                RichText::new(launch_progress_title(&progress))
                                    .size(15.0)
                                    .strong()
                                    .color(if is_failure {
                                        palette.danger
                                    } else {
                                        palette.text_strong
                                    }),
                            );
                        });
                        ui.add_space(SPACE_SM);

                        let folder_display = friendly_path(&progress.worktree_path);
                        plumbing_row(ui, palette, tr("作業フォルダ"), &folder_display);
                        if !progress.project_name.is_empty() {
                            plumbing_row(ui, palette, tr("プロジェクト"), &progress.project_name);
                        }
                        if progress.agent != "terminal" {
                            let agent_label = agent_choice_copy(&progress.agent).0;
                            plumbing_row(ui, palette, tr("エージェント"), agent_label);
                        }
                        if !progress.agent_command.is_empty() {
                            plumbing_row(ui, palette, tr("起動コマンド"), &progress.agent_command);
                        }

                        if !progress.goal.trim().is_empty() {
                            ui.add_space(SPACE_SM);
                            ui.label(
                                RichText::new(tr("依頼内容"))
                                    .size(11.5)
                                    .color(palette.text_faint),
                            );
                            ui.label(
                                RichText::new(truncate_chars(&progress.goal, 200))
                                    .size(12.5)
                                    .color(palette.text),
                            );
                        }

                        ui.add_space(SPACE_MD);
                        hairline(ui, palette);
                        ui.add_space(SPACE_SM);

                        if let Some(error) = &progress.failure {
                            ui.label(
                                RichText::new(error)
                                    .size(12.5)
                                    .color(palette.danger),
                            );
                            ui.add_space(SPACE_MD);
                            ui.horizontal(|ui| {
                                if ui.button(tr("閉じる")).clicked() {
                                    close_modal = true;
                                }
                                if ui.button(tr("再試行")).clicked() {
                                    retry_session = true;
                                }
                                if (error.contains("tmux") || error.contains("PATH"))
                                    && ui.button(tr("設定を開く")).clicked()
                                {
                                    open_settings = true;
                                }
                            });
                        } else {
                            let (_status, action) = launch_progress_footer(
                                ui,
                                palette,
                                progress.started_at.elapsed().as_secs(),
                            );
                            if action
                                .on_hover_text(tr(
                                    "この表示だけ閉じます。起動はバックグラウンドで続き、完了するとターミナルが開きます。",
                                ))
                                .clicked()
                            {
                                dismiss_to_background = true;
                            }
                        }
                    });
            });

        if dismiss_to_background {
            if let Some(p) = self.launch_progress.as_mut() {
                p.dismissed = true;
            }
        }
        if close_modal {
            self.launch_progress = None;
        }
        if open_settings {
            self.launch_progress = None;
            self.open_settings(SettingsSection::Agents);
        }
        if retry_session {
            let session_id = progress.session_id;
            self.launch_progress = None;
            self.start_session(session_id);
        }
    }
}

/// What one row of the quick-action palette opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PaletteKind {
    /// The fixed actions. The key stays the English string the dispatch matches
    /// on; the label beside it is what a person searches and reads.
    Action(&'static str),
    Session(Uuid),
    Project(Uuid),
}

/// How well a row matched, best first.
///
/// Ordered rather than scored: a number invites arithmetic on it, and there are
/// only three answers to "how well does this match" that a person can tell
/// apart — it is what they started typing, it is a word in it, or it is in
/// there somewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PaletteScore {
    Name,
    Word,
    Anywhere,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaletteEntry {
    pub(crate) kind: PaletteKind,
    pub(crate) label: String,
    pub(crate) detail: String,
    /// The key that runs this row, in the macOS spelling, or empty. Read from
    /// the keymap the frame loop listens with, so the palette can only name a
    /// key the application answers to.
    pub(crate) chord: String,
    pub(crate) score: PaletteScore,
}

/// Where `query` sits in a row, or `None` if it is not in it.
///
/// `name` is the row's own name and `haystack` everything else it can be found
/// by, so that typing the start of a session's title beats typing a word from
/// the middle of its request.
pub(crate) fn palette_rank(name: &str, haystack: &str, query: &str) -> Option<PaletteScore> {
    if query.trim().is_empty() {
        return None;
    }
    // Both sides are folded here rather than at the call site. A function that
    // is only correct when its caller happened to lower-case one argument is a
    // function whose next caller gets it wrong.
    let query = query.to_lowercase();
    let query = query.as_str();
    let name = name.to_lowercase();
    let haystack = haystack.to_lowercase();
    if !haystack.contains(query) && !name.contains(query) {
        return None;
    }
    if name.starts_with(query) {
        return Some(PaletteScore::Name);
    }
    // A word here is anything after a space or one of the separators these rows
    // are actually made of — a path, a branch, a chip joined with a middle dot.
    let starts_a_word = haystack
        .split(|character: char| {
            character.is_whitespace() || matches!(character, '/' | '-' | '_' | '.' | ':' | '·')
        })
        .any(|word| word.starts_with(query));
    Some(if starts_a_word {
        PaletteScore::Word
    } else {
        PaletteScore::Anywhere
    })
}

impl PaletteKind {
    /// Actions before sessions before projects, when nothing else separates
    /// them: the verbs are what the palette was for, and a project is reachable
    /// through any of its sessions.
    fn rank(&self) -> u8 {
        match self {
            Self::Action(_) => 0,
            Self::Session(_) => 1,
            Self::Project(_) => 2,
        }
    }
}

/// Running what a repository says about preparing a fresh checkout of itself.
///
/// Split from the rest of the application's methods only because `src/app.rs`
/// is over its size band; the decisions live in `src/git/setup.rs` and this is
/// the wiring that reaches them.
impl OperonApp {
    /// Called once, when a worktree has just been made. Decides between three
    /// outcomes: nothing to do, run it, or ask.
    pub(crate) fn offer_setup(&mut self, project: Uuid, worktree: &Path) {
        match read_setup_script(worktree) {
            SetupScriptRead::Missing => {}
            SetupScriptRead::TooLarge(size) => {
                self.notice = Some(tf!(
                    "{p0} が大きすぎるため実行しません（{p1} バイト、上限 {p2} バイト）。",
                    p0 = SETUP_SCRIPT_RELATIVE_PATH,
                    p1 = size,
                    p2 = SETUP_SCRIPT_MAX_BYTES
                ));
            }
            SetupScriptRead::Found(script) => {
                if is_setup_approved(&self.setup_trust, project, &script.digest) {
                    self.start_setup_session(project, worktree.to_path_buf(), &script);
                } else {
                    self.pending_setup = Some(PendingSetup {
                        project,
                        worktree: worktree.to_path_buf(),
                        script,
                        remember: true,
                    });
                }
            }
        }
    }

    /// The 実行する button. The file is read again here rather than trusted from
    /// the moment it was shown: a script that changed in between is not the
    /// script that was approved, and this is the only place that can tell.
    pub(crate) fn accept_pending_setup(&mut self) {
        let Some(pending) = self.pending_setup.take() else {
            return;
        };
        let script = match decide_setup_run(&pending.worktree, &pending.script.digest) {
            SetupRunDecision::Run(script) => script,
            SetupRunDecision::Changed(script) => {
                self.notice = Some(tf!(
                    "{p0} の内容が変わったため実行しませんでした。もう一度確認してください。",
                    p0 = SETUP_SCRIPT_RELATIVE_PATH
                ));
                // Ask again about what is on disk now, not about what was there
                // when the block was drawn.
                self.pending_setup = Some(PendingSetup {
                    remember: pending.remember,
                    script,
                    ..pending
                });
                return;
            }
            SetupRunDecision::Unreadable => {
                self.notice = Some(tf!(
                    "{p0} を読み直せませんでした。実行していません。",
                    p0 = SETUP_SCRIPT_RELATIVE_PATH
                ));
                return;
            }
        };
        if pending.remember {
            approve_setup(&mut self.setup_trust, pending.project, &script.digest);
            if let Err(error) = save_setup_trust(&self.data_file, &self.setup_trust) {
                self.notice = Some(tf!(
                    "この内容を今後許可する設定を保存できませんでした: {error}",
                    error = error
                ));
            }
        }
        self.start_setup_session(pending.project, pending.worktree, &script);
    }

    /// A setup run is an ordinary terminal session whose command happens to be
    /// the script. Everything that already works on a session — watching it,
    /// scrolling it, stopping it, seeing that it died — works on this for free,
    /// and nothing in the persisted shape had to move to get that.
    pub(crate) fn start_setup_session(
        &mut self,
        project: Uuid,
        worktree: PathBuf,
        script: &SetupScript,
    ) {
        if !self.tools.tmux {
            self.notice = Some(tr("tmux が必要です。「設定」を開いてください。").into());
            return;
        }
        let id = Uuid::new_v4();
        let tmux_name = managed_tmux_name(id);
        let command = setup_command(&script.path);
        let previous_store = self.store.clone();
        self.store.sessions.push(Session {
            id,
            project_id: project,
            name: setup_session_name(&worktree),
            goal: tf!(
                "リポジトリの {p0} を実行しています。",
                p0 = SETUP_SCRIPT_RELATIVE_PATH
            ),
            agent: "terminal".into(),
            tmux_name: tmux_name.clone(),
            created_at: now(),
            launched_at: None,
            status: SessionStatus::Starting,
            worktree_path: Some(worktree.clone()),
            branch: None,
            agent_command: String::new(),
            native_session_id: None,
            native_session_path: None,
            origin: Some(SessionOrigin::PlainTerminal),
            depends_on: Vec::new(),
        });
        if !self.persist() {
            self.store = previous_store;
            return;
        }
        self.setup_sessions.insert(id);
        let hook_environment = self.mint_hook_environment(id);
        self.spawn_background(BackgroundKey::EmptySessionStart(id), move || {
            let result = (|| {
                if !tool_available("tmux") {
                    return Err(tr("tmux が必要です。次のコマンドでインストールしてください: brew install tmux").into());
                }
                start_tmux_agent_session(
                    &tmux_name,
                    &worktree,
                    &command,
                    &hook_variables(hook_environment.as_ref()),
                )
                .map_err(|error| error.to_string())?;
                Ok(git_branch(&worktree).ok())
            })();
            BackgroundResult::EmptySessionStarted {
                session_id: id,
                result,
            }
        });
        self.notice = Some(tr("セットアップを実行しています。").into());
    }

    /// The block agreed before this code existed: what the script is, the first
    /// lines of it, how many lines there are in total, and two answers.
    pub(crate) fn ui_setup_prompt(&mut self, ui: &mut egui::Ui, palette: &Palette) {
        let Some(pending) = self.pending_setup.clone() else {
            return;
        };
        ui.add_space(10.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(
                RichText::new(tr("このリポジトリのセットアップを実行しますか？"))
                    .strong()
                    .color(palette.accent_soft),
            );
            ui.label(tf!(
                "{p0} はリポジトリに入っているシェルスクリプトです。専用のセッションで実行され、途中で止められます。",
                p0 = SETUP_SCRIPT_RELATIVE_PATH
            ));
            ui.add_space(6.0);
            for line in &pending.script.preview {
                ui.monospace(line);
            }
            if pending.script.total_lines > pending.script.preview.len() {
                ui.label(
                    RichText::new(tf!("… 全 {p0} 行", p0 = pending.script.total_lines))
                        .small()
                        .color(palette.text_muted),
                );
            }
            ui.add_space(6.0);
            let mut remember = pending.remember;
            if ui
                .checkbox(&mut remember, tr("内容が変わるまで、今後は確認しない"))
                .changed()
            {
                if let Some(pending) = self.pending_setup.as_mut() {
                    pending.remember = remember;
                }
            }
            ui.horizontal(|ui| {
                if ui.button(tr("実行しない")).clicked() {
                    self.pending_setup = None;
                }
                if ui.button(tr("実行する")).clicked() {
                    self.accept_pending_setup();
                }
            });
        });
    }
}

/// Notes on a diff: keeping them, and handing them to the agent.
impl OperonApp {
    /// Write the notes down if the review pane changed them. Called once a
    /// frame, outside the borrow the pane holds, and does nothing on the frames
    /// where nothing happened — which is almost all of them.
    pub(crate) fn persist_diff_comments(&mut self) {
        if !self.diff_comments_dirty {
            return;
        }
        self.diff_comments_dirty = false;
        let store = DiffCommentStore {
            comments: self.diff_annotations.comments.clone(),
        };
        if let Err(error) = save_diff_comments(&self.data_file, &store) {
            self.notice = Some(tf!(
                "コメントを保存できませんでした: {error}",
                error = error
            ));
        }
    }

    /// Which session a review is handed to: the agent that is running in this
    /// project, most recently launched first.
    ///
    /// Not the selected session — somebody reading a diff is often looking at
    /// something else — and not a plain terminal, which has no agent to read
    /// the message. When there is no answer the send refuses rather than
    /// picking a surprising one.
    pub(crate) fn diff_comment_target(&self, project: Uuid) -> Option<&Session> {
        self.store
            .sessions
            .iter()
            .filter(|session| {
                session.project_id == project
                    && session.status == SessionStatus::Active
                    && session.origin != Some(SessionOrigin::PlainTerminal)
                    && session.agent != "terminal"
            })
            .max_by_key(|session| session.launched_at.unwrap_or(session.created_at))
    }

    /// Hand every note in this project to that agent as one message.
    pub(crate) fn send_diff_comments(&mut self, project: Uuid) {
        let Some(target) = self.diff_comment_target(project) else {
            self.notice = Some(
                tr("このプロジェクトで動いているエージェントのセッションがありません。").into(),
            );
            return;
        };
        let session_id = target.id;
        let tmux_name = target.tmux_name.clone();
        // Mid-turn is exactly when a paste gets cut in half, and half a review
        // read as a whole one is worse than being asked to wait.
        if self.session_activity_for(session_id) == Some(AgentActivity::Working) {
            self.notice = Some(tr("エージェントの応答が終わるまで待ってください。").into());
            return;
        }
        let files = self.diff_file_cache.get(&(project, None)).cloned();
        let annotated: Vec<(DiffComment, bool)> = self
            .diff_annotations
            .comments
            .iter()
            .filter(|comment| comment.project == project && !comment.resolved)
            .map(|comment| {
                let stale = files.as_ref().is_some_and(|files| {
                    files
                        .iter()
                        .find(|file| file.path == comment.file)
                        .and_then(|file| {
                            file.lines
                                .iter()
                                .find(|line| line.new_number == Some(comment.line))
                        })
                        .is_some_and(|line| comment_anchor(&line.text) != comment.anchor)
                });
                (comment.clone(), stale)
            })
            .collect();
        if annotated.is_empty() {
            self.notice = Some(tr("送信する未解決のコメントがありません。").into());
            return;
        }
        let message = diff_comment_message(&annotated);
        if message.len() > DIFF_COMMENT_MESSAGE_MAX_BYTES {
            self.notice = Some(tf!(
                "コメントが大きすぎて一度に送れません（{p0} 件、上限 {p1} バイト）。分けて送ってください。",
                p0 = annotated.len(),
                p1 = DIFF_COMMENT_MESSAGE_MAX_BYTES
            ));
            return;
        }
        match tmux_send_input(
            &tmux_name,
            &[
                TerminalInput::Text(message),
                TerminalInput::Key("Enter".into()),
            ],
        ) {
            Ok(()) => {
                let count = annotated.len();
                self.diff_annotations.mark_sent(project);
                self.diff_comments_dirty = true;
                self.persist_diff_comments();
                self.notice = Some(tf!(
                    "コメント {p0} 件をエージェントに送りました。",
                    p0 = count
                ));
            }
            Err(error) => {
                self.notice = Some(tf!("コメントを送れませんでした: {error}", error = error));
            }
        }
    }

    /// Export review notes formatted as structured Markdown to clipboard.
    pub(crate) fn copy_diff_comments_markdown(&mut self, project: Uuid, ctx: &egui::Context) {
        let files = self.diff_file_cache.get(&(project, None)).cloned();
        let annotated: Vec<(DiffComment, bool)> = self
            .diff_annotations
            .comments
            .iter()
            .filter(|comment| comment.project == project)
            .map(|comment| {
                let stale = files.as_ref().is_some_and(|files| {
                    files
                        .iter()
                        .find(|file| file.path == comment.file)
                        .and_then(|file| {
                            file.lines
                                .iter()
                                .find(|line| line.new_number == Some(comment.line))
                        })
                        .is_some_and(|line| comment_anchor(&line.text) != comment.anchor)
                });
                (comment.clone(), stale)
            })
            .collect();
        if annotated.is_empty() {
            return;
        }
        let markdown = format_diff_comments_markdown(&annotated);
        ctx.copy_text(markdown);
        self.notice =
            Some(tr("レビューメモを Markdown 形式でクリップボードにコピーしました。").into());
    }

    /// The footer under the review pane. Not drawn when there is nothing to
    /// say, which is the ordinary state of this screen.
    pub(crate) fn ui_diff_comment_footer(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        project: Uuid,
    ) {
        let total = self.diff_annotations.count(project);
        if total == 0 {
            return;
        }
        let unresolved = self.diff_annotations.count_unresolved(project);
        let sent = self.diff_annotations.count_sent(project);
        let resolved = self.diff_annotations.count_resolved(project);

        ui.separator();
        ui.horizontal(|ui| {
            let count_text = if resolved > 0 || sent > 0 {
                tf!(
                    "{ICON_COMMENT} {p0} 件（未解決 {p1}、送信済 {p2}、解決済 {p3}）",
                    ICON_COMMENT = ICON_COMMENT,
                    p0 = total,
                    p1 = unresolved,
                    p2 = sent,
                    p3 = resolved
                )
            } else {
                tf!(
                    "{ICON_COMMENT} {p0} 件",
                    ICON_COMMENT = ICON_COMMENT,
                    p0 = total
                )
            };
            ui.label(
                RichText::new(count_text)
                    .size(12.5)
                    .color(palette.text_muted),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(tr("セッションにまとめて送る")).clicked() {
                    self.send_diff_comments(project);
                }
                if ui.button(tr("Markdownをコピー")).clicked() {
                    self.copy_diff_comments_markdown(project, ui.ctx());
                }
                if resolved > 0 && ui.button(tr("解決済みを消去")).clicked() {
                    self.diff_annotations.clear_resolved(project);
                    self.diff_comments_dirty = true;
                    self.persist_diff_comments();
                }
                if ui.button(tr("すべて消す")).clicked() {
                    self.diff_annotations.clear_project(project);
                    self.diff_comments_dirty = true;
                    self.persist_diff_comments();
                }
            });
        });
    }
}

/// Choosing what goes in, saying what it was, and sending it.
impl OperonApp {
    /// Which files are ticked, seeded the first time this project's changes are
    /// seen: tracked modifications yes, untracked no. An agent that left a
    /// scratch file behind should not have it recorded by somebody who did not
    /// look at it.
    pub(crate) fn ensure_staged_selection(&mut self, project: Uuid, files: &[ChangedFile]) {
        let known: HashSet<String> = files.iter().map(|file| file.path.clone()).collect();
        let selected = self.git_staged_files.entry(project).or_insert_with(|| {
            files
                .iter()
                .filter(|file| file.status != "??")
                .map(|file| file.path.clone())
                .collect()
        });
        // A file that stopped being changed stops being a choice. Without this
        // the count on the button would keep naming something that is no
        // longer there.
        selected.retain(|path| known.contains(path));
    }

    pub(crate) fn staged_selection(&self, project: Uuid) -> Vec<String> {
        let mut files: Vec<String> = self
            .git_staged_files
            .get(&project)
            .map(|selected| selected.iter().cloned().collect())
            .unwrap_or_default();
        files.sort();
        files
    }

    pub(crate) fn toggle_staged_file(&mut self, project: Uuid, path: &str, ticked: bool) {
        let selected = self.git_staged_files.entry(project).or_default();
        if ticked {
            selected.insert(path.to_owned());
        } else {
            selected.remove(path);
        }
    }

    /// Stage exactly what is ticked and record it.
    pub(crate) fn record_staged_change(&mut self, project: &Project) {
        let message = self.commit_message_input.trim().to_owned();
        if message.is_empty() {
            self.notice = Some(tr("コミットメッセージを入力してください。").into());
            return;
        }
        let files = self.staged_selection(project.id);
        if files.is_empty() {
            self.notice = Some(tr("コミットするファイルを選んでください。").into());
            return;
        }
        if Self::git_worktree_busy(&self.background_tasks, project.id) {
            self.notice = Some(tr("Git の操作が既に実行中です。").into());
            return;
        }
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::GitMutation(project_id), move || {
            let result = (|| {
                git_stage_exactly(&path, &files).map_err(|error| error.to_string())?;
                git_record_staged(&path, &message).map_err(|error| error.to_string())
            })();
            BackgroundResult::StagedChangeRecorded { project_id, result }
        });
        self.notice = Some(tr("記録しています…").into());
    }

    pub(crate) fn push_project(&mut self, project: &Project) {
        if Self::git_worktree_busy(&self.background_tasks, project.id) {
            self.notice = Some(tr("Git の操作が既に実行中です。").into());
            return;
        }
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::GitMutation(project_id), move || {
            let result = (|| {
                let state = git_upstream_state(&path).map_err(|error| error.to_string())?;
                let branch = git_branch(&path).map_err(|error| error.to_string())?;
                git_push(&path, &state, branch.trim()).map_err(|error| error.to_string())
            })();
            BackgroundResult::PushFinished { project_id, result }
        });
        self.notice = Some(tr("push しています…").into());
    }

    /// Ask a local agent CLI to draft the message from the staged change.
    ///
    /// It stages first, because the prompt is written from the staged patch and
    /// the person's ticks are what "staged" means on this screen. The draft
    /// lands in the field and goes no further: two buttons, so that nothing a
    /// model wrote can become a commit without a person pressing the other one.
    pub(crate) fn draft_commit_message(&mut self, project: &Project) {
        let Some(agent) = commit_message_agent(&self.tools) else {
            return;
        };
        let files = self.staged_selection(project.id);
        if files.is_empty() {
            self.notice = Some(tr("コミットするファイルを選んでください。").into());
            return;
        }
        // The draft rewrites the index too, so it waits for whatever else is
        // writing it — including a commit sitting inside a pre-commit hook,
        // which is minutes on a repository like this one.
        if Self::git_worktree_busy(&self.background_tasks, project.id) {
            self.notice = Some(tr("Git の操作が既に実行中です。").into());
            return;
        }
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::CommitMessage(project_id), move || {
            let result = (|| {
                git_stage_exactly(&path, &files).map_err(|error| error.to_string())?;
                let branch = git_branch(&path).unwrap_or_default();
                let (staged_files, patch) =
                    git_staged_summary(&path).map_err(|error| error.to_string())?;
                let prompt = commit_message_prompt(branch.trim(), &staged_files, &patch);
                let Some((program, arguments)) = commit_message_command(agent, &prompt) else {
                    return Err(tr("このエージェントには下書きの実行方法がありません。").into());
                };
                let mut command = Command::new(program);
                command.args(arguments).current_dir(&path);
                let limited = run_command_with_output_limit(
                    &mut command,
                    Duration::from_secs(COMMIT_MESSAGE_TIMEOUT_SECONDS),
                    COMMIT_MESSAGE_MAX_BYTES,
                    COMMAND_ERROR_MAX_BYTES,
                )
                .map_err(|error| error.to_string())?;
                if !limited.output.status.success() {
                    return Err(String::from_utf8_lossy(&limited.output.stderr)
                        .trim()
                        .to_owned());
                }
                let drafted = clean_generated_commit_message(&String::from_utf8_lossy(
                    &limited.output.stdout,
                ));
                if drafted.is_empty() {
                    return Err(tr("エージェントが何も返しませんでした。").into());
                }
                Ok(drafted)
            })();
            BackgroundResult::CommitMessageDrafted { project_id, result }
        });
    }

    /// Request an automated AI review of the working tree diff.
    pub(crate) fn request_ai_diff_review(&mut self, project: &Project) {
        let Some(agent) = commit_message_agent(&self.tools) else {
            self.notice = Some(tr("このエージェントには下書きの実行方法がありません。").into());
            return;
        };
        if self
            .background_tasks
            .contains(&BackgroundKey::AiDiffReview(project.id))
        {
            self.notice = Some(tr("AIレビュー中…").into());
            return;
        }
        let project_id = project.id;
        let path = project.path.clone();
        self.spawn_background(BackgroundKey::AiDiffReview(project_id), move || {
            let result = (|| {
                let diff = git_working_tree_diff(&path, None).map_err(|error| error.to_string())?;
                if diff.trim().is_empty() {
                    return Err(tr("レビュー対象の差分がありません。").into());
                }
                let branch = git_branch(&path).unwrap_or_default();
                let prompt = ai_diff_review_prompt(branch.trim(), &diff);
                let Some((program, arguments)) = commit_message_command(agent, &prompt) else {
                    return Err(tr("このエージェントには下書きの実行方法がありません。").into());
                };
                let mut command = Command::new(program);
                command.args(arguments).current_dir(&path);
                let limited = run_command_with_output_limit(
                    &mut command,
                    Duration::from_secs(AI_REVIEW_TIMEOUT_SECONDS),
                    AI_REVIEW_MAX_BYTES,
                    COMMAND_ERROR_MAX_BYTES,
                )
                .map_err(|error| error.to_string())?;
                if !limited.output.status.success() {
                    return Err(String::from_utf8_lossy(&limited.output.stderr)
                        .trim()
                        .to_owned());
                }
                let reviewed =
                    clean_generated_ai_review(&String::from_utf8_lossy(&limited.output.stdout));
                if reviewed.is_empty() {
                    return Err(tr("エージェントが何も返しませんでした。").into());
                }
                Ok(reviewed)
            })();
            BackgroundResult::AiDiffReview { project_id, result }
        });
        self.notice = Some(tr("AIレビューを実行しています…").into());
    }

    /// Fix git commit failure by providing failure diagnosis and prompt to the active agent session or clipboard.
    pub(crate) fn fix_commit_failure_with_ai(&mut self, project: &Project, ctx: &egui::Context) {
        let Some((commit_msg, staged_files, error_output)) =
            self.last_commit_failure.get(&project.id).cloned()
        else {
            self.notice = Some(tr("修正対象のコミットエラーがありません。").into());
            return;
        };
        let branch = git_branch(&project.path).unwrap_or_default();
        let prompt = build_fix_commit_failure_prompt(
            branch.trim(),
            &staged_files,
            &commit_msg,
            &error_output,
        );
        let Some(target) = self.diff_comment_target(project.id) else {
            ctx.copy_text(prompt);
            self.notice = Some(tr("アクティブなセッションがありません。修正プロンプトをクリップボードにコピーしました。").into());
            return;
        };
        let session_id = target.id;
        let tmux_name = target.tmux_name.clone();
        if self.session_activity_for(session_id) == Some(AgentActivity::Working) {
            self.notice = Some(tr("エージェントの応答が終わるまで待ってください。").into());
            return;
        }
        match tmux_send_input(
            &tmux_name,
            &[
                TerminalInput::Text(prompt),
                TerminalInput::Key("Enter".into()),
            ],
        ) {
            Ok(()) => {
                self.notice =
                    Some(tr("コミット失敗の修正プロンプトをエージェントに送りました。").into());
            }
            Err(error) => {
                self.notice = Some(tf!(
                    "ターミナルに入力を送れませんでした: {error}",
                    error = error
                ));
            }
        }
    }
}

/// What is listening, and which session is serving it.
impl OperonApp {
    /// Ask for a reading, at most once every `PORT_SCAN_INTERVAL_SECONDS`.
    ///
    /// Called from the frame loop beside the session poll, never from a draw
    /// path: reading the process table is work proportional to the machine, and
    /// the answer is the same for thirty seconds either way.
    pub(crate) fn request_port_scan(&mut self) {
        if self.background_tasks.contains(&BackgroundKey::PortScan) {
            return;
        }
        let due = self
            .ports_scanned_at
            .is_none_or(|at| at.elapsed() >= Duration::from_secs(PORT_SCAN_INTERVAL_SECONDS));
        if !due {
            return;
        }
        // Every folder a session is working in. A session with no worktree path
        // has nothing to attribute against and contributes nothing.
        let mut worktrees: Vec<PathBuf> = self
            .store
            .sessions
            .iter()
            .filter_map(|session| session.worktree_path.clone())
            .collect();
        worktrees.sort();
        worktrees.dedup();
        if worktrees.is_empty() {
            // Nothing could be attributed, so nothing is worth reading. The
            // clock still moves, so this does not spin.
            self.ports_scanned_at = Some(Instant::now());
            self.listening_ports.clear();
            return;
        }
        self.spawn_background(BackgroundKey::PortScan, move || {
            BackgroundResult::ListeningPorts(scan_listening_ports(&worktrees))
        });
    }

    /// The ports of one session's worktree, in port order so the row does not
    /// reshuffle itself between scans.
    pub(crate) fn session_ports(&self, session: &Session) -> Vec<ListeningPort> {
        let Some(worktree) = session.worktree_path.as_ref() else {
            return Vec::new();
        };
        let mut ports: Vec<ListeningPort> = self
            .listening_ports
            .iter()
            .filter(|(_, owner)| owner.as_ref() == Some(worktree))
            .map(|(port, _)| port.clone())
            .collect();
        ports.sort_by_key(|port| port.port);
        ports
    }

    /// One line under the session's title. Not drawn when there is nothing
    /// listening, and not drawn when the machine cannot be read — this is
    /// information, not a task, and a standing complaint about a missing
    /// reading is worse than the missing reading.
    pub(crate) fn ui_session_ports(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        session: &Session,
    ) {
        let ports = self.session_ports(session);
        if ports.is_empty() {
            return;
        }
        meta_separator(ui, palette);
        let badge_text = if ports.len() == 1 {
            format!(":{p0} {p1}", p0 = ports[0].port, p1 = ports[0].process)
        } else {
            tf!("{p0} ポート稼働中", p0 = ports.len())
        };

        let mut opened_url = None;
        let mut copied_url = None;

        let _ = egui::menu::menu_custom_button(
            ui,
            egui::Button::new(
                RichText::new(format!("{ICON_OPEN_EXTERNAL} {badge_text}"))
                    .size(11.5)
                    .color(palette.info),
            )
            .frame(false)
            .min_size(egui::vec2(0.0, 18.0)),
            |ui| {
                ui.label(RichText::new(tr("稼働中のポート")).small().weak());
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        for port in &ports {
                            let url = port_url(port);
                            // Security invariant: only accept http/https scheme with loopback/local host
                            let is_safe_url = (url.starts_with("http://")
                                || url.starts_with("https://"))
                                && !url.chars().any(|c| {
                                    c.is_control() || c.is_whitespace() || c == '"' || c == '\''
                                });
                            if !is_safe_url {
                                continue;
                            }
                            let port_display =
                                format!(":{p0} {p1}", p0 = port.port, p1 = port.process);
                            ui.horizontal(|ui| {
                                if ui
                                    .button(RichText::new(port_display).size(12.0))
                                    .on_hover_text(tf!("ブラウザで開く: {url}"))
                                    .clicked()
                                {
                                    opened_url = Some(url.clone());
                                    ui.close_menu();
                                }
                                if ui
                                    .small_button(RichText::new(ICON_COPY).size(11.0))
                                    .on_hover_text(tr("URL をコピー"))
                                    .clicked()
                                {
                                    copied_url = Some(url);
                                    ui.close_menu();
                                }
                            });
                        }
                    });
            },
        )
        .response
        .on_hover_text(tr("稼働中のローカルポート一覧を表示"));

        if let Some(url) = opened_url {
            self.request_system_action(tr("ブラウザで開く"), move || open_url(&url));
        }
        if let Some(url) = copied_url {
            ui.ctx().copy_text(url);
            self.notice = Some(tr("URL をコピーしました。").into());
        }
    }
}

/// Looking through what a session has already said.
impl OperonApp {
    /// Catch `⌘F` and `Esc` before the pane reads its events.
    ///
    /// `consume_key` removes the event, which is the whole of "the agent never
    /// receives it": the pane reads whatever is left, and this is not left.
    pub(crate) fn handle_terminal_search_keys(&mut self, ctx: &egui::Context) {
        let Some(session_id) = self.selected_session else {
            return;
        };
        if self.command_palette_open {
            // The palette is in front and owns the keyboard while it is.
            return;
        }
        if self.chord_consumed(ctx, "terminal.find") {
            let search = self.terminal_search.entry(session_id).or_default();
            search.open = true;
            search.needs_focus = true;
        }
        let open = self
            .terminal_search
            .get(&session_id)
            .is_some_and(|search| search.open);
        if open
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.close_terminal_search(session_id);
        }
    }

    pub(crate) fn close_terminal_search(&mut self, session_id: Uuid) {
        if let Some(search) = self.terminal_search.get_mut(&session_id) {
            search.open = false;
            search.scroll_to = None;
        }
    }

    /// Recompute where the query occurs.
    ///
    /// Called when the query changes and when the buffer does, never per frame:
    /// the pane holds fifty thousand lines and this walks all of them.
    pub(crate) fn refresh_terminal_search(&mut self, session_id: Uuid) {
        let Some(search) = self.terminal_search.get(&session_id) else {
            return;
        };
        if !search.open {
            return;
        }
        let query = search.query.clone();
        let matches = self
            .terminal_layouts
            .get(&session_id)
            .map(|lines| terminal_search_matches(lines, &query))
            .unwrap_or_default();
        let Some(search) = self.terminal_search.get_mut(&session_id) else {
            return;
        };
        // The current match keeps its index rather than its line number. Output
        // arriving underneath a search moves every line; the third match is
        // still the third match.
        search.current = search.current.min(matches.len().saturating_sub(1));
        search.matches = matches;
    }

    pub(crate) fn step_terminal_search(&mut self, session_id: Uuid, forward: bool) {
        let Some(search) = self.terminal_search.get_mut(&session_id) else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }
        let last = search.matches.len() - 1;
        search.current = if forward {
            if search.current >= last {
                0
            } else {
                search.current + 1
            }
        } else if search.current == 0 {
            last
        } else {
            search.current - 1
        };
        search.scroll_to = Some(search.matches[search.current].line);
    }

    pub(crate) fn terminal_search_overlay<'a>(
        &'a self,
        session_id: Uuid,
        palette: &'a Palette,
    ) -> TerminalSearchOverlay<'a> {
        match self.terminal_search.get(&session_id) {
            Some(search) if search.open && !search.matches.is_empty() => TerminalSearchOverlay {
                matches: &search.matches,
                current: Some(search.current),
                scroll_to: search.scroll_to,
                palette,
            },
            _ => TerminalSearchOverlay {
                matches: &[],
                current: None,
                scroll_to: None,
                palette,
            },
        }
    }

    /// The bar: one row, under the pane, the way a browser does it.
    pub(crate) fn ui_terminal_search(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
        session_id: Uuid,
    ) {
        let Some(search) = self.terminal_search.get(&session_id) else {
            return;
        };
        if !search.open {
            return;
        }
        let mut query = search.query.clone();
        let needs_focus = search.needs_focus;
        let total = search.matches.len();
        let current = search.current;
        let mut changed = false;
        let mut step = None;
        let mut close = false;

        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(ICON_SEARCH)
                    .size(13.0)
                    .color(palette.text_muted),
            );
            let field = ui.add(
                egui::TextEdit::singleline(&mut query)
                    .desired_width(240.0)
                    .hint_text(tr("スクロールバックを検索")),
            );
            if needs_focus {
                field.request_focus();
            }
            if field.changed() {
                changed = true;
            }
            // Enter only means "next" once the composition is committed: a
            // preedit is not yet text, which is what change 014 is about.
            let composing = !self
                .terminal_preedits
                .get(&session_id)
                .is_none_or(String::is_empty);
            if field.has_focus() && !composing {
                let (next, previous) = ui.input_mut(|input| {
                    (
                        input.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                        input.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter),
                    )
                });
                if next {
                    step = Some(true);
                }
                if previous {
                    step = Some(false);
                }
            }
            ui.label(
                RichText::new(if query.trim().is_empty() {
                    String::new()
                } else if total == 0 {
                    tr("一致しません").into()
                } else {
                    tf!("{p0} / {p1}", p0 = current + 1, p1 = total)
                })
                .size(12.0)
                .color(palette.text_muted),
            );
            if ui
                .add_enabled(total > 0, egui::Button::new(ICON_PREVIOUS).small())
                .on_hover_text(tr("前の一致"))
                .clicked()
            {
                step = Some(false);
            }
            if ui
                .add_enabled(total > 0, egui::Button::new(ICON_NEXT).small())
                .on_hover_text(tr("次の一致"))
                .clicked()
            {
                step = Some(true);
            }
            if ui
                .button(ICON_CLOSE)
                .on_hover_text(tr("検索を閉じる"))
                .clicked()
            {
                close = true;
            }
        });

        if let Some(search) = self.terminal_search.get_mut(&session_id) {
            search.query = query;
            search.needs_focus = false;
        }
        if changed {
            if let Some(search) = self.terminal_search.get_mut(&session_id) {
                search.current = 0;
                search.scroll_to = None;
            }
            self.refresh_terminal_search(session_id);
            if let Some(search) = self.terminal_search.get_mut(&session_id) {
                search.scroll_to = search.matches.first().map(|found| found.line);
            }
        }
        if let Some(forward) = step {
            self.step_terminal_search(session_id, forward);
        }
        if close {
            self.close_terminal_search(session_id);
        }
    }
}

/// What the account has left, as the CLI itself reported it.
impl OperonApp {
    /// The chip in the toolbar. Absent until something has been said, which is
    /// most of the time on a machine that has not run Claude Code today.
    pub(crate) fn ui_rate_limits(
        &mut self,
        ui: &mut egui::Ui,
        palette: &Palette,
    ) -> Option<egui::Rect> {
        let (limits, at) = self.rate_limits?;
        let age = at.elapsed().as_secs();
        let stale = age >= STATUSLINE_STALE_AFTER_SECONDS;
        let windows = [
            (tr("5 時間枠"), "5h", limits.five_hour),
            (tr("7 日枠"), "7d", limits.seven_day),
        ];
        let mut hover = Vec::new();
        for (name, _, window) in windows {
            let Some(window) = window else {
                continue;
            };
            hover.push(match window.resets_at {
                Some(resets_at) => tf!(
                    "{p0} {p1}%（{p2}でリセット）",
                    p0 = name,
                    p1 = window.used_percent.round(),
                    p2 = time_until(resets_at)
                ),
                None => tf!("{p0} {p1}%", p0 = name, p1 = window.used_percent.round()),
            });
        }
        if hover.is_empty() {
            return None;
        }
        if stale {
            hover.push(tf!(
                "{p0}の読みです",
                p0 = relative_time(now().saturating_sub(age))
            ));
        }
        let hover = hover.join("\n");
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_XS;
            let mut first = true;
            for (_, short, window) in windows {
                let Some(window) = window else {
                    continue;
                };
                if !first {
                    ui.label(RichText::new("·").size(11.5).color(palette.text_faint));
                }
                first = false;
                // Only the number that is worth noticing changes colour, so a
                // busy window does not make the quiet one look urgent too.
                let colour = if stale {
                    palette.text_faint
                } else if window.used_percent >= STATUSLINE_WARN_PERCENT {
                    palette.warning
                } else {
                    palette.text_muted
                };
                ui.label(
                    RichText::new(tf!(
                        "{p0} {p1}%",
                        p0 = short,
                        p1 = window.used_percent.round()
                    ))
                    .size(11.5)
                    .color(colour),
                )
                .on_hover_text(&hover);
            }
        });
        Some(inner.response.rect)
    }
}

/// Opening what a session's output names.
impl OperonApp {
    /// Answer the questions the pane asked while it was drawing.
    ///
    /// Resolution is a filesystem call, so it happens here, once a frame,
    /// outside the borrow the pane holds and outside the draw path the Rust
    /// rule protects. The set is small: only the rows on screen are read, and
    /// only names the cache has never seen are asked about.
    pub(crate) fn resolve_terminal_paths(&mut self) {
        if self.resolved_paths_session.is_some()
            && self.resolved_paths_session != self.selected_session
        {
            self.resolved_paths.clear();
            self.resolved_paths_generation += 1;
        }
        self.resolved_paths_session = self.selected_session;
        if self.unresolved_paths.is_empty() {
            return;
        }
        let candidates = std::mem::take(&mut self.unresolved_paths);
        let Some((project_path, worktree_path)) = self
            .selected_session
            .and_then(|id| self.store.sessions.iter().find(|session| session.id == id))
            .and_then(|session| {
                let project = self
                    .store
                    .projects
                    .iter()
                    .find(|candidate| candidate.id == session.project_id)?;
                Some((project.path.clone(), session.worktree_path.clone()))
            })
        else {
            // Nothing to resolve against. Recording the miss stops the pane
            // asking about the same names on every frame.
            for candidate in candidates {
                self.resolved_paths.insert(candidate, None);
                self.resolved_paths_generation += 1;
            }
            return;
        };
        for candidate in candidates {
            let resolved =
                resolve_terminal_session_path(worktree_path.as_deref(), &project_path, &candidate);
            self.resolved_paths.insert(candidate, resolved);
            self.resolved_paths_generation += 1;
        }
    }

    /// Switch to the Files tab of a project and open a document, optionally jumping to a specific line.
    pub(crate) fn navigate_to_file(
        &mut self,
        project: &Project,
        root: PathBuf,
        path: PathBuf,
        line: Option<usize>,
    ) {
        self.editor_jump_line = line;
        self.open_document_at(project, root, path);
        if line.is_some() {
            self.editor_view = EditorView::Edit;
        }
        self.page = Page::Projects;
        self.selected_project = Some(project.id);
        self.project_tab = ProjectTab::Files;
    }

    /// Resolve a terminal path against the active session's worktree or project root,
    /// Resolve a path found in terminal output to an existing canonical file on disk,
    /// respecting session worktree, project root, and home/tilde expansion while
    /// preventing directory traversal attempts (`..`). Caches the resolution.
    pub(crate) fn resolve_session_terminal_file(&mut self, path: &str) -> Option<PathBuf> {
        // A path that climbs out of the root is refused before it is looked
        // up, and says so: a click that does nothing reads as a broken link,
        // not as a boundary.
        if path.starts_with("..") || path.contains("/../") {
            self.notice = Some(tf!("{p0} はこのプロジェクトの外にあります。", p0 = path));
            return None;
        }
        let (project, worktree) = match self
            .selected_session
            .and_then(|id| self.store.sessions.iter().find(|session| session.id == id))
            .and_then(|session| {
                let project = self
                    .store
                    .projects
                    .iter()
                    .find(|candidate| candidate.id == session.project_id)
                    .cloned()?;
                Some((project, session.worktree_path.clone()))
            }) {
            Some(pair) => pair,
            None => {
                self.notice = Some(tr("このセッションのプロジェクトが見つかりません。").into());
                return None;
            }
        };

        if self.resolved_paths_session.is_some()
            && self.resolved_paths_session != self.selected_session
        {
            self.resolved_paths.clear();
            self.resolved_paths_generation += 1;
        }
        self.resolved_paths_session = self.selected_session;

        let resolved = match self.resolved_paths.get(path).cloned().flatten() {
            Some(r) => {
                if r.is_file() {
                    Some(r)
                } else {
                    let fresh =
                        resolve_terminal_session_path(worktree.as_deref(), &project.path, path);
                    if fresh.is_some() {
                        self.resolved_paths.insert(path.to_owned(), fresh.clone());
                        self.resolved_paths_generation += 1;
                        fresh
                    } else {
                        Some(r)
                    }
                }
            }
            None => {
                let fresh = resolve_terminal_session_path(worktree.as_deref(), &project.path, path);
                if fresh.is_some() {
                    self.resolved_paths.insert(path.to_owned(), fresh.clone());
                    self.resolved_paths_generation += 1;
                }
                fresh
            }
        };

        let Some(resolved) = resolved else {
            self.notice = Some(tf!("ファイルが見つかりません: {p0}", p0 = path));
            return None;
        };

        Some(resolved)
    }

    /// Resolve a path found in terminal output against the active session's worktree or project root,
    /// ensuring it exists and is contained within the project or worktree boundary.
    pub(crate) fn resolve_session_terminal_path(
        &mut self,
        path: &str,
    ) -> Option<(Project, PathBuf, PathBuf, PathBuf)> {
        let (project, worktree) = match self
            .selected_session
            .and_then(|id| self.store.sessions.iter().find(|session| session.id == id))
            .and_then(|session| {
                let project = self
                    .store
                    .projects
                    .iter()
                    .find(|candidate| candidate.id == session.project_id)
                    .cloned()?;
                Some((project, session.worktree_path.clone()))
            }) {
            Some(pair) => pair,
            None => {
                self.notice = Some(tr("このセッションのプロジェクトが見つかりません。").into());
                return None;
            }
        };

        let resolved = self.resolve_session_terminal_file(path)?;

        let Some((root, relative)) =
            document_root_for(&resolved, &project.path, worktree.as_deref())
        else {
            self.notice = Some(tf!(
                "{p0} はこのプロジェクトの外にあります。",
                p0 = resolved.display()
            ));
            return None;
        };

        Some((project, root, relative, resolved))
    }

    /// Do what an Option (⌥) click on an openable thing means: opens in the built-in editor.
    pub(crate) fn open_terminal_target(&mut self, kind: TerminalTargetKind) {
        match kind {
            TerminalTargetKind::Url(url) => {
                self.request_system_action(tr("ブラウザで開く"), move || open_url(&url));
            }
            TerminalTargetKind::Path { path, line } => {
                if let Some((project, root, relative, _resolved)) =
                    self.resolve_session_terminal_path(&path)
                {
                    self.navigate_to_file(&project, root, relative, line);
                }
            }
        }
    }

    /// Launch an openable terminal target externally (via macOS `open <path>` or default browser).
    pub(crate) fn open_terminal_target_external(&mut self, kind: TerminalTargetKind) {
        match kind {
            TerminalTargetKind::Url(url) => {
                self.request_system_action(tr("ブラウザで開く"), move || open_url(&url));
            }
            TerminalTargetKind::Path { path, .. } => {
                if let Some(resolved) = self.resolve_session_terminal_file(&path) {
                    self.notice = None;
                    self.request_system_action(tr("ファイルを開く"), move || {
                        open_path(&resolved)
                    });
                }
            }
        }
    }
}

/// The two things a right-click on a path offers beyond opening it.
impl OperonApp {
    pub(crate) fn ui_terminal_path_menu(&mut self, ui: &mut egui::Ui, session_id: Uuid) {
        let Some((owner, path)) = self.terminal_path_menu.clone() else {
            return;
        };
        if owner != session_id {
            return;
        }
        let mut dismissed = false;
        egui::Window::new(tr("パスの操作"))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new(path.display().to_string())
                        .size(12.0)
                        .color(self.store.theme.palette().text_muted),
                );
                if ui.button(tr("パスをコピー")).clicked() {
                    ui.ctx().copy_text(path.display().to_string());
                    self.notice = Some(tr("パスをコピーしました。").into());
                    dismissed = true;
                }
                if ui.button(tr("Finder で表示")).clicked() {
                    let reveal = path.clone();
                    self.request_system_action(tr("Finder で表示"), move || {
                        reveal_path(&reveal)
                    });
                    dismissed = true;
                }
                if ui.button(tr("閉じる")).clicked() {
                    dismissed = true;
                }
            });
        if dismissed || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.terminal_path_menu = None;
        }
    }
}
