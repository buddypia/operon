# Spec: 停止・再試行・再開

## Behaviour
1. `session_verb` returns at most one verb per session, by status and by whether a cancellation is pending. A pending cancellation: 「停止中…」 (action Stop; hint says pressing again retries the stop). `Active`: 「停止」 (Stop, danger). `Queued`: 「今すぐ開始」 (Start). `Failed`/`Lost`: 「会話 ID から再開」 (ResumeNative) when `native_session_id` is set, else 「再試行」 (Start). `Exited`/`Cancelled` with a native id: 「会話 ID から再開」. Every other state has no verb. Pinned by `session_verb_names_one_verb_per_state`, `only_an_idle_stop_loses_its_red`.
2. The resume verb's hover lists the carried launch options (「引き継ぐ設定: …」); the verb is `danger` when the carried command holds a dangerous flag. Pinned by `the_resume_hint_names_what_it_will_carry`.
3. `stop_session` refuses anything but an `Active` session (notice 「停止できるのは、実行中またはキャンセル保留中のセッションだけです。」), ignores a second press while a stop task runs, writes the cancellation intent to the store first (`persist_cancellation_intent`), then runs `tmux kill-session` in the background with a 5 s timeout. A failure is accepted when tmux is then observed `Dead` or `Gone`. Pinned by `cancellation_intent_survives_restart_before_external_stop_result_is_saved`.
4. A stop that has not landed keeps its intent; a later poll that finds the terminal gone settles the session as `Cancelled`, closes its tab (unless it is being removed) and clears its unread mark.
5. Retry (`prepare_tmux_for_retry`) asks `tmux_observation` first: `Alive` is refused (「そのターミナルはまだ実行中です。…」), `Dead` is killed then retried, `Gone` proceeds, `Unknown` is refused (「古いターミナルがまだ開いているか判定できませんでした。」). A cleanup error is forgiven if a re-observation shows the terminal gone. Pinned by `retry_state_matrix_refuses_unsafe_terminals_and_cleans_only_dead_ones`, `retry_preparation_preserves_live_tmux_and_removes_dead_tmux`.
6. `can_retry_session` is true only for `Failed` and `Lost`. A stored agent command containing a line break is refused at retry. Pinned by `retry_state_matrix_refuses_unsafe_terminals_and_cleans_only_dead_ones`, `a_stored_agent_command_with_a_line_break_is_refused_at_retry`.
7. Deleting a session: the row's ✕ opens 「実行中のセッションを削除しますか？ 停止後に一覧から取り除きます。」 for a running session; 「削除」 stops it, then `remove_session_record` removes the record after the terminal is gone. Files and worktrees are never deleted.
8. 「ターミナルを閉じる」/「残ったターミナルを閉じる」 kills the leftover tmux session of an ended session (`close_tmux_session_confirming_gone`, 5 s timeout; a disappearance during the call counts as success). Pinned by `terminal_cleanup_accepts_a_concurrent_disappearance_but_not_a_live_failure`.
9. Closing a session tab (✕, ⌘W) only removes it from the screen; the session and its agent keep running; ⇧⌘T reopens it.
10. Queued sessions start when their prerequisite finishes (`ready_queued_session_ids`, `start_ready_queued_sessions`); 「今すぐ開始」 starts one early. A session whose prerequisite failed is blocked and does not start by itself. Cancelling or removing a queued session is durable. Pinned by `cancelling_a_queued_session_is_durable`, `removing_a_queued_session_cancels_then_deletes_its_record`, `persisted_ready_queue_is_discovered_after_restart`.
11. Workspace layout (tabs per group, order, front tab, split shares, focus, selected session, which project the workspace shows) is written to `workspace-layout.json` beside the store within one two-second tick of a change and on quit, and read back on start. Nothing is written before the first layout or when unchanged. Pinned by `the_workspace_comes_back_after_a_relaunch`, `a_damaged_workspace_file_restores_only_what_is_whole`.
12. At start, a session stored as `Starting`/`Active`/`Unknown` whose terminal the first look finds gone or dead (`Lost`/`Failed`) is started again in place (same record, tab, tmux name) without moving the selected tab or page: a CLI with a saved conversation reopens it with its original launch options (`RelaunchKind::Resume`), a plain terminal opens a shell (`RelaunchKind::Shell`). A brief toast reads 「前回開いていたセッションを {count} 件再開しました。」. Pinned by `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`, `the_relaunch_says_how_many_sessions_it_reopened`, `model_check_a_session_is_relaunched_only_at_its_first_stored_look`.
13. During a live run, a forced kill, crash, or lost tmux session of an interactive session with a relaunch kind is auto-restored in the same tab without taking focus. Pinned by `status_291_midrun_crash_auto_restores_in_same_tab_with_toast`, `status_291_auto_restore_cooldown_stops_crash_loop_until_manual_retry`.
14. A clean exit (status 0) of an interactive terminal/agent session (`is_interactive_terminal_session`) leaves a login shell in the same pane and folder; the session's kind becomes `terminal`, the tab stays, and no `Pane is dead` footer is drawn. Open tabs saved as `Exited` by an older build are re-checked at load. Pinned by `pane_288_clean_exit_reopens_interactive_session_as_shell`, `pane_288_configure_tmux_suppresses_dead_pane_banner_and_keeps_scrollback`.
15. Before a native resume the transcript is checked (`check_transcript_resume_readiness`): healthy proceeds; empty or missing falls back to a fresh start with a Japanese notice; a truncated last line is cut back to valid JSONL first. A Claude conversation whose identity rotated is resumed under the corrected id. Pinned by `resume_refuses_empty_transcript_and_falls_back_to_clean_start`, `resume_sanitizes_truncated_trailing_transcript_line`, `resuming_a_session_with_no_transcript_starts_it_fresh_once`.
16. A terminal in which a CLI reports a conversation id takes that CLI's kind (`take_cli_kind`); a session launched as a CLI never changes kind. Pinned by `a_terminal_takes_the_kind_of_the_cli_running_in_it`.

