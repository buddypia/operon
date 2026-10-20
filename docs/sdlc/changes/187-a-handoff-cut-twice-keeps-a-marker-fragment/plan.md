# Plan: a handoff text cut twice by bytes keeps its marker whole

- **Spec**: none — route `bugfix`; the cause below is the intent
- **Approved**: 2026-10-21
- **Status**: approved

## The cause

`shorten_to_bytes` in `src/history.rs` cuts `text` at `max_bytes` less the
room for `HANDOFF_SHORTENED_MARKER`, then appends the marker. A text that an
earlier handoff already shortened ends in that marker, and the byte cut is
taken from the whole text: when the room lands inside the old marker, the
result keeps its first bytes — `[shortened by Op` — before the new one. The
bundle reaches this cut for the task when the task and the person's
instructions together are over `HANDOFF_BUNDLE_MAX_BYTES`, so a re-handoff of
a shortened task with long instructions shows the fragment in the task and in
the session name (`session_display_goal` maps only a trailing marker to `…`).
`shorten_for_handoff` already strips an earlier marker before it cuts (change
185); the byte form does not. Carried in change 185's review as
subprocess-safety-reviewer finding 2/1.

## Files that change

| File | Change |
|---|---|
| `src/history.rs` | `shorten_to_bytes` cuts the body of a text an earlier cut marked |
| `src/tests.rs` | `a_handoff_cut_twice_by_bytes_keeps_its_marker_whole` |
| `docs/sdlc/lessons.md` | entry 083 |

## Order of work

1. The test: a task already ending in the marker, handed off with
   instructions of every length from 300 to 1,200 characters, so that for some
   of them the bundle shortens the task by bytes with the room landing inside
   the old marker. For each bundle, the marker's first four bytes occur exactly
   as often as the whole marker, and the bundle is within its budget; the test
   also asserts that at least one length took the byte cut, so it cannot pass
   by never reaching it. Watched failing on today's `shorten_to_bytes`.
2. The fix: strip a trailing marker and the whitespace before it, cut the
   body on a char boundary within the room, and append one marker — the same
   rule `shorten_for_handoff` follows.
3. The test passes, and so does every `handoff` test.
4. Mutations: put the whole-text cut back; drop the char-boundary walk.
5. Lesson 083, `make q.fast`, `bash scripts/check-bands.sh`. Restore and the
   shared archive do not move, so `cargo test --locked -- --ignored` is not
   needed, and is said so.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The body is shorter than the room, so nothing needs cutting but the text is still over budget | a marker on a text that was not shortened further | the body-fits case returns body plus one marker, which is what the text already was |
| The cut moves off a char boundary in Japanese | a panic on slicing | the char-boundary walk, the Japanese task in the test, and mutation 2 |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked handoff` — every handoff test `ok`.
- `bash scripts/check-bands.sh` — within band.
- CI on the branch, then `bash scripts/package-macos.sh` and the ship skill.

## Departures from the plan

None yet.
