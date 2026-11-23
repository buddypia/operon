# Spec: 要対応

## Behaviour
1. A session is in the 要対応 group when its state is `Waiting` (agent awaits input or permission), `Blocked` (prerequisite failed), `Failed` or `Lost` while it still has a tab in any project workspace, or when it is marked unread (it spoke while nobody was reading it). Cleanly exited and stopped sessions belong to no group. Pinned by `status_291_unrestorable_failure_stays_in_needs_you_only_while_tab_open_or_unread`, `an_unread_session_is_counted_as_needing_a_person`, `the_attention_count_counts_a_waiting_unread_session_once`.
2. `track_attention` runs once per frame before drawing. It keeps `attention_since` (the `Instant` each session entered 要対応) and rebuilds `attention_order` only when a session joins or leaves: oldest entry first, ties broken by newest launch (`launched_at`, else `created_at`). A session that leaves and returns is timed afresh. Pinned by `the_attention_queue_orders_waiting_sessions_by_how_long_they_have_waited`, `a_session_that_stops_needing_a_person_leaves_the_attention_queue`.
3. The queue is drawn by `ui_pinned_sessions` at the head of the history list (`ui_terminal_session_tabs`, the 「プロジェクト別」 view), headed 「要対応」 with the count, in a card of its own. It is drawn only when no status chip is on, or the 要対応 chip is on, and only for queued sessions that pass the agent and status filters; a session drawn there is not drawn again under its project. The Sessions page's left column (`ui_workspace_projects`) does not draw this queue; it shows the 要対応 tile, the per-project counts and the rows of open tabs. Pinned by `the_session_list_pins_waiting_sessions_above_the_projects`, `a_pinned_session_is_not_drawn_again_in_its_project`, `chips_that_exclude_needs_you_hide_the_pinned_section`.
4. Each queue row shows: a dot in the 要対応 ink (ringed when selected), the agent chip, the title (`session_list_title`), the wait right-aligned (`attention_wait_label`: 「たった今」 under a minute, 「{n}分」 under an hour, else 「{n}時間」), a meta line `project · branch` (or the project alone), and, only from a fresh hook status, one line: 「許可または回答を待っています」 for awaiting input, or the agent's last message (trimmed, at most 2 rows, ellipsis). A status carried over from the previous turn is not offered as the question. Pinned by `a_pinned_session_shows_what_its_agent_last_said`.
5. Clicking a row calls `open_waiting_session`: the session opens in the terminal and, when its status is `Active`, the keyboard goes to its pane (`terminal_focus_request`). Pinned by `opening_a_waiting_session_that_is_not_running_asks_for_no_focus`, `opening_the_next_waiting_session_focuses_its_terminal`.
6. A row whose session status is `Active` draws a one-line reply field with the hint 「返信を入力して Enter」. Enter calls `send_waiting_reply`: the text, trimmed, then the Enter key are queued for that session's terminal, the notice 「返信を送りました」 is shown briefly, and the page and the selected session do not change. Pinned by `reply_279_running_rows_offer_a_reply_field`, `reply_279_reply_goes_to_the_sessions_terminal`, `reply_279_typing_then_enter_sends_and_clears`.
7. An Enter that confirms an IME conversion does not send (`reply_enter_sends`: one Enter belongs to each conversion committed in the frame); the field keeps the keyboard. Keys typed in the field reach only that row's session. Pinned by `reply_279_enter_that_confirms_a_conversion_does_not_send`, `reply_279_keys_typed_in_the_field_stay_out_of_the_pane`.
8. ⌘J (`attention.next`, rebindable) calls `open_next_waiting_session`: the queued session after the selected one, wrapping, or the head when the selection is not queued; it works on every page and with a terminal focused. With an empty queue it shows 「返事を待っているセッションはありません」 and changes nothing. Pinned by `the_next_waiting_chord_opens_the_next_session_in_the_queue`, `the_next_waiting_chord_with_an_empty_queue_changes_nothing`.
9. The toolbar shows a secondary button 「要対応 {n}」 (with the chord appended when bound, hover 「次の要対応セッションへ移動します」) only while the queue is not empty; it does what ⌘J does. It is a title-bar control and does not drag or maximise the window. Pinned by `the_title_bar_offers_the_next_waiting_session_while_one_waits`.
10. The Dock icon badge reads the queue length as text, and is removed at 0. `track_dock_badge` runs once per frame after `track_attention` and tells the Dock only when the number changed. Pinned by `dock_281_badge_says_how_many_wait`, `dock_281_badge_changes_only_when_the_count_does`, `dock_281_frame_keeps_the_badge`.
11. With 「設定 → 通知」 on, a macOS notification is sent when an agent finishes a turn or needs input: 「{agent} が完了しました: {session}」 / 「{agent} が入力を待っています: {session}」 (agent named Codex CLI, Claude Code, Antigravity CLI, or カスタム CLI); a session that completes or fails without being relaunched notifies 「セッションが完了しました: {name}」 / 「セッションが失敗しました: {name}」. Pinned by `names_the_agent_and_session_in_activity_notifications`.
12. A turn that ends or a question that arrives while the session is not on screen marks it unread (`note_session_activity_change`) whether or not notifications are on; opening it clears the mark unless it was marked by hand (`mark_session_unread` holds it until another session is looked at).
13. Corner notices (`ui_brief_notice_toast`): a brief notice is a card at the bottom-right of every page with `ICON_STATUS_DONE`, gone after `NOTICE_TOAST_SECONDS` (4 s) or on 「この通知を閉じる」. On the Sessions page a persistent notice (`ICON_NOTICE`) and the durability warning (`ICON_ATTENTION`, no close button) also float there, warning above notice, so nothing is inserted above the terminal workspace. Off the Sessions page they stay top banners. Pinned by `session_notice_289_terminal_workspace_never_shifts_on_notices_or_warnings`, `session_notice_289_persistent_notice_floats_stays_past_toast_timeout_and_closes`, `session_notice_289_durability_warning_floats_without_close_button`, `session_notice_289_other_pages_keep_top_banner_without_duplicate_toast`.
14. 「未読にする」 in a session row's menu marks that session unread.