## States
- tmux missing: no automatic restore, relaunch, or retry is attempted (`self.tools.tmux`).
- Cancel pending and the store cannot confirm it durable: stop does not proceed until a recovery copy is confirmed.
- Restore refused: custom command, CLI session with no conversation id or no transcript, a session stopped before the quit, one that exited cleanly before the quit, or one still alive. It stays `Lost`/`Failed` with its manual verb and no toast.
- Missing, unreadable, over 1 MiB, or malformed `workspace-layout.json`: what is valid is restored, a project whose layout cannot be made whole falls back to opening its live sessions; unreadable file equals no file. Never a pane without a group, a session in two groups, more than `PANE_MAX` panes, or a share outside 0..1.
- A layout write that fails is said once.
- Persist failure after finding tmux gone keeps the session `Lost` and retries the save in the background.

## Invariants
- `AUTO_RESTORE_COOLDOWN` is 15 seconds (`AUTO_RESTORE_COOLDOWN_SECONDS`): a second automatic restore of the same session inside it is refused and the session settles in `Failed`/`Lost`. A manual start or resume clears the cooldown.
- Stop is recorded as intent before tmux is touched, so a crash or restart between the two still ends as `Cancelled`.
- Retry never kills a live terminal and never proceeds when liveness is `Unknown`.
- Every child process of these paths goes through `run_command_with_timeout` (5 s for kill-session).
- A resume with no transcript never re-enters the resume path (it starts fresh once); an auto restore never selects the session, changes the page, or replaces the notice.
- Tmux session names are built through `managed_tmux_name` and checked by `is_safe_tmux_name`; the prefix is spelled in one place.
- A cleanly exited unpinned session closes its tab into 履歴; a pinned one stays.

## Code and tests
- Symbols:
  - `src/tmux.rs::tmux_observation` and `parse_tmux_observation`: Alive/Dead/Gone/Unknown plus exit status of a session's pane.
  - `src/tmux.rs::can_retry_session`, `retry_tmux_action`, `prepare_tmux_for_retry`: retry gate and old-terminal cleanup.
  - `src/tmux.rs::close_tmux_session_confirming_gone`, `tmux_cleanup_failure_result`: kill with re-observation.
  - `src/tmux.rs::launch_stored_session`, `stored_launch_command`: relaunch from the stored record.
  - `src/tmux.rs::respawn_tmux_pane`, `dead_pane_reopen_steps`, `reopen_or_start_plain_terminal`, `configure_tmux_session_steps`: shell fallback and the dead-pane footer suppression.
  - `src/app.rs::session_verb`, `resume_verb`, `SessionVerb`: the one verb per state.
  - `src/app.rs::stop_session`, `remove_session_record`, `start_ready_queued_sessions`: stop, delete, queue start.
  - `src/app/relaunch.rs::relaunch_kind`, `RelaunchKind`, `take_cli_kind`, `terminal_provenance`: what a gone session is restarted as.
  - `src/app/relaunch.rs::reconcile_observed_tmux_gone`, `auto_restore_after_resize_gone`, `relaunch_remembered_sessions`, `restore_forced_sessions`: first-look and mid-run restore.
  - `src/app/relaunch.rs::auto_restore_ready`, `record_auto_restore`, `clear_auto_restore_cooldown`: the 15 s cooldown.
  - `src/app/relaunch.rs::restore_workspace_layout`, `workspace_layout_snapshot`, `save_workspace_layout_if_changed`: tab and split persistence.
  - `src/ui/session_tree.rs::WorkspaceLayout`, `load_workspace_layout`, `save_workspace_layout`: the saved layout types and file.
  - `src/cli.rs::check_transcript_resume_readiness`: transcript validation before resume.
