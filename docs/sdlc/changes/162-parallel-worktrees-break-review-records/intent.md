# Intent: work landing on main from another worktree voids a finished review

- **Status**: draft
- **Opened**: 2026-10-17

## Problem

The person's words, after change 158 could not land: 「AIは並列で複数Worktreeで
作業しているから問題があれば恒久対応して。判断は状況をみて進めて」.

Change 158 was reviewed and approved, its CI was green, and the merge into main
was still refused: two other changes (159, 160) landed on main while 158 was in
review, 158 took them in as the pipeline asks, and the review verdict stopped
matching. Nothing in 158 had changed — the other changes added tests earlier in
the same test file, so 158's additions sat at different line numbers. The
verdict could not be brought up to date either: a commit that only updates the
review record is refused, so the branch had no way to land.

This is not the first time. The review template already counts "a
recombination confirm (another session landed on main first)" as a review
round, from change 135. With several sessions working in parallel worktrees,
whichever change lands second pays for the first one.

A second, smaller collision happened on the way: the next change number was
chosen from the changes already on main, and another session's in-flight
branch already held it.

## Who feels it, and when

Every session whose change is in review while another session lands on main —
which, with parallel worktrees, is most of them. It shows up at the very last
step, after CI, as a refusal to land a change that was approved.

## Desired outcome

- A change that was reviewed lands after taking in unrelated work from main,
  without another review, as long as nothing it touches was changed by that
  work.
- When the work from main does touch the reviewed lines, the change is sent back
  for review as today, and the pipeline says how to record the new verdict so
  that the branch can still land.
- Reviews recorded before this change keep counting, so sessions that are
  mid-review in other worktrees are not sent back by it.
- A new change number is not one another session's branch or worktree already
  uses.

## Constraints this change inherits

- The review gate fails closed, and a verdict stays bound to the code it judged:
  nothing here may let unreviewed code land.
- The gate's own tests pin its digest path line by line; they move with it.
- Sessions on other worktrees run the gate from main's copy at their next
  commit, so the change takes effect for them the moment it lands.

## Systems likely affected

- The review gate script and its tests.
- The pipeline's instructions for opening a change and for taking in main.

## Not in scope

- Letting a commit that only updates the review record land on its own.
- Merges of main that bring in no change directory (a docs-only or trivial
  commit on main that overlaps a reviewed hunk); they keep today's behaviour.
