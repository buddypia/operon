# Spec: Launching a CLI opens its terminal screen immediately without a blocking progress modal

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Calling `launch_session`, `launch_empty_session`, or `start_session` in
   `src/app.rs` immediately selects the launched session (`self.select_session`),
   switches to `Page::Sessions`, closes `self.session_library_open`, and records
   `self.launch_progress` with `dismissed: true` so the full-window in-progress
   `ui_launch_modal` (`{agent} セッションを起動しています…`) does not cover the
   starting terminal screen or block `ui_launch_sheet`. Test:
   `launch_opens_terminal_screen_immediately_without_progress_modal`.
2. When `BackgroundResult::SessionStarted` or
   `BackgroundResult::EmptySessionStarted` arrives with `Ok(branch)` in
   `src/app.rs`, the session transitions to `SessionStatus::Active`,
   `self.launch_progress` for that session is cleared, and when no other session
   was selected in the meantime (`self.selected_session.is_none() ||
   self.selected_session == Some(session_id)`), Operon immediately requests
   terminal output (`self.request_session_output(session_id)`), arms the fast
   terminal echo poll window (`self.last_terminal_input = Some(Instant::now())`),
   and requests terminal keyboard focus (`self.terminal_focus_request =
   Some(session_id)`). Test:
   `session_started_arms_immediate_terminal_output_polling_and_focus`.
3. When `BackgroundResult::SessionStarted` or
   `BackgroundResult::EmptySessionStarted` arrives with `Err(error)` in
   `src/app.rs`, the session transitions to `SessionStatus::Failed` and
   `self.launch_progress` for that session sets `failure = Some(error)` and
   `dismissed = false`, surfacing the diagnostic failure modal
   (`セッションの起動に失敗しました` with `閉じる`, `再試行`, and `設定を開く`).
   Test: `launch_failure_surfaces_diagnostic_modal_after_starting_in_terminal`.
4. `launch_stored_session` in `src/tmux.rs` returns `Ok(git_branch(&worktree_path).ok())`
   immediately after `start_tmux_agent_session` succeeds instead of blocking the
   `SessionStart` worker thread for up to 8 seconds inside
   `auto_approve_workspace_trust_prompt`; when `auto_approve_workspace_prompts`
   is `true`, workspace-trust auto-approval runs asynchronously on a background
   thread and `auto_approve_workspace_trust_prompt` exits early with `Ok(false)`
   if `tmux_capture_visible_screen` reports `TmuxState::Gone`. Test:
   `launch_stored_session_does_not_block_on_workspace_trust_polling`.

## Behaviour

States when launching, retrying, or resuming a CLI session or plain terminal:

- **Loading (`SessionStatus::Starting`, first ~50–80 ms while `tmux new-session` runs)**:
  - The app immediately switches to `Page::Sessions` with the new or retried
    session selected and `session_library_open = false`.
  - The terminal workspace header shows the agent chip, session title, and the
    existing `STARTING` (`起動中`) status chip; the notice bar shows the
    existing `セッションを起動しています…` (or `セッションを再試行しています…` /
    `ターミナルを起動しています…`) message.
  - No `launch-scrim` or `launch-modal` covers the window (`launch_progress`
    has `dismissed: true`).
- **Empty / initial pane before first `tmux capture-pane` completes**:
  - Inside the terminal viewport frame, the existing muted placeholder
    `ターミナル出力を待っています…` is shown until the first capture arrives.
- **Success (`BackgroundResult::SessionStarted` / `EmptySessionStarted` returns `Ok(branch)`)**:
  - Because `launch_stored_session` no longer waits 8 seconds on
    `auto_approve_workspace_trust_prompt`, `SessionStarted` arrives as soon as
    `start_tmux_agent_session` finishes (~50–80 ms).
  - The session status chip changes to `ACTIVE` (`実行中`),
    `request_session_output(session_id)` fetches the initial pane contents
    immediately, `last_terminal_input = Some(Instant::now())` drives 33 ms
    `poll_terminal_echo` captures across `TERMINAL_ECHO_WINDOW` (1.5 s) so the
    CLI banner and prompt render live, and `terminal_focus_request =
    Some(session_id)` gives the terminal pane keyboard focus.
  - Meanwhile, if `auto_approve_workspace_prompts` is enabled, the background
    watcher checks `tmux_capture_visible_screen` every 160 ms for up to 8 s and
    auto-approves a recognized workspace-trust prompt if one appears.
- **Error (`BackgroundResult::SessionStarted` / `EmptySessionStarted` returns `Err(error)`)**:
  - The session status becomes `SessionStatus::Failed`.
  - `launch_progress` sets `failure = Some(error.clone())` and
    `dismissed = false`, so `ui_launch_modal` opens over the screen with
    `セッションの起動に失敗しました`, the folder/project/agent/command details,
    the error text in `palette.danger`, and the recovery actions (`閉じる`,
    `再試行`, and `設定を開く`).

## Design

**Placement: `REPLACE`.** Replaces the blocking in-progress modal display in
`src/app.rs` and the synchronous 8-second `auto_approve_workspace_trust_prompt`
wait in `src/tmux.rs` with immediate terminal navigation and non-blocking trust
prompt polling, while keeping `LaunchProgress` and `ui_launch_modal` for launch
failure diagnostics. Route `modify`.