- Tests:
  - `session_verb_names_one_verb_per_state`: the verb table above.
  - `the_resume_hint_names_what_it_will_carry`: the hover lists carried options.
  - `only_an_idle_stop_loses_its_red`: red hover rule for the stop verb.
  - `retry_state_matrix_refuses_unsafe_terminals_and_cleans_only_dead_ones`: retry gate by tmux state.
  - `retry_preparation_preserves_live_tmux_and_removes_dead_tmux`: retry cleanup.
  - `terminal_cleanup_accepts_a_concurrent_disappearance_but_not_a_live_failure`: kill re-observation.
  - `a_stored_agent_command_with_a_line_break_is_refused_at_retry`: command gate at retry.
  - `cancellation_intent_survives_restart_before_external_stop_result_is_saved`: stop intent is durable.
  - `cancelling_a_queued_session_is_durable`: queued cancel survives.
  - `removing_a_queued_session_cancels_then_deletes_its_record`: delete order.
  - `persisted_ready_queue_is_discovered_after_restart`: queue after relaunch.
  - `the_workspace_comes_back_after_a_relaunch`: layout round trip.
  - `a_damaged_workspace_file_restores_only_what_is_whole`: layout damage.
  - `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`: first-look restore.
  - `the_relaunch_says_how_many_sessions_it_reopened`: the toast.
  - `model_check_a_session_is_relaunched_only_at_its_first_stored_look`: relaunch only at the first look.
  - `status_291_midrun_crash_auto_restores_in_same_tab_with_toast`: mid-run crash restore.
  - `status_291_auto_restore_cooldown_stops_crash_loop_until_manual_retry`: 15 s cooldown.
  - `status_291_clean_exit_and_stop_close_unpinned_tab_but_keep_pinned_and_history_open`: tab closing on clean exit or stop.
  - `pane_288_configure_tmux_suppresses_dead_pane_banner_and_keeps_scrollback`: no dead-pane footer.
  - `pane_288_clean_exit_reopens_interactive_session_as_shell`: clean exit leaves a shell.
  - `resume_refuses_empty_transcript_and_falls_back_to_clean_start`: empty transcript fallback.
  - `resume_sanitizes_truncated_trailing_transcript_line`: torn last line.
  - `resuming_a_session_with_no_transcript_starts_it_fresh_once`: no recursion.
  - `a_terminal_takes_the_kind_of_the_cli_running_in_it`: kind follows the CLI.

## Decisions inherited
- Change 045: validate the transcript before any native resume; an empty one starts fresh and a torn last line is repaired, because blind resumes revived wrong or crashing conversations.
- Change 270: a relaunch restores tabs and splits from `workspace-layout.json` and restarts only sessions found gone at the first look; clean `/exit`, cancelled, and custom-command sessions are never restarted.
- Change 288: a clean exit keeps the pane as a login shell and suppresses tmux's dead-pane footer; crashes restore in the same tab.
- Change 291: automatic restore has a 15 s per-session cooldown so a crash loop ends in 要対応; clean exit and stop close unpinned tabs into 履歴.
- Change 303: a vanished pane must not loop output errors; adopted or untracked CLI terminals recover their conversation on crash.
- Change 123: a tmux connect error must not block deleting a session.
- Change 160: a launch must not fail when a second pty is unavailable.
- Change 156: a launch may carry the person's own arguments or a chosen conversation id.
- Change 027: a restart keeps the options the agent was launched with.
- Change 305: a CLI tab keeps its mark across a relaunch.
