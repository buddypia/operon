# Plan: an eval summary names its conditions and says when little of it was judged

- **Spec**: `./spec.md`
- **Approved**: 2026-10-21
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | `the_eval_runner_passes_the_model_and_effort_through` extended; `the_eval_runner_warns_when_most_runs_were_not_judged` |
| `scripts/run-evals.sh` | the conditions string; its line and the coverage warning in the report awk |
| `evals/README.md` | the Reading section, the headroom row, hillclimb steps 3 and 5 |
| `docs/sdlc/changes/189-an-eval-summary-hides-its-conditions-and-thin-coverage/` | this change's records |

No paused surface: `scripts/run-evals.sh` and `evals/` match no row in
`docs/sdlc/risk.yaml`.

## Order of work

1. The tests, watched failing on today's runner: no `conditions:` line in any
   of the three runs; the 3-of-9 fixture prints `no headroom` and no coverage
   warning. The 3-of-6 fixture passes today and must still pass after.
2. The runner: the conditions string beside `agent_options`, the awk line, the
   scheduled count, the warning, and the headroom warning made conditional on it.
3. `evals/README.md`.
4. Mutations: the conditions line removed; the model printed as `default`
   whatever was given; the coverage warning removed; the headroom warning
   printed beside it; the boundary moved to exactly half; the warning's `S`
   printed as the judged count (the words wrong, the condition right).
   `make q.fast`; `bash scripts/check-bands.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The new line breaks an existing assertion | a runner test red | every runner test asserts with `contains`; `cargo test --locked eval_runner` |
| The warning fires on a healthy suite | a warning beside a rate of a fully judged run | the existing outcome fixture (6 of 10) and the 3-of-6 boundary print none |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked eval_runner` — every runner test `ok`.
- `bash scripts/check-bands.sh` — within band.
- The suite in CI on the pushed branch.

## Departures from the plan

None yet.
