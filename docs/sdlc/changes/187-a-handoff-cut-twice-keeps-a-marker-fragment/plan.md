# Plan: a handoff cut never splits a marker inside the text

- **Spec**: none — route `bugfix`; the cause below is the intent
- **Approved**: 2026-10-21
- **Status**: approved

## The cause

Both cuts in `src/history.rs` — `shorten_for_handoff` (characters, for every
turn, the task, and the note) and `shorten_to_bytes` (bytes, to fit the
bundle) — strip only a marker at the very end of the text before they cut. A
`HANDOFF_SHORTENED_MARKER` with text after it — an earlier handoff a person
pasted or an agent quoted into a turn — is cut through whenever the limit lands
inside it, and the bundle carries its first bytes, `[shortened by Op`, as if
they were the source's words. Watched failing on main's code:
`a_handoff_cut_never_splits_a_marker_inside_the_text` (a turn of 1,115
characters before a quoted marker) and
`a_handoff_byte_cut_never_splits_a_marker_inside_the_text` (instructions of
705 characters). Change 185's review carried the byte form as
subprocess-safety-reviewer finding 2/1, for a text ending in the marker; that
case cannot happen, because `shorten_to_bytes` reserves the marker's room and
so cuts before a trailing marker — measured across 901 instruction lengths
before this plan was revised.

## Files that change

| File | Change |
|---|---|
| `src/history.rs` | `cut_outside_marker`: a cut that would split a marker steps back to the marker's start; both cuts go through it |
| `src/tests.rs` | the two tests above |
| `docs/sdlc/lessons.md` | entry 083 |

## Order of work

1. The tests, each watched failing on main's code with its own message, each
   asserting it reached the case so it cannot pass by never cutting near a
   marker.
2. `cut_outside_marker(text, end)`: when a marker starts before `end` and ends
   after it, return its start; otherwise `end`. `shorten_for_handoff` takes its
   character limit as a byte index and passes it through; `shorten_to_bytes`
   strips a trailing marker as `shorten_for_handoff` does, then cuts on a char
   boundary and passes the cut through. Neither result ever exceeds its limit:
   stepping back only shortens.
3. Both tests pass, and so does every `handoff` test.
4. Mutations: either cut bypassing `cut_outside_marker`; the helper returning
   `end` unchanged.
5. Lesson 083, `make q.fast`, `bash scripts/check-bands.sh`. Restore and the
   shared archive do not move, so `cargo test --locked -- --ignored` is not
   needed, and is said so.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The step back lands off a char boundary | a panic on slicing | the marker is ASCII and starts on a boundary; the Japanese text around it in both tests |
| A marker at the very start of the text | the cut steps back to 0 and the turn is empty | an empty byte cut already falls through to the next rung; the char cut keeps the marker after it |
| The search costs a scan per cut | slower handoff building | it scans from one marker length before the cut, a few kilobytes at most, once per handoff, off the draw path |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked handoff` — every handoff test `ok`.
- `bash scripts/check-bands.sh` — within band.
- CI on the branch, then `bash scripts/package-macos.sh` and the ship skill.

## Departures from the plan

- The first approved version named a byte cut through a trailing marker as
  the cause. Its test passed on main's code across every length it tried, and
  the arithmetic says why: the byte cut reserves the marker's room. Probing
  found the real case, a marker with text after it, in both cuts; this
  revision is the evaluator's second reading.
