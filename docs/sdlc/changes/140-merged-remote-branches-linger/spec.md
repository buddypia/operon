# Spec: a landed branch is deleted from the remote when `main` takes it

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **A landed branch is deleted.** A remote branch whose tip is reachable from
   `origin/main`, but is not on `main`'s first-parent line, is deleted from the
   remote. That is the shape every landing here leaves behind: `git merge --no-ff`
   makes the branch tip the second parent of a merge commit on `main`.
   — `landed_remote_branches_are_deleted_and_nothing_else_is`
2. **Nothing whose work has not landed is touched.** These are kept:
   - a branch with commits `main` does not have;
   - a branch whose tip sits on `main`'s first-parent line, such as one just
     created from `main` and pushed before its first commit;
   - `main` itself and `origin/HEAD`.
   — same test
3. **It runs on its own.** A workflow runs the script on every push to `main`,
   which is the last step of landing. `--dry-run` prints what would go and
   deletes nothing.
   — `the_prune_workflow_runs_the_script_on_every_push_to_main`

## Behaviour

- The script prints one line per remote branch: `pruned <name>` or
  `kept <name> (<reason>)`, where the reason is `not landed` or `on main's line`.
  The person never reads it; it is for the workflow log.
- If a deletion fails, for example because another run already deleted the
  branch, the script prints the failure and carries on with the other branches.
  It exits non-zero only when the remote cannot be fetched.
- With no remote branches besides `main`, it prints nothing and exits 0.

## Design

- **Placement:** NEW_IN_MODULE for repository tooling. The script is
  `scripts/prune-landed-branches.sh` beside the other `scripts/*.sh`, the
  workflow is `.github/workflows/prune-landed-branches.yml`, and the test goes
  in `src/tests.rs`. `.claude/scripts/create-pr/ops.mjs` (`cleanup-worktree`) is
  not changed: it is a managed bundle file and the next sync would overwrite the
  edit.
- **Script:**
  - `git fetch --prune origin`.
  - The first-parent list of `origin/main`.
  - For each `refs/remotes/origin/*` other than `main` and `HEAD`: whether
    `git merge-base --is-ancestor <tip> origin/main` holds, and whether the tip
    is in that list. Delete with `git push origin --delete <name>`.
  - It works on the repository in its working directory. The base branch is
    `main`, and `PRUNE_BASE` overrides it for the test.
- **Workflow:**
  - Runs `on: push: branches: [main]` on `ubuntu-latest`.
  - Uses `permissions: contents: write`.
  - Checks out with `actions/checkout@v4` and `fetch-depth: 0`, so the
    ancestry is there, then runs `bash scripts/prune-landed-branches.sh`.
  - The `GITHUB_TOKEN` that checkout persists is what pushes the deletions.
- **External commands:** `git` only, run by the workflow on a GitHub runner and
  by the test against temporary repositories. The app never runs it.
- **Persisted shape:** none.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | no | no drawing |
| Icons | no | no drawing |
| Identifier SSOT | no | the base name `main` matches `ci.yml` and `gate-merge.sh`, which also write it literally |
| Durability | no | no store |
| Subprocess safety | no | the app spawns nothing new; the test's `git` goes through `git_command` / `forget_inherited_repository` |
| Documentation | yes | no user-facing app behaviour changes; the landing line in `AGENTS.md` names the automatic deletion so a session does not do it by hand |
| Local-first | yes | runs on GitHub's runner against GitHub's copy of the repository, the same place `ci.yml` already runs; the app makes no network call |
| Budgets | no | no scan in the app |

## Flagged concerns

- **A branch fast-forwarded into `main` instead of merged with `--no-ff`** would
  sit on the first-parent line and be kept. Landing here is always `--no-ff`
  (`gate-merge.sh`), so this only matters if that ever changes, and the cost is
  a branch left behind, not one wrongly deleted.

## Acceptance

- The test passes. It fails when the first-parent exception is removed, because
  the fresh branch is then deleted, and when the ancestry check is removed,
  because the in-flight branch is then deleted.
- After this change lands, its own push to `main` runs the workflow, and the
  workflow deletes this change's branch.

## Rejected alternatives

- **GitHub's "Automatically delete head branches".** It deletes only on a merged
  pull request, and nothing here lands through one.
- **Deleting inside `cleanup-worktree`.** It is a managed bundle file, and it
  would only clean up branches whose landing session survived long enough to
  call it.
- **Deleting every branch that is merged into `main`.** That would also delete a
  branch pushed at `main`'s tip before its first commit, which is another
  session's work in flight.
