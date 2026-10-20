# Plan: split eval runs are named, the hillclimb reads them first, and every reviewer can be started

- **Spec**: `./spec.md`
- **Approved**: 2026-10-21
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | three tests and `subagent_definition_problems` |
| `scripts/run-evals.sh` | the `split:` line in the report `awk` program |
| `evals/README.md` | steps 4 and 5 of "Hillclimbing the steering" |
| `docs/sdlc/lessons.md` | entry 081 |
| `docs/sdlc/changes/184-eval-runs-that-disagree-go-unread/` | this change's records |

No paused surface is edited: `.claude/agents/` is read, not written.

## Order of work

1. Write the three tests; run each by name on this tree and watch the first two
   fail for the stated reason (no `split:` line; the README sentences missing).
   The third passes on today's definitions, so it is watched failing through
   its mutants instead.
2. Add the `split:` line to `scripts/run-evals.sh`; the runner tests pass,
   including the existing outcomes test.
3. Edit `evals/README.md` steps 4 and 5; the README test passes.
4. Mutations: remove each check in `subagent_definition_problems` in turn —
   front matter, missing description, empty description, the 1,024-character
   limit, the name, `REVIEW.md` — and watch the test fail on the mutant that
   check owns; weaken the runner's
   condition to `p > 0` alone and watch the split test fail on the all-pass
   eval; drop one README sentence at a time and watch its test fail. Restore
   and confirm the files are byte-identical.
5. Lesson 081; `make q.fast`; `bash scripts/check-bands.sh`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The new line changes a line an existing runner test reads exactly | `the_eval_runner_judges_every_run_and_keeps_infra_and_unstable_out_of_the_rate` fails | run it by name in step 2 |
| The counter file the split check writes collides between parallel test runs | the split test flakes | the counter lives in the fixture's own temporary directory |
| The README test only checks words, and a reworded rule passes while saying less | a rule weakened in a later edit | the test reads one phrase per rule, each watched failing when removed (step 4) |
| The subagent check passes vacuously on an empty directory | a deleted `.claude/agents/` reads as healthy | an explicit "no definitions" problem |

## Proof of completion

- `make q.fast` — no output past the compile lines.
- `cargo test --locked the_eval_runner_` — every runner test `ok`, the new one included.
- `cargo test --locked the_hillclimb_procedure_states_what_harness2_measured` — `ok`.
- `cargo test --locked every_subagent_loads_under_the_name_the_review_gate_asks_for` — `ok`.
- `bash scripts/check-bands.sh` — every metric within its band.
- The suite in CI on the pushed branch.

## Departures from the plan

None yet.
