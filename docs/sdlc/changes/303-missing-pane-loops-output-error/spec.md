# Spec: Stop missing-pane output-read loops, recover un-ID'd CLI conversations, and fall back unrestorable open tabs to plain terminals

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Stop per-frame missing-pane capture loops and reconcile missing active panes** (verified by `pane_303_missing_pane_does_not_loop_output_requests_or_spam_error_banner`):
   - `OperonApp` adds transient in-memory state `terminal_capture_missing: HashSet<Uuid>`, `explicit_output_requests: HashSet<Uuid>`, and `lost_tabs_on_load: HashSet<Uuid>` (not persisted, pruned against `self.store.sessions`, and cleared on session start/launch/open/removal and when `apply_session_poll` observes `TmuxState::Alive` or `set_session_output` succeeds).
   - `ui_terminal_pane` delegates initial capture scheduling to `request_initial_pane_output_if_needed(&session)` in `src/app/terminal_output.rs`, which returns without spawning `BackgroundKey::SessionOutput` if `terminal_layouts`, `terminal_fallback_layouts`, or `session_output` already holds output, OR `terminal_capture_missing`, `lost_tabs_on_load`, or `exited_tabs_on_load` contains `session.id`, OR `session.status == SessionStatus::Queued`, OR `session_launch_running(session.id)` is `true`.
   - `open_session_in_terminal` clears `self.terminal_capture_missing.remove(&session_id)` before calling `self.refresh_session_output(session_id)`.
   - `refresh_session_output(session_id)` (called by both `open_session_in_terminal` in `src/app.rs` and `select_default_terminal_session` in `src/app/screens.rs`) checks the same eligibility guard: it returns early without calling `request_session_output(session_id)` if the session is absent from `self.store.sessions`, `session.status == SessionStatus::Queued`, `self.session_launch_running(session_id)` is `true`, or `terminal_capture_missing`, `lost_tabs_on_load`, or `exited_tabs_on_load` contains `session_id`.
   - The terminal header menu's 「今すぐ同期」 calls `sync_session_output_now(session_id)`, which clears `terminal_capture_missing`, inserts `session_id` into `explicit_output_requests` (pruned against `self.store.sessions`), and calls `request_session_output(session_id)`.
   - `handle_session_output_result` in `src/app/terminal_output.rs`:
     - Removes `BackgroundKey::SessionOutput(session_id)` from `self.background_tasks` and consumes `let explicit = self.explicit_output_requests.remove(&session_id);` unconditionally on both `Ok` and `Err` so a later passive capture cannot inherit a stale explicit flag.
     - On `Ok(capture)`: clears `terminal_capture_missing`, stashes any pre-laid rows, updates output via `set_session_output`, and clears `terminal_prelaid`.
     - On `Err(error)` when `!explicit && tmux_error_state(&error) == TmuxState::Gone && self.tools.tmux`:
       - If `session_launch_running(session_id)` is `true`: ignores the stale pre-launch `Gone` result completely without recording `terminal_capture_missing` or setting `self.notice`.
       - Else if `session_is_live_without_inflight_launch(session_id)` (`Active`, `Starting`, or `Unknown`): marks `record_terminal_capture_missing(session_id)` and calls `reconcile_observed_tmux_gone(session_id, Instant::now())` without emitting `ターミナル出力を読み取れませんでした: ...`.
       - Else (session already `Lost`, `Failed`, `Exited`, `Cancelled`, or removed): marks `record_terminal_capture_missing(session_id)` without setting `self.notice`.
     - On `Err(error)` when `explicit` OR `tmux_error_state(&error) != TmuxState::Gone` OR `!self.tools.tmux`: records `terminal_capture_missing` if `session_output` has no entry for `session_id` and sets `self.notice = Some(tf!("ターミナル出力を読み取れませんでした: {error}", error = error))`.
   - `BackgroundResult::TerminalInputSent` on `Err(error)` when `tmux_error_state(&error) == TmuxState::Gone && self.tools.tmux && self.session_is_live_without_inflight_launch(session_id)` calls `reconcile_observed_tmux_gone(session_id, Instant::now())` instead of leaving a `can't find pane` input error banner. `BackgroundResult::TerminalResized` on `TmuxState::Gone` delegates to the same `reconcile_observed_tmux_gone(session_id, Instant::now())` helper.

