# Intent: the same project is the hero in two places

- **Status**: approved
- **Opened**: 2026-11-05

## Problem

The person's words, verbatim:

- 「セッションとプロジェクトのタブが分かれているけど、これ分けるのって意味あるかな？」
- 「セッションでもプロジェクトでも同じプロジェクトが表示されているからなぜか違和感を感じているんだよね。」

In the session workspace the left column draws each project as a card whose
name is the strongest text in the card, with its sessions underneath. The
Projects page draws the same project, with the same icon and strength, as a
card that opens its detail. The same face does two different things: in the
column it only folds and selects, on the Projects page it navigates. Reading
the app, a project looks like the hero twice, and the sessions it exists to
hold look secondary in the place where the person works on them.

## Who feels it, and when

- Every time the person scans the session column to find or resume a session
  with more than one project registered.
- When they move between the session workspace and the Projects page and meet
  the same project as if it were a different thing.

## Desired outcome

- In the session workspace, a project reads as a heading that groups its
  sessions, not as a destination. The session rows are the strongest text.
- A project is the hero only on the Projects page, where opening it is what
  the page is for.
- Which project the open terminal belongs to stays visible, in smaller text.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese, through `tr`.
- Colours and sizes come from `Palette` roles and the `DESIGN.md` type scale.
- No persisted-shape change: sessions stay grouped by the project they hold.

## Systems likely affected

- The session workspace column and terminal pane header in `src/app/screens.rs`.
- The test suite in `src/tests.rs`.

## Open questions

None. The person reviewed the before/after HTML (`.tmp/session-project-dedup/`,
rev 2) and said 「進めて」.

## Not in scope

- Merging the Projects and Sessions tabs, or removing either page.
- The Projects page list and project detail: they keep their current face.
- The history view's project grouping (`ui_terminal_session_tabs`).
- Collapsing, reordering, or starting a session from the column: all kept.
