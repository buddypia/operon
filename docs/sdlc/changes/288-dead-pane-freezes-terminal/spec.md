# Spec: Keep cleanly exited panes as live shells and auto-restore force-terminated sessions

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Every tmux session configured by `configure_tmux_session` sets `remain-on-exit-format` to `""` alongside `remain-on-exit on`, so tmux never writes a `Pane is dead (...)` footer into a managed pane when its top-level command exits.
2. When `apply_session_poll` observes a clean exit (`TmuxState::Dead` with `exit_status: Some(0)`) for a session with `self.tools.tmux == true`, `!cancellation_pending`, `!self.setup_sessions.contains(&id)`, `queued_dependency_count(&self.store.sessions, id) == 0`, and `is_interactive_terminal_session(&session)` (`"terminal"`, `"existing terminal"`, `"claude"`, `"codex"`, `"gemini"`, or `"custom"`), Operon converts that same `Session` record in place into a plain terminal (`agent = "terminal"`, empty `agent_command` and `goal`, `native_session_id = None`, `native_session_path = None`, `origin = Some(terminal_provenance(&session).unwrap_or(SessionOrigin::PlainTerminal))`) and launches a login shell in place (`LaunchFocus::Keep`), respawning the dead tmux pane without killing the tmux session so prior scrollback stays intact.
3. On startup (`OperonApp::from_state`), after restoring `WorkspaceLayout` when `tools.tmux` is `true`, any interactive session open in `workspace` or `project_workspaces` whose stored status is `SessionStatus::Exited` (with no pending cancellation and zero queued dependents) is queued in `exited_tabs_on_load` without mutating its stored `SessionStatus::Exited`: on its first non-`Unknown` poll, if the pane is `Alive` it becomes `SessionStatus::Active`, and if the pane is `Dead` or `Gone` (and still interactive, uncancelled, non-setup, and dependency-free) it reopens in place as a plain terminal shell; closed `Exited` sessions not in any open workspace tab remain `SessionStatus::Exited`.
4. When `apply_session_poll` observes a forced or abnormal termination (`SessionStatus::Lost` or `SessionStatus::Failed`, from `TmuxState::Gone`, `TmuxState::Dead` with `exit_status: None`, or `TmuxState::Dead` with non-zero `exit_status`) during a live run (`!first_look`) for a session with `self.tools.tmux == true`, `!cancellation_pending`, `!self.setup_sessions.contains(&id)`, and `relaunch_kind(&session).is_some()`, Operon automatically restores the session in place through `relaunch_lost_session` and shows a brief toast: `「強制終了されたセッションを自動で復元しました: {name}」` when one session is restored in that poll, or `「強制終了されたセッションを {count} 件自動で復元しました。」` when two or more are restored.
5. Automatic session restoration records the restoration `Instant` per session and refuses another automatic restore for the same session while less than `AUTO_RESTORE_COOLDOWN` (`15` seconds) has elapsed since its last automatic restore, leaving a repeat failure in `SessionStatus::Failed` or `SessionStatus::Lost`; an explicit manual start or resume (`start_session`, `resume_managed_native_session`) clears that session's cooldown entry.
6. When `BackgroundResult::SessionStarted` succeeds for a quiet launch (`quiet == true`) whose session is currently visible on screen (`self.is_session_on_screen(session_id)`), Operon calls `self.request_session_output(session_id)` immediately without changing selection, page, focus, or notices, so the newly spawned shell or restored CLI renders without waiting for a focus change.

## Behaviour

