# Plan: Keep cleanly exited panes as live shells and auto-restore force-terminated sessions

- **Spec**: `./spec.md`
- **Approved**: 2026-11-12
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `AUTO_RESTORE_COOLDOWN` (`15` seconds) for bounding automatic session restoration. |
| `src/tmux.rs` | Set `remain-on-exit-format ""` in `configure_tmux_session`; add `set_tmux_session_environment` and `reopen_or_start_plain_terminal` so launching a plain terminal on a `Dead` pane respawns the login shell in place while keeping scrollback. |
| `src/app/relaunch.rs` | Add `is_interactive_terminal_session`, `reopen_exited_workspace_tabs_on_load`, `reopen_exited_session_as_shell`, `auto_restore_ready`, `clear_auto_restore_cooldown`, `restore_forced_sessions`, and post-poll transition helpers; stamp `last_auto_restore` on automatic relaunches. |
| `src/app.rs` | Add `last_auto_restore` and `exited_tabs_on_load` fields on `OperonApp`; wire startup tab check in `from_state`, cooldown reset in `start_session` / `resume_managed_native_session`, quiet on-screen output refresh in `BackgroundResult::SessionStarted`, and clean-exit / forced-restore handling in `apply_session_poll` while keeping `src/app.rs` below `10,000` lines. |
| `src/i18n_tables.rs` | Add sorted English (`EN_TABLE`) and Korean (`KO_TABLE`) translations for the two forced-restore toast messages. |
| `src/tests.rs` | Add empirical unit/tmux tests for R1–R6 and an exhaustive `model_check_pane_288_*` test; update change 270's two poll-relaunch tests in place for the new clean-exit and mid-run restore rules. |
| `README.md`, `README.ja.md`, `README.ko.md` | Document clean-exit shell continuation and mid-run forced-termination restoration in all three languages. |
| `docs/sdlc/changes/288-dead-pane-freezes-terminal/*` | SDLC state, intent, spec, screen, plan, approvals ledger, mutation sweep (`mutations.py`), and verification evidence (`verification.yaml`). |

## Order of work

1. Validate `intent.md`, `spec.md`, `screen.md`, and `plan.md` with `scripts/check-readiness.sh` and an independent evaluator subagent, record approvals in `approvals.log`, and commit the Stage 1–2 SDLC documents on the feature worktree branch before touching `src/`.
2. Write the new R1–R6 tests and formal model check in `src/tests.rs` first (with minimal stub signatures if needed to compile) and watch them fail for the expected reasons.
3. Implement `AUTO_RESTORE_COOLDOWN` in `src/config.rs` and the tmux dead-pane suppression and scrollback-preserving shell respawn in `src/tmux.rs`.
4. Implement the clean-exit shell transition, startup legacy `Exited` open-tab recovery, mid-run forced-termination auto-restore with cooldown, quiet on-screen output refresh, and sorted `src/i18n_tables.rs` entries across `src/app/relaunch.rs`, `src/app.rs`, and `src/i18n_tables.rs`; update `README.md`, `README.ja.md`, `README.ko.md`, and the two change-270 tests in `src/tests.rs`.
5. Run named tests, write and run `docs/sdlc/changes/288-dead-pane-freezes-terminal/mutations.py` (including mutations on the Japanese toast claims), run independent empirical/formal verification checks, verify `scripts/check-bands.sh` and `make q.fast`, record `verification.yaml` at the staged index digest, and commit on the worktree branch.
6. Push `feature/288-dead-pane-freezes-terminal`, wait for GitHub Actions CI (`scripts/ci-verified.sh`), merge `--no-ff --no-edit` onto `main`, push `main`, clean up the worktree, and rebuild/install `/Applications/Operon.app` via `.claude/skills/ship/SKILL.md`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `src/app.rs` crosses the 10,000-line `propose` band | `scripts/check-bands.sh` flags `largest_module_lines` | Placing new methods in `src/app/relaunch.rs`, moving post-poll recovery dispatch out of `src/app.rs`, and running `bash scripts/check-bands.sh`. |
| Respawning a plain shell wipes scrollback by running `kill-session` first | Previous agent output disappears when CLI exits with status 0 | `pane_288_configure_tmux_suppresses_dead_pane_banner_and_keeps_scrollback` asserts prior lines survive above the respawned shell output on real tmux. |
| Broken CLI or missing folder triggers an infinite auto-restore loop | Session flips between `Starting` and `Failed` every poll | `AUTO_RESTORE_COOLDOWN`, `pane_288_forced_kill_mid_run_restores_cli_and_shell_with_cooldown`, and `model_check_pane_288_poll_transitions_and_cooldown`. |
| Unsorted or missing translation row in `src/i18n_tables.rs` | `every_message_the_code_shows_has_a_row_in_every_table` or `every_translation_table_is_sorted_by_message_id` fails | Running both translation tests locally before committing. |
| Parent session of a queued dependent turns into a live shell instead of `Exited`, stalling the queue | Dependent `Queued` session never starts | Guarding clean-exit shell conversion on `queued_dependency_count == 0` and testing queued dependency preservation in `pane_288_clean_exit_reopens_interactive_session_as_shell`. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked pane_288_` and `cargo test --locked model_check_pane_288_` — all pass after failing before the implementation.
- `cargo test --locked relaunch` and `cargo test --locked translation_table` — all pass.
- `python3 docs/sdlc/changes/288-dead-pane-freezes-terminal/mutations.py` — every mutation caught.
- `bash scripts/check-bands.sh` and `bash scripts/check-review.sh --index` — exit 0.
- `bash scripts/check-installed-build.sh` after packaging on `main` — `/Applications/Operon.app` matches `HEAD` and reopens existing dead-pane tabs as live terminals.

## Departures from the plan

- Added `preserve_dead_pane_scrollback` (`tmux send-keys -R -t <name>`) before `respawn-pane -k` in `reopen_or_start_plain_terminal` so visible cells of the dead pane are pushed into scrollback history rather than overwritten at cursor `(0, 0)` on `tmux 3.8`.
- Hardened seven edge cases found during independent verification (`pane_288_f1_` through `pane_288_f7_`): ignoring late `NativeSessionResolved` results after shell conversion, keeping `Failed` macOS notifications when `relaunch_lost_session` returns `false`, keeping `SessionStatus::Exited` untouched in `self.store` while queued in `exited_tabs_on_load` before the first poll, atomically rolling back CLI metadata if `launch_session_record` refuses the shell launch (and ignoring stale CLI account bindings on plain terminals), unsetting removed `MANAGED_TMUX_ENV_KEYS` on dead-pane respawn, preserving persistent poll error banners ahead of restoration toasts, and dropping stale `exited_tabs_on_load` poll observations if the session was manually restarted or its tab was closed while the startup poll was in flight.
