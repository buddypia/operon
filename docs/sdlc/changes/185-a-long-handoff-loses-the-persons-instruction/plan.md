# Plan: a long handoff keeps the person's instruction, and says where Operon cut

- **Spec**: none — route `bugfix`; the cause below is the intent
- **Approved**: 2026-10-21
- **Status**: done

## The cause

`build_compact_handoff_bundle` in `src/history.rs` renders the person's handoff
instructions as the last section of the bundle. When even the bundle holding
only the last turn is over `HANDOFF_BUNDLE_MAX_BYTES`, it returns that bundle
through `truncate_utf8_bytes`, which keeps the first 6,000 bytes — so the
instructions are what goes. Japanese is three bytes a character, and the task,
one turn, and the note are each allowed 1,200 characters, so a long Japanese
handoff reaches that branch. Separately, every per-turn cut goes through
`truncate_chars`, which ends the text with a bare `…`: the session receiving
the bundle cannot tell Operon's cut from the source's words, nor that the rest
is in the transcript snapshot the bundle names. Harness² prints its own cuts
as its own for that reason (`harness2/core/context.py`, the truncation marker
"is ours").

Watched failing on main's builder:
`a_long_handoff_keeps_the_persons_instruction_whole` ("the person's handoff
instructions were cut") and `a_handoff_says_where_operon_shortened_the_text`
("a shortened turn does not say Operon shortened it").

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `HANDOFF_SHORTENED_MARKER` |
| `src/history.rs` | a cut that says it is Operon's; the bundle fitted by shortening the excerpt, then the task, and the note last |
| `src/tests.rs` | the two tests above |
| `docs/sdlc/lessons.md` | entry 082 |

## Order of work

1. `shorten_for_handoff(text, max_chars)` and `shorten_to_bytes(text,
   max_bytes)` in `src/history.rs`: unchanged when the text fits, otherwise the
   cut text, a newline, and `HANDOFF_SHORTENED_MARKER`, the byte form on a char
   boundary and never past its budget. The four `truncate_chars` calls in the
   builder go through the first; `truncate_chars` itself is shared with
   `src/git.rs` and stays.
2. The fitting order, every step bounded by `HANDOFF_BUNDLE_MAX_BYTES`: the
   longest tail of turns that fits, as today; then the task, the note, and the
   last turn shortened to the room left; then no turns; then the task shortened
   to the room left; then — only for a note past the budget on its own — the
   note shortened. The bundle is never cut from its end.
3. Both tests pass, and so do
   `compact_handoff_bundle_bounds_turns_and_bytes_and_strips_tool_spam` and
   `compact_handoff_bundle_peels_nested_envelopes_and_preserves_display_goal`.
4. Mutations: put the end-cut back; drop the marker from the per-turn cut; let
   the byte shortening exceed its budget by the marker's length.
5. Lesson 082, `make q.fast`, `bash scripts/check-bands.sh`, then
   `cargo test --locked -- --ignored` is not needed (restore and the shared
   archive do not move) and is said so.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The marker inside the task is carried into a re-handoff's task | a nested bundle repeats the marker | `compact_handoff_bundle_peels_nested_envelopes_and_preserves_display_goal` still passes; the marker is honest there too |
| A shortened task changes the session's display goal | the title shows the marker | the display reads the task; a task over 1,200 characters was already cut there with `…` |
| The byte shortening overruns by the marker | a bundle over 6,000 bytes | the byte assertion in both tests, and mutation 3 |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked handoff` — every handoff test `ok`.
- `bash scripts/check-bands.sh` — within band.
- CI on the branch, then `bash scripts/package-macos.sh` and the ship skill.

## Departures from the plan

- `HANDOFF_SHORTENED_MARKER` was added to `src/config.rs` before this file was
  committed, so the second test could compile and be watched failing; it is a
  constant only, read by nothing but that test until step 1. The commit gate
  runs clippy over the working tree, and an unused constant is a warning, so
  this file could not be committed until step 1 used it: the plan landed in
  its own commit after the code was written and before the code was committed.
- The marker first began `[…`, which the second test reads as the bare
  ellipsis it forbids; the marker lost the `…`, the test is unchanged.
- `src/agents.rs` changes too. A task over 1,200 characters ends in the marker,
  and `session_display_goal` reads the task, so the session's name would have
  shown the English marker where it showed `…` before. It maps the marker back
  to `…`; `a_long_handoff_keeps_the_persons_instruction_whole` reads the name
  and was watched failing before the mapping. Risk takes `src/agents.rs`'s
  surface, subprocess (medium, supervised), the same tier.
