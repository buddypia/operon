# Intent: branches stay on the remote after their work has landed

- **Status**: approved
- **Opened**: 2026-10-09

## Problem

The person asked, after change 139 landed: "古いremote branchとかはマージされたら合わせて削除して不要だから。今後自動で削除するメカニズムにして発生しないようにして" — once a branch is merged, its remote copy is useless, so it should be deleted with the merge, automatically, so that stale branches stop accumulating.

On 2026-10-09 the remote held eighteen branches besides `main`, sixteen of them already merged into it. The landing procedure removes the worktree and the local branch but never the remote one, so every change leaves one or two branches behind (the change itself, then its close-out). They were deleted by hand that day.

## Who feels it, and when

The person, whenever they look at the repository's branch list, and every session that lists remote branches to find its own work: each landed change adds rows that mean nothing.

## Desired outcome

After a branch's work lands on `main`, its remote branch disappears without anyone doing it. A branch whose work has not landed — another session's work in flight, or a branch just created from `main` and pushed before its first commit — is never deleted.

## Constraints this change inherits

- The landing procedure (worktree, CI on the branch, `--no-ff` merge, push) stays as it is; the merge gate reads CI on the branch head before the merge, so deleting the branch afterwards takes nothing it needs.
- `.claude/scripts/create-pr/ops.mjs` belongs to a synced bundle (`.claude/.bundle-receipt.json`); an edit there is overwritten by the next sync, so the mechanism must live in files this repository owns.

## Systems likely affected

A GitHub Actions workflow on pushes to `main`, a script under `scripts/`, and a test in `src/tests.rs`.

## Open questions

None for the person: deleting a branch whose tip is already in `main` loses nothing, and that is the only kind deleted.

## Not in scope

- A branch rewritten after it was pushed, whose old tip never lands (change 139's first branch). That came from one workaround and is recorded in `plan.md`, not automated.
- Local branches and worktrees: `cleanup-worktree` already removes them.
