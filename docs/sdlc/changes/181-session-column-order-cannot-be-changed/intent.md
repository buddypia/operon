# Intent: the session column's order is fixed, and a person cannot arrange it

- **Status**: approved
- **Opened**: 2026-10-20

## Problem

The person's words, verbatim:

- 「プロジェクト別のセッション内の配置も移動できるようにしたい。」

The session column of the terminal workspace, headed 「プロジェクト別セッション」,
can already be moved as a whole: its heading is dragged to another edge of the
workspace. What is inside it cannot. The projects always come in alphabetical
order of their names, and each project's sessions always come newest first. A
person who works mostly in two of ten projects, or who keeps one long-running
session they return to all day, finds them wherever the fixed order puts them,
every time, and has no way to put them where they want them.

## Who feels it, and when

- Every time the person looks down the column for the project they work in
  most, and it sits below projects they rarely open because of its name.
- Every time a new session starts in a project: it takes the top of that
  project's card and pushes the session they keep coming back to down.

## Desired outcome

- The person moves a project's card to another place among the projects, and
  it stays there.
- The person moves a session's row to another place among its own project's
  sessions, and it stays there.
- The arrangement is still there after Operon is quit and opened again.
- A person who never moves anything sees the column exactly as today.
- Projects and sessions added later still appear without the person having to
  find them.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is on real machines and a paused surface; the
  arrangement is a per-machine preference, not work anybody would miss.

## Systems likely affected

- The session column drawing in `src/app/screens.rs`.
- The session workspace module, `src/ui/session_tree.rs`, beside the column's
  dock layout.
- App state in `src/app.rs`; a file name in `src/config.rs`.

## Open questions

None left open. Decided, and recorded in `state.yaml`: both levels move; a
session stays in its own project.

## Not in scope

- The 「履歴」 page's 「プロジェクト別」 and 「グリッド」 lists: they keep their own
  orders.
- Moving a session into another project: its terminal and worktree belong to
  the project's folder.
- The 要対応 queue at the top of the column: it stays in the order sessions
  began waiting.
- Sorting choices (by name, by date): the person arranges by hand.
