# Spec: ホーム

## Behaviour
1. Home is `Page::Home`, reached with ⌘1 (`page.home`). Its header is 「ホーム」 with the line 「いま何が動いていて、どれがあなたを待っているか。」 and no action button: the toolbar's 「新しいセッション」 is the page-independent start control, so Home does not add a second primary one.
2. Directly under the header Home always draws one attention card, above the tiles, so "nothing waits" is distinguishable from "not looked". Pinned by `home_always_says_whether_anything_waits`.
3. The attention count is `attention_count()`: the number of sessions in the 要対応 group (`Waiting`, `Blocked`, `Failed`/`Lost` while the session still has a tab, and any session marked unread). A session that is two of those counts once. Pinned by `the_attention_count_counts_a_waiting_unread_session_once`, `an_unread_session_is_counted_as_needing_a_person`.
4. With a count above 0 the card is a warning-bordered card (`palette.warning`, `ICON_ATTENTION`) reading 「{n} 件のセッションがあなたを待っています」, with a secondary button 「要対応のセッションを見る」 (`show_sessions_needing_you`) and, inside the card, one session row per 要対応 session, newest `created_at` first, at most 5.
5. With a count of 0 the card shows `ICON_STATUS_DONE`, 「あなたを待っているセッションはありません」 and the muted 「要対応になったセッションは、ここの一番上に表示されます」. Pinned by `home_always_says_whether_anything_waits`.
6. Below the card is one metric tile per status group in the chips' order, named with the chips' words (「要対応」「実行中」「アイドル」), counting every session in the store with `status_group_counts` (not limited to sessions with a tab). Clicking a tile calls `show_status_group`: page becomes Sessions, the history library opens in list view, the search is cleared, every filter is cleared, and only that group's chip is on. Pinned by `home_status_entry_opens_filtered_history_with_visible_controls`.
7. Below the tiles, 「最近のセッション」 lists the 6 newest sessions by `created_at` (any status), each as a home row, with a quiet 「すべて表示」 button that switches to `Page::Sessions`.
8. Beside it a 300 px column headed 「プロジェクトから始める」 lists every project in store order with `ICON_PROJECT`, the (truncated) name and a button 「セッション」 that calls `open_project_session_setup(project.id)`. Under 720 px of available width the column is drawn under the recent list instead; with no projects the column is not drawn. Pinned by `starting_from_a_project_on_home_opens_that_projects_launch_form`.
9. A home row (`home_session_row`) is one line: the state mark (hover shows the state label and hint, no permanent state word), title, agent chip, project name (「登録されていないプロジェクト」 if missing), branch with `ICON_BRANCH`, relative time; right-aligned a 「⋯」 menu (hover 「その他の操作」) and the action. Pinned by `home_287_row_draws_no_state_word`, `home_287_state_word_appears_on_hover`.
10. The action is the session verb button when `session_verb` is Start or ResumeNative; otherwise a secondary 「開く」. Clicking the row outside its controls opens the session (`open_session_in_terminal`). Pinned by `a_recent_session_is_one_line_with_its_details_behind_the_menu`.
11. The 「⋯」 menu holds, in order: the Stop verb when offered; 「ターミナルを閉じる」 (Exited) or 「残ったターミナルを閉じる」 (Failed); 「セッションを削除」 (asks for confirmation under the row); then small details: the goal when it differs from the title, `tmux: <name>`, `cwd: <path>`, the native resume command or 「ネイティブ再開 ID を待っています」, the dependency count and the blocking prerequisite.
12. A waiting session that is also recent is drawn twice in one frame; the two rows take their egui ids from the list name (`"waiting"` / `"recent"`) so they never clash. Pinned by `home_always_says_whether_anything_waits`.
13. The state mark on a row is `session_status_icon` over the session's `status_view`; a row's removal confirmation (`session_removal_confirmation`) draws inside its card.
14. Home reads no data of its own: every figure comes from the store and the per-frame attention queue, so it changes as the sessions do.

