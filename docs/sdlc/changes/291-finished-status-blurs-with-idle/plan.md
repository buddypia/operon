# Plan: retire the Finished status group, auto-restore killed sessions, and close clean exits to History

- **Spec**: `./spec.md`
- **Approved**: 2026-11-12
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `AUTO_RESTORE_COOLDOWN_SECONDS: u64 = 15`. |
| `src/ui/widgets.rs` | Remove `SessionStatusGroup::Finished` (`GROUPS: [SessionStatusGroup; 3]`), change `SessionStatusKind::group` to `Option<SessionStatusGroup>` (`Failed \| Lost => Some(NeedsYou)`, `Done \| Stopped => None`), name `Done` `"正常終了"` (with hint `"ターミナルは正常に終了しました"`) and `Stopped` `"停止済み"`, and update `NeedsYou::hint_id()`. |
| `src/app/session_order.rs` | Add `OperonApp::close_finished_session_tab(&mut self, id: Uuid)` (pinned skip without refusal toast, off-screen neighbour `unread_sessions` preservation, `Document` / empty active-tab `selected_session` cleanup, and `project_workspaces` `selected` cleanup) and `clear_cancelled_unread_and_close_tab(&mut self, id: Uuid)`. |
| `src/app/relaunch.rs` | Add `auto_restore_ready`, `record_auto_restore`, `clear_auto_restore_cooldown`, `relaunch_interrupted_sessions` (startup `前回開いていた…` vs mid-run `強制終了されたセッションを {count} 件自動で復元しました。` plus fallback failure names when relaunch declines), and `auto_restore_after_resize_gone`. |
| `src/app.rs` | Change `status_filter` and `status_group_counts*` from length 4 to 3; add `last_auto_restore: HashMap<Uuid, Instant>`; gate `Failed \| Lost` in `kind_in_status_group` on `session_has_a_tab \|\| is_session_unread`; call `close_finished_session_tab` / `clear_cancelled_unread_and_close_tab` on transitions to `Exited` / `Cancelled`; widen `BackgroundResult::TerminalResized` (`TmuxState::Gone`) to `Active \| Starting \| Unknown` and gate auto-restore on `self.persist()`; trigger mid-run auto-restore in `apply_session_poll`; clear cooldown in `start_session` and `resume_managed_native_session`. |
| `src/app/screens.rs` | Draw 3 Home tiles (`ui.columns(SessionStatusGroup::GROUPS.len(), ...)`), 3/2/1-column responsive sidebar tiles with per-row equal widths, `Option<SessionStatusGroup>` in `ui_workspace_projects`, and `[0usize; 3]` in session library counts. |
| `src/i18n_tables.rs` | Insert sorted EN and KO rows for `"ターミナルは正常に終了しました"`, `"停止済み"`, `"回答待ち・異常終了・前提が壊れて止まっているセッション"`, and `"強制終了されたセッションを {count} 件自動で復元しました。"`, and remove the three orphaned rows (`"終了"`, `"ターミナルが終了したセッション"`, `"回答待ち・前提が壊れて止まっているセッション"`). |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | Update status group counts/descriptions from four toggles to three, document mid-run auto-restore with cooldown, unrestorable failures surfacing in 要対応, and clean exits/stops closing unpinned tabs to History in all three languages. |
| `src/tests.rs` | Add `status_291_*` requirement tests (R1–R5); update existing 4-group pins (`every_status_label_is_one_of_the_four_group_words`, `the_documents_name_the_idle_filter_with_the_windows_word`, `the_status_filter_shows_everything_when_nothing_is_selected`, `an_unread_session_is_counted_as_needing_a_person`, `narrowing_the_list_forgets_a_pending_removal_confirmation`, `the_filtered_session_list_says_how_many_it_hides`, `a_group_tile_filters_the_session_list_to_that_group`, `sidebar_251_*`, `sidebar_252_*`, `session_285_column_lists_exactly_the_tabs`), `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`, and `model_check_a_session_is_relaunched_only_at_its_first_stored_look` in place without removing tests or adding `#[ignore]`. |
| `docs/sdlc/changes/291-…/` | `mutations.py`, `verification.yaml`, `state.yaml`, `approvals.log`. |

## Order of work

1. Commit Stage 1–2 artifacts (`state.yaml`, `intent.md`, `spec.md`, `screen.md`,
   `plan.md`, `approvals.log`) after `check-readiness.sh`, `check-approvals.sh`,
   and `harness_documents_only_name_paths_that_exist` pass.
2. Write the five `status_291_*` tests in `src/tests.rs` first covering R1–R5
   (including `TerminalResized` `Gone` + `persist()` failure guard, off-screen
   neighbour unread preservation, `Document`/`project_workspaces` `selected_session`
   cleanup, and `Cancelled` unread clearing vs `Exited` unread retention).
