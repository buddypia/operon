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
/// Cross-CLI handoff destinations, in the order the picker offers them.
/// Each destination receives a compact handoff bundle derived from the archived
/// source transcript and launches a fresh native session with that bundle.
pub(crate) const FULL_HISTORY_RESTORE_TARGETS: &[&str] = &["codex", "claude", "gemini"];
pub(crate) const HANDOFF_BUNDLE_MAX_TURNS: usize = 8;
pub(crate) const HANDOFF_BUNDLE_TURN_MAX_CHARS: usize = 1_200;
pub(crate) const HANDOFF_BUNDLE_MAX_BYTES: usize = 6_000;
/// What a handoff bundle prints where Operon cut text to fit its budget. The
/// session reading the bundle has to tell Operon's cut from the source's own
/// words, and know the whole text is in the archived transcript (change 185).
pub(crate) const HANDOFF_SHORTENED_MARKER: &str =
    "[shortened by Operon to fit the handoff; the full text is in the transcript snapshot]";
/// A handoff carries the last few tool results of the source conversation,
/// each shortened by rule, so the receiving session knows what the tools found
/// without the bundle growing past its budget (change 193). A result past the
/// char limit keeps its first and last lines and the latest lines carrying a
/// signal word between them; the rest is counted, never silently dropped.
pub(crate) const HANDOFF_TOOL_RESULTS_MAX: usize = 3;
pub(crate) const HANDOFF_TOOL_RESULT_MAX_CHARS: usize = 1_000;
pub(crate) const HANDOFF_TOOL_RESULT_HEAD_LINES: usize = 3;
pub(crate) const HANDOFF_TOOL_RESULT_TAIL_LINES: usize = 5;
pub(crate) const HANDOFF_TOOL_RESULT_SIGNAL_LINES: usize = 6;
/// Matched case-insensitively anywhere in a line.
pub(crate) const HANDOFF_TOOL_RESULT_SIGNAL_WORDS: &[&str] =
    &["error", "fail", "panic", "fatal", "exception", "traceback"];
