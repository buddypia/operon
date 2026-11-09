# Spec: a relaunch brings back the sessions the person had, and a terminal takes the kind of the CLI running in it

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: when a CLI's status hook reports a conversation ID from a session that
   began as a plain or recovered terminal (`origin` `PlainTerminal` or
   `Adopted`, or a legacy record whose kind is `terminal` / `existing terminal`),
   the session's kind becomes that CLI (`claude`, `codex`, `gemini`), its
   plain-terminal description (`goal`) is emptied, its origin is kept (and
   written as `PlainTerminal` / `Adopted` when the record had none), and the
   store is written. A later report from a different CLI in the same session
   moves the kind again. A session launched as a CLI never changes kind,
   whatever reports from it. The kind changes even when the reported ID is the
   one already recorded. `a_terminal_takes_the_kind_of_the_cli_running_in_it`.
2. R2: such a session is an agent wherever the app asks: review notes can be
   sent to it, and the error-explain action types into it rather than looking
   for another agent. `a_terminal_running_claude_receives_review_notes`.
3. R3: the workspace — for every project: its tab groups and their session tabs
   in order, each group's front tab, the split layout and its shares, the
   focused group, and the selected session; plus which project the workspace
   shows and which sessions have already been placed — is written to
   `workspace-layout.json` beside the store within one two-second tick of a
   change, and when Operon quits, and is read back when Operon starts, so the
   same tabs are on screen after a relaunch. Nothing is written before the
   workspace has been laid out once, and nothing when it did not change.
   `the_workspace_comes_back_after_a_relaunch`.
4. R4: a session tab the person closed while its session kept running is not
   reopened by the relaunch; a session that started after the last write (none
   can, but a store edited by another build can hold one) is added once, as
   today. Covered by `the_workspace_comes_back_after_a_relaunch`.
5. R5: a missing, unreadable, oversized (over 1 MiB), or malformed file, or one
   naming projects or sessions the store no longer holds, never produces a tab
   group without a pane, a pane without a group, a session in two groups, more
   than `PANE_MAX` panes, a share outside 0..1, or a front-tab index outside its
   group: what remains valid is restored, a project whose saved layout cannot
   be made whole falls back to today's behaviour, and a file that cannot be read
   at all is the same as no file. `a_damaged_workspace_file_restores_only_what_is_whole`.
6. R6: at launch, a session that was running when Operon last closed (stored as
   `Starting`, `Active`, or `Unknown`) and whose terminal the first look finds
   gone — no terminal or a pane dead without a status (`Lost`), or a pane whose
   CLI died with a non-zero status (`Failed`) — is started again in place — the same record, the same tab, the
   same tmux name — without moving the selected tab or the page: a CLI session
   with a saved conversation ID reopens that conversation with the options it
   was launched with (the existing resume command and transcript validation),
   and a plain terminal opens a shell in its folder. The launch's completion
   does not select the session, change the page, focus its terminal, or replace
   the notice either. Not started again: a session the first look finds alive,
   one whose CLI exited cleanly (`Exited`, status 0: the person's `/exit`), a
   cancelled one, one already stopped before the quit, a CLI
   session without a conversation ID or whose transcript is empty or missing (it
   keeps its manual 「再試行」/「再開」), and a custom command.
   `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`.
7. R7: when at least one session was started again, a brief notice says how
   many: 「前回開いていたセッションを {count} 件再開しました。」. A session
   whose relaunch fails ends `Failed` with the existing 「セッションを開始できませんでした: …」
   notice and no launch dialog. `the_relaunch_says_how_many_sessions_it_reopened`.
8. R8: the manual 「再開」 of a stopped CLI session behaves as before — a new
   record resuming the conversation — and a transcript that was never written
   now starts the session fresh instead of re-entering the resume (which
   recursed until the stack overflowed). `resuming_a_session_with_no_transcript_starts_it_fresh_once`.

## Behaviour

- A terminal in which the person runs `claude` shows the Claude mark on its tab
  and 「Claude」 in the left column from the first hook event that carries a
  conversation ID (Claude sends one on start-up). Its row starts showing
  「作業中」/「要対応」 like any agent. The name is unchanged (「セッション3」).
- Relaunch with the CLIs still alive in tmux (the usual quit): the same
  project, the same tabs in the same groups and splits, the same front tabs,
  the same selected session. Nothing is started; no notice.
- Relaunch after the tmux server is gone (restart): the same tabs; within the
  first two-second poll each session that was running shows 「起動中」, then
  「実行中」 with its CLI resuming the previous conversation, and a toast reads
  「前回開いていたセッションを 3 件再開しました。」. The selected tab stays.
- Empty: no file, or no project yet — today's behaviour (the live sessions of
  the first project open as tabs). A session deleted while Operon was closed
  is simply absent from its group; a group left empty disappears and its pane
  closes the way closing its last tab does.
