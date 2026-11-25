# Intent: the session column heading competes with project names, and pin marks sit after the agent mark

- **Status**: draft
- **Opened**: 2026-11-25

## Problem

The person's request, verbatim (2026-11-25):

> 左のサイドバーを改善したい。1. セッションページでプロジェクトというラベルの文字がタイトルみたいな意味合いを思ってると思うけど、このままでいいんだっけ？業界標準はどうしたらいいの？ 2. セッションリストでプロジェクトごとに表示しているが、プロジェクト名を少し大きくした方がよいんじゃない？3. セッションリストのセッションアイテムにピン留めするとアイコンの前に表示しておきたい。中央のセッションアイテムのピン留めのAIアイコンの左にして。

Clarified with the person via two questions:
- Top panel label: `「セッション一覧」に変更し、12.5px の控えめな見出しにする（右に＋・履歴）`
- Sidebar pin alignment: `① ガター配置（左の余白に 📌 を置き、全行の AI アイコンの縦位置を揃える）`

Three issues exist on the Sessions page today:
1. The left column's top label reads 「プロジェクト」 in 15px bold strong ink — like a page title — right below the toolbar that already holds a 「プロジェクト」 page tab and above session filter tiles rather than projects.
2. Each project heading in the left column is 13.5px bold, only 1.5px larger than the 12px bold session titles under it, so project names do not stand out clearly above their sessions.
3. On a pinned session, the pin mark sits after the CLI/agent mark in a sidebar row (`[AI] 📌 title`) and at the right edge of a central workspace tab (`[AI] title 📌`), so scanning pinned items left-to-right does not show the pin before the agent mark.

## Who feels it, and when

Anyone working on the Sessions page scanning the left column across multiple projects, or looking at pinned sessions in the left column and central tab strip.

## Desired outcome

- The left column's top label is a quiet 「セッション一覧」 panel label rather than a prominent 「プロジェクト」 heading, keeping `+` and 「履歴」 beside it.
- Project names in the left column read clearly larger than both the quiet top label and the session titles beneath them.
- Pinned sessions show their pin mark to the left of the AI/status mark in both the left column (in the left indent gutter so every row's AI mark stays vertically aligned) and the central workspace tabs (with no close button on pinned tabs).

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese, with sorted English and Korean entries in translation tables; code, comments, and docs are English.
- Sizes and colours come from `DESIGN.md`; folded project headings stay within 28px top-to-top.

## Systems likely affected

The session column and central workspace tabs in the app UI, layout constants, translation tables, feature documents for `session-column` and `terminal-pane`, and tests.

## Open questions

None.

## Not in scope

- Moving or removing the toolbar's 「プロジェクト」 navigation tab.
- Changing the pin behaviour, sort order (`workspace_session_rank`), or `session-order.json` schema.
- Renaming 「プロジェクト」 on the Projects page or elsewhere in the app.
