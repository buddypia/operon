# Intent: a session's state dot does not say which state it is

- **Status**: approved
- **Opened**: 2026-11-11

## Problem

The person's request, verbatim, with a screenshot of the tab strip after
change 283:
「左サイドバーと画面のようにセッション名にステータスの小さいアイコンがあるけど、このアイコンだけではなんなのかわからないからステータスのアイコンを小さくして表示するのはどう？」
Offered a recommendation — a status glyph scaled to the 5px dot cannot be
read, so make the dot larger (about 9px) and draw a simple state shape inside
it, keeping the state's colour — they answered:
「ならば左サイドバーのアイコンも変更した方がいいんじゃない？絵文字ではなくアイコンを生成してやってみて」

Today a session tab and its row in the left column carry the state as a 5px
coloured dot on the CLI's mark. Only 要対応 holds a shape (an exclamation);
every other state is told by colour alone, so a person cannot tell 実行中 from
アイドル from 完了 without hovering. The left column's project rows count
sessions per state group with font glyphs beside the count, which read as
emoji next to the generated marks the tabs use, and a session with no mark of
its own still draws a font glyph in its row.

## Who feels it, and when

Anyone who keeps several agent sessions open and scans the tab strip or the
left column to find the one that finished, or the one that is free.

## Desired outcome

- A session's state on its mark is told by a shape as well as a colour, for
  every state, readable at the tab's size without hovering.
- The left column uses the same generated marks for states as the tabs: its
  per-project state counts and the row of a session without a CLI mark draw
  the generated mark, not a font glyph.
- A running plain terminal still shows no state, as change 283 decided.
- Hovering still names the CLI and the state.

## Constraints this change inherits

- `eframe`/`egui` 0.31 immediate-mode GUI on macOS; nothing new per frame
  beyond painting.
- Colours come from `Palette` roles, geometry from constants in
  `src/glyphs.rs`; no new palette role.

## Systems likely affected

The session tab strip, the session rows and project rows of the left column.

## Open questions

None.

## Not in scope

- The home screen's state tiles, the status chips and the 要対応 rows, which
  say the state in words beside their glyph.
- Changing which states exist or how they are grouped.