- Error: a file that cannot be written is retried at the next change; the
  first failure in a row shows 「タブの配置を保存できませんでした: {error}」, later
  failures in the same streak stay quiet, and a success ends the streak. A
  file that cannot be read is ignored silently, like `session-order.json`.
- Too large: past 1 MiB the file is not read (as `session-order.json`).
- Interrupted: the file is replaced atomically; a quit while it is being written
  leaves the previous file or the new one, never a torn one. A relaunch
  interrupted by a second quit leaves each session `Starting`, `Active`, or
  `Failed` as the existing launch path does; the next launch judges `Starting`
  ones again.

## Design

Placement: `EXTEND`. The kind change extends the hook intake
(`adopt_native_identity` in `src/app.rs`); the layout extends the workspace
model (`WorkspaceTabs`, `PaneNode` in `src/ui/session_tree.rs`) with the same
beside-the-store file pattern as `SessionArrangement` (change 181); the
relaunch extends the session poll and the start path in `src/app.rs`.

- `src/app.rs`
  - `adopt_native_identity`: before the "same ID" early return, a session
    whose provenance is a terminal (a new helper `terminal_provenance(session)`
    in `src/app.rs`: `origin` `PlainTerminal`/`Adopted`, else what
    `legacy_session_origin` in `src/agents.rs` derives from a legacy record) and whose kind differs from
    `reading.source.agent()` takes that kind, an emptied goal, and its
    provenance written into `origin`; persisted with the existing rollback.
  - `diff_comment_target` and the error-explain target in
    `src/app/screens.rs`: drop the `origin != PlainTerminal` clause; the kind
    test (`agent != "terminal"`) is the one that says "agent".
  - `start_session` splits: the retry-with-conversation branch stays, and the
    rest moves to `launch_session_record(id, LaunchFocus)`. `LaunchFocus::Bring`
    is today's behaviour; `LaunchFocus::Keep` does not select the session,
    change the page, set `launch_progress`, or write the 「起動しています」 notice,
    and records the id in a new `quiet_launches: HashSet<Uuid>`. The
    `SessionStarted` handler takes the id out of that set: for a quiet launch it
    skips its select/page/focus block, and puts back the notice that was showing
    when `persist_external_transition` replaced it with its success sentence
    (a durability warning or a write error it sets is left showing). Its error
    arm is unchanged, and finds no `launch_progress` for a quiet launch, so no
    launch dialog opens.
  - `resume_managed_native_session` keeps its behaviour but takes its
    validation (safe ID, tools on `PATH`, Claude identity correction, transcript
    readiness) from a new `prepare_native_resume(&Session, CliProvider)`,
    which the relaunch also uses. Its `EmptyOrMissing` arm calls
    `launch_session_record(source_id, LaunchFocus::Bring)` instead of
    `start_session`, which re-entered this function forever (R8).
  - New field `relaunch_candidates: HashSet<Uuid>`, filled in `from_state` with
    the sessions stored as `Starting`/`Active`/`Unknown`. `apply_session_poll`
    takes each one out at its first decisive observation (`Alive`, `Dead`,
    `Gone`); the ones that ended `Lost` or `Failed` go to `relaunch_remembered_sessions`,
    which calls `relaunch_lost_session(id) -> bool` for each and writes the R7
    notice. `relaunch_lost_session`: a plain terminal (`terminal` with an empty
    command) goes straight to `launch_session_record(id, Keep)`; a CLI session
    with a conversation ID goes through `prepare_native_resume`, writes
    `agent_command = resume_command_with_launch_options(...)` and the checked ID
    and transcript path into the same record, persists, and launches with
    `Keep`; anything else returns `false`.
  - New fields `saved_workspace_layout: Option<WorkspaceLayout>` and
    `workspace_layout_save_failed: bool`. `from_state` reads the file and calls
    `restore_workspace_layout`; the two-second tick in `update` and `on_exit`
    call `save_workspace_layout_if_changed`, which builds
    `workspace_layout_snapshot()` (projects and placed sessions sorted so the
    comparison does not depend on `HashMap` order) and writes only when it
    differs from the last written one.
