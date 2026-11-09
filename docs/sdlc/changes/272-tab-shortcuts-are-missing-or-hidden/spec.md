# Spec: Tab keys a Mac user brings with them

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. ⌘T opens a plain terminal tab in the focused pane through the same path as
   the strip's + (`launch_workspace_terminal`), from any page, while neither the
   command palette nor the launch sheet is open. Test:
   `shortcut_272_command_t_opens_a_terminal_like_the_plus`.
2. ⌘⇧T never also counts as ⌘T: when one key press matches several bindings
   because egui ignores an extra Shift or Alt, the binding whose Shift and Alt
   match exactly wins, and only a press with no exact match falls back to a
   lenient one (so ⌘⇧K still opens the palette as before). Test:
   `shortcut_272_the_most_specific_chord_wins`.
3. ⌘W closes the active tab while the Sessions workspace is showing (unchanged),
   now through the keymap action `tab.close`; with `"tab.close": null` in the
   keymap file it does nothing. Test: `shortcut_272_close_is_a_keymap_action`.
4. ⌘⇧T reopens the most recently closed tab that can still be shown — a session
   still in the store, or a document whose project still exists — skipping tabs
   already open, selects it, and shows the Sessions workspace. With none left it
   shows 「開き直せるタブはありません。」 briefly and changes nothing. At most
   `CLOSED_TAB_LIMIT` (20) closed tabs are remembered; a closed tab is
   remembered once. A close refused for unsaved changes is not remembered. Test:
   `shortcut_272_reopen_brings_back_the_last_closed_tab`.
