# Intent: an eval summary hides the conditions it ran under and how little of it was judged

- **Status**: approved
- **Opened**: 2026-10-21

## Problem

Harness² (`~/dev/oss/harness2`, Apache-2.0) keeps two things beside every
figure it reports. It records the configuration an experiment ran under and
refuses to judge it under another: "Judge model changed for this experiment;
choose a new HARNESS2_RUNS_DIR" (`harness2/cli/judge.py:169-171`). And its
report warns when the rows behind a figure are not the rows it meant to read —
a stage that fell back, a selection that mostly kept the reference output
(`harness2/core/report.py:133-163`).

Here, `scripts/run-evals.sh` records neither. Its summary ends with the base
commit and the runs per eval; `--model`, `--effort`, `--split`, `--recheck`,
the agent, and `EVAL_TIMEOUT` leave no trace in `summary.txt` or `summary.md`.
The hillclimb in `evals/README.md` compares a baseline summary with a later
round's (steps 3, 5, and 8), and two summaries cannot say whether they ran the
same model: a baseline under `--model haiku` against a round under the default
model reads as a gain or a regression the steering did not cause.

And the summary's headroom warning reads only the judged runs. When 6 of 9
runs end `infra` — every agent timing out on two evals — and the 3 judged runs
pass, the summary prints `3/3 passed` and "no headroom left to show a gain; the
suite needs harder cases". Both point the next session at the wrong problem:
the suite is not too easy, most of it was never judged.

## Who feels it, and when

The next session that hillclimbs the steering, comparing a baseline it did not
run with its own round; and whoever reads a summary after a bad night of
timeouts and is told to write harder evals.

## Desired outcome

Every summary names the conditions its runs were made under, in one line two
summaries can be compared on. A summary where fewer than half the scheduled
runs were judged says so in place of the headroom warning.

The person's request, 2026-10-21: 「~/dev/oss/harness2를 참고해서 하네스 설계를
내 프로젝트에 알맞게 최대한 적용시켜줘.」

## Constraints this change inherits

- Change 184: the rate line keeps its shape; a new fact gets its own line.
- Change 188: a `refused:` summary prints no rate, and still prints what follows it.

## Systems likely affected

`scripts/run-evals.sh`, `src/tests.rs`, `evals/README.md`.

## Open questions

None.

## Not in scope

- Refusing to compare two summaries whose conditions differ: the runner never
  reads an earlier summary, and the comparison is a reading the README asks for.
- Recording the agent's version: one more child call per run of the suite for
  a field the model name mostly already pins.