2. **Show accurate empty-pane messages instead of perpetual waiting** (verified by `pane_303_missing_pane_does_not_loop_output_requests_or_spam_error_banner`):
   - `terminal_empty_placeholder(status: &SessionStatus, capture_missing: bool) -> &'static str` in `src/app/terminal_output.rs` returns:
     - `SessionStatus::Queued` -> `tr("開始待機中のため、まだターミナル出力はありません。")`
     - `SessionStatus::Lost | SessionStatus::Failed | SessionStatus::Exited | SessionStatus::Cancelled` when `capture_missing` is `true` -> `tr("このセッションのターミナルは終了しているため、表示できる出力はありません。")`
     - All other cases (including `Starting`, `Active`, `Unknown`, or initial render before capture returns) -> `tr("ターミナル出力を待っています…")`.
   - `ui_terminal_pane` draws `terminal_empty_placeholder(&session.status, self.terminal_capture_missing.contains(&session_id))` when `lines` is `None`.

3. **Recover CLI conversations from local registries/transcripts even when `native_session_id` was not saved** (verified by `pane_303_adopted_or_untracked_cli_recovers_conversation_on_crash_and_resume`):
   - In `adopt_terminal_cli` (`src/app.rs`), when `take_cli_kind(session, provider)` transitions a terminal to a newly observed foreground CLI, also stamp `session.launched_at = Some(now())` before persisting so subsequent local recovery filters against when the CLI started rather than when the plain shell originally opened.
   - `resolve_or_recover_native_session` in `src/app/relaunch.rs` (backed by directory-injectable `recover_cli_conversation_in`) recovers missing conversation IDs when `source.native_session_id` is `None`:
     1. Checks `native_session_id_from_agent_command(provider, &source.agent_command)`.
     2. If still `None` and `workspace.is_dir()`, consults local CLI stores:
        - `CliProvider::Claude`: first `corrected_managed_claude_conversation_in` (which reads Claude Code's session registry at `launched_at.unwrap_or(created_at)` even when `goal` is empty), then `recover_managed_claude_session_in` at `launched_at.unwrap_or(created_at)` and `&source.goal`.
        - `CliProvider::Codex`: `recover_managed_codex_session_in` at `launched_at.unwrap_or(created_at)`.
        - `CliProvider::Gemini`: `recover_managed_antigravity_session_in` at `launched_at.unwrap_or(created_at)`.
     3. Validates any recovered ID through `is_safe_cli_session_id`, `tools.missing_session_tools(provider)`, and `validate_session_transcript_for_resume`.
   - `prepare_native_resume` delegates to `resolve_or_recover_native_session`, returning the existing `Err("このターミナルにはネイティブ再開 ID がまだ保存されていません。初回起動の完了を待つか、ローカル CLI セッションを検出してください。")` when no ID exists for manual resume, while automatic `relaunch_lost_session` treats an unrecovered missing ID (`Ok(None)`) quietly as `false` (no error banner).
   - `relaunch_kind` includes `native_session_id_from_agent_command(provider, &session.agent_command).is_some()`, `can_attempt_session_relaunch` returns `true` when `relaunch_kind(session).is_some()` or when `cli_provider_for_agent(&session.agent).is_some() && self.session_workspace_path(session).is_dir()`, and `relaunch_lost_session` dispatches directly on `if session.agent == "terminal" && session.agent_command.is_empty()` (plain shell via `launch_session_record(session_id, LaunchFocus::Keep)`) else `let Some(provider) = cli_provider_for_agent(&session.agent) else { return false; };` followed by `self.resolve_or_recover_native_session(&session, provider, &workspace)` (instead of returning early when `relaunch_kind(&session)` is `None`).

4. **Fall back unrestorable open workspace tabs to plain terminal shells within cooldown** (verified by `pane_303_unresumable_open_tab_falls_back_to_plain_terminal_with_cooldown`):
   - Whenever automatic recovery runs on a session transitioning to `SessionStatus::Lost` or `SessionStatus::Failed` (via `apply_session_poll` or `reconcile_observed_tmux_gone` -> `auto_restore_after_resize_gone`) with `!cancellation_pending && self.tools.tmux && !self.setup_sessions.contains(&id)` and `(first_look || self.auto_restore_ready(id, now))`:
     - First try `relaunch_lost_session(id)` when `can_attempt_session_relaunch(&session)` holds; on success, stamp `record_auto_restore(id, now)` and emit the existing startup or forced-restore toast.
     - Otherwise, if `self.session_has_a_tab(id)` is `true`, `is_interactive_terminal_session(&session)` is `true`, and `queued_dependency_count(&self.store.sessions, id) == 0`, call `reopen_exited_session_as_shell(id)` to reopen the open tab in place as `"terminal"` in the session's folder (preserving dead-pane scrollback or opening a fresh shell if gone, rolling back `self.store` on launch refusal/persist failure), stamp `record_auto_restore(id, now)`, and emit a brief toast (unless `persistent_notice || self.store_retry_pending` already holds):
       - 1 session: `tf!("会話を復元できなかったため、ターミナルとして開き直しました: {name}", name = name)`
       - 2+ sessions: `tf!("会話を復元できなかったため、{count} 件をターミナルとして開き直しました。", count = count)`
     - Fallback-shell recoveries do NOT suppress macOS `セッションが失敗しました: {name}` failure notifications for `Failed` CLI sessions, and sessions with `!self.session_has_a_tab(id)` stay `Lost` or `Failed` without opening a shell.
     - Repeat failures inside `AUTO_RESTORE_COOLDOWN` (15 seconds) are refused and settle in `Lost` or `Failed` without looping.

5. **Recover open `Lost`/`Failed` workspace tabs left in store on startup** (verified by `pane_303_lost_or_failed_open_tab_from_previous_run_recovers_on_load` and `model_check_pane_303_missing_pane_recovery_and_output_guard`):
   - In `OperonApp::from_state`, call `reopen_lost_workspace_tabs_on_load(&mut self)` after `reopen_exited_workspace_tabs_on_load()`: when `self.tools.tmux` is `true`, collect interactive sessions open in `workspace` or `project_workspaces` with status `SessionStatus::Lost | SessionStatus::Failed`, no pending cancellation, and `queued_dependency_count == 0` into `self.lost_tabs_on_load` without mutating stored status before the first poll.
   - `request_session_poll` and `apply_session_poll` include sessions in `self.lost_tabs_on_load`: on the first non-`Unknown` poll (dropping stale entries whose status left `Lost | Failed` or whose tab closed), if `!cancellation_pending && self.tools.tmux && !self.setup_sessions.contains(&id)` and observation is `Dead` or `Gone` (and `is_interactive_terminal_session` and `queued_dependency_count == 0`), first try `relaunch_lost_session(id)`, else fall back to `reopen_exited_session_as_shell(id)`, stamping `record_auto_restore(id, poll_now)` and emitting the matching startup or fallback-shell toast (`lost_looked` is removed only after `persist()` succeeds). If observation is `Alive`, restore `SessionStatus::Active`. Closed `Lost`/`Failed` sessions without open tabs stay untouched in **履歴**.

## Behaviour

- **Happy path — missing pane recovered automatically**:
  - Active CLI session crashes or loses its tmux server mid-run (or is opened from `store-v2.json` as `Lost`/`Failed` in an open tab on startup).
  - If a conversation ID exists or can be recovered from Claude Code's registry / local Claude, Codex, or Antigravity transcripts, Operon reopens the CLI conversation in the same tab and tmux session and shows the existing bottom-right toast (`前回開いていたセッションを {count} 件再開しました。` on startup / `強制終了されたセッションを自動で復元しました: {name}` mid-run). No `ターミナル出力を読み取れませんでした: can't find pane` error loop occurs.
- **Fallback path — conversation unrecoverable in open tab**:
  - When a CLI session crashed before writing any transcript (or is `"custom"` / `"existing terminal"`), and its tab is open in the workspace with no queued dependents, Operon converts the open tab in place to a live `"terminal"` shell in the session's folder and briefly displays:
    - 1 session: `「会話を復元できなかったため、ターミナルとして開き直しました: {name}」`
    - 2+ sessions: `「会話を復元できなかったため、{count} 件をターミナルとして開き直しました。」`
  - If a tool or ambiguous multi-session conflict raised a persistent notice explaining why conversation resume was blocked, that persistent notice is preserved ahead of the brief fallback toast.
- **Empty / Waiting / Stopped states in pane**:
  - `Queued` session tab: displays `「開始待機中のため、まだターミナル出力はありません。」` and spawns zero tmux capture attempts.
  - `Starting` / `Active` / `Unknown` before first output arrives (including while `SessionStart` / `EmptySessionStart` is running): displays `「ターミナル出力を待っています…」` and ignores any transient pre-launch `Gone` result while the launch task is in flight.
  - Closed/unrecoverable `Lost`, `Failed`, `Exited`, or `Cancelled` tab whose pane disappeared (e.g. second crash inside 15s cooldown, or a session with queued dependents): stops requesting captures after the first missing-pane result, sets no `ターミナル出力を読み取れませんでした` banner, and displays `「このセッションのターミナルは終了しているため、表示できる出力はありません。」` until manually retried or refreshed.
- **Explicit refresh error state**:
  - Clicking `···` -> 「今すぐ同期」 on a pane whose tmux session does not exist triggers one explicit read and surfaces `「ターミナル出力を読み取れませんでした: {error}」` in the bottom-right corner overlay card on `Page::Sessions`, while still not entering a per-frame loop.
- **Repeat crash / cooldown**:
  - If a restored session or fallback shell crashes again within `AUTO_RESTORE_COOLDOWN` (`15` seconds), Operon refuses another automatic restore, leaves the session in `Lost` or `Failed` (with `terminal_capture_missing` preventing any capture error loop), and surfaces the honest ended-terminal message or stderr output.

## Design

- **Placement verdict**: **EXTEND** existing app modules (`src/app.rs`, `src/app/terminal_output.rs`, `src/app/relaunch.rs`, `src/app/screens.rs`, `src/i18n_tables.rs`). Currently `src/app.rs` is `9,882` lines (`118` lines below the `10,000`-line `largest_module_lines` `propose` band in `docs/sdlc/bands.yaml`) and `src/app/screens.rs` is `9,900` lines (`100` lines below `10,000`); placing new helpers in `src/app/terminal_output.rs` and `src/app/relaunch.rs` and replacing inline blocks in `src/app.rs` keeps both files strictly bounded and below their current line counts.
- **`src/app.rs`**:
  - Add `pub(crate) terminal_capture_missing: HashSet<Uuid>`, `pub(crate) explicit_output_requests: HashSet<Uuid>`, and `pub(crate) lost_tabs_on_load: HashSet<Uuid>` to `OperonApp` (initialized empty in `from_state`).
  - Call `reopen_lost_workspace_tabs_on_load()` in `OperonApp::from_state` after `reopen_exited_workspace_tabs_on_load()`.
  - Promote `session_launch_running` to `pub(crate)`; clear `terminal_capture_missing` when launching/reopening/selecting/receiving `TmuxState::Alive`; clear `terminal_capture_missing` before calling `refresh_session_output` in `open_session_in_terminal`; guard `refresh_session_output` through `session_eligible_for_initial_output`.
  - Stamp `session.launched_at = Some(now())` in `adopt_terminal_cli`.
  - Route `BackgroundResult::SessionOutput` to `handle_session_output_result`, `TerminalResized` `Gone` to `reconcile_observed_tmux_gone`, and `TerminalInputSent` `Gone` (when live without inflight launch and `self.tools.tmux`) to `reconcile_observed_tmux_gone`.
  - Include `self.lost_tabs_on_load.contains(&id)` in `request_session_poll` and `apply_session_poll`, passing both `relaunch` and `fallback_shells` / `lost_tabs_on_load` recovery into `finish_session_poll_transitions`.
- **`src/app/terminal_output.rs`**:
  - Add `terminal_empty_placeholder`, `session_eligible_for_initial_output`, `request_initial_pane_output_if_needed`, `sync_session_output_now`, `record_terminal_capture_missing`, `session_is_live_without_inflight_launch`, and `handle_session_output_result` (consuming `self.explicit_output_requests.remove(&session_id)` at the start of result handling).
  - Clear `terminal_capture_missing` inside `set_session_output`.
- **`src/app/relaunch.rs`**:
  - Extend `relaunch_kind`, add `can_attempt_session_relaunch`, `session_workspace_path`, `recover_cli_conversation_in`, `resolve_or_recover_native_session`, `reopen_lost_workspace_tabs_on_load`, `reconcile_observed_tmux_gone`, and fallback-shell handling inside `relaunch_remembered_sessions`, `restore_forced_sessions`, `auto_restore_after_resize_gone`, and `finish_session_poll_transitions`.
  - Dispatch `relaunch_lost_session` directly on plain terminal vs `cli_provider_for_agent(&session.agent)` and use `resolve_or_recover_native_session` so un-ID'd CLI sessions recover from local transcripts/registries before falling back to a plain terminal shell.
- **`src/app/screens.rs`**:
  - Call `self.sync_session_output_now(session_id)` from 「今すぐ同期」, `self.request_initial_pane_output_if_needed(session)` in `ui_terminal_pane`, and `terminal_empty_placeholder(&session.status, self.terminal_capture_missing.contains(&session_id))` when `lines` is `None`.
- **`src/i18n_tables.rs`**:
  - Add sorted EN and KO rows for the 4 new Japanese message IDs.
- **`README.md`, `README.ja.md`, `README.ko.md`**:
  - Update the workspace/session lifecycle bullet in all three languages to describe recovering un-ID'd CLI conversations and reopening unrestorable open tabs as plain terminal shells.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Does not apply | Uses existing `palette.text_muted` for placeholder labels and existing `notice_briefly` / `notice` corner overlay surfaces; no colour roles change. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Does not apply | Reuses existing `TERMINAL_MARK` and CLI marks via `reopen_exited_session_as_shell`; no icons added or modified. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Applies | Reuses `AUTO_RESTORE_COOLDOWN` and `CLAUDE_STORE_DIRECTORY` / `CLAUDE_PROJECTS_DIRECTORY` / `CLAUDE_SESSION_REGISTRY_DIRECTORY` from `src/config.rs`, and `managed_tmux_name` / `PLAIN_TERMINAL_COMMAND` from `src/tmux.rs`, without duplicating any identifier literal. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Applies | `STORE_SCHEMA_VERSION` stays `5`; no persisted types change. Every `self.store` mutation clones `previous_store` and rolls back on `persist()` failure, keeps `relaunch_candidates` and `lost_tabs_on_load` unspent until `persist()` succeeds, and checks `cancellation_pending` (and `pending_cancellations` sidecar) before any recovery or shell fallback. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Applies | Spawns no new subprocesses; reuses `tmux_capture_with_cursor` / `launch_session_record` (`src/exec.rs` timeouts and `src/agents.rs` `is_safe_agent_command` / `is_safe_cli_session_id` gates). |
| Documentation — user-facing docs change in all three languages together | Applies | `README.md`, `README.ja.md`, and `README.ko.md` are updated together in the same change. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | Purely local tmux observation, local transcript scanning, and local store transitions; zero network calls and zero `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Applies | `terminal_capture_missing` and `explicit_output_requests` are bounded and pruned against `self.store.sessions`; local recovery delegates strictly to bounded helpers in `src/cli.rs` (`TRANSCRIPT_SEARCH_BYTE_LIMIT`, `TRANSCRIPT_SEARCH_LINE_LIMIT`, `24` Claude candidates, `TRANSCRIPT_CANDIDATE_LIMIT` = `600` in `src/config.rs` for `recover_managed_codex_session_in`, and `96` Antigravity candidates). |

## Flagged concerns

- **Preventing infinite auto-restore loops when opening a shell or resuming a broken CLI repeatedly fails** — Answered by recording `record_auto_restore(id, now)` for both resumed CLI sessions and fallback-shell reopens and checking `auto_restore_ready(id, now)` (`AUTO_RESTORE_COOLDOWN = 15s`) before any mid-run retry or fallback, plus keeping `relaunch_candidates` and `lost_tabs_on_load` cleanly consumed after `persist()` so repeat failures settle in `Lost` or `Failed` and stop requesting tmux captures via `terminal_capture_missing`.
- **Preserving `relaunch_270`, `pane_288`, `status_291`, and `terminal_276` test contracts while adding open-tab fallback and recovery** — Answered by gating plain-terminal fallback strictly on `self.session_has_a_tab(id)` so closed/non-tabbed sessions keep settling in `Lost`/`Failed`, gating `SessionOutput` missing-pane suppression on `self.tools.tmux && !explicit`, consuming `explicit_output_requests` on every completion, and keeping manual `prepare_native_resume` error semantics intact when no conversation can be found.
- **Keeping `src/app.rs` and `src/app/screens.rs` below the `10,000`-line `propose` band in `docs/sdlc/bands.yaml`** — Answered by placing output-guard and placeholder helpers in `src/app/terminal_output.rs`, placing conversation-recovery and fallback-shell helpers in `src/app/relaunch.rs`, keeping `src/app/screens.rs` unchanged in line count (`9,900` lines), and replacing the inline `SessionOutput` and `TerminalResized` `Gone` blocks in `src/app.rs` with single-line helper calls so `src/app.rs` shrinks below `9,882` lines.

## Acceptance

- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` pass with zero warnings.
- `cargo test --locked` passes, including new tests `pane_303_missing_pane_does_not_loop_output_requests_or_spam_error_banner`, `pane_303_adopted_or_untracked_cli_recovers_conversation_on_crash_and_resume`, `pane_303_unresumable_open_tab_falls_back_to_plain_terminal_with_cooldown`, `pane_303_lost_or_failed_open_tab_from_previous_run_recovers_on_load`, and `model_check_pane_303_missing_pane_recovery_and_output_guard`, as well as existing `pane_288_`, `status_291_`, `relaunch_270`, `model_check_a_session_is_relaunched_only_at_its_first_stored_look`, `terminal_276_`, `tui_perf_292_`, `launch_modal`, and i18n table tests.
- `cargo test --locked -- --ignored` runs (per `AGENTS.md` because native restore paths changed), or its exact constraint is reported if any live CLI binary is unavailable.
- `python3 docs/sdlc/changes/303-missing-pane-loops-output-error/mutations.py` kills every mutation (including mutations on the new Japanese toast and placeholder claims).
- `bash scripts/check-bands.sh` reports zero `propose` breaches with `src/app.rs` and `src/app/screens.rs` below `10,000` lines.
- In `/Applications/Operon.app`, viewing an open tab whose tmux pane is missing (`Lost`, `Failed`, `Queued`, or mid-run disappeared) never loops `ターミナル出力を読み取れませんでした: can't find pane`, recovers saved/local CLI conversations automatically, and reopens unrecoverable open tabs as live shells in the session's folder.

## Rejected alternatives

- **Suppressing all `SessionOutput` error notices silently without reconciling `Gone` or falling back to a shell** — Rejected because hiding the error alone would leave the user staring forever at 「ターミナル出力を待っています…」 on a dead tab, whereas reconciling `Gone` recovers the session (or opens a shell) and surfaces honest state when ended.
- **Reopening closed historical `Lost`/`Failed` sessions that have no open workspace tab** — Rejected because closed tabs in **履歴** were deliberately put away by the user and silently spawning shells for historical crashes on startup would flood tmux and workspace tabs unexpectedly.
- **Changing the persisted `Session` schema to store an explicit `fallback_shell` or `capture_missing` flag** — Rejected because `src/store.rs` / `src/models.rs` are `paused` high-risk surfaces and existing `SessionOrigin` + transient `OperonApp` sets already express every required transition without a schema migration.
