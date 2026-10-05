---
name: root-cause
description: Find the cause of a failure before changing any code — a failing test, a panic, a wrong result, a gate that will not pass, or behaviour that makes no sense. Produces a named cause and a test that fails for that reason. Use for the bugfix route in docs/sdlc/routes.yaml and any time the same fix has already been attempted once. Do not use to plan a feature or to review a diff that already works.
---

# Find the cause before changing the code

The loop here is one `cargo test --locked` invocation and one `#[test]` file, and
the suite has been green through every failure in `docs/sdlc/lessons.md`.

## The Iron Law

**No fix before the cause is named.** Not "probably the borrow in the loop" —
named, meaning you can say which line produces the wrong value and why, and point
at the observation that proves it.

The reason this is a law and not advice: every entry in `docs/sdlc/lessons.md`
was found *after* a green suite. A guess that makes the symptom go away also
makes the evidence go away, and the next reader inherits a repository that
passes its tests and does the wrong thing.

Skip it for nothing. "Simple bug" is not an exception — a simple bug has a
precise cause and finding it takes a minute. Time pressure is not an exception:
three guesses cost more than one investigation, which is the observation behind
the attempt ceilings in `docs/sdlc/routes.yaml`.

## Phase 0 — a loop that answers in seconds

Before reading code, get a deterministic pass/fail. In descending order of worth:
a failing `#[test]` in `src/tests.rs`, run by name; the same with a fixture built
from real data — a real transcript, a real store — rather than JSONL you typed
(lessons 001 and 007 are both "the fixture only held cases somebody had already
thought of"); or `cargo run` driven by hand with a written record of what was
observed, which is slowest and sometimes the only option for a drawing bug.

If no loop can be built, say "not reproducible" out loud and go to phase 1 for
evidence instead of guessing. Do not proceed to a fix from here.

## Phase 1 — investigate

- **Read the whole failure.** The assertion message, the panic location, the
  clippy span. Not the first line of it.
- **Reproduce, and say how reliably.** Every run, or one in ten? A flake and a
  bug need different work.
- **Check what changed.** `git diff`, `git log -3`, and whether the behaviour
  ever worked. `git stash` is a legitimate diagnostic.
- **Find the code that decides.** The branch, the `match` arm, or the comparison
  that produces the wrong answer, read with enough context to know what it was
  written to do.
- **State the cause in one sentence.** If it needs "and maybe", the phase is not
  done.

Two traps this repository has fallen into, both recorded. **A green guard that
checks nothing** (lessons 004, 009): a passing test over the failing area is not
evidence the area is fine — mutate what it claims to guard and watch it fail. **A
check whose caller never calls it** (lessons 005, 008): test the whole decision,
not the helper the decision was supposed to consult.

## Phase 2 — the test comes first

Write the test that fails *for the stated cause*, and watch it fail. Read the
failure message and confirm it says what you predicted; a test that fails for a
different reason than the one you named has not reproduced the bug.

The test goes in `src/tests.rs` — the only place with tests — and its name says
the behaviour, not the mechanism, in the style of the names already there.

## Phase 3 — fix, without touching the test

Smallest change that makes the named cause stop being true. Nothing else in the
diff. `.claude/hooks/gate-commit.sh` refuses a commit that lost a test or gained
an `#[ignore]`, which is the shape "make the test pass" takes when it goes wrong
here — but the hook is the floor, not the standard.

Then run the three gates in `CLAUDE.md`, and re-read the diff for anything that
went in while you were looking elsewhere.

## Phase 4 — close it

- Was the cause invisible in a way that could recur? Then it is a lesson:
  `docs/sdlc/lessons.md`, with the **Guard** column filled in. An entry with no
  guard is unfinished.
- **Verify the guard by mutation.** Delete or invert the code it guards and watch
  the guard fail. This is the step that separates entries 001–006 from 008 and
  009, and it is the reason those two caught defects that review did not.
- If the guard can only be a prompt, add an eval under `evals/` and say so.
- Did an evaluator approve the artifact the defect came from? Then it is an
  escape too — `.claude/skills/sdlc/references/approval.md` says how to record it,
  and recording it is what takes that kind of approval back to a person.

## When to stop instead

The route's attempt ceiling in `docs/sdlc/routes.yaml` — three for `bugfix`, two
for `security` and `refactor`. On reaching it, stop and write
`handoff.md` into the change directory from `docs/sdlc/templates/handoff.md`. The
handoff's **What was ruled out** table is the point: it is what makes the next
attempt cheaper than this one, and it is the section that gets left empty.

Stop early, before the ceiling, when the answer is outside the repository: a
question about intent, a decision about a trade-off, or evidence that only a
person with the failing machine can gather.

## Report

The named cause, in one sentence. The test that reproduces it and the message it
printed. What changed. Which guard now catches it, and whether that guard was
watched failing. If you stopped, the handoff path and the one thing a person has
to decide.