pub(crate) const HANDOFF_TOOL_RESULT_LINE_MAX_CHARS: usize = 150;
pub(crate) const HANDOFF_TOOL_RESULT_LINE_SHORTENED: &str = "[line shortened by Operon]";
pub(crate) const HANDOFF_TOOL_RESULTS_HEADING: &str = "Latest tool results from the source session (oldest first; the full output of each is in the transcript snapshot):";
pub(crate) const HANDOFF_BUNDLE_FILE_NAME: &str = "handoff-bundle.md";
/// The files a handoff archive holds besides its bundle, and the restore kind
/// its manifest names. The import writes them; the reuse lookup and the expiry
/// sweep recognise an archive by them (change 180), so each is spelled once.
pub(crate) const SHARED_SESSION_SNAPSHOT_FILE_NAME: &str = "source.jsonl";
pub(crate) const HANDOFF_MANIFEST_FILE_NAME: &str = "manifest.json";
pub(crate) const HANDOFF_ARCHIVE_RESTORE_KIND: &str = "compact handoff bundle";
/// Version 6 added the source fingerprint, the note, and `last_used_at`, which
/// is what makes an archive reusable. A version-5 manifest is never reused.
pub(crate) const HANDOFF_MANIFEST_SCHEMA_VERSION: u64 = 6;
/// A handoff archive is deleted once this long has passed since a handoff last
/// used it. Its bundle names the snapshot, so a continuation can still open the
/// whole transcript for this long after it starts.
pub(crate) const HANDOFF_ARCHIVE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
/// One sweep or reuse lookup inspects at most this many entries under the
/// archive root, and reads each manifest within this many bytes. An entry past
/// either is neither reused nor deleted by that pass.
pub(crate) const HANDOFF_ARCHIVE_SCAN_MAX_ENTRIES: usize = 4_096;
pub(crate) const HANDOFF_MANIFEST_MAX_BYTES: u64 = 256 * 1024;
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
/// Share of the session column the session list takes when the side panel is
/// stacked under it; the panel gets the rest.
pub(crate) const SIDEBAR_SPLIT_DEFAULT: f32 = 0.4;
/// Least height the session list shrinks to in the stacked column.
pub(crate) const SIDEBAR_SECTION_MIN_H: f32 = 120.0;
/// Least height the side panel shrinks to under the session list: its tab
/// row and frame take about 56px and its body floors at 100px.
pub(crate) const SIDEBAR_PANEL_MIN_H: f32 = 180.0;
/// Height a panel docked on the workspace's top or bottom edge starts at.
pub(crate) const DOCK_DEFAULT_H: f32 = 240.0;
/// Least height of a top or bottom edge: the side panel's own floor.
pub(crate) const DOCK_MIN_H: f32 = SIDEBAR_PANEL_MIN_H;
/// Most height a top or bottom edge is dragged to.
pub(crate) const DOCK_MAX_H: f32 = 560.0;
/// Least width the terminal keeps whatever is docked beside it.
pub(crate) const DOCK_CENTER_MIN_W: f32 = 300.0;
/// Least height the terminal keeps whatever is docked above or below it: the
/// pane's header and the terminal's own 320px floor.
pub(crate) const DOCK_CENTER_MIN_H: f32 = 360.0;
/// How far into the workspace, as a share of its span, a release still lands
/// on the nearest edge. Past it on every side is the middle, which cancels.
pub(crate) const DOCK_DROP_BAND: f32 = 0.3;
/// The rule between an edge and the terminal, or two sections on one edge.
pub(crate) const DOCK_GAP: f32 = 6.0;
/// The most terminals the workspace shows at once (change 226). Each one on
/// screen is captured on every session poll, so this also bounds that.
pub(crate) const PANE_MAX: usize = 4;
/// Least width of one terminal among several, header included.
pub(crate) const PANE_MIN_W: f32 = 320.0;
/// Least height of one terminal among several, header included.
pub(crate) const PANE_MIN_H: f32 = 200.0;
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
/// How much of a text file the editor shows read-only, past the
/// `EDITOR_FILE_MAX_BYTES` it can edit.
///
/// The viewer lays out only the rows on screen, so layout no longer sets the
/// number; memory and the one pass that splits the file into rows do. 8 MiB is
/// any report, dump, or log an agent writes for a person to read, and the
/// watch rereads an open file every two seconds, which is the other cost.
/// Past this the file is not read at all.
pub(crate) const VIEWER_FILE_MAX_BYTES: usize = 8 * 1024 * 1024;
/// Characters in one row of the read-only viewer. A longer line continues on
/// the next row: minified HTML or JSON is one line, and one row of it would be
/// a layout of the whole file and a scrollbar nobody reaches the end of.
pub(crate) const VIEWER_ROW_MAX_CHARS: usize = 400;
/// Rows the viewer highlights together. A chunk is the unit of work when a
/// row first comes on screen, so it is about one screenful: 64 rows of at
/// most `VIEWER_ROW_MAX_CHARS` is at most ~25 KiB through the highlighter.
pub(crate) const VIEWER_CHUNK_ROWS: usize = 64;
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
/// How often the terminal pane re-reads tmux after a keystroke, and for how
/// long. `send-keys` returns once the byte is in the pane, before the CLI has
/// redrawn its prompt, so the capture that follows it can miss the echo; the
/// pane keeps looking at roughly frame rate for a second after typing rather
/// than leaving the last character for the two-second session poll
/// (change 154). Only while typing: an idle pane is not captured any faster.
pub(crate) const TERMINAL_ECHO_POLL: std::time::Duration = std::time::Duration::from_millis(33);
pub(crate) const TERMINAL_ECHO_WINDOW: std::time::Duration = std::time::Duration::from_secs(1);
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
/// Antigravity's local history index keeps one line per submitted turn, so a
/// restored conversation needs an index line too. Keep its title short: the
/// index is a shared append-only file owned by `agy`.
#[cfg(test)]
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
/// The timeout written into each CLI's hook entry, in seconds. The script's own
/// post is capped at 1.5 s, so a healthy run never comes near this; what it
/// covers is the machine. At load average 100 with 25 sessions running,
/// starting `/bin/sh` alone took 11 s on the path that posts nothing, and 10 s
/// had Claude report `timed out after 10s` on every `UserPromptSubmit`. Codex
/// hashes this value into its trust entries, so changing it re-trusts ours on
/// the next install — the keys stay, the hashes are rewritten.
pub(crate) const HOOK_ENTRY_TIMEOUT_SECONDS: u64 = 60;
/// How many rows the quick-action palette draws. A palette is read by scanning
/// it, and a list longer than a glance is a list nobody scans — the query is
/// the way to narrow it, not the scrollbar.
pub(crate) const PALETTE_RESULT_LIMIT: usize = 12;
/// How many of the choices made inside a folded section its header names before
/// it says how many more there are. The header is one line on a screen that
/// already has three numbered steps, and a header that wraps is a header that
/// pushes the thing it heads off the screen.
pub(crate) const LAUNCH_DETAIL_SUMMARY_MAX: usize = 3;
/// How many characters of a typed value — the extra arguments, a conversation
/// ID — that header shows before `…`. The cap above counts entries, not width,
/// and one 1 024-character line is one entry.
pub(crate) const LAUNCH_DETAIL_SUMMARY_VALUE_MAX: usize = 24;
/// The longest typed argument line a launch takes, remembers, or loads (change
/// 156). The field stops at it, so what is saved is what loads, and a pasted
/// page cannot grow `recent-agent-settings.json` past its read cap.
pub(crate) const LAUNCH_EXTRA_ARGS_MAX_CHARS: usize = 1024;
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
/// Where the bytes of an approved setup or run script are kept, one file per
/// digest, beside the store. What runs is this copy, so an edit to the
/// repository's file after the approval runs nothing (change 209).
pub(crate) const APPROVED_SCRIPTS_DIRECTORY: &str = "approved-scripts";
/// The agent a run-script session records, which is how the menu finds the one
/// still running in a worktree (change 210).
pub(crate) const RUN_SCRIPT_AGENT: &str = "script";
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
/// Where the order a person dragged the session column into lives: beside
/// the store rather than in it, because the store is a paused surface and
/// this is a per-machine arrangement rather than work anybody would miss.
pub(crate) const SESSION_ARRANGEMENT_FILE_NAME: &str = "session-order.json";
/// The most of `session-order.json` that is read. An id is 39 bytes of
/// compact JSON, so this is some 27 000 of them, and every save lists only
/// what the store still holds.
pub(crate) const SESSION_ARRANGEMENT_FILE_MAX_BYTES: u64 = 1024 * 1024;
/// How close to the session column's top or bottom edge a drag is held for
/// the column to scroll towards that edge.
pub(crate) const COLUMN_DRAG_EDGE: f32 = 32.0;
/// How far the session column scrolls per frame while a drag is held at its
/// edge.
pub(crate) const COLUMN_DRAG_SCROLL_STEP: f32 = 12.0;
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
/// The one marker every Claude folder used to share (change 025). Change 203
/// gives each settings file its own; this one is adopted by the machine's own
/// folder on the first install after the upgrade and then removed.
pub(crate) const STATUSLINE_INSTALL_MARKER: &str = "claude-statusline.installed";
/// Records that the usage reader has been in one settings file's slot, so that
/// an empty slot there later is read as the person having removed it rather
/// than as a slot free to fill again. Named after the file, one per file.
pub(crate) const STATUSLINE_MARKER_PREFIX: &str = "claude-statusline-";
pub(crate) const STATUSLINE_MARKER_SUFFIX: &str = ".installed";
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
/// EmbeddingGemma 2, pinned (change 157). One file at one revision, with the
/// size and SHA-256 read from the Hugging Face API and re-hashed locally on
/// 2026-10-16. A download is renamed to `EMBEDDING_MODEL_FILE_NAME` only after
/// its hash matched, so nothing but these bytes is ever loaded.
pub(crate) const EMBEDDING_MODEL_REPOSITORY: &str = "ggml-org/embeddinggemma-2-GGUF";
pub(crate) const EMBEDDING_MODEL_REVISION: &str = "bfcd298762cc34d0357ece5ebdd31791a3a374d8";
pub(crate) const EMBEDDING_MODEL_FILE_NAME: &str = "embeddinggemma-2-Q8_0.gguf";
pub(crate) const EMBEDDING_MODEL_BYTES: u64 = 309_855_456;
pub(crate) const EMBEDDING_MODEL_SHA256: &str =
    "2188ac1deca4b77dffefd603c2776a9d76d9d74ec01841392982ebb840b09135";
