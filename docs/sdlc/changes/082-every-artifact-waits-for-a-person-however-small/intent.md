# Intent: every artifact an agent writes waits for a person, however small

- **Status**: approved
- **Opened**: 2026-09-25

## Problem

Change 035 took the diff away from a person: a reviewer that is not the author
judges it, bound to the digest it read. Everything upstream of the diff still
stops for a person — the intent is written by interviewing them, the plan is
approved by them, the screen is approved by them with "no automatic pass", and
every change to the harness's own configuration is paused until they answer. A
spacing tweak and a new screen wait at the same gate; a hook that gains a check
and a hook that loses one wait at the same gate.

The consequence is not only waiting. A gate that asks about everything teaches
the person to answer without reading, and then the answer carries no
information on exactly the decisions it exists for. The person's attention is
the scarce input, and today it is spread evenly across decisions that differ by
orders of magnitude in what a wrong answer costs.

The second half: an agent's own approval is not yet something anybody has a
reason to trust. When a defect escapes an automated check today, a lesson and a
guard are written for the defect — but nothing records *which approval let it
through*, so there is no way to say whether automated approval of a given kind
of artifact is getting better, worse, or has never been tested at all.

## Who feels it, and when

The owner, at every stage boundary of every change: asked to approve a plan they
would have approved unread, a screen change that moves a label, a hook edit that
only adds a refusal. And, the other way round, at the rare boundary that
matters — a redesigned screen, a store migration, a change of direction — asked
in the same voice as for all the others.

## Desired outcome

- A person is asked only when the decision is significant: it changes what the
  product is or which way it is going, it cannot be undone cheaply, it removes a
  protection, or it changes what a user sees in a way they would notice as a
  different product. The owner named large UI changes, large store rework, and
  pivots as examples — **the category is the judgement, not the list**, and the
  judgement is made from the change's context, not by matching file names alone.
- Everything else is approved by an evaluation that is not the author, that
  records what it judged and why, and that fails closed: no evaluation is no
  approval.
- Every automatic approval leaves a record a person can sample afterwards, and
  the session reports how many there were in one line, not each one.
- When a person finds a defect in something that was approved automatically, it
  is an **escape**: a lesson with a guard watched failing, as today, **and** that
  kind of approval goes back to a person until it has earned automatic approval
  again by a run of clean approvals.
- Whether automatic approval is trustworthy becomes a number that can be read:
  approvals, escapes, and which kinds are currently demoted.

## Constraints this change inherits

- The agent that wrote an artifact does not approve it (change 035).
- A verdict is about a specific artifact; a stale one reads like a fresh one.
- Fail closed. Silence is not approval.
- Hooks and checkers are bash and flat files parsed without a YAML library.
  `docs/sdlc/routes.yaml` rows stay exactly five values (lesson 039).
- `steering_bytes` is past `propose` and `always_loaded_bytes` at `warn`; this
  change may not grow what loads on every turn.
- Local-first: the record of approvals stays in the repository.

## Systems likely affected

The pipeline documents (`docs/sdlc/README.md`, `docs/sdlc/risk.yaml`, the
`sdlc` skill and its screen-approval reference), `REVIEW.md`, the change-state
template, a new ledger beside `docs/sdlc/lessons.md`, `scripts/` for a checker,
and `src/tests.rs` for the guards.

## Open questions

- Which decisions stay with a person? **Answered by the owner: not a fixed list
  — understand the context.** Large UI changes, large store rework, and pivots
  are examples of the category.
- Where is the line between a large and a small UI change? **Answered by the
  owner: by structure** — a new screen or modal, a re-arranged layout, or a
  feature removed or moved is large; wording, spacing, and colour or state within
  existing palette roles is small, and the small kind is judged against its
  agreed layout by an evaluator.
- What happens on an escape? **Answered by the owner: lesson and guard, and that
  kind of approval is demoted to a person until N consecutive clean automatic
  approvals.**
- How does the owner see it? **Answered by the owner: a ledger and after-the-fact
  sampling; one line per session, no per-approval notice.**

## Not in scope

- Replacing change 035's diff review. It already is the automatic approval for
  code; this change extends the same principle upstream and adds the trust loop.
- Settling change 035's own blocked state or change 038's open questions.
- Judging whether a design is *good*. An evaluator can check an artifact against
  its intent, its policies, and its agreed layout; it cannot decide taste, and
  the significance rule exists so that taste goes to a person.
