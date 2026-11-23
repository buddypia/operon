# Spec: セッション一覧

## Behaviour
1. The Sessions page's left column is `ui_workspace_projects`. It is always docked left (`dock.move_panel(DockPanel::Sessions, DockEdge::Left)`). Its heading is 「プロジェクト」 with a 「履歴」 button that toggles `session_library_open`.
2. Inside one vertical scroll area it draws, from the top: three status tiles (要対応, 実行中, アイドル, with counts), the CLI segment, and, while a filter is on, 「クリア」 and 「{count} 件」; then one block per project. Pinned by `sidebar_251_filters_drive_real_rows`, `sidebar_252_status_tiles_count_and_toggle`.
3. Tile counts come from `status_group_counts_for_chosen_agents`: sessions that pass the CLI choice and have a tab in some workspace. The 要対応 tile's hover also shows 「承認・回答待ち {count} 件」. Tiles are laid out 3, 2 or 1 per row by the available width (`STATUS_TILE_MIN_W`). Tiles are independent toggles; none selected means no state filter. Pinned by `sidebar_252_counts_follow_cli_and_clear_only_while_filtering`, `sidebar_252_tiles_fit_narrow_column`.
4. The CLI segment offers Codex, Claude Code, Antigravity and 「その他」 (indices 0..3 of `agent_filter`); unchosen marks fade while any is chosen. Filters are not persisted. Pinned by `sidebar_251_filter_combinations`.
5. Projects are ordered by lower-cased name, then as arranged (`sort_column_projects`). Each block has a heading, not a card: a disclosure (「展開する」 / 「折りたたむ」; while filtering 「フィルタ中はすべて表示されます」), the project name, per-group count chips (要対応 leftmost, then 実行中, アイドル, each hover 「{n} 件」), and 「セッションを開始」 (`open_project_session_setup`). Pinned by `test_change_181_column_keeps_todays_order_until_arranged`, `workspace_project_header_counts_sessions_by_status_group`.
6. A project is folded unless its id is in `SessionArrangement::unfolded`. A filter lists the matching rows of a folded project without changing or saving its fold. A project added later starts folded. Pinned by `column_311_projects_start_folded_and_unfold_on_click`, `column_311_an_older_order_file_keeps_its_order_and_folds_every_project`, `column_311_a_filter_lists_matches_of_a_folded_project`.
7. A project's rows are exactly its session tabs, from every tab group of its workspace (shown or not), whatever their state (`project_tab_sessions`); only the column's filters hide a row. A session without a tab has no row. Pinned by `session_285_column_lists_exactly_the_tabs`.
8. Rows use one order (`workspace_session_rank`): pinned first; then listed in `SessionArrangement::sessions` order; sessions the file does not list come first among the unpinned, newest `created_at` first. The tab strip uses the same key, so rows and tabs always agree. The column no longer sorts by state. Pinned by `session_285_pin_from_row_or_tab_moves_it_first_in_both`, `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`.
9. A row shows the agent mark with the state dot (`session_badge`; a legacy kind with no mark shows the state icon and a short label), a pin icon (hover 「ピン留め中」) when pinned, and the title (`session_list_title`). The selected row has the selection fill and a 2 px accent bar. Hover shows the title, the state label, the hint with working time, and 「ドラッグで並べ替え」.
10. Clicking a row (or a count chip, which opens the first session of that group) calls `open_waiting_session`: the session opens and, if `Active`, the terminal takes the keyboard. Double-click, or 「名前を変更」, starts an inline rename (`begin_session_rename`). Pinned by `session_253_rename_from_sidebar_and_tab`.
11. The row's context menu offers 「名前を変更」, 「ピン留め」 / 「ピン留めを外す」, and 「閉じる」 (absent on a pinned session). 「閉じる」 closes the session's tab wherever it is (`close_session_tab`); the session keeps running. Pinned by `session_286_a_pinned_session_offers_no_close`.
12. Pinning (`toggle_session_pin`) writes `pinned` to `session-order.json`. A pinned tab has no close button; ⌘W, 「画面から外す」 and the other closes leave it open with 「ピン留め中のセッションは閉じられません。ピン留めを外してから閉じてください。」. Deleting the session removes its tab and pin. Pinned by `session_285_pins_survive_a_relaunch`, `session_285_a_pinned_tab_does_not_close`.
13. Dragging a session row, and dropping it among its own project's rows of the same pin state, rewrites `sessions` in the order file and the project's tabs follow (`place`, `follow`). Dropping a tab among its group's tabs does the reverse (`follow_tab_drop`). Dragging a project name reorders `projects`. Holding a drag within `COLUMN_DRAG_EDGE` of the top or bottom scrolls the column; Escape or a release the window never saw ends the drag with no move. Pinned by `test_change_181_session_row_drag_reorders_within_its_project`, `test_change_181_escape_or_a_release_in_place_moves_nothing`, `session_285_arrange_keeps_documents_and_the_front_tab`.
14. Clicking a project name selects that project as the workspace's project (`select_workspace_project`).
15. A session that finishes cleanly or is stopped, and is not pinned, has its tab closed automatically and moves to 履歴; a crash is restored in place into the same tab (`close_finished_session_tab`). Pinned by `status_291_clean_exit_and_stop_close_unpinned_tab_but_keep_pinned_and_history_open`.