- **Clean exit of an agent CLI or shell (happy path)**: When `agy`, `claude`, `codex`, a custom agent, or a login shell exits with status `0`, no `Pane is dead` line appears. The session's tab and sidebar row stay in place, the session's kind becomes `ターミナル` (`"terminal"`), the previous output stays above the new login shell prompt in the same directory/worktree, and keyboard input continues working immediately. If the person subsequently types `claude`, `codex`, or `agy` into that shell, existing `adopt_terminal_cli` / hook detection adopts the new CLI session as usual. If macOS notifications are enabled and the session that exited cleanly was an agent CLI (not already a plain terminal), the ambient completion notification `「セッションが完了しました: {name}」` still fires once.
- **Open tabs left `Exited` by an older build**: On launch, open tabs in `workspace` or `project_workspaces` that were saved as `Exited` are re-checked; dead panes have their old `Pane is dead (...)` overlay cleared by `respawn-pane -k` while keeping scrollback, and gone panes open a fresh shell in the session's folder. Closed historical sessions stay untouched in **履歴**.
- **Forced kill or crash while Operon is running**: Killing a pane's process (`kill -9`, non-zero crash) or losing its tmux session (`tmux kill-session`) while Operon is running restores the session in the same tab without stealing keyboard focus from another tab. A CLI with a saved transcript reopens its conversation (`codex resume`, `claude --resume`, `agy --resume` with original launch flags); a plain terminal reopens a login shell (keeping scrollback if the pane was `Dead`). Operon displays a brief 4-second toast:
  - 1 session: `「強制終了されたセッションを自動で復元しました: {name}」`
  - 2+ sessions: `「強制終了されたセッションを {count} 件自動で復元しました。」`
- **Startup relaunch**: Unchanged in wording from change 270 — sessions that were running when Operon last closed and are found gone or crashed on `first_look` show `「前回開いていたセッションを {count} 件再開しました。」`. Their restoration also stamps `last_auto_restore` so an immediate post-startup crash does not loop.
- **Interrupted / repeat crash (cooldown)**: If a restored session crashes or disappears again within `15` seconds (`AUTO_RESTORE_COOLDOWN`), Operon does not auto-restore it a second time: it settles in `Failed` or `Lost` (emitting the ambient failure notification if `Failed` and notifications are on) so the person can read the error output or click 「再試行」 / 「再開」.
- **No transcript / unresumable CLI**: A CLI session that was killed before writing a conversation transcript on disk (`NativeResume::NoTranscript` or `native_session_id == None`) cannot reopen a conversation without re-sending its prompt; `relaunch_lost_session` returns `false`, no restoration toast is shown, and the session stays `Lost` or `Failed`.
- **Explicit stop, setup scripts, run scripts, queued dependencies**: Stopping or removing a session (`cancellation_pending`), `.operon/setup.sh` (`setup_sessions`), `.operon/run.sh` (`RUN_SCRIPT_AGENT`), and parent sessions with `queued_dependency_count > 0` keep their existing `Cancelled` / `Exited` / `Failed` transitions unchanged.

## Design

Placement: **EXTEND** existing lifecycle modules (`src/tmux.rs`, `src/app/relaunch.rs`, `src/app.rs`, `src/config.rs`, `src/i18n_tables.rs`), keeping `src/app.rs` line count flat or lower by putting the transition and restoration helpers in `src/app/relaunch.rs`.

- `src/config.rs`:
  - Add `pub(crate) const AUTO_RESTORE_COOLDOWN: Duration = Duration::from_secs(15);` (with `std::time::Duration` imported or referenced via `crate::Duration`).
- `src/tmux.rs`:
  - In `configure_tmux_session(name)`: set `remain-on-exit-format` to `""` (`tmux set-option -q -t <name> remain-on-exit-format ""`) right after `remain-on-exit on`.
  - Add `reopen_or_start_plain_terminal(name, cwd, environment)`: inspects `tmux_observation(name).state`.
    - `TmuxState::Gone` -> `start_empty_tmux_session(name, cwd, environment)`.
    - `TmuxState::Dead` -> `set_tmux_session_environment(name, environment)` (unsetting any absent `MANAGED_TMUX_ENV_KEYS` first) + `configure_tmux_session(name)` + `preserve_dead_pane_scrollback(name)` (`send-keys -R`) + `respawn_tmux_pane(name, cwd, PLAIN_TERMINAL_COMMAND)`. This preserves the pane's scrollback history, removes stale hook/account environment variables, and clears any legacy `Pane is dead (...)` overlay.
    - `TmuxState::Alive` -> returns the existing alive-terminal refusal (`そのターミナルはまだ実行中です…`).
    - `TmuxState::Unknown` -> returns the existing unknown-terminal refusal (`古いターミナルがまだ開いているか判定できませんでした。`).
  - In `launch_stored_session`: when `session.agent == "terminal" && session.agent_command.is_empty()`, call `reopen_or_start_plain_terminal` directly (instead of `prepare_tmux_for_retry`, which would `kill-session` a `Dead` pane and wipe scrollback). Non-terminal launches keep `if retrying { prepare_tmux_for_retry(&session.tmux_name)?; }` before `start_tmux_agent_session`.
