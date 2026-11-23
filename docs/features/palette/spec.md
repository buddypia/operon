# Spec: 検索・操作 (⌘K)

## Behaviour
1. The palette opens from the keymap action `palette.open` (default ⌘K) or the toolbar button 「検索・操作」 (with the current chord appended). Opening sets the focus to the input and keeps nothing of an earlier query.
2. It is an `egui::Window` titled 「検索・操作」, non-collapsible and non-resizable, anchored at the top centre with a 68 px offset and a default width of 520. The input's hint is 「操作・セッション・プロジェクトを検索」.
3. With an empty (or whitespace) query the list holds only the fixed actions, in this order: 「新しいセッション」, 「履歴を開く」, 「プロジェクトを開く」, 「セッションを開く」, 「設定を開く」, 「プロジェクトフォルダを選択…」, 「ワークスペースをスキャン…」, 「キー割り当てファイルを開く」, 「空のターミナルを開く」, 「閉じたタブを開き直す」, 「アイドル中のエージェントへ送信…」.
4. With a query, matching is case-insensitive substring matching over: an action's label; a session's title, request, agent short label, branch, project name and the agent's last message (only while its hook status is fresh); a project's name and path; a file's name and relative path; a symbol's name and its file's path.
5. Each match has a rank: `Name` (the row's own name starts with the query), `Word` (a word of its searchable text starts with it, words split at whitespace and `/ - _ . : ·`), or `Anywhere`. Rows sort by rank, then kind (action, session, project, file), then waiting sessions before others, then newer first (sessions by launch time, projects by added time).
6. Files come from the file cache of the selected project (or the first project when none is selected), capped at `MAX_PALETTE_FILE_CANDIDATES` (16), sorted by rank then shorter path. Symbols come from the buffers of the open documents (viewer documents use their disk text), kinds `fn struct enum trait interface class type`, capped at `MAX_PALETTE_SYMBOL_CANDIDATES` (8), and carry the line.
7. The list shows at most `PALETTE_RESULT_LIMIT` (12) rows. When more match, a small note reads 「ほかに {n} 件あります。検索語を足してください。」.
8. A row shows its label; actions show the chord read from the active keymap at the right; sessions show 「{state} · {agent} · {project}」, projects the friendly path, files 「{project} · {path}」, symbols 「{path}:{line}」 with the label 「{kind} {name}」.
9. ↑ and ↓ move the selection (clamped to the visible rows), Enter opens the selected row; a click opens a row. Both go through `activate_palette_entry`. The selection is clamped every frame as results shrink. Keys are consumed before they reach the input.
10. Opening a row: an action runs (new session sheet, new terminal tab in the focused pane, reopen the last closed tab, open the session history, go to Projects / Sessions / Settings, choose a project folder, scan a workspace, open the keymap file, open the broadcast sheet); a session opens in the terminal workspace; a project is selected and the Projects page shown; a file or symbol selects its project, shows the Projects page, and opens the file (at the symbol's line), using the root of an already-open document with that path else the project's.
11. Activating a row, pressing Esc, or closing the window closes the palette and clears the query and selection.

## States
- No match: 「一致する項目がありません」.
- Empty query: no sessions, projects, files or symbols are listed.
- File candidates are absent while the project's file scan has not loaded; symbol candidates are absent when no document is open.
- A project that no longer exists when a file row is activated does nothing beyond closing the palette.
- While the palette is open the launch sheet does not take Esc or ⌘↩.

## Invariants
- The palette lists no row for closing or stepping through tabs: those act on the tab behind it.
- The chord drawn beside an action is read from the live keymap, never written into the label.
- Results are computed from in-memory state only; no filesystem or process call happens while drawing, and a status view is computed only for rows that are drawn.
- Ranking over a full store stays within one frame.
- The click and Enter paths share one activation function, so a row behaves identically either way.
- Action keys stay the English strings the dispatch matches; labels are Japanese and come from the keymap registry where a chord exists.

## Seen in the running app
`python3 scripts/verify-app.py run palette` starts the built application apart from the person's data, follows these steps, and keeps a picture and the window's tree for each (change 314). One `verb words` per line; the words are what the screen shows.

    start empty
    click 検索・操作
    see 操作・セッション・プロジェクトを検索
    see 空のターミナルを開く
    shot palette-open
    click 設定を開く
    gone 操作・セッション・プロジェクトを検索
    shot after-choice

## Code and tests
- Symbols:
  - `src/app/screens.rs::palette_actions`: the fixed actions with label and keymap id.
  - `src/app/screens.rs::palette_entries`: builds and sorts every row for the query.
  - `src/app/screens.rs::palette_visible_entries`: truncates to the limit and counts the overflow.
  - `src/app/screens.rs::activate_palette_entry`: opens a row.
  - `src/app/screens.rs::handle_palette_keys`: arrows and Enter.
  - `src/app/screens.rs::ui_command_palette`: the window.
  - `src/app.rs::PaletteKind`, `PaletteScore`, `PaletteEntry`, `palette_rank`: row types and ranking.
  - `src/app.rs::open_command_palette`, `close_command_palette`: open and close state.
  - `src/files.rs::extract_file_symbols`: symbols; the caps live as `MAX_PALETTE_FILE_CANDIDATES` and `MAX_PALETTE_SYMBOL_CANDIDATES` there.
  - `src/config.rs::PALETTE_RESULT_LIMIT`: the 12-row limit.
- Tests:
  - `an_empty_palette_query_lists_only_the_actions`: empty query.
  - `the_palette_finds_a_session_by_its_request_and_its_project`: session text.
  - `the_palette_finds_a_session_by_what_its_agent_last_said`: last message.
  - `the_palette_ranks_a_name_match_above_a_word_match_above_anywhere`: ranking.
  - `the_palette_ranks_a_waiting_session_above_an_equal_match`: waiting first.
  - `the_palette_selection_is_clamped_when_the_results_shrink`: selection clamp.
  - `ranking_the_palette_over_a_full_store_stays_inside_a_frame`: frame budget.
  - `the_palette_draws_the_chord_the_handler_listens_for`: chord from the keymap.
  - `the_new_session_chord_is_bound_and_listed_first_in_the_palette`: first action.
  - `the_command_palette_sits_centered_near_the_top_of_the_window`: placement.
  - `ide_261_palette_ranks_project_files_and_document_symbols`: files and symbols.
  - `shortcut_272_palette_rows_run_the_tab_actions`: tab actions.
  - `broadcast_282_palette_opens_the_sheet`: broadcast action.

## Decisions inherited
- Change 017: the palette lists only actions when empty ("a palette that opens onto every session is a list"), and ranks by name start, then word start, then anywhere, as three tiers a person can tell apart rather than a score.
- Change 223: the window is anchored at the top centre, clear of the navigation rail, so it opens where the eye goes.
- Change 261: files of the selected project and symbols of open documents are searchable, with fixed caps (16 and 8) so a query stays fast and bounded.
- Change 272: only the tab actions that start something (new terminal, reopen) are rows; closing and stepping act on the tab behind the palette.
- Change 282: broadcasting to idle agents is an action row without a chord.
