# Plan: Eval runs must have a deadline on stock macOS

- **Status**: approved
- **Referent**: previous change's source-audit.md and the first input folder's bounded-loop recommendation.

## Problem and cause

scripts/run-evals.sh leaves bound empty when timeout/gtimeout is absent, then
launches the paid agent without a deadline. Its help explicitly describes that
fallback. scripts/run-bounded.sh already implements a portable Perl process-group
deadline, but the eval runner never calls it. This is the unresolved first-folder
gap recorded in change 227; no second or third input folder has been read.

## Files that change

| File | Change |
|---|---|
| scripts/run-evals.sh | Validate EVAL_TIMEOUT; always invoke existing run-bounded helper; truthful condition line |
| src/tests.rs | Reproduce on PATH without timeout/gtimeout, child cancellation and infra classification; invalid-limit tests; copy helper into existing fixtures |
| evals/README.md | Document portable default deadline and positive integer limit |
| docs/sdlc/changes/228-eval-runs-without-a-deadline | Evidence, mutations and resume cursor |

## Order of work

1. Evaluate and commit this plan; bugfix route skips intent and spec.
2. Watch the no-GNU deadline test fail under an outer test safety bound and invalid-limit test fail before changing runner code.
3. Reuse bash scripts/run-bounded.sh, keep timeout exit 124 classified infra, validate a positive integer no greater than 2147483647 (portable alarm range).
4. Update existing eval fixtures to copy the helper and expect a deadline on every platform; run the named eval runner tests, q.fast and bands.
5. Mutate the call, invalid-limit guard and recorded timeout claim; obtain independent review and pass branch CI before landing.
6. This closes the remaining first-folder application gap; then read hanakoxbt_article only, before kyr0stack_video.

## Risks

| Risk | Guard |
|---|---|
| Deadline stops parent but leaves agent children | Isolated agent writes child PID; after infra return the child has stopped |
| Zero, negative or oversized limit cancels the alarm | Invalid limits fail before agent invocation |
| Test proves only installed GNU timeout works | Test PATH includes required utilities but excludes timeout and gtimeout |
| Timeout falsely counts as steering fail or pass | results.tsv says infra; summary excludes it from judged rate |
| Existing fixtures miss helper | Copy helper into fixture repository and run existing eval-runner subset |

## Proof of completion

New tests fail before runner fix, all the_eval_runner_ tests pass after; q.fast
and check-bands exit 0; claim mutations caught; both reviewers approve; CI green.
No app release is needed. No real model, key or paid agent is used by these tests.

## Departures from the plan

This worktree branches from change 227 while its CI runs. Local builds are
serialized using that worktree's already repaired target cache; its cleanup waits
until this change no longer uses the cache. This avoids reusing the stale seeded
CMake/OpenSSL cache and does not touch the main checkout's build output.