- `src/app/relaunch.rs` & `src/app.rs`:
  - Add `last_auto_restore: HashMap<Uuid, Instant>` and `exited_tabs_on_load: HashSet<Uuid>` to `OperonApp`.
  - `pub(crate) fn is_interactive_terminal_session(session: &Session) -> bool`: returns `matches!(session.agent.as_str(), "terminal" | "existing terminal" | "claude" | "codex" | "gemini" | "custom")`.
  - `OperonApp::reopen_exited_workspace_tabs_on_load(&mut self)`: called in `from_state` after `restore_workspace_layout` when `self.tools.tmux` is `true`. Collects session IDs from `self.workspace.session_ids()` and `self.project_workspaces.values().flat_map(|(tabs, _)| tabs.session_ids())`. For each `SessionStatus::Exited` interactive session in those open tabs with no pending cancellation and `queued_dependency_count == 0`, inserts `session.id` into `self.exited_tabs_on_load` while keeping `session.status == SessionStatus::Exited` on disk until the first non-`Unknown` poll observes the pane.
  - `OperonApp::reopen_exited_session_as_shell(&mut self, session_id: Uuid) -> bool`: updates the stored `Session` in memory (`agent = "terminal"`, `agent_command.clear()`, `goal.clear()`, `native_session_id = None`, `native_session_path = None`, `origin = Some(terminal_provenance(&session).unwrap_or(SessionOrigin::PlainTerminal))`), launches via `self.launch_session_record(session_id, LaunchFocus::Keep)` (which persists once and validates launch preconditions, rolling back `self.store` on any refusal), and clears stale `self.hook_status` and in-flight `NativeSessionResolution`.
  - `OperonApp::auto_restore_ready(&self, session_id: Uuid, now: Instant) -> bool`: true when `self.last_auto_restore.get(&session_id).is_none_or(|at| now.saturating_duration_since(*at) >= AUTO_RESTORE_COOLDOWN)`.
  - `OperonApp::clear_auto_restore_cooldown(&mut self, session_id: Uuid)`: called from manual `start_session` and `resume_managed_native_session`.
  - `OperonApp::restore_forced_sessions(&mut self, sessions: Vec<(Uuid, String)>, now: Instant)`: calls `relaunch_lost_session(id)` for each, stamps `self.last_auto_restore.insert(id, now)` when started, and displays the single- or multi-session Japanese toast via `notice_briefly`. `relaunch_remembered_sessions` also stamps `self.last_auto_restore.insert(id, Instant::now())` when a startup relaunch starts.
  - Fold the post-persist recovery dispatch of `apply_session_poll` into a helper in `src/app/relaunch.rs` so `src/app.rs` stays well below `10,000` lines.
  - In `BackgroundResult::SessionStarted` (`src/app.rs`): when `quiet && self.is_session_on_screen(session_id)`, call `self.request_session_output(session_id)` so the pane redraws immediately without changing selection or focus.
- `src/i18n_tables.rs`:
  - Add sorted entries to `EN_TABLE` and `KO_TABLE` for:
    - `"強制終了されたセッションを {count} 件自動で復元しました。"`
    - `"強制終了されたセッションを自動で復元しました: {name}"`
