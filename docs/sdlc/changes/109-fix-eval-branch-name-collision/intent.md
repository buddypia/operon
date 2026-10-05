# Prevent eval branch collision during rapid sequential runs

## Problem

In CI environments, tests like `the_eval_worktree_is_on_a_branch_the_worktree_policy_lets_the_agent_edit` run `run-evals.sh` multiple times rapidly within the same second.
Previously, the eval branch name was computed as `eval/$stamp-$id-$run`, where `$stamp` was only second-precision (`date -u +%Y%m%dT%H%M%SZ`).
Because the first run used `--keep` which intentionally leaves the branch for inspection, the second run attempted to recreate the same branch `eval/$stamp-$id-$run`, failing with `fatal: a branch named '...' already exists` and causing `could not create a worktree at HEAD`.

## Proposed Change

- Include process ID (`$$`) in the eval branch name: `eval/$stamp-$$-$id-$run`.
- This guarantees uniqueness between distinct runs even when invoked in the exact same second.

## Risk and Review

Low risk. Only affects temporary test worktree branch names; does not affect application runtime logic.