5. ⌘⇧] and ⌘⇧[ select the next and previous tab of the focused pane, wrapping
   round, whether the press arrives as `}` / `{` (what macOS reports with Shift
   held, on US and JIS layouts) or as `]` / `[`. ⌃Tab goes forward and ⌃⇧Tab
   now goes back. Test: `shortcut_272_brackets_and_control_tab_step_through_tabs`.
6. The five actions are in `KEYMAP_ACTIONS`, so Settings → キー lists them with
   their chords and the keymap file can move or remove them; the palette has
   rows 「空のターミナルを開く」 and 「閉じたタブを開き直す」 with their chords,
   and choosing either does what the key does. Tests:
   `the_keys_section_lists_every_action`,
   `the_palette_draws_the_chord_the_handler_listens_for` (existing, cover the
   new rows), `shortcut_272_palette_rows_run_the_tab_actions`.
7. The keyboard table in `docs/MANUAL.md`, `docs/MANUAL.ja.md` and
   `docs/MANUAL.ko.md` shows every remappable action's default chord as the
   app draws it (so ⇧⌘T, ⇧⌘], ⇧⌘[ and ⌘T appear in all three), and no longer
   lists ⌘W among the fixed keys. Test:
   `shortcut_272_every_manual_lists_every_default_chord`.

## Behaviour

Strings a person reads (new rows in every `src/i18n_tables.rs` table):

| Japanese | Where |
|---|---|
| 「空のターミナルを開く」 (existing) | Settings → キー row and palette row for ⌘T |
| 「タブを閉じる」 (existing) | Settings → キー row for ⌘W |
| 「閉じたタブを開き直す」 | Settings → キー row and palette row for ⌘⇧T |
| 「次のタブへ」 | Settings → キー row for ⇧⌘] |
| 「前のタブへ」 | Settings → キー row for ⇧⌘[ |
| 「開き直せるタブはありません。」 | brief notice, ⌘⇧T with nothing to reopen |

States:

- ⌘T with no project selected anywhere: the existing
  「先にプロジェクトを選んでください。」. With no tmux: the existing
  「tmux が必要です。「設定」を開いてください。」 and no tab. While a worktree
  operation runs: the existing 「worktree 操作の完了を待ってください。」.
- ⌘W on a document with unsaved changes: the existing refusal notice; the tab
  stays and is not remembered for ⌘⇧T.
- ⌘W, ⌘⇧[ ], ⌃Tab with no tab, or on another page, or over the session
  library: nothing happens. With one tab, next/previous keep it.
- ⌘⇧T on a session closed earlier: the terminal tab comes back in the focused
  pane, selected; its tmux session was never stopped by closing the tab. On a
  document: it opens again from disk (「読み込んでいます…」 until the read lands);
  edits discarded before closing are not restored — closing with unsaved edits
  is refused, so there are none.
- While the command palette or the launch sheet is open, none of the tab keys
  act (as ⌘N today).
- A keymap file that binds one of the new chords to another action is refused
  whole with the existing duplicate sentence
  「⇧⌘T が「…」と「…」で重複しています。」; the file can rebind the tab action first.
- The terminal pane never receives the ⌘ chords (unchanged: it drops every ⌘
  chord) nor ⌃Tab (consumed before the pane draws, as today).

## Design

Placement: `EXTEND`. The keymap (`src/app/keymap.rs`, `KEYMAP_ACTIONS`,
`resolve_keymap`) and the app-wide dispatch (`handle_app_shortcuts` in
`src/app.rs`) already own remappable chords; ⌘W and ⌃Tab live hard-coded in
`ui_tab_workspace` (`src/app/screens.rs`); the + calls
`launch_workspace_terminal`. Evidence: `git grep -n "consume_key(egui::Modifiers::COMMAND, egui::Key::W)"`,
`git grep -n "fn launch_workspace_terminal"`.

- `src/app/keymap.rs`: five `KeymapAction` rows — `tab.new` `Mod+T`,
  `tab.close` `Mod+W`, `tab.reopen` `Mod+Shift+T`, `tab.next` `Mod+Shift+]`,
  `tab.previous` `Mod+Shift+[`. New `Keymap::action_for(key, modifiers)` decides
  which action, if any, one key event triggers: candidates are bindings whose key
  matches (equal, or — with Shift held — the event's `{`/`}` for a binding's
  `[`/`]`) and whose modifiers `matches_logically`; one that also
  `matches_exact` wins, else the first candidate. Pure, no I/O.
- `src/app.rs`: `chord_pressed` and `chord_consumed` ask `action_for` of each
  pressed `Event::Key` in this frame instead of comparing one chord;
  `chord_consumed` removes the events it matched. `handle_app_shortcuts` gains
  `tab.new` and `tab.reopen` (gated like `session.new`) and `tab.close`,
  `tab.next`, `tab.previous`, ⌃⇧Tab then ⌃Tab (also gated on
  `page == Sessions && !session_library_open`). New `step_workspace_tab(forward)`
  is the cycling now inlined in `ui_tab_workspace`, with an empty-group guard.
  New `reopen_closed_tab()`. `close_workspace_tab` pushes onto a new
  `closed_tabs: Vec<WorkspaceTab>` field when `workspace.close` removed it,
  removing an earlier copy and dropping the oldest past `CLOSED_TAB_LIMIT`.
- `src/app/screens.rs`: the hard-coded ⌘W and ⌃Tab blocks in `ui_tab_workspace`
  go (⌘S stays); `palette_actions` gains rows "New terminal tab" (`tab.new`) and
  "Reopen closed tab" (`tab.reopen`); `activate_palette_entry` dispatches them.
- `src/config.rs`: `CLOSED_TAB_LIMIT: usize = 20`; the stale "Seven actions"
  sentence on `KEYMAP_FILE_MAX_BYTES` stops counting.
- `src/i18n_tables.rs`: four new rows in each table.
- `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: §8 table and the
  tab paragraph in §4.
- Nothing persisted changes: `closed_tabs` lives as long as the window; the
  keymap file's shape is unchanged (new ids are optional keys).
- Per frame: `chord_pressed` walks this frame's events (usually none) for each of
  about fifteen actions; no allocation, I/O, or process. `reopen_closed_tab`
  runs only on a key press; a document reopen reaches the existing background
  `FileOpen` read.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Nothing is painted in a new colour; Settings rows and palette rows reuse their existing drawing. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No icon is added; chords use the existing `⌘⇧⌥⌃` text glyphs in `keymap.rs`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Action ids live once in `KEYMAP_ACTIONS`; `every_default_chord_survives_being_written_and_read_back` round-trips the new defaults (`⇧⌘]` back to `Mod+Shift+]`). `CLOSED_TAB_LIMIT` is in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No store, sidecar, migration, cancellation, or byte limit is touched; the keymap file is read as before. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | ⌘T calls the existing `launch_workspace_terminal`; no new child or launch path. |
| Documentation — user-facing docs change in all three languages together | yes | §8 of `docs/MANUAL.md`, `docs/MANUAL.ja.md` and `docs/MANUAL.ko.md` gains ⌘T, ⇧⌘T, ⇧⌘] and ⇧⌘[, moves ⌘W out of the fixed list, and the tab paragraph in each names ⇧⌘[ ]; requirement 7 holds all three. The READMEs name ⌘N, ⌘K and ⌘J in feature prose and keep no shortcut table, so they do not change. Message tables gain rows in every table. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | No scan or output; the one new collection, `closed_tabs`, is capped at `CLOSED_TAB_LIMIT` items. |

## Flagged concerns

- **⌘⇧] arrives as `}`** — answered: winit 0.30.13 on macOS
  (`platform_impl/macos/event.rs`, `create_key_event`) gives the logical key
  "heeding just SHIFT" when ⌘ is held, and egui-winit 0.31.1 sends the logical
  key, so `Mod+Shift+]` must also match `Key::CloseCurlyBracket`. Requirement 5
  pins both spellings.
- **egui ignores extra Shift** — answered: `consume_key` uses
  `matches_logically`, under which ⌘⇧T matches ⌘T and ⌃⇧Tab matches ⌃Tab (why
  ⌃⇧Tab moves forward today). Requirement 2's most-specific rule and the
  ⌃⇧Tab-first order answer it without making every existing chord strict.

## Acceptance

- `make q.fast` passes; `cargo test --locked shortcut_272_`,
  `cargo test --locked the_keys_section_lists_every_action`,
  `cargo test --locked the_palette_draws_the_chord_the_handler_listens_for` and
  `cargo test --locked every_default_chord_survives_being_written_and_read_back`
  pass.
- The mutation sweep catches: the exact-match preference removed, the
  shifted-bracket match removed, the closed-tab push removed, the ⌃⇧Tab order
  reversed, and the reopen existence check removed.
- In the running app: ⌘T adds a terminal tab; ⌘W then ⌘⇧T brings it back;
  ⇧⌘] and ⇧⌘[ step through tabs; Settings → キー lists the five rows.
- CI passes before landing; package from main and install.

## Rejected alternatives

- Strict Shift/Alt matching for every chord: fixes ⌘T vs ⌘⇧T but breaks a
  person's `Mod+=` on a JIS keyboard, where `=` needs Shift.
- Several chords per action in the keymap file: a file-format change for one
  pair; ⌃Tab stays a fixed alternate as browsers keep it.
- ⌘1–⌘9 for the Nth tab: taken by Home, Projects and Sessions.
- Keeping ⌘W in the workspace draw: it would stay out of Settings → キー and the
  keymap file, which is half of what the person could not find.
- Persisting closed tabs: browsers that do restore them across a relaunch do so
  as part of session restore, which change 270 owns separately.
