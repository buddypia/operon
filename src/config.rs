pub(crate) const APP_NAME: &str = "Operon";
/// The prefix every managed tmux session is named with, and the only thing
/// `is_safe_tmux_name` has to recognize one by. It gates opening a terminal,
/// resizing a pane, and discovering unregistered sessions, so a name built from
/// anything else makes all three refuse a session this app itself created.
pub(crate) const MANAGED_TMUX_PREFIX: &str = "operon-";
/// What that prefix was before the rename. Sessions named under the previous
/// product name are still running on people's machines and still recorded in
/// their stores, so the gate keeps accepting them; nothing new is ever named
/// with it.
pub(crate) const MANAGED_TMUX_LEGACY_PREFIX: &str = "xirp-copy-";
/// The environment variables that name a repository, which no child Operon
/// spawns may inherit.
///
/// `current_dir` does not decide which repository git works on. These two do,
/// and they win over the working directory — so a child that inherits them is
/// aimed at whatever put them in Operon's own environment, whatever path was
/// passed beside it. Every external tool goes through
/// `run_command_with_output_limit`, and that is where they are dropped; the git
/// call sites additionally build through `git_command` so a `Command` handed
/// straight to `.output()` is covered too.
///
/// Measured, and it did damage rather than merely failing. Claude Code exports
/// both for a worktree session, so the git tests — which drive `src/git.rs` —
/// ran git against this repository instead of the temporary ones they had built:
/// `lists_main_and_linked_git_worktrees` re-initialised
/// `.git/worktrees/trust-prompt-and-readiness-gate`, and one of them committed a
/// change's staged files onto the branch as `initial`, author
/// `Operon test <test@example.invalid>`. The suite was green while it did it,
/// and only went red — 22 tests — once the review gate began refusing those
/// commits, which is the only reason anybody looked.
pub(crate) const INHERITED_REPOSITORY_POINTERS: [&str; 2] = ["GIT_DIR", "GIT_WORK_TREE"];
pub(crate) const AGENTS: &[&str] = &["codex", "claude", "gemini", "custom"];
/// `gemini` is retained as the persisted agent ID for backward compatibility.
/// New sessions use Antigravity's current CLI executable.
pub(crate) const ANTIGRAVITY_COMMAND: &str = "agy";
/// Full-history restore destinations, in the order the picker offers them.
/// Each one can create a real conversation that already holds the restored
/// turns, so every supported CLI is both a source and a destination and any
/// conversation can move to any other CLI.
pub(crate) const FULL_HISTORY_RESTORE_TARGETS: &[&str] = &["codex", "claude", "gemini"];
/// Bumped to 5 when `SessionStatus::Lost` joined the vocabulary a saved session
/// can carry. A build that predates it cannot read a record holding one, and the
/// guard in `parse_store_contents` is what says so rather than letting an older
/// build treat the file as corrupt.
pub(crate) const STORE_SCHEMA_VERSION: u32 = 5;
/// The first schema whose `Failed` records were read off an exit status.
///
/// Everything written before it called any terminal it could not find a
/// failure, so one restarted Mac turned a whole list red. Those verdicts are
/// re-judged once, on load — see `OperonApp::from_state`.
pub(crate) const EVIDENCE_BASED_FAILURE_SCHEMA_VERSION: u32 = 5;
/// How long a success toast stays before it leaves on its own. Failures are
/// banners and stay until closed.
pub(crate) const NOTICE_TOAST_SECONDS: u64 = 4;
/// The most paths the session file filter lists at once. A filter that
/// matches more is asking to be narrowed, and a draw of every match would be
/// the tree again without its folding.
pub(crate) const FILE_FILTER_RESULT_LIMIT: usize = 200;
/// The most files 「ターミナルに出たファイル」 lists above the tree.
pub(crate) const TOUCHED_FILES_LIMIT: usize = 8;
/// Minimum width for resizable workspace sidebars.
pub(crate) const SIDEBAR_MIN_W: f32 = 180.0;
/// Maximum width for resizable workspace sidebars.
pub(crate) const SIDEBAR_MAX_W: f32 = 480.0;
/// Default width for workspace sidebars.
pub(crate) const SIDEBAR_DEFAULT_W: f32 = 268.0;
pub(crate) const FILE_SCAN_VISIT_LIMIT: usize = 100_000;
pub(crate) const DIRECTORY_ENTRY_BUFFER_LIMIT: usize = 10_000;
pub(crate) const COMMAND_OUTPUT_MAX_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const COMMAND_ERROR_MAX_BYTES: usize = 256 * 1024;
pub(crate) const GIT_DIFF_MAX_BYTES: usize = 180_000;
/// How much of a file the editor opens. Past this it is shown read-only with a
/// notice.
///
/// The ceiling is a layout cost, not a memory one. A text edit is one galley:
/// egui re-lays out the whole buffer on the frame its text changes, so the file
/// size is what a keystroke costs. The diff and the preview beside it are
/// virtualised and would not care; the editable pane is the one that sets this
/// number, and 128 KiB is a very long document and about three thousand lines.
pub(crate) const EDITOR_FILE_MAX_BYTES: usize = 128 * 1024;
/// Maximum file size for image previews in the built-in editor (10 MiB).
/// Images are decoded once into an egui texture and virtualized, so memory
/// is bounded while allowing high-resolution assets generated by agents.
pub(crate) const EDITOR_IMAGE_MAX_BYTES: usize = 10 * 1024 * 1024;
/// The size every monospaced thing in this app is drawn at — the terminal, the
/// editor, and the diff. One size, so that a diff and the terminal that
/// produced it line up column for column.
pub(crate) const MONOSPACE_SIZE: f32 = 15.0;
/// How many files the project tree holds. The flat list this replaced stopped
/// at 2 000, which is a small repository — a tree is only useful if the file
/// you are looking for is in it.
pub(crate) const FILE_TREE_LIMIT: usize = 12_000;
/// How far back the in-app terminal can be scrolled. Managed sessions hold
/// 50 000 lines of tmux scrollback, so this is a budget for reading them, not
/// the limit of what was kept: every poll captures, parses, and colours this
/// many lines for the terminal on screen. The layout cost is what forces a
/// ceiling — the pane draws one label per row and only for the rows in view,
/// but laying out even 2000 rows at once costs about 150 ms, so this number
/// must not grow without keeping that viewport virtualised.
pub(crate) const TERMINAL_SCROLLBACK_LINES: usize = 5_000;
/// Claude Code's local store, and the two directories inside it Operon reads.
///
/// Spelled once for the reason `MANAGED_TMUX_PREFIX` is: five call sites built a
/// path into `projects/` by hand, and a sixth reader — the session registry —
/// was about to be the seventh place a rename would have to reach.
pub(crate) const CLAUDE_STORE_DIRECTORY: &str = ".claude";
pub(crate) const CLAUDE_PROJECTS_DIRECTORY: &str = "projects";
/// One file per live Claude Code process, named after its PID, naming the
/// conversation that process is on *now*. `/clear` starts a new conversation in
/// the same terminal under a new ID, and this entry is rewritten when it does;
/// the `--session-id` a terminal was launched with goes on naming the first one
/// for ever. Nothing in the transcripts themselves distinguishes a terminal that
/// moved on from a second terminal in the same workspace, so this registry is
/// the only local account of the rotation that is not a guess.
pub(crate) const CLAUDE_SESSION_REGISTRY_DIRECTORY: &str = "sessions";
/// A registry entry is a few hundred bytes of process metadata. The ceilings
/// bound a directory that accumulates one file per Claude Code process ever run
/// on the machine, and that Operon reads while a person waits for a restore.
pub(crate) const CLAUDE_SESSION_REGISTRY_VISIT_LIMIT: usize = 4_096;
pub(crate) const CLAUDE_SESSION_REGISTRY_FILE_MAX_BYTES: u64 = 64 * 1024;
pub(crate) const TRANSCRIPT_CANDIDATE_LIMIT: usize = 600;
pub(crate) const TRANSCRIPT_SEARCH_BYTE_LIMIT: u64 = 64 * 1024 * 1024;
pub(crate) const TRANSCRIPT_SEARCH_LINE_LIMIT: usize = 500_000;
pub(crate) const TRANSCRIPT_FILE_MAX_BYTES: u64 = 20 * 1024 * 1024;
pub(crate) const TRANSCRIPT_FILE_LINE_LIMIT: usize = 100_000;
pub(crate) const HISTORY_IMPORT_MAX_ITEMS_PER_REQUEST: usize = 64;
pub(crate) const HISTORY_IMPORT_MAX_REQUEST_BYTES: usize = 768 * 1024;
/// Antigravity's local history index keeps one line per submitted turn, so a
/// restored conversation needs an index line too. Keep its title short: the
/// index is a shared append-only file owned by `agy`.
pub(crate) const ANTIGRAVITY_HISTORY_TITLE_MAX_CHARS: usize = 200;
/// `agy` writes a plain JSONL transcript of every conversation under this name.
/// It is the readable record a restore reads, unlike the global history index.
pub(crate) const ANTIGRAVITY_TRANSCRIPT_FILE_NAME: &str = "transcript.jsonl";
/// `agy` writes one log per CLI process into this directory of its store, named
/// after the process start time. That log is the only local record naming both
/// the workspace a process opened and every conversation it created, which is
/// what the history index cannot supply for a conversation of a single turn.
pub(crate) const ANTIGRAVITY_LOG_DIRECTORY_NAME: &str = "log";
pub(crate) const ANTIGRAVITY_LOG_FILE_PREFIX: &str = "cli-";
pub(crate) const ANTIGRAVITY_LOG_FILE_SUFFIX: &str = ".log";
/// The CLI server names its workspace while starting, well inside this many
/// lines. A file that has not named one by then is not an `agy` process log, so
/// its body is never read: the log directory can hold multi-gigabyte logs of
/// unrelated workspaces.
pub(crate) const ANTIGRAVITY_LOG_HEAD_LINE_LIMIT: usize = 256;
pub(crate) const ANTIGRAVITY_LOG_VISIT_LIMIT: usize = 4_096;
pub(crate) const ANTIGRAVITY_LOG_SCAN_BYTE_LIMIT: u64 = 64 * 1024 * 1024;
/// Keep a title read from a transcript's opening request as short as the titles
/// the history index carries; a first turn can be a pasted document.
pub(crate) const ANTIGRAVITY_TRANSCRIPT_TITLE_MAX_CHARS: usize = 200;
/// Codex's own configuration key for the release check it runs before opening a
/// session. Passed per launch, never written to the user's `config.toml`.
pub(crate) const CODEX_STARTUP_OPTIONS: &str = "-c check_for_update_on_startup=false";
/// Codex wraps a standing objective in several kilobytes of its own
/// continuation instructions and sends the whole thing as a `user` message.
pub(crate) const CODEX_GOAL_CONTEXT_MARKER: &str = "<codex_internal_context source=\"goal\">";
/// Native CLIs persist a new conversation asynchronously. Retry only within
/// this bounded window, and keep the combined transcript-read budget small
/// enough that a missing record never turns into an unbounded background scan.
pub(crate) const NATIVE_RESOLUTION_DELAYS_MS: [u64; 7] = [0, 250, 750, 1_500, 3_000, 4_500, 5_000];
pub(crate) const NATIVE_RESOLUTION_SCAN_BYTES: u64 = 2 * 1024 * 1024;
pub(crate) const NATIVE_RESOLUTION_SCAN_LINES: usize = 20_000;
pub(crate) const NATIVE_RESOLUTION_SCAN_VISITED_ENTRIES: usize = 4_000;
/// Only the bottom of a managed terminal is read for the agent's run state.
/// Every supported CLI draws its interrupt hint and status line there, and the
/// limit keeps conversation text that merely quotes those words — a pasted log,
/// a transcript, this file — from being read as a running turn.
pub(crate) const ACTIVITY_STATUS_TAIL_LINES: usize = 12;
/// How many consecutive polls must agree before an activity change is acted on.
/// A single sample can catch a CLI between two redraws.
pub(crate) const ACTIVITY_CONFIRMATIONS: u8 = 2;
/// Where the agent status hooks live, inside the data directory: the managed
/// scripts each CLI runs, the endpoint file those scripts source, the socket
/// they post to, and the file holding the toggle. One directory, so removing
/// the mechanism is removing one directory.
pub(crate) const HOOK_DIRECTORY: &str = "agent-hooks";
pub(crate) const HOOK_ENDPOINT_FILE: &str = "endpoint.env";
pub(crate) const HOOK_SOCKET_FILE: &str = "hook.sock";
pub(crate) const HOOK_SETTINGS_FILE: &str = "settings.json";
/// The variables a managed tmux session carries so that a hook running inside
/// it can say which session it speaks for. Spelled once: the launch writes
/// them, the script text reads them, and the endpoint file sets the last two.
pub(crate) const HOOK_SESSION_ENV: &str = "OPERON_SESSION";
pub(crate) const HOOK_LAUNCH_ENV: &str = "OPERON_LAUNCH_TOKEN";
pub(crate) const HOOK_ENDPOINT_ENV: &str = "OPERON_HOOK_ENDPOINT";
pub(crate) const HOOK_SOCKET_ENV: &str = "OPERON_HOOK_SOCKET";
/// Antigravity passes no event name on stdin, so its managed command sets one
/// in the script's environment instead.
pub(crate) const HOOK_EVENT_ENV: &str = "OPERON_HOOK_EVENT";
/// The name of Operon's bundle inside Antigravity's hooks file, beside any
/// other tool's bundle.
pub(crate) const HOOK_ANTIGRAVITY_BUNDLE: &str = "operon-status";
/// The names of the other CLIs' stores and the settings files inside them that
/// the hook installer edits. `CLAUDE_STORE_DIRECTORY` above is the third.
pub(crate) const CLAUDE_SETTINGS_FILE: &str = "settings.json";
pub(crate) const CODEX_STORE_DIRECTORY: &str = ".codex";
pub(crate) const CODEX_HOOKS_FILE: &str = "hooks.json";
pub(crate) const CODEX_CONFIG_FILE: &str = "config.toml";
pub(crate) const GEMINI_STORE_DIRECTORY: &str = ".gemini";
pub(crate) const GEMINI_CONFIG_DIRECTORY: &str = "config";
pub(crate) const GEMINI_HOOKS_FILE: &str = "hooks.json";
/// What one hook post may carry. A Claude `Stop` payload holds the last
/// assistant message and stays well under this; anything larger is not a hook
/// event this app understands and is answered 413.
pub(crate) const HOOK_BODY_MAX_BYTES: usize = 64 * 1024;
pub(crate) const HOOK_HEADER_MAX_BYTES: usize = 8 * 1024;
/// How long a hook-reported state outranks the screen. Silence is not
/// evidence: past this the screen heuristic decides again, and the state is
/// never promoted to "done" by the clock alone.
pub(crate) const HOOK_STALE_AFTER_SECONDS: u64 = 30 * 60;
pub(crate) const HOOK_LAST_MESSAGE_MAX_CHARS: usize = 200;
/// How much of another CLI's settings file the installer will read. Every one
/// seen is a few kilobytes; a file past this is left alone with a notice.
pub(crate) const HOOK_SETTINGS_MAX_BYTES: u64 = 1024 * 1024;
/// The timeout written into each CLI's hook entry, in seconds. The script posts
/// under 1.5 s, so this only ever matters when `curl` itself hangs.
pub(crate) const HOOK_ENTRY_TIMEOUT_SECONDS: u64 = 10;
/// How many rows the quick-action palette draws. A palette is read by scanning
/// it, and a list longer than a glance is a list nobody scans — the query is
/// the way to narrow it, not the scrollbar.
pub(crate) const PALETTE_RESULT_LIMIT: usize = 12;
/// How many of the choices made inside a folded section its header names before
/// it says how many more there are. The header is one line on a screen that
/// already has three numbered steps, and a header that wraps is a header that
/// pushes the thing it heads off the screen.
pub(crate) const LAUNCH_DETAIL_SUMMARY_MAX: usize = 3;
/// How many names a worktree creation will try before it gives up: the one
/// asked for, then that name numbered `-2`, `-3`, and so on. A person racing
/// several attempts at one task is what the numbering is for; someone who has
/// reached the hundredth is doing something this loop cannot help with.
pub(crate) const WORKTREE_NAME_MAX_ATTEMPTS: usize = 100;
/// Where a repository says how a fresh checkout of it is prepared. Inside the
/// tree, so a branch that changes its own setup is set up its own way, and a
/// shell script rather than a configuration file, because the only field such a
/// file would have held is a shell script.
pub(crate) const SETUP_SCRIPT_RELATIVE_PATH: &str = ".operon/setup.sh";
/// What will be read as a setup script. A file past this is not treated as a
/// truncated script — a preview that does not match what would run is worse
/// than no preview — so it is reported and left alone.
pub(crate) const SETUP_SCRIPT_MAX_BYTES: u64 = 64 * 1024;
/// How much of the script is shown before it is approved. Enough to recognise a
/// familiar setup at a glance; the total line count is drawn beside it so the
/// preview never reads as the whole file.
pub(crate) const SETUP_SCRIPT_PREVIEW_LINES: usize = 8;
/// Where the per-project setup approvals live: beside the store rather than in
/// it, because the store is a paused surface and this is a per-machine
/// approval rather than work anybody would miss.
pub(crate) const SETUP_TRUST_FILE_NAME: &str = "setup-trust.json";
/// Where the notes a person wrote against a diff live: beside the store rather
/// than in it, because the store is a paused surface and this is a working set
/// for one machine. A review interrupted is a review finished later, so they
/// are written down rather than kept in memory.
pub(crate) const DIFF_COMMENTS_FILE_NAME: &str = "diff-comments.json";
/// Where the last used CLI launch options (model, mode, effort, flags) live:
/// beside the store rather than in it, because the store is a paused surface
/// and this is a per-machine working preference rather than a repository record.
pub(crate) const RECENT_AGENT_SETTINGS_FILE_NAME: &str = "recent-agent-settings.json";
/// Maximum byte ceiling for reading recent agent settings file.
pub(crate) const RECENT_AGENT_SETTINGS_FILE_MAX_BYTES: u64 = 64 * 1024;
/// How tall a note is in the review pane, in multiples of one monospace line.
/// The pane's rows are no longer all one height, and this is the other one.
pub(crate) const DIFF_COMMENT_ROW_LINES: f32 = 5.0;
/// One note. Long enough for a paragraph of review, short enough that a paste
/// accident does not become the message.
pub(crate) const DIFF_COMMENT_BODY_MAX_CHARS: usize = 2000;
/// What one handover may carry. A set past this is refused with its count
/// rather than truncated: half a review read as a whole one is worse than being
/// asked to send it in two goes.
pub(crate) const DIFF_COMMENT_MESSAGE_MAX_BYTES: usize = 32 * 1024;
/// What a commit-message prompt may carry. The staged patch is the part that
/// grows without limit; past this it is replaced by a sentence saying so,
/// because a patch cut off mid-hunk reads as a change that ends there.
pub(crate) const COMMIT_PROMPT_PATCH_MAX_BYTES: usize = 200 * 1024;
pub(crate) const COMMIT_PROMPT_FILES_MAX_BYTES: usize = 6 * 1024;
/// What is read back from the drafting CLI. A commit message is a paragraph;
/// anything past this is a model that misunderstood the question.
pub(crate) const COMMIT_MESSAGE_MAX_BYTES: usize = 8 * 1024;
/// How long a draft may take. Long enough for a large staged patch on a slow
/// model, short enough that a hung CLI does not hold the button for ever.
pub(crate) const COMMIT_MESSAGE_TIMEOUT_SECONDS: u64 = 120;
/// How long an AI diff review may take.
pub(crate) const AI_REVIEW_TIMEOUT_SECONDS: u64 = 45;
/// Maximum output bytes retained from an AI diff review response.
pub(crate) const AI_REVIEW_MAX_BYTES: usize = 32 * 1024;
/// How often the listening sockets are read. A dev server that has just come up
/// is worth finding within half a minute; reading the whole process table more
/// often than that costs more than the answer is worth.
pub(crate) const PORT_SCAN_INTERVAL_SECONDS: u64 = 30;
pub(crate) const PORT_SCAN_TIMEOUT_SECONDS: u64 = 8;
/// A machine with more listeners than this has something other than dev servers
/// on it, and the list stops being readable long before it stops being true.
pub(crate) const PORT_SCAN_MAX_ENTRIES: usize = 200;
pub(crate) const PORT_SCAN_MAX_BYTES: usize = 512 * 1024;
/// What reads the listening sockets. macOS ships it; a machine without it gets
/// no port line and no complaint about one.
pub(crate) const PORT_SCAN_PROGRAM: &str = "lsof";
/// How many places a query may be found before the list stops growing. A
/// one-character query against fifty thousand lines would otherwise build a
/// vector nobody could use, once per keystroke.
pub(crate) const TERMINAL_SEARCH_MAX_MATCHES: usize = 2000;
/// The event name the usage reader posts under. It is not an agent state, so it
/// never reaches the state machine; the listener routes on this name alone.
pub(crate) const STATUSLINE_EVENT: &str = "StatusLine";
pub(crate) const STATUSLINE_SCRIPT_FILE_NAME: &str = "claude-statusline.sh";
/// How often the usage reader may post, in milliseconds of the CLI's own clock.
/// A status line runs several times a second while a response streams, and the
/// number it carries moves about once a turn.
pub(crate) const STATUSLINE_MIN_POST_INTERVAL_MS: u64 = 15_000;
/// Past this, a usage reading is drawn faintly and says how old it is. It is
/// not wrong, but it is no longer news, and a percentage with no age on it
/// reads as current.
pub(crate) const STATUSLINE_STALE_AFTER_SECONDS: u64 = 30 * 60;
/// The share of a window at which its number is worth noticing.
pub(crate) const STATUSLINE_WARN_PERCENT: f32 = 80.0;
/// Records that the usage reader was installed once, so that an empty slot
/// later is read as the person having removed it rather than as a slot free to
/// fill again.
pub(crate) const STATUSLINE_INSTALL_MARKER: &str = "claude-statusline.installed";
/// The longest run of characters after a dot that will still be read as a file
/// extension. Past this it is a sentence, not a name.
pub(crate) const TERMINAL_PATH_MAX_EXTENSION: usize = 12;
/// How many unresolved paths one frame may hand to the prober. The rows on
/// screen are the only ones asked about, so this is a ceiling on a number that
/// is already small.
pub(crate) const TERMINAL_PATH_PROBE_LIMIT: usize = 64;
/// The file the keyboard is read from, beside the store and the sidecars rather
/// than in a second home under a dotfile directory of its own.
pub(crate) const KEYMAP_FILE_NAME: &str = "keybindings.json";
/// Past this a keymap is not a keymap. Seven actions is the whole vocabulary,
/// so a file this size is a file that is not what it says it is.
pub(crate) const KEYMAP_FILE_MAX_BYTES: u64 = 64 * 1024;
/// The variable each CLI reads to be told which directory holds its login,
/// settings, and history. Named once here because two places have to agree on
/// them: the launch that sets one, and the hook installer that writes into the
/// directory it names. Confirmed against the installed binaries on 2026-09-06 —
/// a `claude` run with `CLAUDE_CONFIG_DIR` pointed at an empty folder created
/// its state there, and `codex resume --help` names `$CODEX_HOME` as where a
/// profile is layered from.
///
/// Antigravity has no documented equivalent, which is why it has no row: a
/// guessed variable in a launch path is a launch that silently ignores the
/// choice a person made.
pub(crate) const CLAUDE_ACCOUNT_ENV: &str = "CLAUDE_CONFIG_DIR";
pub(crate) const CODEX_ACCOUNT_ENV: &str = "CODEX_HOME";
/// Which directories hold a login, and which session used which. Beside the
/// store rather than in it, because the store is a paused surface.
pub(crate) const AGENT_ACCOUNTS_FILE_NAME: &str = "agent-accounts.json";
/// Past this an accounts file is not an accounts file. The same ceiling the
/// keymap gets, for the same reason.
pub(crate) const AGENT_ACCOUNTS_FILE_MAX_BYTES: u64 = 64 * 1024;
pub(crate) const RUN_SCRIPT_RELATIVE_PATH: &str = ".operon/run.sh";
pub(crate) const CHECKPOINTS_REF_PREFIX: &str = "refs/operon/checkpoints/";
pub(crate) const PROMPT_TEMPLATES_RELATIVE_DIR: &str = ".operon/prompts";