## States
- Empty queue: no 要対応 card, no toolbar button, no Dock badge; ⌘J says 「返事を待っているセッションはありません」.
- A queued session whose terminal is not running: the row shows no reply field; clicking opens it without a focus request.
- A reply that is empty or whitespace, or for a session that is not `Active`: nothing is sent and nothing is said.
- The session stops waiting while a reply is half typed: the draft is dropped (`waiting_reply_drafts` is pruned when the queue changes); a draft is never persisted.
- A filter hiding a queued row keeps its draft while the session still waits.
- Not the bundled app (`cargo run`, tests): the badge call returns without touching AppKit.
- A send tmux refuses reports through the existing 「ターミナルに入力を送れませんでした: …」 notice.
- Notifications off: the unread mark and the queue still work; nothing leaves the app.
- A hook report older than 30 minutes (`HOOK_STALE_AFTER_SECONDS`) no longer speaks for the session, so the row shows no line and the screen reading takes over.

## Invariants
- The queue length equals `attention_count` for sessions in the store; the toolbar number, the Dock badge and Home's banner read the same set.
- Queue order is by entry time, never by state; it is not changed by filters.
- A reply is typed text then Enter, exactly as from the terminal pane; the row never sends a bare Enter.
- `track_attention` and `track_dock_badge` do no allocation unless the membership changed; the Dock is not messaged per frame.
- A restart drops wait timing, unread marks and drafts.
- A terminal on its way out (stop pending) is not waiting on anybody and leaves the queue.
- The notice overlay never changes the height or top of the terminal workspace (`dock_area`).

