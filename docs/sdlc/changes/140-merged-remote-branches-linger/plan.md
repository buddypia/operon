# Plan: a landed branch is deleted from the remote when `main` takes it

- **Spec**: `./spec.md`
- **Approved**: 2026-10-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/prune-landed-branches.sh` | new: the rule, run against `origin` |
| `.github/workflows/prune-landed-branches.yml` | new: runs the script on push to `main` |
| `src/tests.rs` | two tests: the rule against temporary repositories, and the workflow wiring |
| `AGENTS.md` | the landing line says the remote branch is deleted automatically |

## Order of work

1. The test of the rule, against a script that deletes nothing, watched failing.
2. The script, making that test pass.
3. Mutations: remove the first-parent exception, then the ancestry check; each
   must turn the test red.
4. The workflow and its wiring test.
5. The `AGENTS.md` line.

## Risks

- **The token cannot delete branches.** `contents: write` is enough for
  `git push --delete` with the checkout-persisted `GITHUB_TOKEN`. If branch
  protection ever forbids it, the run fails visibly and deletes nothing.
- **Two pushes to `main` close together** run two prunes. The second one's
  deletions fail on branches the first already removed, and the script carries
  on past them.

## Proof of completion

- `cargo test --locked landed_remote_branches` and
  `cargo test --locked the_prune_workflow`.
- The mutation results recorded here.
- After landing, the workflow run on `main` shows `pruned fix/140-…`, and
  `git branch -r` lists only `origin/main` and whatever is in flight.

## Departures from the plan

- **Three points from the evaluator, taken in the build rather than by
  re-opening the spec:** the deletion carries a lease on the tip it judged
  (`--force-with-lease=refs/heads/<name>:<sha>`), so a commit pushed between
  the fetch and the delete keeps the branch; the rule test also runs
  `--dry-run` and asserts nothing was deleted; and the flagged concern's
  "always `--no-ff`" should read "`--no-ff` or `--ff-only`" — a fast-forwarded
  branch sits on the first-parent line and is kept either way. A zero-commit
  branch made from a landed branch's tip would also be deleted; only its name
  goes, its commits are all in `main`.
