# Intent: a session's tab and row read as two marks side by side

- **Status**: draft
- **Opened**: 2026-11-10

## Problem

The person's request, verbatim, with a screenshot of the session workspace:
「タブ名と左のサイドバーにステータスとターミナルアイコンが表示されるけど、なんか二つも表示されて違和感を感じているけど、どうしたら良いか？」
Offered a recommendation — keep the CLI's mark as the one mark and carry the
state as a small dot on its lower right corner, with the waiting state keeping
a shape; give a plain terminal the same mark in the left column instead of the
word ターミナル; and let a terminal say nothing while it is simply running —
they answered 「進めて」.

Today a session tab draws two 14px marks of equal weight before its title — a
circled state mark and the CLI's mark — and a row in the left column draws a
state glyph, then a vendor mark for an agent but the word 「ターミナル」 for a
terminal. Two marks of the same size and weight compete, neither leads, and
the two places disagree about how a terminal is drawn. A plain terminal's
state is 実行中 for as long as it exists, so its mark repeats a fact that never
changes on every terminal row and tab.

## Who feels it, and when

Anyone who keeps several sessions open in the workspace and scans the tab
strip or the left column to find one.

## Desired outcome

- A session's tab and its row each lead with one mark: the CLI's.
- The state is still visible without hovering, as a dot on that mark, coloured
  as the state is coloured today; a session waiting for the person is still
  told apart by shape, not by colour alone.
- A plain terminal that is simply running shows no state; one that stopped,
  failed or was lost shows it.
- The left column draws a terminal with the same mark as its tab.
- Hovering the mark still names the CLI and the state, with its explanation.
- A session with no mark of its own (a custom command) keeps today's state
  mark, so its state is not lost.

## Constraints this change inherits

- `eframe`/`egui` 0.31 immediate-mode GUI on macOS; nothing new per frame
  beyond painting.
- Colours come from `Palette` roles, geometry from constants in
  `src/glyphs.rs`; no new palette role.
- User-facing text is Japanese through `tr`.

## Systems likely affected

The session tab strip and the session rows of the left column.

## Open questions

None.

## Not in scope

- Status chips, the 要対応 rows, the home screen's session rows and the
  header line, which already say the state in words.
- A setting to choose between the two layouts.
