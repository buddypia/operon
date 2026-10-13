# Intent: a docked panel can be moved somewhere it is not drawn, and the app says it moved

- **Status**: approved
- **Opened**: 2026-10-13

The person asked, after change 145 landed: 「批判的思考と敵対的レビューに基づいて最終検証を
行うこと。発見された問題点・改善点は根本的かつ長期運用が可能で、多角的な視点から検証されて
いること。AIがタスクを適切に理解して進行できるメカニズムによって再発を防止しつつ、
オーバーエンジニアリングは回避すること。」 This is what that review found.

## Problem

1. In a window too small for every edge, the workspace gives up an edge to keep
   the terminal its minimum. A move does not know that. Dropping a panel on an
   edge that is given up says 「…を下部に移動しました」 and draws nothing; moving a
   panel to the top can push the bottom edge — and a panel that was on screen —
   off it; putting the session list on the bottom hides the views on the top.
   The panel that vanished takes its own grip and menu with it, so the only way
   back is to make the window bigger, which the person is not told.
2. Change 145's review saw the first case and carried it to the person as
   "no action needed". Its rubric has no line for "the app says something
   happened that the screen does not show", so the finding was graded a nit.
3. Change 144 put grips on tabs that could only move within the side column, and
   the person had to point out that a grip which cannot move a panel where a
   grip promises is meaningless. Its intent stated the cue — an icon, a cursor —
   as the outcome.
4. Smaller, same area:
   - Escape's test passes with the Escape handling deleted, and no test drives a
     real press, move, and release.
   - Over its own edge, the drag says 「中央で離すとキャンセル」.
   - A right-button wobble on a grip starts a drag instead of opening its menu.
   - The placement menu button is drawn differently from the fold button beside it.
   - English reads "Moved Session list to the top".

## Who feels it, and when

Anyone on a laptop-sized window who moves a panel to the top or bottom, or puts
the session list there: the panel disappears and the app has just said it moved.

## Desired outcome

- Every panel that is on screen before a move is on screen after it. A move that
  could not keep that is not made, and the person reads why and what to do: the
  drop zone shows it while dragging, the menu entry is greyed with the reason,
  and a release there says 「ウィンドウが小さいため…には置けません」.
- A drag driven by real pointer events moves a panel; Escape cancels it; a
  release on the panel's own edge or in the middle changes nothing.
- A review grades "the app says something happened that the screen does not
  show" as Important, and an intent states what the person does and then sees,
  not the cue that invites it.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted shape changes.

## Systems likely affected

`src/ui/session_tree.rs`, `src/app/screens.rs`, `src/app.rs`, `src/ui/widgets.rs`,
`src/i18n_tables.rs`, `src/tests.rs`, `REVIEW.md`, `docs/sdlc/templates/intent.md`,
`docs/sdlc/lessons.md`.

## Open questions

None.

## Not in scope

- Remembering the layout across restarts. That changes the store's shape and is
  its own change.
- What happens when the window shrinks after a move. An edge given up for room
  comes back when the window grows, as the side panel did before change 145.
