# Intent: some buttons and labels ignore the sizes every other control uses

- **Status**: approved
- **Opened**: 2026-10-09

## Problem

The person, quoting a screenshot of the "パスを手入力" toggle on the project
page: "the button is not done properly, there is no consistency — other buttons
have margin and padding, and some buttons exist where it was not applied. Make
the other buttons and the text consistent, and change the mechanism first so an
AI developing UI reads DESIGN.md and knows how to build it, so this never
happens again."

Some controls are visibly shorter than the ones beside them, with the label
pressed against the frame. Some text is drawn at sizes that are not on the type
scale the design document names.

## Who feels it, and when

Anyone opening the project page (the manual path toggle), confirming a session
removal in the sidebar, using the find bar's previous and next arrows, or
resolving a diff note: those buttons are cramped next to every other button on
the same screen.

## Desired outcome

- Every button is one of the two documented heights, 28px or 24px in a list
  row, with the shared padding, so no button on any screen is cramped.
- Every text size in the app is a level on the type scale in `DESIGN.md`.
- An agent writing UI is told, before it writes a button or a text size, which
  helper and which sizes to use; and a test fails if it does not, so the rule
  holds whether or not the agent read it.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/ui/widgets.rs`, `src/app.rs`, `src/app/screens.rs`, `src/ui/`,
`DESIGN.md`, `.claude/rules/palette-and-glyphs.md`, `src/tests.rs`.

## Open questions

None. The person asked for the mechanism and the fix together.

## Not in scope

Colours and icons: both already have a rule and a test. Menu items inside an
open popup menu, which egui lays out as a menu list, not as buttons on a page.
