# Spec: ターミナルペイン

## Behaviour
1. The centre of the Sessions page is a tab workspace (`ui_tab_workspace`): groups of tabs (session terminals and open documents), each group a pane, laid out as a split tree (`PaneNode`). At most `PANE_MAX` = 4 panes are on screen; each is at least `PANE_MIN_W` × `PANE_MIN_H` = 320 × 200 pt, header included. Dividers are draggable; each project keeps its own tabs and layout across project switches. Pinned by `two_split_sessions_are_both_drawn`, `a_split_past_four_or_too_small_is_refused`.
2. A tab dropped within 30% of a pane's edge splits that pane on that side (left/right side by side, top/bottom stacked, each half); dropped in the middle it swaps or replaces. A session is never on screen twice: dropping a shown one moves it. A drop that would exceed four panes or the minimum size is refused and changes nothing. The drop hint reads 「左に分割」「右に分割」「上に分割」「下に分割」「ここに表示」, or 「これ以上分割できません」 when refused. A session's `⠿` grip in a split pane header, hover 「ドラッグして分割位置を変更」, starts the drag. Pinned by `pane_drop_follows_the_pointer_like_an_editor_group`, `dropping_on_an_edge_splits_the_terminal_on_that_side`, `pane_drop_hint_names_the_zone`.
3. The tab strip's last element is 「＋」 (hover/label 「ターミナルや AI を起動」). It opens a four-row menu (ターミナル, Claude Code, Codex CLI, Antigravity; `WORKSPACE_LAUNCH_CHOICES`); choosing 「ターミナル」 starts a login shell in the pane's folder, choosing an agent starts that CLI with its remembered or pinned launch options in the pane's project and folder. A CLI the machine lacks is disabled with the hover 「未準備（設定で確認）」. Options needing the danger acknowledgement open the launch sheet instead. Escape closes the menu and is consumed before any terminal sees it. ⌘T opens a plain terminal in the focused pane from any page. A pinned session tab draws `ICON_PIN` (`11.5px`, `selected_secondary_ink()` when active else `text_muted`, hover 「ピン留め中」) to the left of `session_badge` and omits `ICON_CLOSE` (change 321). Pinned by `session_273_agent_launch_uses_pane_and_remembered_options`, `session_260_plus_follows_last_tab`, `column_321_tab_pin_sits_left_of_agent_mark_without_close_button`.
4. Tab chords: ⇧⌘] / ⇧⌘[ and ⌃Tab / ⌃⇧Tab move to the next/previous tab within the focused pane; ⌘W closes the shown tab (the agent keeps running); ⇧⌘T reopens the last closed tab, skipping deleted sessions and crossing back to the tab's project. Pinned by `shortcut_272_brackets_and_control_tab_step_through_tabs` and `shortcut_272_reopen_brings_back_the_last_closed_tab`.
5. Clicking a pane focuses it for typing. Printable text, IME preedit and commit, control chords and keys are sent to the session's tmux pane through `queue_terminal_input` (`terminal_input_events`, `terminal_key_binding`). While a conversion is being composed, the preedit draws at the cursor and an Enter that confirms it is not sent. Shift+Enter inserts a newline in the agent's input instead of sending. Wide characters take two cells and the caret never splits one. Pinned by `terminal_input_events_handles_ime_preedit_and_commit_enter_suppression`, `shift_enter_puts_a_newline_in_the_agents_input_instead_of_sending`, `a_full_terminal_pane_still_answers_a_click`.
6. The pane keeps at most `TERMINAL_SCROLLBACK_LINES` = 5 000 lines for the app view (tmux keeps `history-limit` 50 000). Only the rows in view are laid out. A 「最新へ」 button (hover 「ターミナルの最新行へ移動」) at the pane's lower right scrolls to the last row.
7. ⌘F (`terminal.find`) opens the find bar with the hint 「スクロールバックを検索」, showing 「{current} / {total}」 or 「一致しません」, with previous (「前の一致」) and next buttons. Enter steps forward, ⇧Enter backward, Esc closes (but not during an IME conversion). Typing in the bar never reaches the agent. At most `TERMINAL_SEARCH_MAX_MATCHES` = 2 000 matches are collected. Pinned by `the_find_bar_steps_on_enter_but_not_on_a_conversions_enter`, `esc_closes_the_find_bar_into_the_pane_but_not_mid_conversion`.
8. Paths and URLs in the output are links. ⌘-click or ⌥-click opens a path in the built-in editor at its line (`file:line` and stack-trace forms), a URL in the browser; a plain click only focuses the pane. A right-click on a path opens a menu of exactly 「Operon で開く」「既定のアプリで開く」「Finder で表示」「パスをコピー」 (`TERMINAL_PATH_MENU`); it closes on a choice, Escape, or a press outside. Pinned by `terminal_293_cmd_and_option_click_open_a_path_inside_operon`, `terminal_293_path_menu_lists_four_choices_in_order`, `finds_a_path_and_its_line_in_a_stack_trace`.
9. A path opens only when it resolves to an existing file inside the session's own folders (worktree, then project root); `~` paths and CLI tool-call forms are understood. A bare file name resolves only when the scan of the session folder holds exactly one file of that name; several matches stay unresolved, a name symlinked outside the root stays unresolved, and a truncated scan does not prove uniqueness. Paths under macOS-protected home folders are never probed unless inside the session's own folders. At most `TERMINAL_PATH_PROBE_LIMIT` = 64 unresolved paths are probed per frame. Pinned by `refuses_a_path_that_escapes_the_session_folder`, `terminal_293_bare_name_resolves_to_the_only_file_with_that_name`, `terminal_293_bare_name_with_two_matches_stays_unresolved`.
10. The pane header shows 「{project} / {title}」, the state (with working time after one minute), the session verb button (stop / start / resume), 「画面から外す」 on split panes, 「サイドパネルを表示」 when the panel is hidden, and the 「⋯」 menu (「今すぐ同期」, 「表示中のログをすべてコピー」, 「Terminal.app で開く（マウス操作・tmux 操作用）」 and the session's other actions).
11. The header's port badge reads `:5173 vite` for one own port, 「{n} ポート稼働中」 for several, 「プロジェクトで {n} ポート稼働中」 (muted) when only other folders of the project listen. Its menu splits 「この作業フォルダ」 and 「プロジェクト内のほかの場所」, each row offering open, copy, and stop (stop confirms and checks the same process still holds the port before SIGTERM). Ports are read with `lsof` every `PORT_SCAN_INTERVAL_SECONDS` = 30. Pinned by `ports_262_session_splits_own_from_elsewhere`, `ports_262_badge_counts_own_folder_or_names_the_project`, `ports_262_menu_sections_name_folders_and_offer_stop`, `ports_262_stop_refuses_operon_and_init`.
12. The side panels (ファイル, 会話, 変更) are dockable on the left, right, top or bottom edge, one by one (grip `⠿` drag, right-click on the grip, or 「パネルの配置を変更」). A release within 30% of an edge's span docks there and shows 「{panel}を{edge}に移動しました」; a release in the middle or on its own edge, or Escape, cancels. An edge the window cannot hold shows a warning frame and 「ウィンドウが小さいため{edge}には置けません」. The terminal keeps at least `DOCK_CENTER_MIN_W` × `DOCK_CENTER_MIN_H` = 300 × 360. The session list panel is forced to the left edge each frame. Pinned by `test_dock_layout_moves_one_panel_and_leaves_the_rest`, `test_dock_drop_edge_partitions_the_workspace`, `test_dock_rects_keep_the_terminal_its_minimum`, `test_dock_refused_move_says_why`.
13. The workspace (project shown, every project's groups and tabs in order, front tabs, split layout and shares, focus, selected session, already-placed sessions) is written to `workspace-layout.json` within one two-second tick of a change and at quit, and read back at start. A session tab the person closed is not reopened by the next start. Pinned by `the_workspace_comes_back_after_a_relaunch`.
14. At start, a session that was running at the last quit whose terminal is gone is started again in place; a clean exit, or a stale Exited tab with a dead pane, becomes a plain login shell in the same tab (scrollback kept); a forced or abnormal end during a live run is restored in place with a toast 「強制終了されたセッションを自動で復元しました: {name}」 (or 「…を {count} 件自動で復元しました。」). Consecutive crashes within 15 seconds stop auto-restore and leave the session in 要対応. Pinned by `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`, `status_291_midrun_crash_auto_restores_in_same_tab_with_toast` and `status_291_auto_restore_cooldown_stops_crash_loop_until_manual_retry`.
15. Output reaches the pane from an event-driven tmux watcher for each visible Active session (at most 4); a snapshot identical in theme, text and cursor wakes nothing. Text is laid out where it is captured, not on the frame, and cached per pane and row. Pinned by `every_terminal_on_screen_is_polled_and_read`, `terminal_276_unchanged_text_is_not_laid_out_again`.

## States
- Queued session: 「開始待機中のため、まだターミナル出力はありません。」
- Lost, Failed, Exited or Cancelled session whose pane is gone: 「このセッションのターミナルは終了しているため、表示できる出力はありません。」 and the capture is not retried every frame.
- Otherwise with no output yet: 「ターミナル出力を待っています…」.
- Empty workspace (no tab): a heading 「セッションを選択」, 「一覧からセッションを選んでください。」 and the 「＋」 menu, disabled without a project.
- A saved layout that is missing, unreadable, over `WORKSPACE_LAYOUT_FILE_MAX_BYTES` (1 MiB) or damaged restores only what is whole and never makes a pane without a group, a session in two groups, more than 4 panes, a share outside 0..1 or a front tab outside its group.
- A path outside the session folders, or one that does not exist, opens nothing and sets a notice.
- A pane too narrow for a find bar or a header keeps its controls inside the pane (long metadata is cut short).
- tmux missing: choosing 「ターミナル」 reports it instead of starting.

## Invariants
- A session is on screen at most once and every pane on screen is polled and counts as read.
- Never more than 4 panes; never one under 320 × 200 pt; the dock never takes the terminal below its centre minimum.
- App scrollback ≤ 5 000 lines; the search holds ≤ 2 000 matches; neither limit is raised without keeping the viewport virtualised.
- ⌘/⌥-click is the only gesture that opens a path in Operon; a plain click never steals the agent's input.
- A resize is sent to tmux only once the size has settled (the first one at once), not on every frame of a drag.
- Terminal layout work is never done on the draw path for text already laid out off-frame; unchanged captures cause no repaint.
- Every tmux session sets `remain-on-exit-format` to empty so no "Pane is dead" footer is written into a managed pane.
- A stop of a port holder is refused for PID 0, PID 1, Operon's own PID, and when the holder changed.

## Seen in the running app
`python3 scripts/verify-app.py run terminal-pane` starts the built application apart from the person's data, follows these steps, and keeps a picture and the window's tree for each (change 314). One `verb words` per line; the words are what the screen shows.

    start project
    click プロジェクト
    click セッション
    see 一覧からセッションを選んでください。
    click 検索・操作
    click 空のターミナルを開く
    see 空のターミナルセッションを tmux で起動しました
    see 停止
    shot terminal-running

## Code and tests
- Symbols: `src/app/screens.rs::ui_tab_workspace` lays out the docks, tab groups, dividers, empty workspace and drop zones.
- `src/app/screens.rs::ui_terminal_pane` draws one pane: header, output rows, caret, IME, 「最新へ」, links, find bar.
- `src/app/screens.rs::ui_terminal_search` and `handle_terminal_search_keys` draw and drive the find bar; `ui_session_ports` draws the port badge and menu.
- `src/app/launch_menu.rs::ui_workspace_launch_menu`, `launch_workspace_agent`, `workspace_agent_launch` and `WORKSPACE_LAUNCH_CHOICES` are the + menu and its checked launches.
- `src/app/terminal_paths.rs::TERMINAL_PATH_MENU`, `resolve_terminal_path_with_scan`, `open_terminal_path_menu`, `apply_terminal_path_menu_choice` are the right-click menu and bare-name resolution.
- `src/app/terminal_output.rs::TerminalStreamManager`, `lay_out_capture`, `terminal_empty_placeholder`, `flush_terminal_input`, `request_terminal_resize` feed and size the pane.
- `src/app/relaunch.rs::restore_workspace_layout`, `workspace_layout_snapshot`, `save_workspace_layout_if_changed` keep the workspace across relaunches; `relaunch_remembered_sessions` and `reopen_exited_session_as_shell` bring terminals back.
- `src/ui/terminal.rs::terminal_input_events`, `terminal_key_binding`, `terminal_targets`, `resolve_terminal_session_path`, `terminal_search_matches`, `TerminalPaneRenderCache`, `IncrementalTerminalLayout` parse, link, search and cache output.
- `src/ui/session_tree.rs::PaneNode`, `DockLayout`, `dock_drop_hint`, `dock_no_room` hold the split tree and the panel docking.
- `src/config.rs::PANE_MAX`, `PANE_MIN_W`, `PANE_MIN_H`, `TERMINAL_SCROLLBACK_LINES`, `TERMINAL_SEARCH_MAX_MATCHES`, `TERMINAL_PATH_PROBE_LIMIT`, `PORT_SCAN_INTERVAL_SECONDS`, `WORKSPACE_LAYOUT_FILE_MAX_BYTES` are the limits.
- Tests: `terminal_input_events_handles_ime_preedit_and_commit_enter_suppression` and `shift_enter_puts_a_newline_in_the_agents_input_instead_of_sending` pin keyboard input.
- `a_focused_terminal_pane_claims_the_ime_without_redrawing_every_frame` and `a_full_terminal_pane_still_answers_a_click` pin IME claim and click focus.
- `the_find_bar_steps_on_enter_but_not_on_a_conversions_enter` and `esc_closes_the_find_bar_into_the_pane_but_not_mid_conversion` pin the find bar.
- `pane_drop_follows_the_pointer_like_an_editor_group`, `dropping_on_an_edge_splits_the_terminal_on_that_side`, `a_split_past_four_or_too_small_is_refused`, `closing_a_terminal_gives_its_room_to_the_neighbour`, `two_split_sessions_are_both_drawn`, `every_terminal_on_screen_is_polled_and_read`, `pane_drop_hint_names_the_zone` pin splitting.
- `session_273_agent_launch_uses_pane_and_remembered_options` pins the + launch; `session_260_plus_follows_last_tab` pins its place.
- `terminal_293_cmd_and_option_click_open_a_path_inside_operon`, `terminal_293_path_menu_lists_four_choices_in_order`, `terminal_293_bare_name_resolves_to_the_only_file_with_that_name`, `terminal_293_bare_name_with_two_matches_stays_unresolved`, `terminal_293_menu_closes_on_escape_or_outside_click` pin path opening.
- `protected_folders_265_terminal_link_probe_skips_protected_folders` and `refuses_a_path_that_escapes_the_session_folder` pin where a path may resolve.
- `finds_a_path_and_its_line_in_a_stack_trace` pins link detection.
- `ports_262_session_splits_own_from_elsewhere`, `ports_262_badge_counts_own_folder_or_names_the_project`, `ports_262_menu_sections_name_folders_and_offer_stop`, `ports_262_stop_refuses_operon_and_init` pin ports.
- `test_dock_layout_moves_one_panel_and_leaves_the_rest`, `test_dock_drop_edge_partitions_the_workspace`, `test_dock_rects_keep_the_terminal_its_minimum`, `test_dock_refused_move_says_why` pin panel docking.
- `the_workspace_comes_back_after_a_relaunch`, `sessions_running_at_the_quit_come_back_when_their_terminal_is_gone`, `shortcut_272_reopen_brings_back_the_last_closed_tab` pin relaunch and reopen.
- `session_255_terminal_resize_waits_until_the_size_settles` pins resize; `terminal_276_unchanged_text_is_not_laid_out_again` pins off-frame layout.
- `pane_288_configure_tmux_suppresses_dead_pane_banner_and_keeps_scrollback` and `pane_303_missing_pane_does_not_loop_output_requests_or_spam_error_banner` pin dead and missing panes.
- `counts_a_cjk_character_as_two_terminal_columns` pins wide-character cells.

## Decisions inherited
- Change 014: IME text is entered at the terminal cursor, and the Enter that confirms a conversion is not sent.
- Change 226: up to four terminals side by side by dropping on an edge; one session on screen once; every visible pane is polled.
- Change 145: each side panel docks on any of four edges; drop within 30% of an edge.
- Change 146: a panel can never be docked where it is not drawn; a refused move says why.
- Change 255: resizing is debounced so a window drag does not redraw every terminal repeatedly.
- Change 264: Shift+Enter is a newline, not a send.
- Change 265: protected home folders are not probed for links, to avoid permission prompts naming Operon.
- Change 270: the workspace, splits and running sessions come back after a relaunch; a closed tab stays closed.
- Change 273: the + chooses what to start; options needing acknowledgement open the sheet instead of starting.
- Change 276 and 292: output is laid out off the frame and streamed by a watcher; unchanged snapshots wake nothing.
- Change 288 and 303: a dead pane becomes a shell or is restored; a missing pane is not polled in a loop.
- Change 293: ⌘/⌥-click opens in Operon, other destinations live in a four-row menu, and a bare name opens only when it is unique.