- `src/ui/session_tree.rs`: `PaneNode` derives `Serialize`/`Deserialize`;
  new `WorkspaceLayout { project, workspaces: Vec<SavedWorkspace>, placed }`,
  `SavedWorkspace { project, groups: Vec<SavedGroup>, focused, layout, selected }`,
  `SavedGroup { id, sessions, active }`; `WorkspaceTabs::to_saved(project,
  selected)` (session tabs only; a group's front tab mapped to its position
  among them; groups left without a session dropped and their panes retained
  out), `WorkspaceTabs::from_saved(&SavedWorkspace, exists) -> Option<Self>`
  (the R5 checks; `None` when the layout's panes are not exactly its groups),
  `workspace_layout_path`, `load_workspace_layout`, `save_workspace_layout`
  (compact JSON through `write_file_atomically`, as `save_session_arrangement`).
- `src/config.rs`: `WORKSPACE_LAYOUT_FILE_NAME` (`workspace-layout.json`) and
  `WORKSPACE_LAYOUT_FILE_MAX_BYTES` (1 MiB), documented beside
  `SESSION_ARRANGEMENT_FILE_NAME`.
- `src/i18n_tables.rs`: English and Korean rows for the two new sentences.
- `README.md`, `README.ja.md`, `README.ko.md`: one bullet after the
  `session-order.json` one.
- Persisted shape: the store is not touched — `Session` fields keep their types;
  only values change (`agent`, `goal`, `origin`, `agent_command`, IDs), through
  the existing `persist`. The new file is new; nothing migrates.
- External commands: none new. The relaunch runs the existing
  `launch_stored_session` on the existing background key
  `BackgroundKey::SessionStart(id)`; the commands are the ones the manual retry
  and resume build and gate today.
- Draw path: nothing new per frame. The snapshot is built every two seconds on
  the existing tick (a few small `Vec`s); the file is read once at start, and
  written only on change and at quit. `prepare_native_resume` reads the
  transcript on the UI thread once per relaunched session at start-up, which is
  what one click of 「再開」 does today.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | Nothing is painted that is not painted today; the tab mark and label come from the session's kind through the existing helpers. |
| Icons | No | No glyph or mark is added; the CLI marks of change 267 are reused. |
| Identifier SSOT | Yes | The file name and its ceiling are constants in `src/config.rs`; the CLI kind written into a record is `CliProvider::agent()`, the same value the launch path writes, and the R1 test reads it through `cli_provider_for_agent` (round trip), not a literal. |
| Durability | Yes | The store's shape and write order are unchanged; record changes go through `persist` with the existing rollback. The new file is a per-machine preference written with `write_file_atomically` (temp file in the same folder, file sync, rename, folder sync); `CommittedButNotSynced` counts as written, as for `session-order.json`. Read through `take(max + 1)` on the open handle, not `metadata`. No cancellation, migration, or tmux stop/retry code changes; the relaunch calls the existing retry preparation. |
| Subprocess safety | Yes | No new `Command`; the relaunch reaches tmux only through `launch_stored_session`, whose command passes `stored_launch_command`'s `is_safe_agent_command` gate, and a resume command only through `resume_command_with_launch_options`, which falls back to the bare resume when the carried line fails the gate. `src/agents.rs`, `src/tmux.rs`, `src/exec.rs` are not edited. |
| Documentation | Yes | The README bullet in English, Japanese, and Korean together. |
| Local-first | Yes | A local file beside the store; no network, telemetry, account, or `unsafe`. |
| Budgets | Yes | The file read stops at 1 MiB + 1 byte and is then ignored; the relaunch is bounded by the sessions in the store, each one launch on its own background key. |
| Draw path — `.claude/rules/rust.md` | Yes | See Design: no I/O or subprocess per frame; snapshot on the two-second tick only. |
| Transcripts — `.claude/rules/transcripts.md` | Yes | No terminal's conversation is inferred: the kind follows the ID the CLI's own hook reported, and resume uses the existing identity correction. |

## Flagged concerns

None.

## Acceptance

R1–R8 tests pass and each is watched failing first; a mutation sweep catches a
terminal that keeps its kind, a launched CLI that changes kind, a closed tab
reopened, a damaged pane tree accepted, an exited session started again, a
relaunch that moves the selected tab or whose completion steals the notice, a
crashed CLI left stopped, a missing notice, and the recursion put
back. `make q.fast` passes. CI passes before merging into `main`. After landing,
package from `main` and replace the installed app per `.claude/skills/ship/SKILL.md`.

## Rejected alternatives

- Showing the CLI only while it is known to run (in-memory): the person chose
  the persistent switch, and an in-memory mark would fall back to the terminal
  mark after every relaunch until the CLI spoke again.
- Resuming through the existing 「再開」 path, which creates a new record per
  resume: every restart would leave one stopped copy of each session behind,
  and its tab would point at the stopped copy.
- Relaunching every `Lost` session in the store: sessions lost weeks ago that
  the person left alone would all start; only those running at the last quit
  were "the sessions they had".
- Putting the layout in the store: a paused surface and a schema bump for a
  per-machine preference whose loss costs one rearrangement.
- Writing the layout on every frame that changes it: the change points are
  scattered over drag, close, split, and project switch; the two-second tick
  catches all of them with one comparison.
