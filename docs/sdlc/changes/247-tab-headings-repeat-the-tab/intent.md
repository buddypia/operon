# Intent: tab contents repeat their tab as a big heading

- **Status**: approved
- **Opened**: 2026-11-05

## Problem

The person's words, verbatim:

- 「セッションタブの左側のサイドバーなんだけど、プロジェクトという文字がダサすぎるんだよね」
- 「他のタブはどうする？ここはA案だとしても」
- 「実装を進めて」
- On the overlap with change 245, the person chose 「245に寄せて全部進める」.

Change 245 handles the session column: its project groups and the column's
「プロジェクト」 label. What remains is the same shape everywhere else. A tab's
content opens by saying the tab's name again as a large heading: the 変更 tab
opens with 「Git」, worktree with 「worktree」, PR with 「プルリクエスト」, the
Sessions page with 「セッション」. Section labels inside a page — 設定 in the
settings sidebar, スキル and ルールと指示 inside エージェント設定 — are drawn at
page-title size too. Big type that carries no new information is what read as
dated in the column, and it is repeated across the app.

## Who feels it, and when

- Every time the person opens a project tab or the Sessions page and reads a
  heading that only repeats the tab they just clicked.
- When they compare pages: the same words at three different strengths
  (navigation tab, page header, in-tab heading) make the hierarchy unclear.

## Desired outcome

- One large heading per page at most: the existing `page_header` (20px, with
  subtitle and hairline) stays the page's title.
- A heading that only repeats its tab is gone; the controls that shared its
  row (refresh, view switches, search) stay.
- A label that groups content inside a page reads as a section, not a page
  title: 15px strong for in-page sections, 12.5px muted for panel labels,
  matching the form change 245 set in the session column.
- Tabs themselves do not change: their selected marks already say where the
  person is.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese, through `tr`.
- Colours and sizes come from `Palette` roles and the `DESIGN.md` type scale
  (section 15/600, label 12.5, chip 11.5).
- No persisted-shape change; no new string, icon, or colour role.
- Change 245 is in flight on the same file by another session and lands
  first; this change touches none of its regions and merges `main` in before
  landing.

## Systems likely affected

- In-tab headings in `src/app/screens.rs` (project detail tabs, Sessions
  page, settings sidebar, agent-settings sections).
- The test suite in `src/tests.rs`.

## Open questions

None. The person reviewed the classification (keep tabs and page headers,
remove repeated headings, demote section/panel labels) in the before/after
HTML proposals and said 「実装を進めて」.

## Not in scope

- The session column and terminal crumb: change 245 owns them.
- Top navigation tabs, project detail tabs, inspector tabs, and their
  selected marks.
- `page_header` itself and the pages that use it (Home, Projects, project
  detail title, Settings sections).
- Empty-state headings (「まずはプロジェクトを開いてください」,
  「セッションを選択」): the heading is the message there.
