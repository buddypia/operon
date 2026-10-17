# Intent: an empty project offers to start a session twice, in words it does not need

- **Status**: approved
- **Opened**: 2026-10-17

## Problem

The person's words, verbatim, on seeing change 158 land: 「セッションを開始は冗長
じゃない？どう？+ボタンとかでもいいかも」

In the session column, an empty project now shows two controls that do the same
thing: the ＋ at the end of its header line and, under it, a framed
「＋ セッションを開始」. The second one repeats the first and spends a line and a
sentence saying what the ＋ already means. The history's two by-project views
show the framed button too, while their headers offer no ＋ at all — so the same
action sits in a different place, with a different look, depending on the view.
In 「プロジェクト別」 the header's terminal-window glyph reads 「プロジェクトを開く」 and in fact
opens the launch sheet.

## Who feels it, and when

- Right after adding a project, in the session column: two ＋ for one action,
  stacked.
- In the history's 「プロジェクト別」 and 「グリッド」 views, where the start sits
  under the project's name in one state and nowhere in another.

## Desired outcome

Every project, in every by-project session list, has exactly one ＋ at the end of
its header line, and clicking it opens the launch sheet for that project — the
same in the session column, 「プロジェクト別」, and 「グリッド」, whether the project
has sessions or not. An empty project shows nothing under its header but that
header: no framed button, no 「セッションはまだありません」. Under a search, a
project with nothing matching still says 「一致するセッションはありません。」.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Every way to start a session goes through the launch sheet.
- Change 158's search wording stays.

## Systems likely affected

- The session column of the terminal workspace and the history's two views.
- The test suite.

## Not in scope

- What the launch sheet itself shows.
- The project overview page's 「新しいセッション」.
