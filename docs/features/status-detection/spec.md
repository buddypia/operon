# Spec: 状態の検知

## Behaviour
1. Each live session's activity is `Working`, `AwaitingInput`, or `Idle` (`AgentActivity`). `session_activity_for` returns the fresh hook status when there is one, else what the screen tracker last settled on. The row state is `session_status_kind` over that plus queue, cancellation, and the not-started flag.
2. Status groups shown to the person are exactly three, in this order: 「要対応」 (waiting for an answer, or crashed with no auto restore, or blocked by a failed prerequisite), 「実行中」 (starting, working, stopping, queued), 「アイドル」 (turn finished, or not started).
3. 「未開始」 is a live `Active` Claude Code or Codex session that has had no first request: the hook `SessionStart` with a startup/clear source sets it, a first `UserPromptSubmit` clears it, a resume leaves 「アイドル」. It counts in the アイドル group, has its own centred-dot mark and the hover 「まだ最初の依頼を受けていません」, and survives a stale hook but not a turn or a new launch.
4. Hook mechanism: every managed session gets `OPERON_SESSION`, `OPERON_LAUNCH_TOKEN`, `OPERON_HOOK_ENDPOINT`, `OPERON_HOOK_SOCKET` and `OPERON_EXECUTABLE` in its environment. Operon listens on a Unix socket in `agent-hooks/` of the data directory; one managed shell script per CLI posts each event to it. An event whose launch token differs from the recorded one, or whose session is unknown, is ignored.
5. Events registered: Claude `SessionStart`, `UserPromptSubmit`, `Stop`, `StopFailure`, `PostCompact`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest` (never `PreCompact`); Codex `SessionStart`, `UserPromptSubmit`, `Stop`, `PreToolUse`, `PostToolUse`, `PermissionRequest`; Antigravity `PreInvocation`, `PostInvocation`, `Stop`, `PreToolUse`, `PostToolUse`. Claude's three tool events are registered with `"async": true`; every other handler is synchronous.
6. Install writes into `~/.claude/settings.json`, `~/.codex/hooks.json` (plus the feature flag and `trusted_hash` entries in `~/.codex/config.toml`), and the `operon-status` bundle of `~/.gemini/config/hooks.json`, plus every registered account folder. Foreign entries are kept; removal deletes only what Operon added. The settings screen reports 「フック 導入済み」/「フック 未導入」/「フック 失敗」 per tool.
7. A hook reading is authoritative for `HOOK_STALE_AFTER_SECONDS` (30 minutes) after its last event. Older, or none, and the screen decides. Dropping a stale reading also drops the screen tracker, so the handover is a baseline and announces nothing.
8. Screen reading (`detect_agent_activity`) looks only at the last `ACTIVITY_STATUS_TAIL_LINES` (12) non-blank lines: `esc to interrupt`, `esc to cancel`, `ctrl+c to interrupt`, or a Claude turn timer (an ellipsis followed by `(N s`) means `Working`; a workspace-trust prompt or numbered yes/no list means `AwaitingInput`; anything else is `Idle`. Working wins over the prompt shapes.
9. `observe_agent_activity` acts on `Working` at once but needs `ACTIVITY_CONFIRMATIONS` (2) agreeing polls to leave it.
10. Notices (`activity_notice`): Working→Idle is 「…が完了しました」, Working/Idle→AwaitingInput is 「…が入力を待っています」; the first settled reading is a baseline and a session boundary (`SessionStart` startup/resume/clear, manual `PostCompact`) never notifies. macOS notifications are sent only when 「ターンの完了・入力待ち・終了を通知する」 is on.
11. Working time (`track_working`, `working_time_label`): a session whose kind is `Working` is timed from when this run first saw it; under one minute shows nothing, 1 to 59 shows 「{n}分」, 60 and over shows 「{n}時間」, appended as 「 · 」 to the pane header, the project row, and the state hover. It restarts when the agent stops and resumes, and is not persisted.
12. A hook status keeps its turn's start time across tool events in the turn, and keeps its last message (at most `HOOK_LAST_MESSAGE_MAX_CHARS`, 200).
13. The workspace-trust prompt is answered automatically only when 「初回起動時のフォルダ信頼プロンプトを自動承認する」 is on, only for a prompt matched on the visible screen with a single affirmative option, by moving to that option; anything else is left alone.
14. The window's state per session is published to `agents.json` (see agents-cli) using the same reading.

## States
- Hook toggle off or CLI missing: that tool shows 「フック 未導入」 with the reason on hover; the screen reading decides.
- Hook install error: 「フック 失敗」; the reason shows on hover; nothing else is changed.
- A registered account folder that does not exist is skipped and said, never created.
- Hook body over `HOOK_BODY_MAX_BYTES` (64 KiB) is dropped; headers over `HOOK_HEADER_MAX_BYTES` (8 KiB) likewise; an idle connection is dropped.
- Unrecognised screen wording reads `Idle`, so no false completion is announced.
- A Claude tool post that arrives late (async) after a Stop or after the question it preceded does not turn the row back to working or hide the question.
- Antigravity's tool event after its Stop does not turn a finished row yellow again.

## Invariants
- Hook scripts answer the CLI first and exit 0 on every path; their post is capped at 1.5 s; the hook entry timeout is `HOOK_ENTRY_TIMEOUT_SECONDS` (60).
- Antigravity's `PreToolUse` answer is `ask`, never `allow`.
- Install is idempotent, preserves foreign entries and file mode, and writes only when the document changed.
- The last `HOOK_SEEN_TOOL_IDS_MAX` (64) Claude tool-call ids are kept for ordering.
- `PreCompact` is never registered (an aborted compaction would pin a session at working).
- A reading is never used from a session whose launch token does not match.
- Status labels stay in the four group words plus 「未開始」; no colour-only distinction.

## Code and tests
- Symbols:
  - `src/tmux/hooks.rs::hook_events`, `hook_runs_async`, `hook_stdout_answer`, `hook_script`: what is registered and how it answers.
  - `src/tmux/hooks.rs::apply_hooks`, `install_claude_hooks`, `install_codex_hooks`, `install_antigravity_hooks`, `apply_codex_config`: install and removal per CLI.
  - `src/tmux/hooks.rs::HookListener`, `serve_hook_connection`, `parse_hook_envelope`: the socket.
  - `src/tmux/hooks.rs::normalise_hook_event`, `HookReading`: an event to working/waiting/done.
  - `src/tmux/hooks.rs::apply_hook_reading`, `HookStatus`, `hook_status_is_fresh`: state machine, ordering guards, 30 minute freshness.
  - `src/tmux.rs::detect_agent_activity`, `is_agent_working`, `has_running_turn_timer`, `is_agent_input_prompt`, `agent_status_tail`: screen reading.
  - `src/tmux.rs::observe_agent_activity`, `activity_notice`, `activity_notification_body`: confirmation and notices.
  - `src/tmux.rs::auto_approve_workspace_trust_prompt`, `workspace_trust_answer_keys`: trust prompt.
  - `src/app/working_time.rs::working_time_label`, `track_working`, `with_working_time`: working clock.
  - `src/app.rs::session_activity_for`, `fresh_hook_status`, `drain_hook_events`, `apply_hook_event`, `session_status_kind_for`: where readings meet the row.
  - `src/config.rs::HOOK_STALE_AFTER_SECONDS`, `ACTIVITY_STATUS_TAIL_LINES`, `ACTIVITY_CONFIRMATIONS`: limits.
- Tests:
  - `reads_running_and_finished_screens_of_every_agent_cli`: working versus finished screens.
  - `reads_a_prompt_that_needs_an_answer_as_awaiting_input`: prompt shapes.
  - `ignores_interrupt_hints_that_scrolled_above_the_footer`: the 12-line tail.
  - `reads_run_state_from_the_live_pane_rather_than_its_scrollback`: live pane only.
  - `announces_a_prompt_that_appears_while_idle_or_working`: notices.
  - `a_session_boundary_never_notifies`: boundary events are silent.
  - `a_hook_state_keeps_its_turn_start_and_then_goes_stale`: turn start and 30 minute expiry.
  - `a_bookkeeping_event_after_an_antigravity_stop_keeps_the_row_finished`: agy ordering.
  - `a_late_tool_event_does_not_reopen_a_finished_turn`: late Claude post after Stop.
  - `a_late_tool_event_does_not_hide_a_question`: late post versus a wait.
  - `claude_tool_hooks_run_async_and_every_other_handler_waits`: async rule.
  - `claude_hook_install_is_idempotent_and_keeps_every_foreign_entry`: install.
  - `removing_hooks_restores_each_settings_file_to_its_foreign_entries`: removal.
  - `a_missing_cli_is_reported_rather_than_registered`: 「フック 未導入」.
  - `codex_hook_install_writes_a_config_codex_can_load`: Codex config.
  - `antigravity_hook_install_owns_only_its_bundle`: agy bundle.
  - `the_hook_socket_round_trips_a_posted_event`: socket round trip.
  - `an_installed_hook_command_posts_a_reading_end_to_end`: script end to end.
  - `every_managed_hook_script_is_valid_shell_that_cannot_fail`: scripts exit 0.
  - `working_278_clock_runs_only_while_the_agent_works`: working clock.
  - `working_278_time_is_said_in_minutes_then_hours`: label format.
  - `unstarted_308_a_session_is_not_started_until_its_first_request`: 未開始 rule.
  - `unstarted_308_the_mark_outlives_a_stale_hook_but_not_a_turn_or_a_launch`: 未開始 persistence.
  - `recognizes_only_workspace_trust_prompts_for_auto_approval`: trust prompt match.
  - `a_menu_that_is_not_a_trust_prompt_is_never_answered`: no other answers.
  - `status_291_groups_are_three_and_done_stopped_avoid_finished_word`: three groups.

## Decisions inherited
- Change 015: state comes from the CLIs' own hooks first and from the bottom of the screen second; a hook reading is authoritative for 30 minutes; an unknown screen reads idle so no false completion is announced.
- Change 060: write only keys the CLI's own schema accepts (Codex rejected an Operon `hooks` key).
- Change 056: the trust auto-answer must find the affirmative option, not press whatever is highlighted.
- Change 278: working time is derived in memory from when this run first saw the turn; it is not stored.
- Change 281: waiting agents are counted on the Dock icon.
- Change 291: three status groups; the Finished state no longer blurs with 「アイドル」.
- Change 301: Claude's tool hooks run async so every tool call is not blocked; the ordering guard (seen tool ids) absorbs the resulting races.
- Change 308: 「未開始」 is a separate live state inside 「アイドル」, so a broadcast line cannot become another agent's first request by accident.
- Change 292: the output watcher coalesces bursts and does not poll blindly.
