# Merged worktrees are left behind

## The report

> なんでAIがworktreeを放置しているかわからない。コミットやPRなど終わったらしっかり片付けるようにして。

On 2026-09-23 the repository carried six linked worktrees and six stray
branches. Three of the worktrees were fully merged into `main` and had nothing
in them but the two generated `GIT_DIR` pointer files. Nobody had removed them.

## Root cause

Three causes, and all three have to be fixed. Any one of them left in place
still leaves the worktree behind.

1. **The landing procedure stops at the merge.** `AGENTS.md` says work lands
   from a worktree and "`main` takes it with `git merge --no-ff`". It stops
   there. The standard flow's `ship-worktree` removes the worktree after it
   merges, but it needs `gh` and an SSH `origin`. `AGENTS.md` already says this
   repository has neither, so every landing here is the manual merge, and the
   manual merge has no removal step after it.
2. **The removal command cannot succeed here.**
   `ops.mjs cleanup-worktree` checks safety against `origin/main` and then runs
   `git fetch origin main`. This repository has no remote, so both fail every
   time:
   - Without `--force` it refuses before it deletes anything
     (`unmerged-commit check failed`).
   - With `--force` it removes the worktree and then exits 1 with `fetch_failed`.

   Measured on the three merged worktrees above. An agent that sees the only
   sanctioned removal path fail stops using it. `destructive-git-guard` blocks
   the raw `git worktree remove` and `rm -rf .worktrees`, so no other path is
   left.
3. **Nothing notices a merged worktree that still exists.**
   `worktree-shipping-guard` blocks Stop only on uncommitted changes or unmerged
   commits. A worktree that has been merged but not removed has neither, so it
   is skipped as finished. A merged, still-present worktree is exactly the state
   cause 1 produces, and this guard is the one check that could have caught it.

`worktree-session-owner-guard` then makes the leftovers permanent: once the
session that created a worktree ends, every other session is refused access to
it. That rule is deliberate and it stays. What this change has to ensure is that
the session that owns a worktree removes it before it ends.

## The fix

- `.claude/scripts/create-pr/ops.mjs`
  - Add `baseRef()`. It returns `origin/<base>` when that ref resolves, and
    otherwise the local `<base>`. `assessCleanupSafety` measures "ahead" and
    "tree matches base" against `baseRef()`. The `origin/<branch>` push probe
    runs only when an origin exists.
  - `syncMainAfterCleanup` returns `null` with a `no_remote` status when there
    is no `origin`. Local `main` is the authority, the same answer
    `worktree-new.mjs` already gives ("no 'origin' remote — skipped fetch").
- `.cli/hooks/worktree-shipping-guard.mjs`
  - A worktree this session owns is now also a candidate when it is clean, has
    no commits ahead of base, **and its tip is a non-first parent of a merge
    commit on base**. That last condition is what a `--no-ff` landing leaves
    behind. "Ahead is 0" alone would not do: a worktree created a minute ago
    and not yet committed to is also 0 ahead, and a session that stops there to
    ask a question must not be told to delete it. The block names
    `cleanup-worktree`, and the 5-minute attempt marker applies as it does
    elsewhere. A fast-forward landing is not detected. This repository does not
    land that way.
  - A merged worktree the session does not own goes into the existing
    `not_owned` stderr notice, so a person can see it.
- `AGENTS.md` — the landing sentence gains its last step: remove the worktree
  with `node .claude/scripts/create-pr/ops.mjs cleanup-worktree --worktree
  <path>` from the main checkout, in the same session that merged it.

## What was considered and rejected

- **Loosening `worktree-session-owner-guard` so any session may clean up.** The
  guard exists because a mistyped path destroyed another session's live work.
  The owner removing its own worktree needs no loosening.
- **Auto-removing merged worktrees inside the Stop hook.** A hook that deletes
  directories has side effects nobody asked for. The hook states what is left
  and the session removes it.
- **Adding an `origin`.** That is out of scope. The repository is deliberately
  local.

## The guards

Three tests in `src/tests.rs`, each watched failing:

- `cleanup_removes_a_merged_worktree_in_a_repository_with_no_remote` builds a
  repository with no remote and a merged linked worktree, runs `node ops.mjs
  cleanup-worktree`, and requires exit 0, the directory gone and the branch
  deleted. It also requires that an unmerged worktree is refused.
- `the_stop_guard_names_a_merged_worktree_the_session_still_owns` drives the
  exported `evaluate` with injected probes, and requires a block that names
  `cleanup-worktree`.
- `the_landing_procedure_ends_by_removing_the_worktree` requires that the
  `AGENTS.md` landing sentence names `cleanup-worktree`.

## Departures from the plan

- **A fourth cause, found while cleaning up.** `ops.mjs` runs every git command
  with a 30-second timeout. A worktree that holds a Rust `target/` (94,000 files
  in the case measured) takes longer than that to remove, so `worktree remove`
  was killed partway through and left a half-deleted directory that git still
  listed. The removal now gets 600 seconds.
- **Review round 1, Important (subprocess-safety-reviewer).** The no-remote
  success path turned a mistyped path into a reported cleanup: nothing existed,
  so nothing was removed, and the result said `worktree_cleaned: true`.
  `cleanup-worktree` now refuses any path that is not a linked worktree in
  `git worktree list`, compared after resolving symlinks. That also refuses the
  main checkout, which is clean and 0 ahead of itself, and which the no-remote
  path would otherwise have accepted (N3).
- **The sync decision uses `git remote get-url origin`**, not whether
  `origin/main` resolves. A remote that has not been fetched is still a remote
  (N2).
- **One env-scrub list.** `forget_inherited_repository` in `src/git.rs` is now
  shared by `git_command` and the test driver `node_in`, and
  `no_git_command_in_the_crate_can_inherit_the_session_repository` follows the
  chain (rust-reviewer nit 1).
- **`relativizePath` in the Stop guard required the separator.** A sibling
  directory named `<project>-worktrees` shares the project directory's prefix, so
  it was printed as `worktrees/...` — a `--worktree` argument that removes
  nothing. The Stop test now pins the exact command it prints.
- Moved onto main f46635f after 049 landed, by cherry-picking the plan commit
  and re-applying the diff. The first worktree was cut from 27f72f0.
