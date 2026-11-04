# Plan: a change lands on evidence the machine re-runs, not on a reviewer's opinion

- **Spec**: `./spec.md`
- **Approved**: pending — gate configuration loses a check (the verdict
  requirement), which `.claude/skills/sdlc/references/approval.md` gives to a person
- **Status**: draft

## Files that change

| File | Change |
|---|---|
| `docs/sdlc/verification-methods.yaml` | new: methods, prefixes, surface and route requirements |
| `docs/sdlc/templates/verification.yaml` | new: the record's shape |
| `scripts/lib/verification.sh` | new: parse and validate a record (shared by gate and runner) |
| `scripts/check-review.sh` | the evidence rule, switched on by the table in the first parent; verdict rule kept for history |
| `scripts/run-obligations.sh` | new: CI runner |
| `.github/workflows/ci.yml` | `verify` job |
| `.claude/agents/empirical-verifier.md`, `.claude/agents/formal-verifier.md` | new |
| `src/tests.rs` | `exhaustive_check` and the ten tests in the spec |
| `REVIEW.md` | rewritten around empirical and formal evidence |
| `docs/sdlc/README.md`, `.claude/skills/sdlc/SKILL.md`, `.claude/skills/feature-pilot/SKILL.md`, `docs/sdlc/routes.yaml` (comments only), `docs/sdlc/risk.yaml` (comments only) | stage 5 described as verification |
| `docs/sdlc/lessons.md` | lesson: an opinion is not evidence, with its guards |
| `docs/sdlc/changes/242-…/verification.yaml`, `review.yaml`, `mutations.py` | this change's own evidence, and the old-style verdicts `main`'s judge still requires |

## Order of work

1. Method table and record template. The suite still passes: nothing reads them yet.
2. `exhaustive_check` and its two tests, with the failing test first.
3. `scripts/lib/verification.sh` and the record tests (requirements 2, 3, 4, 7),
   failing first.
4. `scripts/check-review.sh` switch, plus the tests for requirements 1 and 8
   in temporary repositories. Watch the verdict rule still pass this branch.
5. `scripts/run-obligations.sh` and the `verify` job (requirement 6).
6. Verifier agents (requirement 9).
7. Documents and the lesson.
8. This change's `verification.yaml` and `mutations.py`. The sweep removes each
   rule from the gate or the runner and requires a guard to go red.
9. Review by `rust-reviewer` and `subprocess-safety-reviewer` into
   `review.yaml`. This is the last time the old rule runs, because `main`'s
   judge requires it for this commit.
10. Push, CI green including `verify`, merge, then a scratch-branch check of
    the acceptance items that need the table on `main`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The switch reads the wrong tree and turns the old rule off for history | old commits refused on reflog-less re-judge | `review_gate_judges_history_before_the_method_table_by_verdicts` |
| `main`'s judge cannot read this branch's tables | every code commit on the branch refused | `routes.yaml` keeps 5 values; the new table is a file the judge does not read; step 4 runs the gate |
| Runner executes something the record smuggles in | arbitrary command in CI | prefix plus metacharacter refusal, and its mutation |
| A `model_` test that does not enumerate passes as formal | weak proof labelled formal | `every_model_test_goes_through_exhaustive_check` |
| CI time past the job ceiling | `verify` red with `時間切れ` | the ceiling; measured on the first runs |
| bash 3.2 cannot parse the new scripts | gate silently passes | the existing `bash -n` suite test covers `scripts/` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked` in CI — `test result: ok`, 0 failed.
- `cargo test --locked exhaustive_check_reports_the_shortest_counterexample` — fails before step 2, passes after.
- `bash scripts/check-review.sh --all` — `✅ SHIP` by verdicts on this branch.
- CI `verify` — green, and its log lists every obligation in this change's `verification.yaml`.
- `python3 docs/sdlc/changes/242-review-verdicts-rest-on-opinion/mutations.py` — every mutation caught.

## Departures from the plan

None yet.
