# Intent: the editor can only open files that live under the project folder

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

Change 026 made the path in a stack trace a way to the file: hover it in a
terminal, click, and the editor opens at that line. It works in a session
running in the project folder. It never works in a session running in a
worktree — which is how this application runs agents in parallel, and therefore
most of the time.

Two conditions decide it, and they read different places. A path underlines when
the file is inside the *session's worktree*. A click opens it when the file is
inside the *project folder*. A worktree created here sits beside the project
rather than inside it, so for a worktree session those two sets never overlap:
the path underlines, the click reports 「このプロジェクトの外にあります」, and the
editor never opens.

The guarantee change 026 was written around — nothing underlined is
unfollowable — does not hold where it matters most.

## Who feels it, and when

Anybody who clicks a path in a terminal running in a worktree, which is every
parallel session. The workaround is to add the worktree itself as a second
project, which multiplies the project list by the number of branches.

## Desired outcome

Clicking a path in a worktree session opens that file, in that worktree, and the
person can tell from the tab which copy they are editing. Saving writes back to
the file that was opened.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface. `OpenDocument` lives in `src/app.rs`
  and is not persisted, so this needs no schema move.
- The failure this must not introduce is worse than the one it fixes: a document
  whose root and path are paired wrongly is a save into the wrong copy of a
  file. Today the application refuses and writes nothing.

## Systems likely affected

The editor's open, save, and external-change paths in `src/app.rs`, the tab row
and the review pane in `src/app/screens.rs`.

## Open questions

- Where does a person see which copy they are editing? — answered by the person
  on 2026-09-06: a branch badge on the tab.

## Not in scope

- Opening a file that is outside both the project and any of its worktrees. The
  refusal stays for that, because there is no root to root it at.
- The file tree, which keeps showing the project. A worktree document is opened
  by following a link, not by browsing.
