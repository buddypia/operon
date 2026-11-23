# Spec: キー割り当て

## Behaviour
1. `KEYMAP_ACTIONS` is the registry of remappable actions, in palette order, with id, message-id label and default chord: `palette.open` 「検索・操作を開く」 Mod+K; `session.new` 「新しいセッション」 Mod+N; `attention.next` 「次の要対応セッションへ」 Mod+J; `page.home` 「ホームを開く」 Mod+1; `page.projects` 「プロジェクトを開く」 Mod+2; `page.sessions` 「セッションを開く」 Mod+3; `page.settings` 「設定を開く」 Mod+,; `project.add` 「プロジェクトフォルダを選択…」 Mod+O; `terminal.find` 「ターミナル内を検索」 Mod+F; `panel.toggle` 「サイドパネルを開閉」 Mod+Alt+B; `tab.new` 「空のターミナルを開く」 Mod+T; `tab.close` 「タブを閉じる」 Mod+W; `tab.reopen` 「閉じたタブを開き直す」 Mod+Shift+T; `tab.next` 「次のタブへ」 Mod+Shift+]; `tab.previous` 「前のタブへ」 Mod+Shift+[. Pinned by `every_default_chord_survives_being_written_and_read_back`.
2. The keymap file is `keybindings.json` in the app data directory. It is read once when the app state is built, not watched; a change takes effect on the next launch.
3. File shape: `{"version": 1, "keybindings": {"<id>": "<chord>" | null}}`. An absent `keybindings` section means defaults. Ids not mentioned keep their default; `null` unbinds the action (it is never "pressed"). Pinned by `a_keymap_rebinds_an_action_by_id`.
4. A chord is modifiers then exactly one key joined by `+`, spaces around parts ignored. Modifier names are `Mod` (⌘), `Shift`, `Alt`, `Ctrl`. Keys: A–Z, 0–9, `, . / ; [ ] \ - = ' \``, `Space`, `Tab`, `Enter`. Anything else (unknown modifier, trailing `+`, modifier-only) is unparsable. Pinned by `every_default_chord_survives_being_written_and_read_back`, `a_keymap_that_cannot_be_read_keeps_the_defaults_and_says_why`.
5. Chords are drawn in macOS spelling, glyphs adjacent in menu-bar order ⌃⌥⇧⌘ then the key (for example ⇧⌘P). The palette row chord and the Settings 「キー割り当て」 list both come from the live keymap; no message may contain a remappable chord. Pinned by `every_chord_modifier_glyph_is_drawn_by_the_ui_font`, `the_palette_draws_the_chord_the_handler_listens_for`, `no_message_spells_a_remappable_chord`.
6. Settings → 「キー割り当て」 draws one row per action with its chord in monospace, or 「なし」 when unbound, then the button 「キー割り当てファイルを開く」. Pinned by `the_keys_section_lists_every_action`.
7. That button writes the defaults as a pretty-printed file when none exists and reveals the data folder in Finder, notice 「キー割り当てファイルを作成しました。」; when the file exists it only reveals the folder, notice 「キー割り当てファイルの場所を開きました。」; a write failure gives 「キー割り当てファイルを作成できませんでした: {error}」. It never overwrites. Pinned by `writing_the_default_keymap_never_overwrites_one_that_is_there`.
8. Dispatch: once per frame `handle_app_shortcuts` asks the keymap which action each pressed key event triggers. `palette.open` and `project.add` act unless the launch sheet or the broadcast sheet is open. `session.new`, `attention.next`, `panel.toggle`, `tab.new`, `tab.reopen` and the four `page.*` chords act only while neither the command palette nor those sheets are open. Pinned by `broadcast_282_f1_page_shortcuts_do_not_reach_past_the_sheet`, `the_next_waiting_chord_with_an_empty_queue_changes_nothing`, `the_panel_chord_folds_and_reopens_the_side_panel`, `shortcut_272_command_t_opens_a_terminal_like_the_plus`.
9. `tab.close`, `tab.previous`, `tab.next` and the fixed ⌃Tab / ⌃⇧Tab act only on the Sessions page with the session library closed. `tab.new`, `tab.close`, `tab.reopen`, `tab.next`, `tab.previous` and ⌃Tab are consumed from the input events before any pane draws, so a terminal never receives them. Pinned by `shortcut_272_brackets_and_control_tab_step_through_tabs`, `shortcut_272_close_is_a_keymap_action`, `shortcut_272_reopen_brings_back_the_last_closed_tab`.
10. ⌘2 (`page.projects`) also clears the selected project before switching page.
11. Matching: a press goes to the binding whose Shift and Alt match exactly; only when none matches exactly does it fall back to the first binding that matches leniently (egui ignores an extra Shift or Alt). So ⇧⌘T is `tab.reopen`, not `tab.new`, while ⇧⌘K still opens the palette. With Shift held, `{` and `}` count as `[` and `]`. Pinned by `shortcut_272_the_most_specific_chord_wins`.
12. ⌃Tab steps forward and ⌃⇧Tab steps back, fixed; a press that the keymap has given to some action is left to that action. Pinned by `shortcut_272_a_control_tab_binding_is_the_keymaps`.
13. `chord_pressed` reads without consuming; `chord_consumed` removes the matching events.
14. Reasons shown for a refused file (prefix 「キー割り当てファイルを読み込めませんでした: …既定のキーで起動しました。」): 「JSON として読めません。」, 「いちばん外側が JSON のオブジェクトではありません。」, 「keybindings が JSON のオブジェクトではありません。」, 「「{id}」という操作はありません。」, 「「{chord}」はキーの書き方として読めません。」, 「「{id}」の値は文字列か null にしてください。」, 「{chord} が「{first}」と「{second}」で重複しています。」. Pinned by `a_keymap_that_cannot_be_read_keeps_the_defaults_and_says_why`, `a_chord_claimed_twice_refuses_the_whole_file`.

## States
- No file: defaults, no notice.
- File larger than `KEYMAP_FILE_MAX_BYTES`: defaults and a notice quoting the byte limit.
- File not readable as text: defaults and 「…テキストとして読めません。…」.
- Any parse, unknown-id, bad-value or duplicate-chord error: the whole file is refused, defaults are used, one notice states the reason. Nothing is applied halfway.
- A duplicate is detected on the finished map, so a binding colliding with a default it did not mention is also refused.
- An action with no chord is never triggered and shows 「なし」.

## Invariants
- Action ids are the contract with files people wrote; they are never renamed without keeping the old spelling readable.
- Every default chord parses and survives write-and-read-back (⇧⌘] returns as `Mod+Shift+]`).
- Terminal keys, palette navigation keys (arrows, Enter, Esc), ⌘↩, Esc in the launch sheet, ⌘S in the editor, ⌃Tab/⌃⇧Tab and clicks are not remappable.
- One chord per action; no sequences.
- `resolve_keymap` does no I/O so the whole decision is testable without a window; the file ceiling is 64 KiB.
- The default file is written through `write_file_atomically` and only when no file exists.
- The pane drops every ⌘ chord rather than sending it to the agent, which is what lets app chords work while a terminal has focus.
- Every manual (`docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`) lists every default chord as the app draws it.
- Manual note: `docs/MANUAL.ja.md` lists ten action ids; the code has fifteen (the five `tab.*` ids are also remappable).

