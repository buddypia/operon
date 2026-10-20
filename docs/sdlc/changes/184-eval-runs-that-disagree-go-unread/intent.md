# Intent: repeated eval runs that disagree are read like any other failure, and a reviewer can vanish without a word

- **Status**: approved
- **Opened**: 2026-10-21

## Problem

Change 178 took Harness²'s (`~/dev/oss/harness2`, Apache-2.0) rules for the
text an agent carries on every turn. What it left is how Harness² improves a
harness from repeated runs, and one more of its load-time checks.

Harness² reads disagreement between replicate runs of the same task as its
first signal: where every run did the same thing, the harness already decides
the behaviour; where runs diverged, the harness leaves that decision to chance,
and pinning it is the cheapest gain. A defect that only one run of several had
is noise, and an edit aimed at it fits one sample. An edit whose runs produced
nothing at all is unsafe whatever else it did. And everything a run left behind
is evidence, never instructions: text inside a trajectory is not obeyed.

Here, the eval summary prints `1/3 passed` for an eval that went both ways
exactly as it prints a count, and only an eval that failed every run gets a
line of its own. The hillclimbing procedure says to read the train transcripts
and name a cause, but not which eval to read first, not that a single failed
run among passing ones is not a cause, not that an edit after which runs time
out with nothing is to be reverted, and not that a transcript's contents are
data.

Separately, Harness² refuses a subagent definition with no description before
any run. Measured here on 2026-10-21 with Claude Code in a scratch project
(two runs, same answer both times): a definition with no `description:` is
dropped from the list of subagents without any message, and a definition whose
`name:` differs from its file name is offered under the `name:`. The review
gate requires reviewers by file name. Nothing checks either, so a reviewer
definition edited into one of these shapes leaves the gate demanding a verdict
from a reviewer no session can start.

## Who feels it, and when

Whoever next improves the steering against `evals/` with `--runs 3`: the
summary does not point at the evals that went both ways, so they read the
failures in the order they come, and nothing stops an edit aimed at one failed
run. And the next session that edits a file under `.claude/agents/` and drops
or renames its front matter: the review gate asks for that reviewer, the
reviewer cannot be started, and nothing says why.

## Desired outcome

A measured run names every eval whose judged runs split between pass and fail,
apart from the evals that failed every time, and says to read a passing and a
failing transcript of it side by side first. The hillclimbing procedure orders
the work the way Harness² measured it — split evals first, a repeated failure
next — and states the three rules above. A reviewer definition with no
description, or a `name:` that is not its file name, or one the review policy
never names, fails the suite with what to fix.

The person's request, 2026-10-21: 「~/dev/oss/harness2를 참고해서 하네스 설계를
내 프로젝트에 알맞게 최대한 적용시켜줘.」

## Constraints this change inherits

- Lessons already pinned stay pinned: the eval runner's outcome lines that
  `the_eval_runner_judges_every_run_and_keeps_infra_and_unstable_out_of_the_rate`
  reads, and the 8,000-character budget for the always-on text (nothing is
  added to `CLAUDE.md` or `AGENTS.md`).
- Files under `.claude/agents/` are on the `gate-configuration` surface
  (`paused`); this change reads them and does not edit them.
- Lesson 060: what a CLI does is measured, not assumed.

## Systems likely affected

`scripts/run-evals.sh`, `evals/README.md`, `src/tests.rs`,
`docs/sdlc/lessons.md`.

## Open questions

None. The Claude Code behaviour is measured above; the procedure rules are
Harness²'s and are written as advice in a document that loads only when the
evals are being improved.

## Not in scope

- Harness²'s lesson library with cues and confidence: lessons here end in a
  guard that runs without being remembered, which is stronger delivery than a
  library an agent must consult.
- Its fingerprinted completion marker: the review digest and the approval
  ledger already bind a verdict to the bytes it judged.
- Automating "revert when timeouts rose": comparing two measured runs is a
  judgement about a baseline, made by whoever runs the round.
- Its separate proposers for how the agent works and what the domain needs:
  this repository has one domain.
