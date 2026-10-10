use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Project {
    pub(crate) id: Uuid,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) added_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Session {
    pub(crate) id: Uuid,
    pub(crate) project_id: Uuid,
    pub(crate) name: String,
    pub(crate) goal: String,
    pub(crate) agent: String,
    pub(crate) tmux_name: String,
    pub(crate) created_at: u64,
    /// When this terminal's agent process was last started, which is not when
    /// the record was created: a queued session waits for its dependencies and
    /// a failed one is retried later. Recovering the conversation a CLI created
    /// for this terminal needs the launch, not the record. `None` on records
    /// written before this was kept.
    #[serde(default)]
    pub(crate) launched_at: Option<u64>,
    pub(crate) status: SessionStatus,
    #[serde(default)]
    pub(crate) worktree_path: Option<PathBuf>,
    #[serde(default)]
    pub(crate) branch: Option<String>,
    #[serde(default)]
    pub(crate) agent_command: String,
    /// The provider-owned conversation created for this managed terminal.
    ///
    /// This is intentionally separate from Operon's record UUID: the CLI
    /// owns the conversation and its resume contract. Once captured, later
    /// launches reopen the destination's restored native conversation.
    #[serde(default)]
    pub(crate) native_session_id: Option<String>,
    #[serde(default)]
    pub(crate) native_session_path: Option<PathBuf>,
    /// How this terminal came to exist, when it did not start as a plain
    /// launch. `None` on records written before this was kept, and on ordinary
    /// launches, where there is nothing extra to say.
    #[serde(default)]
    pub(crate) origin: Option<SessionOrigin>,
    #[serde(default)]
    pub(crate) depends_on: Vec<Uuid>,
}

/// What the agent inside a managed terminal is doing right now.
///
/// A pane stays alive for the whole conversation, so pane death only reports
/// that the CLI quit. Finishing a turn is instead visible on screen: every
/// supported CLI prints an interrupt hint while a turn runs and drops it once
/// the turn is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentActivity {
    Working,
    AwaitingInput,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivityNotice {
    Finished,
    NeedsInput,
}

