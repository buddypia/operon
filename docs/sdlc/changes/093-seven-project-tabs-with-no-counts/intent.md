# Intent: seven project tabs, and none says where something is happening

- **Status**: approved
- **Opened**: 2026-09-27

## Problem

The owner's Before/After review (section ③ プロジェクト詳細, row 2) found the
project page's tab row hard to read:

- Seven tabs — 概要 / Git / worktree / PR / エディタ / スキル / ルール — sit
  side by side with equal weight. スキル and ルール are two lists of files the
  agents read, and each is a whole tab.
- 「Git」 names the tool, not what is inside it (uncommitted changes, diffs,
  history). 「エディタ」 names one mode of a tab whose first job is browsing
  the project's files.
- No tab carries a number, so nothing says where there is something to look
  at — changes waiting, worktrees open, pull requests — without opening each.

## Who feels it, and when

Every time the owner opens a project to see where it stands or to find a
file, a diff, or an instruction file.

## Desired outcome

As in the approved `/tmp/operon-ui-before-after/view-projects-after.html`
(a copy sits in the worktree's `.tmp/ui-before-after/`),
③ row 2:

- Six tabs: 概要 / 変更 / worktree / PR / ファイル / エージェント設定.
- スキル and ルール become one tab, 「エージェント設定」, showing both lists.
- 変更, worktree, and PR carry a count once it is known, and 変更's count is
  emphasised when there is something uncommitted.
- Nothing that was reachable becomes unreachable.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31. User-facing text is Japanese, through
  `tr`/`tf!` with a row in every table.
- No persisted-store change; the selected tab is in-memory today.
- The draw path does no I/O: counts come from caches the tabs already fill.

## Systems likely affected

`src/app.rs` (`ProjectTab`), `src/app/screens.rs` (the tab row, the merged
tab), `src/ui/widgets.rs` (a tab with a count), `src/ui/session_tree.rs` (a
message that names the Git tab), `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None. The owner answered "全項目 OK、進めて" to the Before/After on 2026-09-27
and "続けて" after changes 091 and 092.

## Not in scope

- The contents of 変更, worktree, PR, and ファイル: only their tab labels.
- Settings (④): the next change.