- `README.md`, `README.ja.md`, `README.ko.md`:
  - Update the workspace/session lifecycle bullet in all three languages to describe clean exits returning to a shell and forced terminations restoring automatically.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Uses existing session status badges and existing `notice_briefly` toast rendering; no new colour roles. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Reuses existing `TERMINAL_MARK` and CLI marks; no new icons or glyph constants. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `AUTO_RESTORE_COOLDOWN` is defined once in `src/config.rs`; `PLAIN_TERMINAL_COMMAND` remains a single constant in `src/tmux.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | `STORE_SCHEMA_VERSION` remains `5` (no new enum variants or fields). Every store mutation clones `previous_store` and rolls back in memory when `persist()` fails; `pending_cancellations` sidecar precedes all automatic shell/restore actions. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | All tmux calls (`set-option`, `set-environment`, `respawn-pane`) run through `tmux_command()` + `run_command_with_timeout` in `src/exec.rs` with `-t` session targets and separated arguments; resumed CLI commands continue passing through `stored_launch_command` and `is_safe_agent_command`. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ja.md`, and `README.ko.md` are updated together in the same commit. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Purely local tmux and store state transitions; zero network calls and zero `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new file scan or unbounded output buffer; reuses existing `run_command_with_timeout(..., 5s)` and `TERMINAL_SCROLLBACK_LINES`. |

## Flagged concerns

- **Preserving scrollback and removing legacy `Pane is dead (...)` text on respawn** — Verified empirically on `tmux 3.8` using an isolated server socket before writing code: setting `remain-on-exit-format ""` suppresses the dead-pane footer completely while preserving `#{pane_dead}=1` and `#{pane_dead_status}=0`, and running `respawn-pane -k` on a legacy pane that already rendered `Pane is dead (status 0, ...)` clears that dead-pane banner while preserving the prior lines in scrollback above the new shell's output.
- **Preventing infinite restart loops when a command or shell fails immediately** — Resolved by stamping `last_auto_restore` per session on every automatic restoration (both startup `first_look` and mid-run forced termination) and requiring `AUTO_RESTORE_COOLDOWN` (`15` seconds) before another automatic restoration can trigger for the same session. Manual user retries clear the timestamp so the person is never blocked from retrying immediately.
- **Updating change 270's existing tests that encoded the old dead-pane and first-look-only rules** — `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone` and `model_check_a_session_is_relaunched_only_at_its_first_stored_look` in `src/tests.rs` previously asserted that a clean exit (`Dead, Some(0)`) stays `Exited` and that a mid-run disappearance (`!first_look`) stays `Lost`. Following the precedent of change 285, both tests are updated in place (never deleted or ignored) to verify the new contract: clean exit reopens as `"terminal"` rather than re-running Codex, and mid-run disappearance restores once and respects the cooldown on immediate repeat failure.

## Acceptance

- `cargo test --locked` passes, including new tests `pane_288_configure_tmux_suppresses_dead_pane_banner_and_keeps_scrollback`, `pane_288_clean_exit_reopens_interactive_session_as_shell`, `pane_288_exited_open_tab_from_previous_run_reopens_as_shell`, `pane_288_forced_kill_mid_run_restores_cli_and_shell_with_cooldown`, `pane_288_quiet_launch_on_screen_requests_immediate_output`, and `model_check_pane_288_poll_transitions_and_cooldown`.
- `python3 docs/sdlc/changes/288-dead-pane-freezes-terminal/mutations.py` kills every mutation.
- In the running `/Applications/Operon.app`, exiting `agy` / `claude` / `codex` cleanly leaves a live shell in the same tab without `Pane is dead`, existing dead-pane tabs reopen as live shells on launch, and `kill -9` on an active session restores it in place with the Japanese toast.

## Rejected alternatives

- **Setting `remain-on-exit off` on managed tmux sessions** — Rejected because a fast startup crash would destroy the tmux window before Operon could read `#{pane_dead_status}` or capture the error message printed on failure (`start_tmux_agent_session` relies on `remain-on-exit on`).
- **Wrapping every agent launch command in `agent; exec "$SHELL" -l` inside the shell string** — Rejected because `exec "$SHELL" -l` would replace the exit status before tmux ever observed it, hiding non-zero crashes from Operon, breaking setup/run script completion detection, blocking queued dependent sessions from ever seeing `Exited`, and leaving Operon's stored `session.agent` falsely claiming the CLI is still running.
- **Re-running a killed CLI session from scratch when no transcript exists on disk** — Rejected because re-launching with the initial `goal` would replay a user prompt that may already have mutated files or external state.
