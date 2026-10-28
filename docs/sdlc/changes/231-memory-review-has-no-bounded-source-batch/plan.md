# Plan: Make cross-session memory review inspectable

- **Spec**: `./spec.md`
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| scripts/memory-review.py | Bounded explicit committed-source manifest using the receipt reader |
| src/tests.rs | Scope, immutability, read-only behavior, invalid IDs and byte budgets |
| .claude/skills/sdlc/SKILL.md | Discoverable conditional maintenance reference |
| .claude/skills/sdlc/references/memory-review.md | Dedicated review budget, evidence/prevalence, untrusted input and proposal acceptance procedure |
| This change directory | Source coverage, approvals, real batch result, verification and mutations |

## Order of work

1. Commit intent, evaluate/commit spec and plan before implementation.
2. Watch two named tests fail without the tool; implement the bounded reader.
3. Add the conditional guide; run named tests, document paths, q.fast and bands.
4. Independently review explicit completed changes227–230 with the guide, preserving
   already-fixed patterns and reporting only selected-batch prevalence.
5. Mutate scope, snapshot, total budget and review-only claims; get independent
   source/implementation reviews, pass branch CI, land and clean all owned trees.
6. Audit all three original folders against current main before completing goal.

## Risks

| Risk | What catches it |
|---|---|
| Broad/private input discovery | Explicit canonical IDs, fixed file allowlist, no default and unrelated sentinel fixture |
| Untrusted notes become executable instructions | Read-only tool; guide refuses embedded instructions and routes any accepted proposal through Git review |
| Dirty/current data mixed into a historical batch | Immutable revision and dirty worktree fixture |
| Context grows without bounds | Eight IDs, actual-byte aggregate and oversize tests |
| Old evidence promoted after base changed | Guide requires fresh target hashes/review, existing versioned worktree gates |
| Sample counts sound project-wide | Actual independent batch must name numerator, denominator and selected scope |

## Proof of completion

Named tests pass after observed red runs, q.fast/bands/path checks exit 0,
mutations caught, source coverage approved, real batch report cites evidence,
independent reviewers approve, CI green and main/remote match after cleanup.

## Departures from the plan

Reuse change229's repaired target cache for serialized local builds, and clean it
when this final change no longer needs it. New worktree231 was also created through
the fixed seeder: active native caches0, preserved configuration copies2.

Extract canonical change-name validation and immutable revision resolution into
small shared functions in scripts/change-receipt.py rather than duplicating those
contracts in the new tool. Existing receipt tests must remain green. Dynamic
imports disable bytecode writes so the read-only tool does not create __pycache__.