## Code and tests
- `src/app/keymap.rs::KEYMAP_ACTIONS`, `KeymapAction`: the registry.
- `src/app/keymap.rs::Keymap` (`defaults`, `chord_for`, `label_for`, `action_for`): the live map and lookup with the most-specific-wins rule.
- `src/app/keymap.rs::resolve_keymap`, `keymap_or_defaults`, `read_keymap`, `keymap_path`: parse, refuse-whole, and read from disk.
- `src/app/keymap.rs::parse_chord`, `chord_label`, `key_matches`: chord grammar, macOS spelling, shifted-bracket matching.
- `src/app/keymap.rs::write_default_keymap`, `write_default_keymap_at`: the never-overwrite default file.
- `src/app/keymap.rs::OperonApp::handle_app_shortcuts`, `chord_pressed`, `chord_consumed`, `consume_fixed_control_tab`: per-frame dispatch.
- `src/app/screens.rs::ui_settings_keys`: the Settings list and button; `src/app.rs::open_keymap_file`: the button's action.
- `src/config.rs::KEYMAP_FILE_NAME`, `KEYMAP_FILE_MAX_BYTES`, `CLOSED_TAB_LIMIT`: file name, ceiling, reopen-memory size.
- Tests: `every_default_chord_survives_being_written_and_read_back` pins defaults parse and round-trip.
- Tests: `every_chord_modifier_glyph_is_drawn_by_the_ui_font` pins the ⌘⇧⌥⌃ glyphs in the bundled font.
- Tests: `a_keymap_rebinds_an_action_by_id` pins rebind, `null`, and the drawn label.
- Tests: `a_chord_claimed_twice_refuses_the_whole_file` pins the duplicate rule naming chord and both actions.
- Tests: `a_keymap_that_cannot_be_read_keeps_the_defaults_and_says_why` pins each refusal reason.
- Tests: `writing_the_default_keymap_never_overwrites_one_that_is_there` pins never-overwrite.
- Tests: `the_palette_draws_the_chord_the_handler_listens_for` pins palette chords equal the live map.
- Tests: `no_message_spells_a_remappable_chord` pins that no message hard-codes a chord.
- Tests: `the_keys_section_lists_every_action` pins the Settings list.
- Tests: `the_new_session_chord_is_bound_and_listed_first_in_the_palette`, `the_next_waiting_chord_opens_the_next_session_in_the_queue`, `the_next_waiting_chord_with_an_empty_queue_changes_nothing`, `the_panel_chord_folds_and_reopens_the_side_panel` pin those actions.
- Tests: `shortcut_272_command_t_opens_a_terminal_like_the_plus`, `shortcut_272_the_most_specific_chord_wins`, `shortcut_272_close_is_a_keymap_action`, `shortcut_272_reopen_brings_back_the_last_closed_tab`, `shortcut_272_brackets_and_control_tab_step_through_tabs`, `shortcut_272_palette_rows_run_the_tab_actions`, `shortcut_272_a_control_tab_binding_is_the_keymaps`, `shortcut_272_reopen_crosses_back_to_the_tabs_project` pin the tab chords.
- Tests: `shortcut_272_every_manual_lists_every_default_chord` pins the manuals against the defaults.
- Tests: `broadcast_282_f1_page_shortcuts_do_not_reach_past_the_sheet` pins that page chords are dead behind the broadcast sheet.

## Decisions inherited
- Change 028: shortcuts are data in a file, the palette draws what the handler reads, and a self-contradicting file is refused whole because half a keymap cannot be told from none.
- Change 028: the file is the editor; there is no in-app binding editor, no chord sequences, no per-scope bindings, no remapping of anything that reaches a terminal.
- Change 028: action ids are the contract with the file and do not change without a migration story.
- Change 272: ⌘T, ⌘W, ⇧⌘T, ⇧⌘], ⇧⌘[ and ⌃Tab joined the registry so Settings and the file can see and move them; ⌘W left the hard-coded workspace draw.
- Change 272: lenient Shift/Alt matching is kept (a JIS `=` needs Shift) but an exact match wins, so ⌘T and ⇧⌘T are two chords.
- Change 272: ⌘⇧] arrives as `}` on macOS, so shifted brackets match through Shift; other shifted symbols are not guessed because layouts differ.
- Change 272: closed tabs are remembered up to `CLOSED_TAB_LIMIT` (20) for the life of the window only; persisting them belongs to relaunch restore (change 270).
- Change 273: ⌘T is the same action as the + menu's ターミナル row.
- Change 282: chords that would discard typed text (palette, page changes) are not live while the launch sheet or the broadcast sheet is open.