## States
- No sessions and no projects: 「セッションはまだありません。プロジェクトを追加すると、ここに履歴が並びます。」 in an empty state.
- No sessions but projects exist: 「セッションはまだありません。ツールバーの「新しいセッション」から始められます。」
- Nothing waiting: the green line of Behaviour 5, never an empty gap.
- A session whose project is not in the store shows 「登録されていないプロジェクト」.
- A session with no verb (running, working) shows 「開く」, not a start button.
- Notices on Home draw as top banners (change 289 leaves non-Sessions pages as they were).

## Invariants
- The Home attention number and the 要対応 chip on the Sessions page are the same predicate (`session_in_status_group` with `NeedsYou`); neither may be redefined separately.
- Tiles count all sessions through `status_group_counts`; the column's own tiles use `status_group_counts_for_chosen_agents` (tabs only), so the two sets of numbers may differ by design.
- At most 5 waiting rows and 6 recent rows are drawn; both are cloned only for those rows (a row takes `&mut self`).
- Home draws no state word on a row; the word lives in the hover only.
- Home never starts a session by itself: every start goes through `open_project_session_setup` or the toolbar.
- Drawing code reads colours from `Palette` and glyphs from `ICON_*` (policy tests); the warning card uses `palette.card` with a `palette.warning` stroke, never the selection surface.

## Seen in the running app
`python3 scripts/verify-app.py run home` starts the built application apart from the person's data, follows these steps, and keeps a picture and the window's tree for each (change 314). One `verb words` per line; the words are what the screen shows.

    start project
    see あなたを待っているセッションはありません
    see プロジェクトから始める
    see verify-demo
    shot home

## Code and tests
- Symbols: `src/app/screens.rs::ui_home` draws the header, attention card, tiles and the two columns.
- `src/app/screens.rs::home_recent_sessions` draws 「最近のセッション」 and its empty states.
- `src/app/screens.rs::home_project_starts` draws 「プロジェクトから始める」.
- `src/app/screens.rs::home_session_row` draws one session line with its action and 「⋯」 menu; `home_session_details` fills the menu's detail lines.
- `src/app.rs::attention_count` counts the 要対応 group; `session_in_status_group` / `kind_in_status_group` decide membership.
- `src/app.rs::status_group_counts` counts every group in one allocation-free pass for the tiles.
- `src/app.rs::show_status_group` and `show_sessions_needing_you` open the filtered history list.
- Tests: `home_always_says_whether_anything_waits` pins the always-drawn card, the two rows of one session and the unique ids.
- `a_recent_session_is_one_line_with_its_details_behind_the_menu` pins the one-line row and the menu details.
- `home_287_row_draws_no_state_word` and `home_287_state_word_appears_on_hover` pin the mark-only state.
- `home_287_clicking_the_mark_opens_the_session` pins the click on the mark.
- `home_status_entry_opens_filtered_history_with_visible_controls` pins a tile and the banner button landing on the filtered list.
- `starting_from_a_project_on_home_opens_that_projects_launch_form` pins the per-project start button.
- `the_attention_count_counts_a_waiting_unread_session_once` pins one count per session.
- `an_unread_session_is_counted_as_needing_a_person` pins unread inside 要対応.

## Decisions inherited
- Change 091: the waiting line goes first and is always drawn; sessions are one line each, the old tall card's details moved behind 「⋯」; a project column offers one start per project.
- Change 287: a home row spells no state word; the mark carries it and the hover names it.
- Change 139: 要対応 is one queue (waiting, blocked, unread) shared by Home, the column and ⌘J; Home's banner opens the list narrowed to it.
- Change 246: the banner uses a neutral card with a warning stroke, not the selection colour, so a warning is not read as a selection.
- Change 252: tile counts follow the same group predicate as the chips; the waiting subset lives in the hover.
- Change 291: cleanly exited or stopped sessions belong to no group, so they never inflate Home's numbers; a failed session counts only while it has a tab or is unread.
- Change 284: the state mark differs in shape per state so a row is readable without colour.
- Change 289: notices float over the Sessions page only; Home keeps top banners.