- `src/app.rs`:
  - In `launch_session` (around line 4309): initialize `LaunchProgress` with
    `dismissed: true` before calling `self.start_session(id)`.
  - In `launch_empty_session` (around line 4378): initialize `LaunchProgress`
    with `dismissed: true`, and call `self.select_session(id)`,
    `self.page = Page::Sessions`, and `self.session_library_open = false` so a
    plain terminal also opens on `Page::Sessions` immediately.
  - In `start_session` (around line 6010): when creating `LaunchProgress`, set
    `dismissed: true`; if an existing `LaunchProgress` for `session_id` is
    reused on retry, clear `failure = None` and set `dismissed = true`. Call
    `self.select_session(session_id)`, `self.page = Page::Sessions`, and
    `self.session_library_open = false`.
  - In `BackgroundResult::SessionStarted` and `BackgroundResult::EmptySessionStarted`
    (around lines 1604–1738):
    - On `Ok(branch)`: clear `self.launch_progress` when its `session_id`
      matches; if `self.selected_session.is_none() || self.selected_session ==
      Some(session_id)`, ensure `self.select_session(session_id)`,
      `self.page = Page::Sessions`, `self.session_library_open = false`, call
      `self.request_session_output(session_id)`, set `self.last_terminal_input =
      Some(Instant::now())`, and set `self.terminal_focus_request =
      Some(session_id)`.
    - On `Err(error)`: on `self.launch_progress` matching `session_id`, set
      `progress.failure = Some(error.clone())` and `progress.dismissed = false`
      so `ui_launch_modal` displays the error card.
- `src/tmux.rs`:
  - In `launch_stored_session` (around line 281): when
    `auto_approve_workspace_prompts` is `true`, clone `session.tmux_name` and
    spawn a background thread calling
    `let _ = auto_approve_workspace_trust_prompt(&tmux_name);` so
    `launch_stored_session` returns `Ok(git_branch(&worktree_path).ok())` right
    after `start_tmux_agent_session` completes.
  - In `auto_approve_workspace_trust_prompt` (around line 1327): when
    `tmux_capture_visible_screen(name)` returns `Err(error)` whose
    `tmux_error_state(&error.to_string()) == TmuxState::Gone`, return
    `Ok(false)` immediately instead of sleeping until the 8-second deadline.
- `src/tests.rs`:
  - Add the four tests named in Requirements 1–4.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Uses only existing terminal workspace and failure modal palette roles. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No icons added or changed. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new shared identifier or config key is introduced. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes (read) | Session status persistence before spawning `SessionStart` (`SessionStatus::Starting` persisted before background work, `Active` / `Failed` persisted on completion) is unchanged. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new subprocess command is added; `auto_approve_workspace_trust_prompt` still uses `tmux_capture_visible_screen` and `tmux_send_input` via `tmux_command()` and `run_command_with_timeout`, and stops early when the `tmux` session is `TmuxState::Gone`. |
| Documentation — user-facing docs change in all three languages together | No | No README or CLI flag surface changes. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Purely local `tmux` session management and UI state transitions; no network calls or `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `auto_approve_workspace_trust_prompt` remains bounded by `Duration::from_secs(8)` with 160 ms polling intervals (`<= 50` visible-screen captures) and exits early on `TmuxState::Gone`. |

## Flagged concerns

- **Requesting `tmux capture-pane` before `start_tmux_agent_session` has created the `tmux` session would fail with `can't find session` and overwrite `self.notice`.** Answered: `start_session` and `launch_empty_session` only switch the UI selection and page (`select_session`, `page = Page::Sessions`, `session_library_open = false`) without calling `request_session_output`. `request_session_output(session_id)` and `last_terminal_input = Some(Instant::now())` are triggered in `BackgroundResult::SessionStarted` / `EmptySessionStarted` after `start_tmux_agent_session` has succeeded, which is now ~50–80 ms after launch because `auto_approve_workspace_trust_prompt` no longer blocks `launch_stored_session`.
- **A background workspace-trust polling thread could keep polling for 8 seconds if a short-lived `tmux` session exits immediately.** Answered: `auto_approve_workspace_trust_prompt` checks `tmux_error_state(&error.to_string()) == TmuxState::Gone` when `tmux_capture_visible_screen` returns an `Err`, returning `Ok(false)` on the first poll after the `tmux` session disappears.

## Acceptance

- `cargo test --locked launch_opens_terminal_screen_immediately_without_progress_modal` passes.
- `cargo test --locked session_started_arms_immediate_terminal_output_polling_and_focus` passes.
- `cargo test --locked launch_failure_surfaces_diagnostic_modal_after_starting_in_terminal` passes.
- `cargo test --locked launch_stored_session_does_not_block_on_workspace_trust_polling` passes.
- Existing launch modal and workspace trust prompt tests (`cargo test --locked launch_modal`, `cargo test --locked trust_prompt`, `cargo test --locked handoff`) pass.

## Rejected alternatives

- **Delete `ui_launch_modal`, `LaunchProgress`, and `launch_progress_footer` completely.**
  Rejected because `ui_launch_modal` is also the diagnostic error surface when
  a launch fails (`セッションの起動に失敗しました` with `再試行` and `設定を開く`),
  and existing tests in `src/tests.rs` (`the_launch_modal_*`,
  `every_message_row_is_one_the_code_shows`) guard the modal widget and its
  i18n rows. Keeping `LaunchProgress` with `dismissed: true` while starting and
  `dismissed: false` on `Err(error)` removes the blocking progress modal while
  preserving the failure diagnostic modal and all existing tests.
- **Keep `auto_approve_workspace_trust_prompt` synchronous inside `launch_stored_session` and only hide the modal.**
  Rejected because `launch_stored_session` would still block
  `BackgroundResult::SessionStarted` for 8 full seconds on every already-trusted
  folder; during those 8 seconds `session.status` would remain
  `SessionStatus::Starting`, `request_session_poll` would skip the session, and
  the user would stare at `ターミナル出力を待っています…` unable to see output
  or type into the CLI for 8 seconds.