## States
- No projects: 「プロジェクトがありません」.
- Filter on and nothing matches: 「一致するセッションはありません。」; projects with no match are omitted.
- Unfolded project with no tab: 「開いているセッションはありません」.
- Order file missing, unreadable, unparsable or over `SESSION_ARRANGEMENT_FILE_MAX_BYTES` (1 MiB, counted on the open handle): the default order applies and the next move rewrites it. An older file with `folded` and no `unfolded` keeps its order and pins and folds every project.
- Order file write fails: the arrangement stays in effect for the run and the notice 「並び順を保存できませんでした: {error}」 is shown.
- A pinned session's 「閉じる」 is not offered; a close attempt gives the notice above.

## Invariants
- Rows and tab groups follow one key, `workspace_session_rank`; a group's session tabs take the slots session tabs already hold so document tabs keep their place.
- `commit_session_arrangement` keeps only ids the store still holds (projects, sessions, unfolded, pinned), so the file is bounded by the store; it is written compactly with an atomic write, and unlisted items are never lost at the bottom.
- The file holds only ids, never titles; it sits beside the store, not in it.
- Filters, fold-while-filtering and drag state are never saved.
- A session is never shown as a row twice and never without a tab.
- Pinned sessions are never closed by any path other than unpin or delete.
- The order and pin file name lives once, in `SESSION_ARRANGEMENT_FILE_NAME`.
- Row and heading sizes come from the compact type scale (`COLUMN_SESSION_MARK_SIZE`, `SIDEBAR_ROW_GAP`); folded headings sit within 28 px top to top.

## Code and tests
- Symbols: `src/app/screens.rs::ui_workspace_projects` draws the heading, filters, project blocks, rows, drag and menus.
- `src/app/screens.rs::ui_sidebar_session_filters`, `ui_sidebar_agent_segment` and `ui_sidebar_filter_summary` draw the tiles, CLI segment, 「クリア」 and the count.
- `src/app/screens.rs::toggle_project_fold` saves the fold.
- `src/app/session_order.rs::sync_workspace_live_sessions` gives live sessions tabs and puts every workspace in the column's order.
- `src/app/session_order.rs::project_tab_sessions` yields a project's rows; `toggle_session_pin`, `close_session_tab`, `close_finished_session_tab`, `follow_tab_drop`, `commit_session_arrangement` do the edits.
- `src/ui/session_tree.rs::SessionArrangement` holds `projects`, `sessions`, `unfolded`, `pinned`; `load_session_arrangement` and `save_session_arrangement` read and write `session-order.json`.
- `src/ui/session_tree.rs::workspace_session_rank` and `sort_workspace_sessions` are the one order; `sort_column_projects`, `place`, `follow`, `drop_slot` support projects and drags.
- `src/app.rs::session_matches_filters`, `toggle_status_filter`, `toggle_agent_filter`, `clear_session_filters`, `status_group_counts_for_chosen_agents` hold the filter state and counts.
- `src/config.rs::SESSION_ARRANGEMENT_FILE_MAX_BYTES` and `SESSION_ARRANGEMENT_FILE_NAME` are the file's ceiling and name.
- Tests: `session_285_column_lists_exactly_the_tabs` pins rows equal tabs.
- `session_285_pin_from_row_or_tab_moves_it_first_in_both` and `session_285_pins_survive_a_relaunch` pin pinning.
- `session_285_a_pinned_tab_does_not_close` and `session_286_a_pinned_session_offers_no_close` pin the close refusal.
- `session_285_arrange_keeps_documents_and_the_front_tab` pins document tabs keeping their place.
- `column_311_projects_start_folded_and_unfold_on_click`, `column_311_an_older_order_file_keeps_its_order_and_folds_every_project`, `column_311_folded_projects_sit_closer_together` and `column_311_a_filter_lists_matches_of_a_folded_project` pin folding.
- `test_change_181_column_keeps_todays_order_until_arranged`, `test_change_181_session_row_drag_reorders_within_its_project`, `test_change_181_project_name_drag_reorders_the_column` and `test_change_181_escape_or_a_release_in_place_moves_nothing` pin drag ordering.
- `test_change_181_arrangement_file_is_bounded_and_fails_towards_default` and `test_change_181_save_failure_is_said` pin the file's limits.
- `sidebar_251_filters_drive_real_rows` and `sidebar_251_filter_combinations` pin filtering.
- `sidebar_252_status_tiles_count_and_toggle`, `sidebar_252_counts_follow_cli_and_clear_only_while_filtering` and `sidebar_252_tiles_fit_narrow_column` pin the tiles.
- `workspace_project_header_counts_sessions_by_status_group` pins heading counts.
- `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs` pins the compact rows.
- `session_253_rename_from_sidebar_and_tab` pins rename.

## Decisions inherited
- Change 181: the column's order is the person's; unlisted items come first so nothing is lost; the file is ids only, bounded, and fails towards the default.
- Change 285: the column lists exactly the open tabs in one shared order, and pins exist; state no longer sorts the column.
- Change 286: 「閉じる」 on a row and on a tab menu closes the same tab; a pinned session offers none.
- Change 311: projects start folded so the column is a list of projects; a filter shows matches without touching the saved fold.
- Change 251 and 252: state and CLI are independent toggles; the tile count follows the CLI choice and what the column will actually draw.
- Change 237: each project heading carries its per-state counts, which open the first session of that state.
- Change 245: a project is a heading, not a card, because it is a destination only on the Projects page.
- Change 291: clean exits and stops close their unpinned tab into 履歴; pinned tabs stay.
- Change 302: compact rows and headings to cut vertical scrolling.
- Change 310: session search must not say "nothing matches" for words the row shows.