3. Update `src/config.rs`, `src/ui/widgets.rs`, `src/i18n_tables.rs` (adding the
   4 new keys and removing the 3 orphaned keys), `src/app/session_order.rs`,
   `src/app/relaunch.rs`, `src/app.rs`, and `src/app/screens.rs`; update existing
   4-group tests and `model_check_a_session_is_relaunched_only_at_its_first_stored_look`
   (adding cooldown state and manual cooldown reset) in `src/tests.rs`.
4. Update `README*.md` and `docs/MANUAL*.md` in all three languages alongside
   `the_documents_name_the_idle_filter_with_the_windows_word`; run targeted
   tests (`status_291_`, `model_check_a_session_is_relaunched_only_at_its_first_stored_look`,
   `every_message_row_is_one_the_code_shows`, `every_translation_table_is_sorted_by_message_id`,
   `every_message_id_has_a_row_in_every_table`, `every_status_label_is_one_of_the_four_group_words`,
   `the_documents_name_the_idle_filter_with_the_windows_word`, `sidebar_251_`,
   `sidebar_252_`, `home_287_`, `session_285_`).
5. Create and run `mutations.py` in `docs/sdlc/changes/291-finished-status-blurs-with-idle/`,
   invoke an independent verification subagent to attempt falsification, run
   `make q.fast` and `bash scripts/check-bands.sh`, record `verification.yaml`
   at the staged diff digest, and commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Old closed `Failed` / `Lost` records in `store.json` stay in `要対応` forever | Inflated Home warning banner, Dock badge, and `⌘J` jumping to old closed failures | `status_291_unrestorable_failure_stays_in_needs_you_only_while_tab_open_or_unread` and a mutation removing the `session_has_a_tab` check |
| Crash loop when a resumed CLI exits non-zero immediately every poll | Repeated tmux launches every 2 seconds | `status_291_auto_restore_cooldown_stops_crash_loop_until_manual_retry`, `model_check_a_session_is_relaunched_only_at_its_first_stored_look`, and cooldown mutation |
| `TerminalResized` (`Gone`) clobbers store-retry notice on `!persist()` or misses `Unknown` | Store-retry warning disappears or pre-first-poll session stays `Unknown` | `status_291_midrun_crash_auto_restores_in_same_tab_with_toast` testing `TerminalResized` on `Unknown` and read-only store failure |
| Off-screen auto-close marks unseen neighbour tab read or leaves stale `selected_session` / `Cancelled` unread | Unseen session loses unread badge, closed ID stays in `selected_session`, or cancelled session stays in `要対応` | `status_291_clean_exit_and_stop_close_unpinned_tab_but_keep_pinned_and_history_open` and off-screen unread / `Cancelled` unread mutations |
| Orphaned, unsorted, or missing translation row in `src/i18n_tables.rs`, or stale 4-toggle doc wording | `gate-commit.sh` or CI fails | `every_message_row_is_one_the_code_shows`, `every_translation_table_is_sorted_by_message_id`, `every_message_id_has_a_row_in_every_table`, and `the_documents_name_the_idle_filter_with_the_windows_word` |
| `src/app.rs` crosses the 10,000-line `propose` ceiling | `check-bands.sh` reports a new `propose` breach | Placing helpers in `src/app/relaunch.rs` and `src/app/session_order.rs` and running `bash scripts/check-bands.sh` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past compile lines.
- `cargo test --locked status_291_` — `5 passed; 0 failed`.
- `cargo test --locked model_check_a_session_is_relaunched_only_at_its_first_stored_look` — `1 passed; 0 failed`.
- `cargo test --locked every_message_row_is_one_the_code_shows`, `cargo test --locked every_translation_table_is_sorted_by_message_id`, `cargo test --locked every_message_id_has_a_row_in_every_table`, `cargo test --locked every_status_label_is_one_of_the_four_group_words`, and `cargo test --locked the_documents_name_the_idle_filter_with_the_windows_word` — `ok`.
- `cargo test --locked home_287_`, `cargo test --locked session_285_`, `cargo test --locked sidebar_251_`, and `cargo test --locked sidebar_252_` — `ok`.
- `bash scripts/check-bands.sh` — no metric at `propose`.
- Mutation sweep (`mutations.py` in this change directory) — every mutation caught.
- In packaged `/Applications/Operon.app`: Home and the session sidebar show only 要対応・実行中・アイドル, mid-run killed sessions auto-restore in place, and `/exit` closes unpinned tabs to History.

## Departures from the plan

None yet.
