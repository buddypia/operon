# Intent: dimmed text and keyboard focus fall below WCAG 2.1 AA

- **Status**: approved
- **Opened**: 2026-09-12

## Problem

Two visibility gaps sit below the WCAG 2.1 AA floor the app otherwise holds.
The dimmed grey used for metadata, counts, and labels measures around
3.3:1 to 4.2:1 on the surfaces it is actually read on, but it is drawn at
11–12px — ordinary body size, which needs 4.5:1. And the custom clickable
cards and rows give no visible mark when they hold keyboard focus, so a
person tabbing through the app cannot tell where they are.

## Who feels it, and when

Anyone reading secondary lines — session metadata, file-tree marks, code
block language tags, zero-value counts — on the Dark or Light theme, every
time those lines are on screen. And anyone operating the app by keyboard,
every time focus lands on a custom card or row instead of a standard button.

## Desired outcome

Afterwards, every line of text at body size clears 4.5:1 on every surface
in all three themes, and every element reachable by Tab shows where focus
is with a mark that clears 3:1. The existing contrast suite pins both, so a
future colour change that reopens either gap fails the build instead of
shipping.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

The theme tables and the contrast tests, plus the custom drawing code for
clickable cards and rows.

## Open questions

None. Scope (colour plus focus and operability, all three themes, Light and
terminal/diff weighted first) was agreed with the author before this intent
was written.

## Not in scope

- Text resizing and OS-level scaling behaviour: the app follows the OS
  setting and adds no control of its own.
- Tooltips on keyboard focus for icon-only buttons: hover text stays the
  mechanism; every icon-only control already keeps its wording as hover
  text, and irreversible actions keep their words outright.
- The diff washes and word marks keep their documented low ratios: the
  `+`/`-` signs carry the meaning and already clear 4.5:1.
