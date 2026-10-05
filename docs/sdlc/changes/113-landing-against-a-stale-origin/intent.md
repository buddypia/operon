# Intent: landing measured against an `origin` that trails `main`

- **Status**: approved
- **Opened**: 2026-10-04

## Problem

Landing here is local: `git merge --no-ff` on `main`, then
`ops.mjs cleanup-worktree`. Since 2026-10-03 the repository has an `origin`,
pushed by hand, so `origin/main` trails `main` by every landing not yet pushed.
Two worktree tools still take `origin/main` as where work lands:

- `cleanup-worktree` refused the worktree of change 112 after it landed — "6
  commit(s) not provably on origin" — although all six were on `main`. The
  only ways past were `--force` or raw git that the guards refuse.
- `make wt.new`, with `main` ahead of `origin/main`, reported `main` up to date
  and created the next worktree from `origin/main`: without the landings not yet
  pushed, change 112 among them.

`AGENTS.md` still says this repository has no remote.

## Who feels it, and when

Every session that lands a change and then pushes later, or not at all: its
cleanup is refused, and the next worktree starts from old code without a
warning.

## Desired outcome

A worktree whose commits are all on local `main` is removed by
`cleanup-worktree` without `--force`, and a new worktree starts from whichever
of `main` and `origin/main` contains the other. Unmerged work is still refused,
and the root documents describe the repository as it is.

## Constraints this change inherits

- No push and no pull request from a session; `origin` is a person's to update.
- Nothing that is on no local branch is ever removed without `--force`.

## Systems likely affected

`.claude/scripts/create-pr/ops.mjs`, `.claude/scripts/worktree-new.mjs`,
`AGENTS.md`, `src/tests.rs`. No application module.

## Open questions

None.

## Not in scope

Pushing `main`, and when to push it: a person's decision.
