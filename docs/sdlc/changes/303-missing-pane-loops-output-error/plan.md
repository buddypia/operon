# Plan: Stop missing-pane output-read loops, recover un-ID'd CLI conversations, and fall back unrestorable open tabs to plain terminals

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | Add `terminal_capture_missing`, `explicit_output_requests`, and `lost_tabs_on_load` transient sets on `OperonApp`; make `session_launch_running` `pub(crate)`; clear missing-capture markers on launch/start/open/alive/remove; clear `terminal_capture_missing` before `refresh_session_output` in `open_session_in_terminal` and guard `refresh_session_output` (covering `open_session_in_terminal` and `select_default_terminal_session`); stamp `launched_at` in `adopt_terminal_cli`; call `reopen_lost_workspace_tabs_on_load` in `from_state`; route `SessionOutput`, `TerminalInputSent` `Gone`, and `TerminalResized` `Gone` to helpers, shrinking `src/app.rs` below its current `9,882` lines (`10,000` ceiling). |
| `src/app/terminal_output.rs` | Add `terminal_empty_placeholder`, `session_eligible_for_initial_output`, `request_initial_pane_output_if_needed`, `sync_session_output_now`, `record_terminal_capture_missing`, `session_is_live_without_inflight_launch`, and `handle_session_output_result` (removing `session_id` from `explicit_output_requests` on every `Ok`/`Err` completion); clear `terminal_capture_missing` in `set_session_output`. |
| `src/app/relaunch.rs` | Add `session_workspace_path`, `recover_cli_conversation_in`, `resolve_or_recover_native_session`, `can_attempt_session_relaunch`, `reopen_lost_workspace_tabs_on_load`, and `reconcile_observed_tmux_gone`; update `relaunch_lost_session` to dispatch directly on plain terminal vs `cli_provider_for_agent(&session.agent)` instead of short-circuiting on `relaunch_kind(&session)`; extend `relaunch_kind`, `auto_restore_after_resize_gone`, `relaunch_remembered_sessions`, `restore_forced_sessions`, and `finish_session_poll_transitions` to recover un-ID'd CLI sessions and fall back unrestorable open tabs to plain shells with cooldown and toasts. |
| `src/app/screens.rs` | Route 「今すぐ同期」 through `sync_session_output_now`, initial pane read through `request_initial_pane_output_if_needed`, and `None` pane placeholder through `terminal_empty_placeholder`, keeping `src/app/screens.rs` at or below its current `9,900` lines (`10,000` ceiling). |
| `src/i18n_tables.rs` | Add sorted English (`EN_TABLE`) and Korean (`KO_TABLE`) rows for the 4 new Japanese message IDs. |
| `src/tests.rs` | Add tests `pane_303_*` and exhaustive `model_check_pane_303_*` covering R1–R5, `select_default_terminal_session` guard, `explicit_output_requests` consumption, and regression cases. |
| `README.md`, `README.ja.md`, `README.ko.md` | Document un-ID'd CLI conversation recovery and plain-terminal fallback for unrestorable open tabs in all three languages. |
| `docs/sdlc/changes/303-missing-pane-loops-output-error/*` | SDLC state, intent, spec, screen, plan, approvals ledger, mutation sweep (`mutations.py`), and verification evidence (`verification.yaml`). |

## Order of work

