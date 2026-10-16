# Intent: a project with no sessions says so instead of offering to start one

- **Status**: approved
- **Opened**: 2026-10-17

## Problem

The person's request, verbatim: 「プロジェクト別セッションがあるじゃん。セッションが
まだありませんじゃなくて+ボタンなどですぐセッションを作れるようにして。どの
UI/UXが適切か考えてから進めて。」

Wherever sessions are listed by project, a project that has none shows the line
「セッションはまだありません」 in its place. The line is an answer to a question
nobody asked: a person looking at an empty project wants to start something in
it, and the line names the absence without offering the way out. The one control
that does start a session there is a 24px glyph at the far end of the project's
header, recognisable only on hover.

## Who feels it, and when

- Right after adding a project: it appears in the session column with nothing
  under it but 「セッションはまだありません」, and the person goes looking for
  how to begin.
- In the history's 「プロジェクト別」 and 「グリッド」 views, the same line stands
  in each project that has no sessions.

## Desired outcome

In every by-project session list, a project that has no sessions shows, in the
place its sessions would be, an action reading 「セッションを開始」. One click on it
opens the launch sheet for that project, over the page the person is on, with
their remembered launch settings — so the next ⌘↩ starts the session. Under a
search, a project with nothing matching says 「一致するセッションはありません。」
rather than offering a start: what is missing is a match, not a session.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- Every way to start a session goes through the launch sheet, which is where the
  dangerous-mode acknowledgement and the argument checks live.

## Systems likely affected

- The session column of the terminal workspace and the history's two views.
- The test suite.

## Not in scope

- Launching without the sheet (a one-click start with the last settings).
- The project overview page, which already shows 「新しいセッション」 directly
  above its list.
- The empty states that are not per project (no projects at all, no sessions at
  all on the home page).
