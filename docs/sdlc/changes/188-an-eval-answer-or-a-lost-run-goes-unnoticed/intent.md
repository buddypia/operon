# Intent: an eval's answer in the steering, or a run counted twice, goes unnoticed

- **Status**: approved
- **Opened**: 2026-10-21

## Problem

Harness² (`~/dev/oss/harness2`, Apache-2.0) guards its measurement in two
places this repository does not. Before any context reaches an agent, it scans
what the agent will read — including a harness an agent wrote — for the
grader's own text, and refuses on a hit: "a leak, not a false positive to
widen" (`harness2/core/context.py`). And before it reports a rate, it checks
that the runs it counted are exactly the runs it scheduled, duplicates first,
because a duplicate can make the counts come out right (`core/ledger.py`).

Here, `no_eval_prompt_is_pasted_into_the_steering` keeps an eval's prompt out of
`CLAUDE.md`, `AGENTS.md`, `REVIEW.md`, `.claude/rules/`, and `.claude/skills/`,
but not its setup or its check — the part that says what passes. A hillclimb
that copies a check's command into a skill teaches the agent the answer, and
the suite stays green. And `scripts/run-evals.sh` counts whatever rows
`results.tsv` holds: two eval files that share an id merge into one line of the
summary, with twice the runs, and the rate is printed as if nothing were wrong.

## Who feels it, and when

The next session that hillclimbs the steering: an edit that pastes a check line
is kept because the score rose, and the rise is the leak. And whoever adds an
eval numbered like one a parallel change also added: the summary reads 6 of 6
for a suite of seven evals run three times, and nobody is told.

## Desired outcome

The suite refuses a steering document that holds a line of any eval's setup or
check, naming the eval and the document. The runner refuses to print a rate
when its rows are not one per eval per run, naming what is duplicated or
missing.

The person's request, 2026-10-21: 「~/dev/oss/harness2를 참고해서 하네스 설계를
내 프로젝트에 알맞게 최대한 적용시켜줘.」

## Constraints this change inherits

- Change 184: the runner's report shape — the rate line, `infra`, `split:` —
  stays as it is when the rows reconcile.
- Lesson 004: the guard reads what it guards — the eval files themselves, not a
  copied list of lines.

## Systems likely affected

`src/tests.rs`, `scripts/run-evals.sh`, `evals/README.md`.

## Open questions

None.

## Not in scope

- Scanning transcripts or the main checkout for check text: the runner already
  flags a run that read `evals/` as `peeked`.
- Banning measurement words from always-loaded text: the steering names the
  eval suite on purpose, as the place a hillclimb is measured.
