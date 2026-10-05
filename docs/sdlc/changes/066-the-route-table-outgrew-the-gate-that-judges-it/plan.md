# Plan: the route table outgrew the gate that judges it

- **Spec**: none — `bugfix` route. The refusal below is the intent.
- **Approved**: 2026-09-21
- **Status**: done

## The cause, named

Change 065's commit was refused by the review gate:

```
route `bugfix` is not in docs/sdlc/routes.yaml on either side of this diff,
or is not the 5 positional values with a reviewer set of full/craft/none
```

`.githooks/reference-transaction` does not run this tree's
`scripts/check-review.sh`. It runs `refs/heads/main:scripts/check-review.sh`,
deliberately — a commit is the one place that can edit its own judge. main's copy
has `[ "$#" -eq 5 ] || continue`, so a route row of six values is passed over on
both sides of the diff and every route becomes unreadable at once.

Change 064 added that sixth value, the review-round ceiling. Its own commit
landed because the route table is read from both sides and its *parent* still had
five values. The defect was latent by exactly one commit.

Measured, not inferred: `git show refs/heads/main:scripts/check-review.sh` line
423 is `[ "$#" -eq 5 ] || continue`; `bash scripts/check-review.sh --index` in
this tree parses the same table without complaint; and the 065 plan commit passed
only because a paper-trail-only diff exits before the route is read.

**Nothing on this branch could land a code change until this is fixed.**

## Files that change

| File | Change |
|---|---|
| `docs/sdlc/review-rounds.yaml` | new — the ceiling, keyed by route name, with the reason it is not a column |
| `docs/sdlc/routes.yaml` | rows back to five values; header says why five is a contract and not a style |
| `scripts/check-review.sh` | `-ge 5` back to `-eq 5`; the ceiling read from the new file, both sides, stricter wins |
| `src/tests.rs` | route guard back to five; `every_route_row_is_the_width_the_gate_that_judges_commits_reads`; `every_route_has_a_review_round_ceiling`; 064's fixture rewritten in the shape that parses |
| `REVIEW.md`, `.claude/skills/sdlc/SKILL.md` | both named the sixth column |
| `docs/sdlc/lessons.md` | entry 039 |

## Why a separate file rather than any of the alternatives

- **Keep the column and land the parser on main first.** The right end state, and
  not reachable from here: a commit on `main` is refused, and the merge is the
  user's chosen direction with main taking the branch. That would leave this
  branch unable to commit until then.
- **A trailing comment after the closing quote.** main's route `sed` requires the
  line to end at the quote, so the row stops matching entirely. Measured against
  main's blob.
- **Encode the ceiling in the fifth value (`none/2`).** main's `as_set` passes it
  through, `rank` answers `-1`, and the row is skipped. Same failure.
- **Drop per-route ceilings and default everything to 2.** Works, and throws away
  the part of 064 that was thought about: `docs` gets one round because prose is a
  nit, `feature` and `security` get three.

A file main's gate does not read can be added freely. That is the whole of it,
and it is now written at the top of both tables.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The two files drift — a route with no ceiling, or a ceiling for a route that is gone | a ceiling silently falls back to 2, or bounds nothing | `every_route_has_a_review_round_ceiling`, both directions |
| Someone widens `routes.yaml` again | every commit with a code diff is refused, for a table that looks correct | `every_route_row_is_the_width_the_gate_that_judges_commits_reads`, with the width read out of the parser and the operator checked, not just the number |
| The premise moves — the hook stops reading main's gate | the rule above is enforced with no cause behind it | the same guard asserts the premise first, and says so rather than going on |
| main's copy changes its width to something other than 5 | this tree is green and commits are refused | **not caught here, and deliberately.** A test that reads another branch is green in a fresh clone that has no main, which is a silent skip — lesson 009's shape. The argument lives in the two table headers instead |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 490 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — the two standing warn-tier metrics and nothing new.
- `bash scripts/pipeline-indicators.sh --lessons` — 39 of 39.
- Four mutations watched failing, each at a different assertion — `./mutations.py`.
- **The commit itself.** This change lands or it does not; the refusal it exists
  to remove is the same gate that judges it.

## Departures from the plan

None. The alternatives above were measured before the shape was chosen, not after.
