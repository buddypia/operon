# Intent: section headings read as labels, and the session rows are cramped

- **Status**: approved
- **Opened**: 2026-11-25

## Problem

The person's request, verbatim:
「セッションページの左のサイドバーのプロジェクトのラベル、プロジェクトページのプロジェクトという文字のラベルとかインパクトかないというかそんなもんかな？業界標準の見出しやタイトル的な印象があまりない気がして改善したいけど
左のサイドバーのプロジェクトの中のセッション一覧に少しpaddingなど間隔調整をしたい。ぎゅうぎゅうになっている。」

Measured in the running build (change 314's harness, 2026-11-25):

- The session column's heading 「プロジェクト」 is 13.5px in the muted ink — the
  same size as the project names under it and dimmer than them, so it reads as
  a caption, not as the heading of the column.
- A page's title (「プロジェクト」 on the Projects page, 「ナレッジ」) is 20px,
  one step above the 16px body, and sits right under the toolbar's rule; it
  does not read as the title of a page.
- A session row is 20px tall (a 16px mark and 2px above and below) with 4px
  between rows, and starts at the column's left edge, left of its project's
  name: the rows are packed together and do not read as belonging to the
  project above them.

## Who feels it, and when

Anyone on the session page with a project unfolded (several sessions packed
into 24px each), and anyone opening the Projects or Knowledge page.

## Desired outcome

- The column's heading reads as the heading: larger and stronger than the
  project names it heads.
- A page's title reads as a title: clearly larger than anything else on the
  page and with room above it.
- Session rows have room to breathe and sit indented under their project's name.

## Constraints this change inherits

- Sizes, colours and spacing stay on `DESIGN.md`'s scale and palette roles; a
  new size is a new documented level.
- Nothing new per frame.

## Systems likely affected

The session column, the page header shared by pages, `DESIGN.md`, `src/tests.rs`.

## Open questions

None.

## Not in scope

- The project names' size, the filter tiles, the tab strip.
- Pages that draw their own title outside the shared page header, except
  「ナレッジ」, which draws the same 20px title by hand.
