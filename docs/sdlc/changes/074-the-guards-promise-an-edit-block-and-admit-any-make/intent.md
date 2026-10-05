# Intent: the guards promise an edit block Operon lacks, and admit any make on main

- **Status**: approved
- **Opened**: 2026-09-24

## Problem

Change 073 updated the worktree guards. Three issues remained in the harness scripts:

- The `[test-lock]` notice after a `fix/*` commit tells the agent that editing a
  locked regression test is denied. Operon has no edit-time guard for that
  (`coverage-threshold-guard` is not in the bundle); only the commit is refused.
  The notice promises a block that does not exist.
- `worktree-new.mjs` picks its default base from `origin/HEAD`, not from the
  create-pr `base_branch` the worktree will ship onto.

Also, 073's review carried a nit (rust-reviewer/1): on `main`, the make pattern in
`trunk_bash_allowlist` fences off only `-f`, so `make wt.new --eval=...` or
`make q.check SHELL=...` runs any command on the trunk checkout.

## Who feels it, and when

An agent on a `fix/*` branch that reads the notice and relies on the edit block
it promises, and every session on `main`: the allowlist exists to keep a
session from building there, and make is a way around it.

## Desired outcome

- The harness scripts are updated, with clean separation between policy and implementation.
- On `main`, make runs only the targets 073 allowed (`wt.new` and `q.check`),
  with only the arguments those targets take.
- The suite goes red if the make pattern loosens again.

## Constraints this change inherits

- Standardized guard files maintain consistent policy evaluation across CLIs.
- Local-first: the guards run locally under node, with no network.

## Systems likely affected

Three guard files (`.claude/scripts/worktree-new.mjs`,
`.cli/hooks/worktree-owner-tracker.mjs`, `.cli/lib/test-lock.mjs`), the receipt,
`.claude/config/worktree-policy.json`, and the 073 test in `src/tests.rs`.

## Open questions

None left. The user approved the whole task, including the landing merge on
`main`, before it started.

## Not in scope

- Nits 2 to 5 carried from 073's review, apart from the allow-side coverage the
  new make asserts add.
- Env-assignment prefixes such as `MAKEFLAGS=... make` or `MAKEFILES=... make`. The guard strips
  assignments before matching, so no pattern can see them (see `spec.md`).
