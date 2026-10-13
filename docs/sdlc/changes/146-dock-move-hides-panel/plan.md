# Plan: refuse a dock move that would take a panel off the screen

- **Status**: approved

## Cause

`dock_panel_to` (src/ui/session_tree.rs) commits `move_panel` without asking
`dock_rects`. Meanwhile `fit_axis` drops the last non-kept edge of an axis when
the centre would fall below its minimum. So whether a panel is drawn is decided
after the move, by a function the move never consults. The menus and the drop
zones read only `edge != from`.

## Files that change

- `src/ui/session_tree.rs`:
  - Adds `DockPanel::ALL`, `DockLayout::{rects, keeps_on_screen}` and
    `dock_drop_hint`.
  - Changes `dock_grip` and `dock_panel_to`, adds `dock_views_to`, both through
    `dock_commit`; the placement menu's button takes `icon_button`'s look.
- `src/app.rs` — the `dock_area` field.
- `src/app/screens.rs`:
  - `ui_terminal_workspace` draws from `dock.rects(area, views)` and records the
    area.
  - `ui_dock_drop` paints refused zones and uses the hint.
- `src/i18n_tables.rs`:
  - adds 「ウィンドウが小さいため{edge}には置けません」;
  - replaces 「中央で離すとキャンセル」 with 「ここで離すとキャンセル」;
  - changes the EN row to "the session list".
- `src/tests.rs` — the four tests, and the Escape test now holds the button down.
- `REVIEW.md` — an Important line for a message the screen does not bear out.
- `docs/sdlc/templates/intent.md` — an outcome is the thing done, not its cue.
- `docs/sdlc/lessons.md` — 070.

## Order of work

1. Write the four tests, and make the Escape test hold the button down. Compile
   them against stubs: `DockPanel::ALL`, a `dock_views_to` that calls
   `move_views`, and a `dock_drop_hint` that returns the old text. The tests
   never set `dock_area`: drawing the workspace records it (step 3). Watch
   requirements 1, 2 and 4 fail on main's behaviour.
2. The model: `DockPanel::ALL`, `DockLayout::rects`, `keeps_on_screen`.
3. The app: the `dock_area` field written by `ui_terminal_workspace`, which now
   draws from `dock.rects`; `dock_fits` and `dock_commit`; `dock_panel_to` and
   `dock_views_to` through them; refused menu entries greyed.
4. The drag: refused zones in warning, and `dock_drop_hint`. Run the four tests;
   then the two mutations for requirement 3 and the Escape test.
5. The placement menu's button, and the strings.
6. `REVIEW.md`, the intent template, lesson 070.
7. Gates and release: fmt, clippy, and the dock, sidebar, translation, and
   design tests. Then review, CI, landing, packaging, and the swap.

## Risks

- `keeps_on_screen` checks a panel's tab, not its content. A view whose tab is
  drawn counts as reachable even while another tab is showing. This is intended.
- The test draws the workspace in a bare `CentralPanel`, so a test window's area
  is the whole window. The sizes in the tests are therefore areas.
- Judging with every view open refuses a move that would be fine while the views
  are folded. That is the price of a move not hiding them later; the notice
  says why.
- A window that shrinks after a move can still give up an edge (intent: not in
  scope); growing it brings the panel back.

## Proof of completion

- Requirements 1, 2 and 4 fail on main (step 1); requirement 3 passes there
  and fails under its two mutations (step 4). All four pass after step 4.
- These tests pass: `cargo test --locked dock`, `sidebar`, `translation`,
  `every_button`, `every_text_size`.
- fmt and clippy are clean, and CI is green on the branch head.

## Departures from the plan

- **Two small helpers beside the gate.** `dock_no_room(edge)` is the one place
  the refusal text is built (hint, menu hover, notice), and `dock_menu_entry` is
  the one menu row both menus draw. They keep the three places that say "no
  room" from drifting apart.
- **The manual** (three languages) says a too-small edge is outlined and refuses
  the panel, and that a release on the panel's own edge cancels.
- **Escape test.** The Escape test is the one that fails with the Escape branch
  deleted; the pointer test fails under both mutations.
