# Intent: in the session column, one project's sessions cannot be told from the next, and a running session cannot be stopped from its row

- **Status**: approved
- **Opened**: 2026-10-17

## Problem

The person's words, verbatim:

- 「プロジェクト別の区切りやセッションの区切りの見分けが付かない。UI/UXを考慮して直して」
- 「セッションの終了するUIもなくて不便」

In the session column of the terminal workspace, a project's name is drawn
smaller than the names of its own sessions, and the gap between two sessions is
the same as the gap between two projects. Nothing — no frame, no line — marks
where one project's sessions end and the next project begins, or where one
session's two lines end and the next session's begin. Reading down the column,
the eye cannot group it.

A running session can be removed from its row (✕ on hover) but not stopped.
Stopping is only in the terminal header of the session currently open, so to
stop a session the person has to open it first; nothing on the row says that
stopping is possible at all. The row's time is also cut off at the column's
right edge (「12時」 for 「12時間前」).

## Who feels it, and when

- Every time the person scans the session column with more than one project
  registered, looking for a session.
- When they want to end a session they are not looking at — finished work, a
  stray launch — from the list.

## Desired outcome

- Each project and its sessions read as one block, visibly apart from the next
  project's block; the project's name is the strongest text in its block.
- Inside a block, each session is visibly separated from the next.
- The person stops a running session from its row in the column, without
  opening it first, and finds that action both on the row and in its menus.
- The row's time is shown whole.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese, through `tr`.
- Colours and sizes come from `Palette` roles and the `DESIGN.md` type scale.

## Systems likely affected

- The session column drawing in `src/app/screens.rs`.
- The test suite.

## Open questions

None: the person delegated the design ("UI/UXを考慮して直して").

## Not in scope

- The home screen's session rows (their ··· menu already has 停止 and
  セッションを削除).
- Collapsing a project's block.
- What stopping does: the same `stop_session` the terminal header calls.