/// How a managed terminal came to exist, when it was not a plain launch.
///
/// The list shows what a conversation is about, not how it was created, so this
/// is not drawn on its own. It is the fallback title for a session whose
/// conversation has no request to quote, and it stays on the record either way:
/// it is persisted history, and dropping the field would rewrite the user's own
/// data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum SessionOrigin {
    /// Every recorded turn of another CLI's conversation was copied in.
    Restored { from: CliProvider },
    /// The provider's own conversation was reopened with its native resume ID.
    Resumed { from: CliProvider },
    /// A tmux session Operon did not launch, adopted after a restart.
    Adopted,
    /// A terminal opened without an agent.
    PlainTerminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TmuxState {
    Alive,
    Dead,
    Gone,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TmuxObservation {
    pub(crate) state: TmuxState,
    pub(crate) exit_status: Option<i32>,
}

/// A managed terminal's activity as it has settled after debouncing, plus the
/// observation currently being confirmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Worktree {
    pub(crate) path: PathBuf,
    pub(crate) branch: Option<String>,
    pub(crate) is_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrphanedTmuxSession {
    pub(crate) name: String,
    pub(crate) created_at: u64,
    pub(crate) cwd: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PullRequest {
    pub(crate) number: u64,
    pub(crate) title: String,
    #[serde(rename = "headRefName")]
    pub(crate) head_ref_name: String,
    #[serde(rename = "isDraft")]
    pub(crate) is_draft: bool,
    pub(crate) url: String,
    #[serde(rename = "reviewDecision")]
    pub(crate) review_decision: Option<String>,
    #[serde(rename = "statusCheckRollup", default)]
    pub(crate) checks: Vec<PrCheck>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PrCheck {
    #[serde(default)]
    pub(crate) conclusion: Option<String>,
    #[serde(default)]
    pub(crate) status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChangedFile {
    pub(crate) status: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct GitChangesSnapshot {
    pub(crate) status: String,
    pub(crate) files: Vec<ChangedFile>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FileScanSnapshot {
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) truncated: bool,
    pub(crate) unreadable_directories: usize,
}

/// A node of the project tree. The scan behind it returns file paths only, so
/// the directories here are the ones those paths imply: a directory this app
/// cannot see inside, or one holding nothing but ignored files, has no node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileTreeNode {
    /// The last component, which is what the row shows.
    pub(crate) name: String,
    /// The whole path, relative to the project root. This is what a document,
    /// a git status, and a diff are all keyed by.
    pub(crate) path: PathBuf,
    /// Empty for a file. A directory always has at least one child, or the
    /// scan would not have implied it.
    pub(crate) children: Vec<FileTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum SessionStatus {
    Starting,
    Active,
    Queued,
    Exited,
    Failed,
    Cancelled,
    /// The terminal is gone and this app never saw how it ended.
    ///
    /// Managed panes are launched with `remain-on-exit on`, so an agent that
    /// finishes on its own leaves a dead pane carrying its exit status behind
    /// it — that is what `Exited` and `Failed` are read from. A session can
    /// therefore only vanish outright when something outside this app removed
    /// it: the machine restarted, the tmux server was killed, the user ran
    /// `tmux kill-session` by hand. None of those is the agent failing, and
    /// none of them is the agent succeeding either.
    Lost,
    Unknown,
}

/// How a file changed, and why it is being shown that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffFileChange {
    Added,
    Removed,
    Renamed,
    Modified,
}

/// What one tmux observation is allowed to say about a session's state.
///
/// Only an exit status the app actually read is grounds for a verdict: `0` is
/// `Exited` and anything else is `Failed`. A pane that is dead without a status,
/// or a session that is not there at all, ends as `Lost` — the run is over and
/// the outcome was never observed. Calling either of those a failure puts a red
/// mark on work that most likely finished, which is what happens to every
/// session on the list after a reboot takes the tmux server with it.
pub(crate) fn session_status_after_observation(
    current: &SessionStatus,
    observation: TmuxObservation,
) -> Option<SessionStatus> {
    if !matches!(
        current,
        SessionStatus::Starting | SessionStatus::Active | SessionStatus::Unknown
    ) {
        return None;
    }
    match (observation.state, observation.exit_status) {
        (TmuxState::Alive, _) => {
            (*current != SessionStatus::Active).then_some(SessionStatus::Active)
        }
        (TmuxState::Dead, Some(0)) => Some(SessionStatus::Exited),
        (TmuxState::Dead, Some(_)) => Some(SessionStatus::Failed),
        (TmuxState::Dead, None) | (TmuxState::Gone, _) => Some(SessionStatus::Lost),
        (TmuxState::Unknown, _) => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetryTmuxAction {
    Ready,
    RemoveDead,
    RefuseAlive,
    RefuseUnknown,
}

/// A keystroke or committed text fragment destined for the active tmux pane.
/// Keeping text distinct from named keys is important: tmux's literal mode
/// preserves Japanese/IME text and punctuation, while named keys preserve TUI
/// controls such as arrows, Tab, and Ctrl+C.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalInput {
    Text(String),
    Key(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalCapture {
    pub(crate) output: String,
    pub(crate) cursor: Option<(u16, u16)>,
}

/// How a file changed, and why it is being shown that way.
/// What one tmux observation is allowed to say about a session's state.
///
/// Only an exit status the app actually read is grounds for a verdict: `0` is
/// `Exited` and anything else is `Failed`. A pane that is dead without a status,
/// or a session that is not there at all, ends as `Lost` — the run is over and
/// the outcome was never observed. Calling either of those a failure puts a red
/// mark on work that most likely finished, which is what happens to every
/// session on the list after a reboot takes the tmux server with it.
/// What the editor read when it opened a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileOpenOutcome {
    Text(String),
    Image {
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    },
    /// The file is there and this pane cannot edit it — it is binary, or past
    /// `EDITOR_FILE_MAX_BYTES`. The reason is shown in place of the content.
    Unopenable(String),
}

/// A file open in the editor.
///
/// `disk` is the whole point of this struct. Agents in this app write to the
/// same working tree a person is reading, so "what I loaded" and "what is there
/// now" are routinely different, and a save that did not check would delete an
/// agent's work without saying so. It is also what the unsaved-changes mark and
/// the editor's own diff are computed from.
/// The three ways the editor shows the file it has open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorView {
    Edit,
    /// Markdown, rendered. The reason this app has an editor at all is that
    /// agents write Markdown — plans, notes, reviews — and reading it as
    /// source is reading it in the wrong format.
    Preview,
    /// This file's diff against the last commit, which is the question
    /// "what did the agent just do to this file" asked one file at a time.
    Diff,
}

impl EditorView {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Edit => tr("編集"),
            Self::Preview => tr("プレビュー"),
            Self::Diff => tr("差分"),
        }
    }
    pub(crate) fn icon(self) -> &'static str {
        match self {
            Self::Edit => ICON_EDIT,
            Self::Preview => ICON_PREVIEW,
            Self::Diff => ICON_DIFF,
        }
    }
}

/// What one line of a unified diff is, decided once while the diff is parsed so
/// that the drawing code never looks at a leading character again. Reviewing an
/// agent's work is the reason this app exists, and a diff drawn as one block of
/// uncoloured monospace is a diff nobody reads.
/// One line of a hunk, already split from its marker and numbered against both
/// sides of the change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffLine {
    pub(crate) kind: DiffLineKind,
    /// Where the line sits in the file before and after the change. A line
    /// exists on only one side of an addition or a removal, so the gutter
    /// leaves the other number blank rather than guessing at one.
    pub(crate) old_number: Option<usize>,
    pub(crate) new_number: Option<usize>,
    /// The line without its `+`, `-`, or leading space. A hunk header and a
    /// note keep their text as git wrote it.
    pub(crate) text: String,
    /// Byte ranges into `text` that differ from the line this one replaced.
    ///
    /// Empty for every line that is not half of a rewrite: a context line, a
    /// hunk header, a note, and an addition or removal with no counterpart all
    /// carry none. Filled while the diff is parsed, never while it is drawn —
    /// the pane lays out its visible rows every frame and the diff behind them
    /// does not change between two of them.
    pub(crate) highlights: Vec<std::ops::Range<usize>>,
}

/// One file's worth of a unified diff. `git diff` writes every file into one
/// stream; splitting it back apart is what lets the review screen show a list
/// of files with a count beside each one instead of a wall of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffFile {
    /// What to call this file: the path it has after the change, or the one it
    /// had where the change deletes it.
    pub(crate) path: String,
    /// The name it had, where the change renames it.
    pub(crate) renamed_from: Option<String>,
    pub(crate) change: DiffFileChange,
    pub(crate) lines: Vec<DiffLine>,
    pub(crate) added: usize,
    pub(crate) removed: usize,
    /// The longest line in characters, which is how far the review pane has to
    /// let a person scroll sideways. Measured while parsing: doing it while
    /// drawing would mean walking every line of a 180 KiB diff on every frame.
    pub(crate) widest: usize,
    /// Set where git described the change instead of showing it — a binary
    /// file, or a mode change with no content behind it.
    pub(crate) note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffLineKind {
    /// An `@@ -a,b +c,d @@` line. Its text keeps the section name git puts
    /// after the second `@@`, which is usually the enclosing function.
    Hunk,
    Added,
    Removed,
    Context,
    /// `\ No newline at end of file`, and anything else git writes that is
    /// about the diff rather than in it.
    Note,
}

/// One line of a hunk, already split from its marker and numbered against both
/// sides of the change.
/// One file's worth of a unified diff. `git diff` writes every file into one
/// stream; splitting it back apart is what lets the review screen show a list
/// of files with a count beside each one instead of a wall of text.
pub(crate) fn dependencies_finished(sessions: &[Session], dependencies: &[Uuid]) -> bool {
    !dependencies.is_empty()
        && dependencies.iter().all(|dependency| {
            sessions
                .iter()
                .find(|session| session.id == *dependency)
                .is_some_and(|session| session.status == SessionStatus::Exited)
        })
}

pub(crate) fn ready_queued_session_ids(sessions: &[Session]) -> Vec<Uuid> {
    sessions
        .iter()
        .filter(|session| {
            session.status == SessionStatus::Queued
                && dependencies_finished(sessions, &session.depends_on)
        })
        .map(|session| session.id)
        .collect()
}

/// Why a queued session will never start.
#[derive(Debug, Clone, Copy)]
pub(crate) enum BlockingDependency<'a> {
    /// A prerequisite that ended in a state it cannot leave.
    Ended(&'a Session),
    /// A prerequisite whose session record is gone, so its outcome is
    /// unknowable rather than merely bad.
    Missing,
}

/// The first prerequisite that will not finish, if there is one. Which
/// dependency states count as "will not finish" is judged here and nowhere
/// else; `blocking_dependency_label` is this with a name attached.
///
/// The predicate is separate from the label because the session list counts
/// blocked sessions on every frame to fill in a filter chip, and a count that
/// cloned a name to find out it was non-empty would be allocating in the draw
/// path — see `.claude/rules/rust.md`.
pub(crate) fn blocking_dependency<'a>(
    sessions: &'a [Session],
    session: &Session,
) -> Option<BlockingDependency<'a>> {
    session.depends_on.iter().find_map(|dependency| {
        match sessions
            .iter()
            .find(|candidate| candidate.id == *dependency)
        {
            Some(candidate)
                if matches!(
                    candidate.status,
                    SessionStatus::Failed | SessionStatus::Cancelled | SessionStatus::Lost
                ) =>
            {
                Some(BlockingDependency::Ended(candidate))
            }
            None => Some(BlockingDependency::Missing),
            _ => None,
        }
    })
}