1. Run `bash scripts/check-readiness.sh docs/sdlc/changes/303-missing-pane-loops-output-error` and have the independent `research` evaluator subagent approve `spec.md`, `screen.md`, and `plan.md`; record approvals in `approvals.log`, update `state.yaml`, and commit Stage 2 before editing `src/`.
2. Write the TDD tests (`pane_303_*` and `model_check_pane_303_*`) in `src/tests.rs` first (with minimal stub signatures if needed to compile) and watch them fail for the predicted causes.
3. Implement `src/app/terminal_output.rs`, `src/app/relaunch.rs`, `src/app.rs`, `src/app/screens.rs`, `src/i18n_tables.rs`, and `README*.md`.
4. Run `cargo test --locked pane_303_`, `cargo test --locked model_check_pane_303_`, regression tests (`pane_288_`, `status_291_`, `relaunch_270`, `model_check_a_session_is_relaunched_only_at_its_first_stored_look`, `terminal_276_`, `tui_perf_292_`, `launch_modal`, `opening_a_session_selects_it`, `terminal_workspace_prefers_a_running_session`, `harness_documents_only_name_paths_that_exist`), all i18n table tests, and `cargo test --locked -- --ignored` (per `AGENTS.md`, since `prepare_native_resume` / `relaunch_lost_session` changed).
5. Write and run `docs/sdlc/changes/303-missing-pane-loops-output-error/mutations.py` (covering code guards and all 4 new Japanese user-facing claims), run `bash scripts/check-bands.sh` and `make q.fast`, invoke `empirical-verifier` and `formal-verifier` subagents, fix any reproduced findings, record `verification.yaml`, close `state.yaml` and `plan.md`, commit, push, wait for CI, merge onto `main`, clean up the worktree, and package/install `/Applications/Operon.app`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `src/app.rs` (`9,882` lines) or `src/app/screens.rs` (`9,900` lines) crosses `10,000` lines | `bash scripts/check-bands.sh` reports `largest_module_lines` breach | Placing new methods in `src/app/terminal_output.rs` and `src/app/relaunch.rs`, sharing `refresh_session_output` guard in `src/app.rs`, and replacing inline blocks in `src/app.rs` with single-line calls. |
| Off-tab closed `Lost`/`Failed` sessions accidentally turn into plain terminal shells | Existing tests `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`, `status_291_midrun_crash_auto_restores_in_same_tab_with_toast`, or `model_check_*` fail | Gating fallback shell conversion strictly on `self.session_has_a_tab(id)` and running full `relaunch_270`, `pane_288_`, `status_291_`, and `model_check_*` tests. |
| Stale `explicit_output_requests` entry or `select_default_terminal_session` fallback bypasses the missing-pane guard | Later passive `Gone` capture emits `ターミナル出力を読み取れませんでした`, or opening workspace with `Queued` session runs tmux capture | Removing `session_id` from `explicit_output_requests` on every `handle_session_output_result` completion, guarding `refresh_session_output`, and asserting both in `pane_303_missing_pane_does_not_loop_output_requests_or_spam_error_banner`. |
| Pre-launch `SessionOutput` race sets `terminal_capture_missing` while tmux session is still starting | Newly started session pane stays stuck showing placeholder without output | Ignoring non-explicit `Gone` errors while `session_launch_running(id)` is true and clearing `terminal_capture_missing` in `SessionStarted` / `EmptySessionStarted` / `apply_session_poll(Alive)`. |
| Unsorted or missing row in `src/i18n_tables.rs` | `every_message_the_code_shows_has_a_row_in_every_table` or `every_translation_table_is_sorted_by_message_id` fails | Running all translation table tests locally before commit. |
| Auto-restore + fallback shell enters an infinite restart loop on persistent error | `AUTO_RESTORE_COOLDOWN` ignored on fallback shells | Recording `record_auto_restore(id, now)` for both resumed CLIs and fallback shells and testing cooldown in `pane_303_*` and `model_check_pane_303_*`. |

## Proof of completion

- `cargo fmt --check` — no output, exits 0.
- `cargo clippy --locked -- -D warnings` — compiles with zero warnings.
- `cargo test --locked pane_303_` and `cargo test --locked model_check_pane_303_` — all pass after failing prior to implementation.
- `cargo test --locked` (in CI) and local regression suites (`pane_288_`, `status_291_`, `relaunch_270`, `terminal_276_`, `tui_perf_292_`, `launch_modal`, `opening_a_session_selects_it`, `terminal_workspace_prefers_a_running_session`, translation table tests) — all pass.
- `cargo test --locked -- --ignored` — six live-CLI restore/archive tests pass (or any environment limit is explicitly recorded).
- `python3 docs/sdlc/changes/303-missing-pane-loops-output-error/mutations.py` — every mutation caught.
- `bash scripts/check-bands.sh`, `bash scripts/check-approvals.sh docs/sdlc/changes/303-missing-pane-loops-output-error`, and `bash scripts/check-review.sh --index` — all exit 0.
- `bash scripts/check-installed-build.sh` after packaging and swapping `/Applications/Operon.app` on `main` — reports the installed app matches `HEAD`.

## Departures from the plan

- `start_session` checks `resolve_or_recover_native_session` when `native_session_id` is `None` so manual retry on an un-ID'd CLI session with a recoverable local transcript reopens its conversation instead of discarding it.
- Added `RelaunchAttempt` (`Launched | ConversationUnavailable | LaunchRefused | PersistFailed`) and `RecoveryOutcome::PersistFailed` (verifier findings `F1` and `F2`), keeping `lost_tabs_on_load` and `exited_tabs_on_load` unspent when `persist()` fails during startup recovery, preventing `LaunchRefused` from wiping resumable CLI metadata into `"terminal"`, and making `recover_cli_conversation_in` idempotent on `Claude`. Marked `relaunch_lost_session` and `reopen_exited_session_as_shell` `#[cfg(test)]` after moving production callers to `*_attempt`.
- Guarded `poll_terminal_echo` and `finish_layout_drag` (verifier finding `F3`) with `session_eligible_for_initial_output` / `refresh_session_output`.
- Kept `BackgroundResult::TerminalResized` `Gone` independent of `self.tools.tmux` because existing Change 128 tests run with `ToolStatus::default()` (`tools.tmux == false`) while expecting `Gone` to transition `Active` to `Lost`/`Cancelled` (with `auto_restore_after_resize_gone` gating recovery on `self.tools.tmux`).
- Guarded `flush_terminal_resize` with `session_launch_running(session_id)` so rendering an `Active` session whose `SessionStart` / `EmptySessionStart` is still in flight does not spawn a premature `tmux_resize` background task before the tmux session exists.
