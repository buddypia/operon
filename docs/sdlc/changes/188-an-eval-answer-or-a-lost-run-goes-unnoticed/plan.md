# Plan: an eval's answer stays out of the steering, and every run is counted once

- **Spec**: `./spec.md`
- **Approved**: 2026-10-21
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | `no_eval_prompt_is_pasted_into_the_steering` extended; `EVAL_ANSWER_LINE_MIN_CHARS`; `the_eval_runner_refuses_a_rate_over_runs_it_did_not_schedule` |
| `scripts/run-evals.sh` | the reconciliation before the rate |
| `evals/README.md` | the `refused:` line; step 6 |
| `docs/sdlc/changes/188-an-eval-answer-or-a-lost-run-goes-unnoticed/` | this change's records |

No paused surface: `scripts/run-evals.sh` and `evals/` match no row in
`docs/sdlc/risk.yaml`.

## Order of work

1. The fixture test, watched failing on today's runner: two passing evals
   `001-a` and `001-b`, both `id: 001`, one run each; today the summary prints
   `eval 001 2/2 passed`, a rate, and exits 0. The test asserts exit 1, a
   `refused:` line naming eval 001 run 1, and no `evals:` rate line.
2. The duplicate check in the report awk: count `(id, run)` pairs; on a
   duplicate print `refused:` and no rate. The runner already exits 1 on any
   row that is not `pass`; it exits 1 as well when the summary holds a
   `refused:` line, even when every row passed.
3. The steering test: setup and check lines added, the count asserted, and a
   mutant — the test's own steering list with one check line appended — refused.
   Watched failing by deleting the new sections from the comparison.
4. `evals/README.md`.
5. Mutations: the duplicate check removed; the rate printed despite a
   refusal; the exit on a refusal removed; the setup and check sections dropped from the steering
   test; the threshold raised past every line. `make q.fast`;
   `bash scripts/check-bands.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The duplicate check fires on a healthy run | a refusal on every run | the existing runner fixtures, with unique ids, still print their rate |
| The steering test grows slow | a longer suite | 145 lines against about 40 documents, substring search, milliseconds |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked eval` — every eval test `ok`.
- `bash scripts/check-bands.sh` — within band.
- The suite in CI on the pushed branch.

## Departures from the plan

None yet.