pub(crate) fn blocking_dependency_label(sessions: &[Session], session: &Session) -> Option<String> {
    blocking_dependency(sessions, session).map(|blocking| match blocking {
        BlockingDependency::Ended(candidate) => candidate.name.clone(),
        BlockingDependency::Missing => tr("セッション記録がありません").to_owned(),
    })
}

pub(crate) fn queued_dependency_count(sessions: &[Session], dependency: Uuid) -> usize {
    sessions
        .iter()
        .filter(|session| {
            session.status == SessionStatus::Queued && session.depends_on.contains(&dependency)
        })
        .count()
}

pub(crate) fn worktree_has_reserved_session(sessions: &[Session], path: &Path) -> bool {
    sessions.iter().any(|session| {
        session_reserves_its_directory(&session.status)
            && session.worktree_path.as_deref() == Some(path)
    })
}

/// The states in which a session may still write the directory it runs in.
fn session_reserves_its_directory(status: &SessionStatus) -> bool {
    matches!(
        status,
        SessionStatus::Starting
            | SessionStatus::Active
            | SessionStatus::Queued
            | SessionStatus::Unknown
    )
}

/// Whether a session stands in the way of landing `worktree` into its project.
///
/// The merge reads the worktree's branch and writes the project checkout, so a
/// session holding either counts: one in the worktree, or one of the project
/// with no worktree of its own, which runs in the project checkout.
pub(crate) fn landing_blocked_by_session(
    sessions: &[Session],
    project_id: Uuid,
    worktree: &Path,
) -> bool {
    worktree_has_reserved_session(sessions, worktree)
        || sessions.iter().any(|session| {
            session_reserves_its_directory(&session.status)
                && session.project_id == project_id
                && session.worktree_path.is_none()
        })
}

