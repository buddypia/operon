# Intent: the development process is stronger in the middle than at either end

- **Status**: approved
- **Opened**: 2026-08-27

## Problem

Everything this repository knows how to enforce, it enforces while code is being
written. Nothing it knows, it enforces before the work starts or after it ships.

Before: there is no record of why a change was wanted. A change arrives as a
prompt, becomes a diff, and the reasoning is in a chat log nobody can find. The
second time the same question is asked, it gets answered from scratch.

After: the rules that matter are promises in prose. `AGENTS.md` has always said
"before packaging, run the three gates" — and a suite the agent never ran looks
exactly like a suite that passed. The most consequential action in the project,
replacing `/Applications/Operon.app`, has no gate at all: an agent that decides
to swap the installed app can do it, and the user finds out afterwards.

## Who feels it, and when

Every time a change is larger than one file. The author re-derives constraints
that were settled weeks ago, an agent re-learns a convention it was corrected on
twice, and nobody can tell from the repository whether the gates ran.

And every time something goes wrong: the fix lands, the lesson does not, and the
next agent repeats it. Two such incidents are already recorded — one in
`CLAUDE.md`, one in a test's doc comment — in places nothing reads as a ledger.

## Desired outcome

- A change worth a paper trail has one, and it is in git.
- A rule that must hold fails a check rather than asking to be remembered.
- The installed app cannot be replaced without a person saying so.
- A mistake that happened once is caught by something that runs without being
  remembered — a test, or an eval.
- The cost of all of the above is visible as a number, so a mechanism that stops
  earning its keep can be argued out again.

## Constraints this change inherits

- Local-first: no telemetry, no accounts, no cloud calls in the app.
- One developer, one Mac, no remote yet. Anything GitHub-shaped is written for
  later and must degrade honestly until then.
- `CLAUDE.md` and `AGENTS.md` load on every turn, so anything added to them is a
  tax on every task in the repository.
- The existing strengths — 267 CI tests, 11 invariant tests, two subagents, two
  skills, `scripts/harness-metrics.sh` — are not to be rearranged. This adds
  stages; it does not restructure stage 3.

## Systems likely affected

None of the application modules. `src/tests.rs` gains guards.
`scripts/harness-metrics.sh` gains counters. Everything else is new files and the
steering documents.

## Open questions

Answered before the spec was written:

- **GitHub-dependent plays, with no remote?** Write them. They degrade to a
  skipped job with a stated reason, exactly as `.github/workflows/ci.yml`
  already does.
- **An eval suite that calls a paid API?** Yes, with manual runs as the default
  and CI opt-in behind a configured key.
- **`CLAUDE.md` size?** Add only what cannot live elsewhere, and report the byte
  delta.

## Not in scope

- Restructuring `src/app.rs`, which is the largest module and a separate problem.
- Chat-based on-call, hosted security scanning, and DORA metrics — see "What this
  project does not do" in `docs/sdlc/README.md`.
