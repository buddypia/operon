# Intent: landing a worktree can undo the main checkout's work

- **Status**: approved
- **Opened**: 2026-10-04

## Problem

The owner asked, of the branch this change was started on:

> feature/land-worktree の未コミット・PLAN.mdなど状況を分析してから残タスクがあれば進めてからコミットPRマージクリーンアップまで

That branch held the paperwork for a "land a finished worktree" feature.
Before it was built, change 110 (`bcb1dfc`) shipped the same action —
"main にマージ" on a worktree. 110's own documents promise to "abort on conflict
without corrupting" (its `intent.md`) and to "return the conflict details" and
refuse "if the agent is actively working" (its `spec.md`, requirement 1). What
is left of this branch's work is that safety, which the shipped code does not
keep:

- **A merge somebody else started can be thrown away.** When the main checkout
  is in the middle of a merge whose resolution leaves no difference from `HEAD`
  — every conflict taken as "ours" — `git status --porcelain` is empty, so the
  dirty-checkout refusal passes. Landing then runs its own merge, git refuses it
  (`You have not concluded your merge (MERGE_HEAD exists)`), and landing runs
  `git merge --abort` unconditionally, which discards the person's resolution.
  Measured on a scratch repository, both halves.
- **A conflict is reported as git's raw output**, not as the files that
  conflicted, and the notice claims the merge was undone without checking it was.
- **Landing and Operon's own commit, draft, and push write one checkout at the
  same time.** Landing runs under its own background key, so the 変更 tab's
  commit button stays live while the merge holds the index lock.
- **An agent mid-turn in the main checkout is not waited for.** Landing waits
  for sessions in the worktree, but the merge writes the project checkout, and a
  session working there gets a merge under its feet.

## Who feels it, and when

The owner, whenever they land a worktree while the main checkout is not idle:
mid-way through resolving an earlier conflict, mid-commit in the 変更 tab, or
with an agent still working in the project directory.

## Desired outcome

- Landing never discards a merge it did not start.
- A conflict names the conflicting files, and the notice says the checkout was
  restored only when it was.
- Landing waits for, and is waited for by, Operon's other writes to that
  checkout.
- Landing waits for an agent working in the main checkout, as it already does
  for one in the worktree.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted-store change and no new dependency.

## Open questions

None put to the owner. Every choice is recoverable by a revert and is recorded
in `spec.md` and `state.yaml`.

## Not in scope

- New landing UI, rebasing, pushing, or resolving conflicts inside Operon.
- The other features change 110 shipped (run scripts, checkpoints, PRs, editors,
  issues).
