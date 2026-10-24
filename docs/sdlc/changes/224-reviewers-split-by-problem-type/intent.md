# Intent: a diff on the store is reviewed twice by agents that read the same rules

- **Status**: draft
- **Opened**: 2026-10-25

## Problem

Review is split by problem type: one reviewer for Rust craft, one for
subprocesses and gates, one for durability. The durability reviewer's own text
says the durability skill states the guarantees and it adds only an order to
check in, so the domain knowledge already lives in a skill and the agent is a
second wrapper around it.

The records say the split costs more than it buys. Of 173 recorded reviews,
durability ran 29 times and 28 of them beside another reviewer; it raised one
Important, in change 067, and that one was raised first by the subprocess
reviewer and taken up by durability only when it was asked again. In a blind
replay of three past changes (`eval.md`), the Rust reviewer reading the
durability skill found as many of the recorded durability findings as the
durability reviewer did. Every extra reviewer is another context window, another
lossy summary back to the session, another verdict line to keep fresh, and
another definition the gate, the CI prompt, and the docs must name in step.

This is the pattern Anthropic's "When to fold subagents into your main agent
loop" describes: specialists divided by problem type and nearly always called
together are merge candidates, the procedure belongs in a skill that loads when
needed, and a subagent stays only where an eval shows the folded version is
worse.

## Who feels it, and when

The session landing any change on the store, the bundle swap, or a `full`
route: it spawns and waits for a third reviewer, reconciles three reports, and
keeps three verdicts fresh across rounds. The person pays the tokens and the
wait.

## Desired outcome

A change on the store or the bundle swap is still refused without a review that
checked it against the durability guarantees, from one reviewer fewer. The
durability procedure is read from the skill, where the session itself also
reads it. The eval used to decide this stays in the repository, so the next fold
(Rust and subprocess reviewers ran together 89 times) can be measured the same
way rather than argued.

## Constraints this change inherits

- `refs/heads/main:scripts/check-review.sh` judges this branch's commits, so the
  branch may not need a reviewer main's judge cannot find (lesson 039).
- `gate-configuration` is `paused`: removing a reviewer is a relaxation, which
  `.claude/skills/sdlc/references/approval.md` gives to a person.

## Systems likely affected

`.claude/agents/`, `.claude/skills/durability-invariants/SKILL.md`,
`scripts/check-review.sh`, `REVIEW.md`, `docs/sdlc/routes.yaml` comments,
`.github/workflows/claude-review.yml`, the gate fixtures in `src/tests.rs`.

## Open questions

- Does the person accept the relaxation? Answered by the person before editing.

## Not in scope

- Folding the subprocess reviewer. It raised Importants in ten changes; it is
  the next candidate and needs its own measured fold.
- Rewriting closed change records that name `durability-reviewer`.
