# Plan: subagent worktrees outlive the change

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-16
- **Status**: approved

## Root cause

Antigravity CLI gives each subagent a linked worktree of the repository under
`~/.gemini/antigravity-cli/worktrees/<conversation>/` and a branch named after
the worktree, and does not remove them when the subagent returns. Operon's only
removal path, `ops.mjs cleanup-worktree`, removes the one worktree it is handed;
nothing else in the repository looks at worktrees agy created, so each verifier
run leaves a worktree and a branch behind for good.

## The fix

`cleanup-worktree` runs on every landing (AGENTS.md, feature-pilot §4), under
every CLI. After it removes its own worktree it sweeps the linked worktrees
under `~/.gemini/antigravity-cli/worktrees/` and removes each one, with its
branch, only when all of these hold:

- the tree is clean (`git status --porcelain` prints nothing — untracked files
  count, ignored build output does not);
- its `HEAD` is an ancestor of the base branch, so no commit is lost;
- the worktree directory was last modified more than six hours ago, so a
  subagent of a session still running is not pulled out from under it.

Anything else is left alone. Removal is `git worktree remove` without
`--force`, so git itself refuses a tree that changed between check and removal.
Each removal is reported as a line in the command's `warnings`. A failure in the
sweep never fails the cleanup.

Worktrees elsewhere (`.worktrees/`, Claude Code's own) are not touched: their
owners already remove them.

## Files that change

| File | Change |
|---|---|
| `.claude/scripts/create-pr/ops.mjs` | `sweepAgentWorktrees`, called from `cmdCleanupWorktree` |
| `src/tests.rs` | `cleanup_sweeps_leftover_subagent_worktrees_and_nothing_else` |
| `docs/sdlc/changes/296-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. The test, watched failing on the unedited `ops.mjs`.
2. The sweep and its call.
3. `mutations.py`: dropping the call, the root check, the clean check, the
   ancestry check, or the age check each turns the test red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A live subagent's worktree is removed | an agy subagent loses its checkout mid-run | six-hour age floor and clean-tree check; the test's fresh worktree survives |
| Work is lost | a commit or edit in a removed tree | ancestry and clean checks, `worktree remove` without `--force`; the test's ahead and dirty worktrees survive |
| A worktree outside agy's root is removed | a person's own worktree disappears | root-prefix check; the test's sibling worktree survives |

## Proof of completion

- `cargo test --locked cleanup_sweeps_leftover_subagent_worktrees_and_nothing_else` passes; it failed before the fix.
- `cargo test --locked cleanup_removes_a_merged_worktree_in_a_repository_with_no_remote` still passes.
- `python3 docs/sdlc/changes/296-subagent-worktrees-outlive-the-change/mutations.py` — every mutation caught.
- Landing this change removes the two worktrees change 290 left.
