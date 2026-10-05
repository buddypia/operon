# Intent: A rewritten line says only that it was rewritten

- **Status**: approved
- **Opened**: 2026-08-30

## Problem

The review pane marks a change by the row: the whole removed line sits in one
wash, the whole added line in another. That answers "how much did this agent
touch" and stops there. It does not answer "what did it change", which is the
question the person is actually holding when they open the pane.

When an agent rewrites a line in place — renames one identifier, flips one
argument, corrects one string — the two rows are nearly identical and the
difference between them is a few characters somewhere across eighty columns.
The screen presents that as two full-width coloured bands and leaves the reader
to find the difference themselves, character by character, on every such pair.
The colour is doing the opposite of its job: it draws the eye to the whole row
and says nothing about the part that moved.

## Who feels it, and when

Every time an agent edits existing code rather than adding new code — which is
most of what an agent does after the first pass. It is worst on the lines that
matter most: a long call whose one argument changed, a condition whose operator
flipped, a path string with one segment different. Those are exactly the edits
a person needs to catch and exactly the ones a row wash hides.

## Desired outcome

Opening the review pane on an in-place rewrite, the changed words are visibly
marked inside their rows, so the difference between the two lines is found by
looking rather than by reading. Where the two lines are not a rewrite of each
other — a wholesale replacement, an unrelated insertion — nothing extra is
marked, because a mark that is usually wrong is worse than no mark.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The diff pane draws one galley per visible row every frame, so nothing added
  here may be computed while drawing.
- A colour role is only added through `DESIGN.md` and all three palettes, and it
  has to survive the contrast checks.

## Systems likely affected

`src/git.rs` (where a diff is parsed into what the pane reads), `src/models.rs`
(what one diff line carries), `src/ui/diff.rs` (how a row is drawn),
`src/theme.rs` and `DESIGN.md` (the marks are a colour).

## Open questions

- How two sides of a change are paired into "these two lines are the same line".
  Answered in `spec.md`: positionally within a replacement block, gated on
  similarity.
- What happens on a line too long to compare word by word. Answered in
  `spec.md`: a stated token ceiling with an honest, cheaper fallback.

## Not in scope

- Syntax highlighting inside the diff pane. The diff answers what changed; the
  editor answers what the code is. Colouring both at once puts two meanings on
  one surface.
- A side-by-side diff layout. This change is about what a row says, not about
  how many columns the pane has.
- Word marks in the terminal pane or the Markdown preview.
