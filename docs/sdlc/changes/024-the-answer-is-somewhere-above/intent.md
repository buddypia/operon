# Intent: the thing you need is somewhere in four thousand lines of scrollback

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

An agent's turn produces a great deal of output, and the useful part of it is
one line. Which test failed, which file the stack trace names, the port the dev
server actually chose. Operon holds fifty thousand lines of scrollback per
session and offers no way to look through it except scrolling and reading. So
people scroll, miss it, scroll back, and eventually give up and ask the agent to
say it again — which costs a turn and gets a paraphrase instead of the line.

## Who feels it, and when

Every long turn, which is most of them. It is worst immediately after a failure,
when the output is longest and the thing being looked for is furthest up.

## Desired outcome

Type what you are looking for, see where it is, and step through the places it
occurs, without leaving the pane or losing your place in it. The gesture is the
one every other application uses, so nobody has to learn it.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- The terminal pane virtualises its rows and must keep doing so: a search must
  not make the pane do work proportional to fifty thousand lines every frame.
- Japanese is typed through an IME, and change 014 exists because a commit key
  arriving mid-composition is not a keystroke.
- While the search field has focus, keys belong to the field and not to the
  agent: a stray character typed into a running agent is a real cost.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/ui/terminal.rs` for finding matches and drawing them. `src/app.rs` for the
per-session search state and the key handling. `src/app/screens.rs` for the bar.

## Open questions

- **Where does the bar go?** Answered before the code: under the pane, one row,
  the way a browser does it.

## Not in scope

- Regular expressions, whole-word, and case-sensitivity switches. Each is a
  control on a bar that is meant to be one row.
- Searching across sessions. That is the palette's job and a different question.
- Detecting file paths and URLs in output and offering actions on them. It is
  worth having; it is a separate change with a separate screen.
