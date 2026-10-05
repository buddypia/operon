# Intent: quality holds only as long as somebody reads the diff

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

Everything this repository automates ends at the same place: a person reading a
diff. Six routes, three reviewer definitions, a policy document listing what
counts as Important — and the step that decides whether any of it happened is a
person finding the time. Two of the six routes require it outright, and the list
of surfaces where it is mandatory is the list of surfaces where a mistake is
unrecoverable.

That is a single point of failure with no signal when it fails. A change that was
never reviewed and a change that was reviewed and approved leave the repository in
identical states: same commit, same green suite, nothing recording which happened.
The gates can say the tests passed; nothing can say the diff was read.

The second half is that when a review does happen, its output is shaped for
someone with the time to read it — passes, findings, arguments — which is the
form least likely to be read by the person the whole mechanism is waiting on.

## Who feels it, and when

At the end of every change on the `feature` and `security` routes, and on any
change touching the store, `unsafe`, the `PATH` setup, the bundle swap, the gate
configuration, or a new dependency. It bites hardest in exactly the case the rule
exists for: a long session, a correct-looking diff, and nobody with fresh
attention left to read it.

## Desired outcome

Merging is decided by something that runs, not by someone being available. What a
test can decide is decided by a test; what needs judgement is judged by a reviewer
that is not the author, bound to the exact diff it judged, so a verdict cannot
outlive the code it was about. A change with no fresh verdict does not merge.

And the answer is legible in one look: whether it shipped, what refused it, and
what was checked to say so — before any finding is read.

## Constraints this change inherits

- The rule that survives: the agent that wrote a change does not approve it.
- A verdict must be about a specific diff. A stale approval is worse than none,
  because it reads exactly like a fresh one.
- Whatever replaces the person must fail closed: no verdict is a refusal.
- `steering_bytes` is at its `propose` threshold and `REVIEW.md` is inside the
  counted set. This change may not grow it.
- Hooks are bash. A mechanism that needs a runtime to explain itself is one this
  repository has already decided not to carry.

## Systems likely affected

`REVIEW.md` states the policy. `docs/sdlc/routes.yaml` carries the column that
says a person must look. `.claude/hooks/gate-commit.sh` is the boundary that can
refuse. The three definitions in `.claude/agents/` are the judgement that is left
once the deterministic half is subtracted.

## Open questions

- How far does this go? **Answered by the owner: repository-wide, not just for
  the change in flight.**
- What does the result look like at a glance? **Answered by the owner: a verdict
  line, then a band of indicators, then findings only when there are Important
  ones.**

## Not in scope

- Judging design quality. A gate can establish that a review happened on this
  diff and that it found nothing Important. Whether the design is *good* is not a
  thing this or any check decides, and claiming otherwise would be the failure
  this repository names first.
- The three reviewer definitions' contents. What they look for is already
  written; this change is about when they run and what happens to their answer.
- `/code-review` and the GitHub workflow, beyond reading the same policy.
