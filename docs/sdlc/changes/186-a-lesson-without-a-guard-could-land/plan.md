# Plan: the suite admits a lesson only with its rule and a guard that ran

- **Spec**: `./spec.md`
- **Approved**: 2026-10-21
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | the test, `lesson_admission_problems`, `FIRST_LESSON_WITH_WATCHED_GUARD`, the script fixture |
| `docs/sdlc/lessons.md` | the header sentence on what the suite refuses |
| `docs/sdlc/changes/186-a-lesson-without-a-guard-could-land/` | this change's records |

No paused surface: `scripts/pipeline-indicators.sh` is run, not edited.

## Order of work

1. Write the test with its mutants and the script fixture. On today's tree it
   passes — every entry already conforms — so it is watched failing through
   its mutants and through a copy of `lessons.md` with one entry's guard
   renamed to a test that does not exist.
2. The header sentence in `lessons.md`.
3. Mutations: remove each check in `lesson_admission_problems` in turn and
   watch the test fail on the mutant it owns; lower the threshold to 001 and
   watch the real file fail; make the test ignore the script's exit code and
   watch the fixture assertion fail.
4. `make q.fast`; `bash scripts/check-bands.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The script reads `src/tests.rs` from its repository, so the fixture needs one | the fixture passes vacuously | the fixture asserts exit 1 and the entry named, and exit 0 when the guard exists |
| A wrapped "watched\nfailing" is missed | a real entry refused | whitespace normalised before the search; 081 and 082 pass |
| An entry added by a parallel change takes a number already used | a skipped or repeated number | the numbering check names the entry |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked every_lesson_` — every lesson test `ok`.
- `bash scripts/check-bands.sh` — within band.
- The suite in CI on the pushed branch.

## Departures from the plan

None yet.
