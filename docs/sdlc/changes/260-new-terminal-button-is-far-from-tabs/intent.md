# Intent: The new-terminal button sits far from the tabs it adds to

- **Status**: draft
- **Opened**: 2026-11-09

## Problem

The person asked: 「セッションのタブ名の右側に+ボタンとか押したら新しいターミナルが立ち上がるようにして」

A button that starts a plain terminal already exists in the terminal tab strip,
but it is pinned to the far right edge of the pane. With the usual two or three
short tabs it sits several hundred pixels away from the last tab name, on the
other side of empty space, so it does not read as "add another tab here" and the
person asked for it as if it did not exist.

## Who feels it, and when

The person working in the Sessions workspace who wants one more terminal next to
the ones already open, looking at the end of the tab names for the place to add
one — as browsers and editors put it.

## Desired outcome

Right after the last tab name there is a + button. Pressing it opens a new plain
terminal tab in that pane's project and working directory, and that tab appears
where the + was, with the + moving along to its right. When the tabs no longer
fit the pane, the + stays visible at the pane's right edge instead of scrolling
away with them. In an empty workspace the + sits where the first tab will appear.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

The terminal workspace drawing in `src/app/screens.rs`; tests in `src/tests.rs`.

## Open questions

None. What the button starts (a plain terminal in the pane's project and
worktree, change 254) is unchanged; only where it sits changes.

## Not in scope

- What the + starts, or a menu choosing an agent: it keeps starting a plain
  terminal through the existing bounded launch.
- The document tab strip and the sidebar's per-project + (「セッションを開始」).
