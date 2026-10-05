# Intent: reading an agent's diff and telling it what is wrong are two different applications

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

Operon shows an agent's diff well: every file, every hunk, folded, numbered on
both sides, with the words that changed inside a rewritten line marked. A person
reads it and finds four things wrong. Then they have to say so, and there is
nothing here for that. They switch to the terminal, and now they are typing from
memory: which file, roughly which line, what it was they objected to. By the
third note they are scrolling back to the diff to check, and by the fourth they
have quietly dropped the one that was hardest to describe.

The review and the reply are the same activity, and they happen in two panes
with nothing between them.

## Who feels it, and when

Every time an agent finishes and its work is read rather than accepted. It is
worst when the review finds several separate things, which is the normal case
and the case where a human reviewer is worth the most.

## Desired outcome

A note can be written against the line it is about, while reading. The notes
collect. When the reading is done, one action hands the whole set to the agent
working in that project, in a form that names each file and line, so the agent
does not have to guess which of eighty lines "the null check" meant.

Notes survive the application being closed, because a review interrupted is a
review that gets finished later.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- The diff pane virtualises its rows: it lays out only what the viewport shows,
  and nothing in it may become work proportional to the whole diff per frame.
- The persisted store is a paused surface, so notes cannot live in it.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Nothing may be typed into a person's session without their asking for it.

## Systems likely affected

The diff drawing in `src/ui/diff.rs`, which currently assumes every row is one
line high. A new module for the notes themselves and the text they turn into.
`src/app.rs` for the footer, the delivery, and the choice of which session
receives it.

## Open questions

- **Where do the notes live, given the store is paused?** Answered in `spec.md`:
  a sidecar file beside the store, the way cancellation intents and the setup
  approvals already do.
- **Which session receives them?** Answered in `spec.md`.

## Not in scope

- Comments on a range of lines, on a whole file, or on rendered markdown. One
  line is what the diff pane can anchor to today.
- Re-anchoring a note when the line it was written against moves. The note
  records what the line said, and a note whose line has changed is marked as
  such rather than silently moved to the wrong place.
- Resolving, replying, or a history of what was sent. The terminal already holds
  what was sent.
- Attribution marks for AI-written lines. There is no established design to
  follow.
