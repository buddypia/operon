# Intent: The launch modal hides the starting CLI terminal

- **Status**: approved
- **Opened**: 2026-10-24

## Problem

> 「operonで各CLIを起動する際に`セッションを起動しています…`モダールが出るが、すぐ起動中のCLI画面を見せた方が良いと思う」

When a person launches, retries, or resumes an agent CLI session (Claude Code,
Codex, Antigravity, or a custom CLI) or opens a plain terminal in Operon, two
things keep them from seeing the starting CLI screen right away:

1. `launch_session`, `launch_empty_session`, and `start_session` in `src/app.rs`
   open the full-window scrim and modal (`ui_launch_modal`, titled
   `{agent} セッションを起動しています…`) with `dismissed: false` and wait for
   `BackgroundResult::SessionStarted` or `BackgroundResult::EmptySessionStarted`
   before selecting the session and switching to `Page::Sessions`.
2. Inside `launch_stored_session` in `src/tmux.rs`, right after
   `start_tmux_agent_session` has already created the `tmux` pane and started
   the CLI (which finishes in roughly 50–80 ms),
   `auto_approve_workspace_trust_prompt(&session.tmux_name)` runs synchronously
   on the `SessionStart` worker thread for up to `Duration::from_secs(8)`.
   Whenever the folder is already trusted or the CLI does not show a trust
   prompt — the common case on almost every launch — `launch_stored_session`
   blocks for the entire 8-second timeout before returning `Ok(branch)`. During
   all 8 seconds the modal covers the window, `session.status` stays
   `SessionStatus::Starting`, and no terminal output is captured or shown.

## Who feels it, and when

- Anyone starting, retrying, or resuming a Claude Code, Codex, Antigravity,
  custom CLI, or plain terminal session: instead of watching the CLI boot in
  the terminal pane, they stare at a modal spinner for up to 8 seconds unless
  they manually click `バックグラウンドで続行` (`Esc`).

## Desired outcome

- Launching, retrying, or resuming a CLI session or plain terminal immediately
  selects that session, closes the session library overlay if open, and switches
  to the in-app terminal workspace (`Page::Sessions`) without showing the
  in-progress `セッションを起動しています…` modal over the screen.
- `launch_stored_session` returns as soon as the `tmux` session is created so
  the session transitions to `SessionStatus::Active` and begins streaming
  terminal output within milliseconds, while workspace-trust prompt
  auto-approval continues asynchronously without blocking `SessionStart`.
- If launching the session fails (`Err(error)`), the diagnostic launch failure
  modal (`セッションの起動に失敗しました` with `閉じる`, `再試行`, and
  `設定を開く`) still appears so the person can see why and recover.

## Constraints this change inherits

- macOS only; local-first with no telemetry or network calls.
- Subprocess safety: all `tmux` commands continue to go through `tmux_command()`
  and `run_command_with_timeout`, and workspace-trust auto-approval continues to
  inspect only the visible screen (`tmux_capture_visible_screen`) for a bounded
  8-second window.
- Existing tests and i18n message table coverage (`every_message_row_is_one_the_code_shows`
  and the `ui_launch_modal` / `launch_progress_footer` unit tests) must remain
  intact.

## Systems likely affected

Session launch state transitions and background result handling in `src/app.rs`,
workspace-trust auto-approval scheduling in `src/tmux.rs`, and unit/integration
tests in `src/tests.rs`.

## Open questions

None: the user's request directly specifies showing the starting CLI terminal
screen immediately instead of the in-progress launch modal, and all steps are
recoverable.

## Not in scope

- Changing the pre-launch configuration sheet (`ui_launch_sheet`) where the
  person picks the agent, model, or worktree before clicking launch.
- Changing the diagnostic error modal shown when `tmux` or the CLI executable
  fails to start.
