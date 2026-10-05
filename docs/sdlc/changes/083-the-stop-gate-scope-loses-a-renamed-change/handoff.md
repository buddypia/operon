# Handoff: change 083 is reviewed and green, but cannot land without a third review round

- **Route**: bugfix
- **Attempts spent**: 0 of 3 (the closing gate never failed); review rounds 2 of 2
- **Written**: 2026-09-26
- **Status**: resolved — change 084 made a re-verdict caused only by main moving part of the round it re-confirms, and made `git merge --abort` the session's to run; both reviewers re-confirmed round 2 at 7b1464d180e4a44b

## The failure, exactly

`git merge --no-ff fix/stop-gate-scope-misses-renames` from the main checkout
stopped before its commit; the review gate, asked of the index it left:

```
❌ BLOCK — 082-the-stop-gate-scope-loses-a-renamed-change · 判定が古い diff のものです
判定は diff 9554480c0f5e4131 のもの、現在の diff は 845a1223d58256ed。
古い承認は無い承認より悪いので、再レビューしてから commit してください。
```

The main checkout was left with that merge staged (`MERGE_HEAD` present).
`git merge --abort` is not on the trunk allowlist, so this session could not
undo it.

## What was tried, in order

1. Landed 080 and 081 the same way without trouble.
2. Wrote this change as 082, reviewed in two rounds (round 1 do-not-approve
   with one Important, taken; round 2 both approve-with-nits) at digest
   9554480c0f5e4131.
3. Merged into main. Meanwhile another session had landed its own change 082
   (`082-every-artifact-waits-for-a-person-however-small`, approval ledger),
   which also edits `.claude/hooks/gate-stop.sh` and `src/tests.rs`. The
   landing diff moved, so the gate refused the merge commit.
4. In the worktree: merged main into the branch (clean), renumbered this
   change 083, suite green on the result (603 passed, 0 failed, 6 ignored;
   fmt and clippy silent). Landing digest is now da149b329960acbf.

## What was ruled out

| Hypothesis | Ruled out by |
|---|---|
| The two changes conflict | auto-merge clean; the other 082 only reads `in_play` for its approvals check; the full suite passes on the merge |
| The verdicts can be reused | the gate hashes the landing diff, and it moved (context lines and the renumbering) |

## Narrowest remaining hypothesis

None needed: the code is done. What remains is a review of the rebased diff.

## What a person has to decide

Whether to allow a third review round past the bugfix ceiling of 2 — a
re-verdict on da149b329960acbf with no logic change, only the merge of main
and the 082→083 renumbering — or to land it some other way.

## State of the tree

- Branch `fix/stop-gate-scope-misses-renames`, worktree
  `.worktrees/fix/stop-gate-scope-misses-renames`, everything committed
  (b22511d).
- Gates pass as the tree stands.
- **The main checkout is dirty**: a stalled merge of the pre-renumber branch.
  Undo with `git -C /Users/someone/dev/buddypia/operon merge --abort`.