## Code and tests
- Symbols: `src/app/waiting_reply.rs::ui_pinned_sessions` draws the 要対応 card, rows and reply fields.
- `src/app/waiting_reply.rs::send_waiting_reply` queues text and Enter and says 「返信を送りました」.
- `src/app/waiting_reply.rs::reply_enter_sends` decides whether an Enter belongs to the person or an IME conversion.
- `src/app/dock_badge.rs::dock_badge_label` and `dock_badge_change` / `track_dock_badge` keep the Dock badge in step.
- `src/app/toast.rs::ui_brief_notice_toast` draws the corner overlay of brief notices, persistent notices and the durability warning.
- `src/app.rs::track_attention` builds the queue; `attention_wait`, `session_attention_line`, `open_waiting_session`, `open_next_waiting_session`, `mark_session_unread`, `note_session_activity_change` live beside it.
- `src/app.rs::next_in_attention_queue` and `attention_wait_label` are the pure helpers.
- `src/app/screens.rs` draws the toolbar button 「要対応 {p0}」 and the Sessions page tiles.
- `src/tmux.rs::activity_notification_body` words the macOS notification.
- Tests: `the_attention_queue_orders_waiting_sessions_by_how_long_they_have_waited` pins the order.
- `a_session_that_stops_needing_a_person_leaves_the_attention_queue` pins leaving the queue.
- `the_next_waiting_chord_opens_the_next_session_in_the_queue` pins wrapping.
- `opening_the_next_waiting_session_focuses_its_terminal` pins the keyboard focus.
- `opening_a_waiting_session_that_is_not_running_asks_for_no_focus` pins the no-focus case.
- `the_next_waiting_chord_with_an_empty_queue_changes_nothing` pins the empty queue.
- `the_title_bar_offers_the_next_waiting_session_while_one_waits` pins the toolbar button.
- `the_session_list_pins_waiting_sessions_above_the_projects` pins the card and `a_pinned_session_is_not_drawn_again_in_its_project` the no-duplicate rule.
- `chips_that_exclude_needs_you_hide_the_pinned_section` pins the filter rule; `opening_an_unread_pinned_session_returns_it_to_its_project` the unread exit.
- `a_pinned_session_shows_what_its_agent_last_said` pins the line; `the_needs_you_queue_is_its_own_block` pins the card.
- `reply_279_running_rows_offer_a_reply_field`, `reply_279_reply_goes_to_the_sessions_terminal`, `reply_279_empty_reply_sends_nothing`, `reply_279_typing_then_enter_sends_and_clears`, `reply_279_draft_goes_when_the_session_stops_waiting`, `reply_279_enter_that_confirms_a_conversion_does_not_send`, `reply_279_keys_typed_in_the_field_stay_out_of_the_pane` pin the reply field.
- `dock_281_badge_says_how_many_wait`, `dock_281_badge_changes_only_when_the_count_does`, `dock_281_frame_keeps_the_badge` pin the badge.
- `session_notice_289_terminal_workspace_never_shifts_on_notices_or_warnings` and `session_notice_289_persistent_notice_floats_stays_past_toast_timeout_and_closes` pin the overlay.
- `session_notice_289_durability_warning_floats_without_close_button` and `session_notice_289_other_pages_keep_top_banner_without_duplicate_toast` pin the rest of the overlay.
- `the_attention_count_counts_a_waiting_unread_session_once` and `an_unread_session_is_counted_as_needing_a_person` pin membership.
- `status_291_unrestorable_failure_stays_in_needs_you_only_while_tab_open_or_unread` pins failed sessions.
- `names_the_agent_and_session_in_activity_notifications` pins the notification text.

## Decisions inherited
- Change 139: one queue, longest-waiting first, shared by the list, ⌘J, the toolbar and Home; a session pinned there is not repeated under its project.
- Change 279: a reply is typed text plus Enter, sent without opening the session; the IME's confirming Enter never sends; drafts die with the wait.
- Change 281: the Dock badge is the queue length, told to the Dock only on a change, and only in the bundled app.
- Change 289: notices and the durability warning float bottom-right on the Sessions page so the terminal never shifts.
- Change 291: cleanly exited and stopped sessions retire to history; a crash that was not auto-restored stays in 要対応 only while its tab is open or unread.
- Change 252: 要対応 folds 「承認・回答待ち」 in; its count under the CLI choice is shown in the tile's hover.
- Change 285: the left column lists exactly the open tabs in one order and no longer sorts by state, so the queue lives in the history list rather than the column.
- Change 308: a session that has had no first request reads 未開始 and is idle, not waiting.
