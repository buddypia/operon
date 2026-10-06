# Intent: the session list and the session's side panel take two columns from the terminal

- **Status**: approved
- **Opened**: 2026-10-06

## Problem

In the session workspace the left edge holds two columns side by side: the
project-grouped session list (「プロジェクト別セッション」, 268px) and, since
change 122 put it on the left, the side panel with 「ファイル」「会話」「変更」
(240–300px). Together they take about 540px before the terminal starts, on a
screen whose main content is the terminal. The person asked whether the two
should be one sidebar, and asked for the placement to be decided from industry
practice and usability — including placing one above the other — and then built:

> プロジェクト別セッションのサイドバーととファイル、会話、変更のサイドバーを一緒にした方が良いのでは？
> 業界標準やユーザービリティを考えてどう配置するか考えて。上に配置とかも多角度で考えて結論を出して進めて。

## Who feels it, and when

Anyone supervising agents on a laptop-width window: the terminal is narrowed to
whatever is left after two side columns, so agent output wraps, while both
columns are mostly white space below their content.

## Desired outcome

1. The session list and the file / conversation / changes panel share one left
   column, so the terminal gets back the width of a whole column.
2. Both stay visible at once: a person reviewing a diff can still see which
   other session needs them.
3. The person can choose how much of that column each half gets.
4. Folding the side panel, or moving it to the right edge (changes 122/125),
   still works; moved right, it is a column of its own again.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese through `tr()`/`tf!` and `src/i18n_tables.rs`.
- No persisted shape change: layout stays in-memory `OperonApp` state, as in 122.

## Systems likely affected

- the session workspace layout and its side panel drawing code
- the inline test suite

## Open questions

None. The person delegated the decision ("結論を出して進めて"); the options
weighed are in the spec's Rejected alternatives.

## Not in scope

- Persisting the split across launches.
- Restyling the session list or the side panel contents.