/// What `meta.model` holds in a knowledge database, so a database built by a
/// different model is recognised and rebuilt rather than compared with this one.
pub(crate) const EMBEDDING_MODEL_ID: &str =
    "ggml-org/embeddinggemma-2-GGUF@bfcd298762cc34d0357ece5ebdd31791a3a374d8/embeddinggemma-2-Q8_0.gguf";
pub(crate) const EMBEDDING_DIMENSIONS: usize = 768;
/// The model's context, and the most tokens one locked `embed` call encodes.
/// Eight texts of 225 tokens took 0.13 s on Metal, so a query waits behind at
/// most that.
pub(crate) const EMBEDDING_CONTEXT_TOKENS: usize = 2_048;
pub(crate) const EMBEDDING_TEXTS_PER_CALL: usize = 8;
/// A loaded model holds about 400 MB; after this long unused it is dropped.
/// How often the frame looks for an idle model to unload.
pub(crate) const EMBEDDING_IDLE_CHECK_SECONDS: u64 = 60;
/// How long exit waits for a cancelled download's `curl` to be reaped: the
/// poll that kills it sleeps at most 25 ms, and a finished fetch may still be
/// hashing its 310 MB.
pub(crate) const EMBEDDING_DOWNLOAD_EXIT_WAIT_MILLIS: u64 = 3000;
pub(crate) const EMBEDDING_IDLE_UNLOAD_SECONDS: u64 = 600;
/// curl's own ceiling on one download, which also bounds an orphan left by a crash.
pub(crate) const EMBEDDING_DOWNLOAD_SECONDS: u64 = 1_800;
/// The word after the executable that makes Operon a command-line tool for
/// agents rather than a window: `Operon knowledge search "…"`.
pub(crate) const KNOWLEDGE_CLI_SUBCOMMAND: &str = "knowledge";
/// `Operon knowledge embed-golden <fixture>`, the fidelity check's entry point.
pub(crate) const KNOWLEDGE_EMBED_GOLDEN_SUBCOMMAND: &str = "embed-golden";
/// The shipped runtime's vectors stay this close to the reference ones
/// (spec Req 15); the Metal batch measured 0.9999.
pub(crate) const EMBEDDING_FIDELITY_MIN_COSINE: f32 = 0.995;
pub(crate) const KNOWLEDGE_DIRECTORY_NAME: &str = "knowledge";
/// Where judge CLIs run: outside every project, so no judge run is listed among
/// a project's conversations and no project hook fires.
pub(crate) const KNOWLEDGE_JUDGE_DIRECTORY_NAME: &str = "judge";
pub(crate) const KNOWLEDGE_DATABASE_EXTENSION: &str = "sqlite3";
pub(crate) const KNOWLEDGE_SCHEMA_VERSION: u32 = 1;
pub(crate) const KNOWLEDGE_DAILY_JUDGE_CALLS: u32 = 30;
pub(crate) const KNOWLEDGE_MAX_ENTRIES: usize = 2_000;
/// Measured 2026-10-16: paraphrases 0.907–0.927, unrelated statements ≤ 0.727.
pub(crate) const KNOWLEDGE_DEDUPE_COSINE: f32 = 0.85;
/// Operon's own restore copies a conversation into another CLI's store; within
/// this of a recorded source, an exchange is that copy, not a recurrence.
pub(crate) const KNOWLEDGE_COPY_COSINE: f32 = 0.98;
pub(crate) const KNOWLEDGE_TAU_DAYS: f64 = 30.0;
pub(crate) const KNOWLEDGE_REMOVE_FRESHNESS: f64 = 0.10;
pub(crate) const KNOWLEDGE_CANDIDATE_MIN_SCORE: f32 = -0.02;
pub(crate) const KNOWLEDGE_CANDIDATES_PER_CONVERSATION: usize = 8;
pub(crate) const KNOWLEDGE_EXCHANGE_MIN_CHARS: usize = 120;
pub(crate) const KNOWLEDGE_EXCHANGE_MAX_CHARS: usize = 4_000;
pub(crate) const KNOWLEDGE_EXCHANGES_PER_CONVERSATION: usize = 400;
/// Measured 2026-10-16, query to entry: right answers 0.746–0.897, the best
/// wrong one 0.717, an off-topic query's best 0.628.
pub(crate) const KNOWLEDGE_SEARCH_MIN_COSINE: f32 = 0.70;
pub(crate) const KNOWLEDGE_SESSION_MIN_COSINE: f32 = 0.70;
pub(crate) const KNOWLEDGE_SESSIONS_LISTED_PER_RUN: usize = 200;
pub(crate) const KNOWLEDGE_SESSIONS_PER_RUN: usize = 20;
pub(crate) const KNOWLEDGE_EMBEDDINGS_PER_RUN: usize = 1_500;
pub(crate) const KNOWLEDGE_RUN_SECONDS: u64 = 180;
/// A transcript changed within this long is still being written.
pub(crate) const KNOWLEDGE_IDLE_SECONDS: u64 = 300;
pub(crate) const KNOWLEDGE_RUN_FIRST_DELAY_SECONDS: u64 = 30;
pub(crate) const KNOWLEDGE_RUN_INTERVAL_SECONDS: u64 = 600;
pub(crate) const KNOWLEDGE_JUDGE_PROMPT_MAX_BYTES: usize = 65_536;
pub(crate) const KNOWLEDGE_JUDGE_ANSWER_MAX_BYTES: usize = 16 * 1024;
pub(crate) const KNOWLEDGE_JUDGE_SECONDS: u64 = 120;
pub(crate) const KNOWLEDGE_JUDGE_PERSON_CHARS: usize = 800;
pub(crate) const KNOWLEDGE_JUDGE_AGENT_CHARS: usize = 1_600;
pub(crate) const KNOWLEDGE_JUDGE_MAX_ENTRIES: usize = 5;
pub(crate) const KNOWLEDGE_TITLE_MAX_CHARS: usize = 40;
pub(crate) const KNOWLEDGE_BODY_MAX_CHARS: usize = 400;
/// Three unparseable answers mark a conversation `failed`; it is tried again
/// after this many days or when its transcript changes.
pub(crate) const KNOWLEDGE_FAILED_ATTEMPTS: u32 = 3;
pub(crate) const KNOWLEDGE_FAILED_RETRY_DAYS: u64 = 7;
pub(crate) const KNOWLEDGE_CLI_DEFAULT_LIMIT: usize = 5;
/// The knowledge map (change 171): neighbours kept per entry, and lines drawn.
pub(crate) const KNOWLEDGE_MAP_NEIGHBOURS: usize = 8;
pub(crate) const KNOWLEDGE_MAP_EDGES_PER_ENTRY: usize = 4;
/// Measured 2026-10-17 through the shipped runtime on entry-shaped texts (three
/// topics of four, six unrelated): same-topic pairs 0.664–0.803, median 0.755;
/// other pairs median 0.664, 95th percentile 0.710. At 0.72 for both, eleven
/// of twelve topic entries fall in their topic's group and all six unrelated
/// ones stay ungrouped; at a 0.70 baseline four unrelated ones were absorbed.
pub(crate) const KNOWLEDGE_MAP_EDGE_MIN_COSINE: f32 = 0.72;
pub(crate) const KNOWLEDGE_MAP_GROUP_BASELINE: f32 = 0.72;
pub(crate) const KNOWLEDGE_MAP_MIN_GROUP: usize = 3;
pub(crate) const KNOWLEDGE_MAP_GROUP_LABEL_CHARS: usize = 16;
pub(crate) const KNOWLEDGE_MAP_ITERATIONS: usize = 300;
pub(crate) const KNOWLEDGE_MAP_SETTLE_ITERATIONS: usize = 60;
/// While settling, a placed entry moves at most this share of the map's extent.
pub(crate) const KNOWLEDGE_MAP_SETTLE_MAX_SHIFT: f32 = 0.05;
pub(crate) const KNOWLEDGE_MAP_MAX_LABELS: usize = 60;
/// Release-build layout time at `KNOWLEDGE_MAX_ENTRIES` (spec Req 9); measured
/// 0.52 s on 2026-10-17. Only the test reads it.
#[cfg(test)]
pub(crate) const KNOWLEDGE_MAP_BUDGET_SECONDS: u64 = 5;
/// Below this content width the panel goes under the map.
pub(crate) const KNOWLEDGE_MAP_NARROW_WIDTH: f32 = 700.0;
pub(crate) const KNOWLEDGE_MAP_HEIGHT: f32 = 420.0;
pub(crate) const KNOWLEDGE_MAP_NARROW_HEIGHT: f32 = 360.0;
pub(crate) const KNOWLEDGE_MAP_PANEL_WIDTH: f32 = 290.0;
pub(crate) const KNOWLEDGE_MAP_MIN_RADIUS: f32 = 3.0;
pub(crate) const KNOWLEDGE_MAP_MAX_RADIUS: f32 = 9.0;
pub(crate) const KNOWLEDGE_MAP_MIN_OPACITY: f32 = 0.25;
pub(crate) const KNOWLEDGE_MAP_DIMMED_OPACITY: f32 = 0.15;
/// Entry titles appear once the view is this many times closer than the fit.
pub(crate) const KNOWLEDGE_MAP_LABEL_ZOOM: f32 = 1.8;
/// The directory under the app data directory that holds the model file.
pub(crate) const EMBEDDING_MODEL_DIRECTORY_NAME: &str = "models";
