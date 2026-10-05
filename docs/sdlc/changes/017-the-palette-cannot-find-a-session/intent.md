# Intent: the quick-action palette cannot find a session

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

`⌘K` opens a palette of five fixed actions: open a page, add a project, scan a
workspace. It cannot find the thing the app is actually full of. With a dozen
sessions across four projects, reaching one means going to the sessions page and
reading down a sidebar that groups by project, or opening the session library
and typing into its own search field — two different searches, in two different
places, neither of which is the one that opens on a keystroke.

The palette also cannot be driven from the keyboard. It focuses its field, and
then every result has to be clicked: there is no selection, no `Enter`, and the
`command_selection` field that would hold one is written on open and read by
nothing.

## Who feels it, and when

Anyone past their third session. Switching between agents is the app's inner
loop, and it currently costs a page change, a scan of a grouped list, and a
click on a small row.

## Desired outcome

- `⌘K` finds sessions and projects by typing any part of what is on their row —
  a name, the request, the agent, the branch, the project.
- The first match is selected when the results change, arrow keys move the
  selection, and `Enter` opens it without the mouse.
- Choosing a session opens it in the terminal workspace; choosing a project
  opens that project.
- The five actions that are there today are still there, and still first when
  nothing is typed.
- A result says enough to be told from its neighbours: what it is, which project
  it belongs to, and what state it is in.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls, no new dependency.
- User-facing text is Japanese; code, comments, and docs are English.
- Nothing is persisted by this change.
- The palette is drawn every frame while it is open, so matching and ranking
  must not walk the store more than once per frame.

## Systems likely affected

`src/app.rs` (the palette, its ranking, its keyboard handling), `src/tests.rs`.

## Open questions

None. Ranking, the result cap, and the field set are decisions this change
makes and the spec records.

## Not in scope

Searching files, transcripts, or commands; sections and headings in the result
list; digit shortcuts; a separate recents list when nothing is typed; and any
change to the session library's own search.
