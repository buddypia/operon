# Verification evidence

## Choosing the design

The person compared the current sidebar with three interactive HTML proposals (counted tiles, a filter popover, a mailbox-style list) rendered in Operon's dark palette, and chose the counted tiles with 「承認・回答待ち」 folded into 要対応. The mockups lived under the ignored `.tmp/session-filters/` and are not part of the change.

## Failing guards before implementation

All four `sidebar_252_` tests failed against the 251 sidebar for the reasons they name: 「CLI」 still drawn, tile counts absent under a Codex choice, no waiting count on 要対応's hover, and tiles side by side at 180px.

## Executed checks

- Ten `sidebar_25` tests pass (four new, six from change 251 following the removed words and the folded waiting dimension).
- 195 related tests pass: sidebar, status, column, pinned, filter, session tab, change 181 and 237 interactions, Home, attention, unread, and every `every_` design and translation guard.
- `make q.fast` passes.
- The mutation sweep catches 7 of 7: tiles never pairing, tile clicks ignored, counts ignoring the CLI choice, the summary shown while unfiltered, 「クリア」 keeping the CLI choice, the hover's waiting count replaced with 0, and tiles pairing in the narrowest column.
- An independent empirical verifier attacked tile layout at 180–268px with 120 and 1200 sessions, and compared each tile count with the rows drawn and with `matching_session_count`, including unread and ended sessions. Nothing failed; it left no test.

## Native render

`sidebar-filtered.png` (要対応 and 実行中 chosen, Claude Code chosen, dark theme) and `sidebar-narrow.png` (180px column, 指示待ち chosen) are the egui output of `ui_terminal_workspace` on a disposable seven-session fixture, tessellated by egui and rasterised in software at 2x by a throwaway test that was removed before commit. They show the real fonts, palette, glyphs and vendor marks; they are not a capture of the installed application. Light theme, the default state and the no-match state were inspected the same way.

## First CI test failure

Run 42283114399 failed `only_selections_wear_the_selection_colour`: a chosen CLI mark was filled with `row_selected`, a role reserved for the session row and toolbar page selections. The chosen mark now uses `control_active` with the `border` hairline; the guard passes unchanged, and the full suite passed locally (1173 passed, 6 ignored) before the fix was pushed.
