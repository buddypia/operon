# Plan: landing measured against an `origin` that trails `main`

- **Spec**: `./spec.md`

## Steps

1. Close change 112 in its own commit: `status: done`.
2. Write both tests with an `origin` fixture, and watch them fail on the
   current scripts.
3. Fix `assessAheadCommits` in `ops.mjs` and the base choice in
   `worktree-new.mjs`.
4. Correct `AGENTS.md`.
5. `make q.check`, review, land, `cleanup-worktree` without `--force`.

## Proof of completion

- Both new tests red before step 3 and green after.
- `make q.check` passes.
- This change's own worktree is removed by `cleanup-worktree` without
  `--force` after landing.

## Departures from the plan
