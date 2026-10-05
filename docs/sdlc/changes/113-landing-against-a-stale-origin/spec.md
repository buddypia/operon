# Spec: landing measured against an `origin` that trails `main`

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. With an `origin` whose `main` trails local `main`, `cleanup-worktree`
   removes a worktree and its branch once every commit is on local `main`, and
   still refuses one with a commit that is on neither —
   `cleanup_removes_a_worktree_landed_on_main_while_origin_trails`.
2. With local `main` ahead of `origin/main`, `make wt.new` bases the new
   worktree on local `main`; with `main` behind and not fast-forwardable from
   where it runs, it still bases it on `origin/main` —
   `a_new_worktree_starts_from_main_when_origin_trails_it`.
3. `AGENTS.md` no longer says the repository has no remote, and says that
   `origin` trails `main` until a person pushes.

## Behaviour

Nothing in the application changes. A session that lands and cleans up sees
`cleanup-worktree` succeed; `make wt.new` reports the base it used, and that
base holds every landing.

## Design

- **Placement verdict.** `EXTEND`: the two worktree scripts change where they
  look; `git grep -n "origin/\${base}"` and `baseRef` locate both decisions.
- **Root cause.** Both scripts treat `origin/<base>` as the landing point
  whenever `origin` exists. `assessAheadCommits` in `ops.mjs` accepts only
  `origin/<branch>` or a tree equal to `origin/main`. `worktree-new.mjs` fixes
  `baseRef` to `origin/<base>` before the fast-forward step, which treats
  "local ahead, behind 0" as up to date.
- **Fix.** `assessAheadCommits` also accepts HEAD when it is an ancestor of the
  local base branch: those commits survive the removal. `worktree-new.mjs`
  chooses the base after the fast-forward step: local `<base>` when it contains
  `origin/<base>`, else `origin/<base>`, and reports `base_sha` from that
  choice.
- No external command is added: both run `git merge-base --is-ancestor`
  through helpers that already exist.

## Policy conformance

| Policy | Owner | Reading |
|---|---|---|
| Landing is a local `--no-ff` merge, no push | `AGENTS.md` | kept; only the "no remote" claim is corrected |
| Removal never loses unlanded work | `.cli/hooks/destructive-git-guard.mjs` | kept: ancestry of a local branch is the same proof `git branch -d` uses |
| Tests live in `src/tests.rs` | `CLAUDE.md` | kept |

## Flagged concerns

- None.

## Acceptance

`make q.check` passes; both new tests fail on the current scripts and pass
after; the existing cleanup tests for no remote and a steered session pass.

## Rejected alternatives

- **Push `main` after each landing.** A session does not push here.
- **`--force` in the landing step.** It removes unmerged work as readily as
  landed work, and makes the safety check meaningless.
- **Return the local branch from `baseRef` in `ops.mjs`.** `ship-worktree`
  uses that ref for pushed branches too; the narrower check changes only the
  removal decision.
