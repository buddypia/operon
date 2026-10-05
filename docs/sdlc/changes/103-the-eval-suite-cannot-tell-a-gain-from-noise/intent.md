# Intent: the eval suite cannot tell a gain from noise

- **Status**: approved
- **Opened**: 2026-10-02

## Problem

The person asked for the practices in "Automating eval design and hillclimbing"
(claude.dev, 2026-09-28) to be applied to this repository as far as they fit.
Quoted request: 「この内容をなるべくこのプロジェクトの状況に合わせて適用して」.

The steering evals already follow the article's first rule — every case is
seeded from a mistake that happened, not from what today's model fails. What
they do not have is the rest of what makes a score trustworthy:

- A run says pass or fail **once** per eval. Agent runs vary, so a steering edit
  that flips one eval cannot be told apart from noise, and nobody can say how
  noisy the suite is.
- A run that died for a reason outside the steering — a timeout, an API error, a
  transcript cut short — is reported as the steering failing.
- Nothing records what the agent did. A failure leaves a verdict and no
  transcript, so the reason has to be guessed.
- The agent under test can read the answer. The check that judges it sits in the
  same checkout the agent works in.
- Nothing says how to improve the steering against the evals without fitting
  the steering to the evals: no held-out cases, no rule against pasting a failed
  case into the instructions, no rule for when to revert.

## Who feels it, and when

Whoever edits `CLAUDE.md`, `AGENTS.md`, `.claude/`, or `REVIEW.md` and wants to
know whether the edit helped. Today they get seven verdicts from one run; the
same tree run again can give different ones.

## Desired outcome

- Running the suite several times gives a pass rate per eval and overall, with
  an interval wide enough to be honest, and a warning when the suite is too easy
  to show a gain.
- Failures caused by the infrastructure are counted apart from failures of the
  steering.
- A check can be run twice on the same tree, and a check that changes its mind
  is reported as a broken check rather than as a steering result.
- The same suite can be run against a weaker and a stronger agent
  configuration, so that a suite whose score does not rise with capability is
  visible.
- Every run leaves its transcript and its check output on disk, with a summary
  that links them.
- The agent cannot read an eval's check in the checkout it works in.
- Evals are split into a set used to improve the steering and a held-out set
  that only scores it, and a written procedure says how one round of
  improvement is accepted or reverted.
- The steering documents never contain an eval's prompt, and a test says so.

## Constraints this change inherits

- Local-first: the suite already needs an agent CLI; nothing new reaches the
  network on its own.
- The runner must still work where it works today (macOS here, Ubuntu in CI),
  with the same default behaviour: one run, exit 1 on any failure.

## Systems likely affected

`scripts/run-evals.sh`, `evals/`, `evals/README.md`, `src/tests.rs`, and the
paragraph on evals in `docs/sdlc/README.md`.

## Open questions

None the repository cannot answer. The split's membership is decided at random
once and recorded, with the seed that drew it, as the article does.

## Not in scope

- A model-graded check. Every check is a shell command; an LLM judge is added
  only when an eval has an open-ended output, and none does yet.
- Writing new evals. Each one waits for its lesson.
- Running the hillclimb loop on the steering itself. This change gives it a
  procedure and the instruments; a round is its own change.
- Keeping CI's run results as a downloadable artifact. That is a workflow edit
  of its own.
