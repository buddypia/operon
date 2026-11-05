# Intent: The selected session row reads as a warning instead of a selection

- **Status**: approved
- **Opened**: 2026-11-05

## Problem

The row a person has selected in the session column, and the current page in
the toolbar, sit on a warm brown tint. At a glance the tint reads as age, dirt,
or a warning rather than as "this is the one you picked": it sits in the same
warm family as the colour that means a session is waiting on the person, the
colour of one vendor's mark, and the colour of a recovered session, so the eye
cannot separate "selected" from "needs you" without stopping to read.

## Who feels it, and when

A person triaging several running sessions from the session column — the
app's main screen — deciding which terminal to open next. Every time more
than one session is live, some of them waiting and one of them selected.

## Desired outcome

Afterwards, the selected row is recognisable as *selected* at a glance, in a
colour that no status in the app also uses, and the statuses on that row stay
readable. The one orange button that starts a session becomes the only warm
thing claiming attention on the screen. This holds in all three themes.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Colour is chosen by role, never by literal at a call site; a value that
  changes in the binary changes in `DESIGN.md` in the same breath.

## Systems likely affected

The theme tables and every surface that paints the selected-row role today:
the session column, the pinned waiting list, the project cards, the toolbar
navigation, and two places that borrow the tint for something that is not a
selection (a hover, an attention banner).

## Open questions

- Whether the left accent bar the design document already prescribes for a
  selected sidebar row lands in this change or a later one. Answered in
  `spec.md`: later — this change is the colour and the inks that sit on it,
  and the bar is a separate drawing change with its own screen.

## Not in scope

- The 2px accent bar prescribed by `DESIGN.md` for a selected sidebar row.
- Any new colour role or value: the replacement comes from a role the table
  already carries.
- Layout, density, typography, and the status model.