impl FileTreeNode {
    pub(crate) fn is_directory(&self) -> bool {
        !self.children.is_empty()
    }
}

/// A managed terminal's activity as it has settled after debouncing, plus the
/// observation currently being confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ActivityTracker {
    pub(crate) settled: Option<AgentActivity>,
    pub(crate) candidate: Option<(AgentActivity, u8)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LandResult {
    pub(crate) mainline_branch: String,
    pub(crate) worktree_branch: String,
    pub(crate) commits_merged: usize,
    pub(crate) cleaned_up_worktree: bool,
    pub(crate) deleted_branch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Checkpoint {
    pub(crate) commit: String,
    pub(crate) turn_index: usize,
    pub(crate) timestamp: u64,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalEditor {
    VsCode,
    Cursor,
    Zed,
    Finder,
}

impl ExternalEditor {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::VsCode => "VS Code",
            Self::Cursor => "Cursor",
            Self::Zed => "Zed",
            Self::Finder => "Finder",
        }
    }

    pub(crate) fn command(self) -> &'static str {
        match self {
            Self::VsCode => "code",
            Self::Cursor => "cursor",
            Self::Zed => "zed",
            Self::Finder => "open",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct GitHubIssue {
    pub(crate) number: u64,
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) body: String,
    #[serde(default)]
    pub(crate) labels: Vec<GitHubIssueLabel>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct GitHubIssueLabel {
    pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptTemplate {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) is_custom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SearchEngineMode {
    #[default]
    Keyword,
    EmbeddedGemma2,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub(crate) enum EmbeddedModelDownloadState {
    #[default]
    Idle,
    Downloading {
        progress: f32,
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    Ready {
        cached_bytes: u64,
    },
    Error(String),
}
