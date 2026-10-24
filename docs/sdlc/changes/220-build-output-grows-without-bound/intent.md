# Intent: build output keeps growing until a person notices the disk is full

- **Status**: approved
- **Opened**: 2026-10-24

## Problem

The person looked at the project folders `dist` and `target` and found they take a
lot of disk space. Their request, verbatim: "このプロジェクトフォルダーであるdist,
targetをみたら容量をかなり食っていて、これらを定期的に掃除したら片付けはできない？
作業を効率化したい。"

Cleaning happens in one place only: when a new worktree is opened, the main
checkout's cargo output is trimmed. Every other worktree's own build output, which
is a copy of main's at the moment it was opened and grows from there with each test
run, is never trimmed, and nothing runs on a timer. So the space comes back only if
the person remembers to ask.

## Who feels it, and when

The person running several worktrees in parallel on one Mac. Each worktree builds
and tests on its own, so each one's build output keeps piling up old test programs
and old incremental caches for as long as the worktree lives. They notice when the
disk is short, and then stop to clean by hand.

## Desired outcome

- Once a week, without the person doing anything, the old test programs and stale
  incremental caches under the build output of the main checkout **and of every
  open worktree** are gone, and the newest ones stay, so the next build is not cold. "Newest" means the newest test program per
  build profile; incremental caches go when untouched for seven days.
- A weekly pass never runs while a build or test is in progress.
- The person can see what the last pass reclaimed, and can turn the weekly pass off
  with one command.
- The packaged app folder is left as it is: it holds only the current bundle and
  one previous bundle, and replacing it is a separate, atomic operation.

## Constraints this change inherits

- macOS only; local-first, nothing leaves the machine.
- The existing trim when a worktree is opened must behave as it does today.
- Never delete the newest build output or anything a running build is writing.

## Systems likely affected

The existing trim script, the Makefile, one new install script for the weekly
schedule, and tests in `src/tests.rs`. No app source changes, so the installed app
does not go stale.

## Open questions

None. Asked how far to automate, the person chose: "extend the trim to every
worktree's build output and run it automatically every week, skipping it while
cargo is running".

## Not in scope

- Deleting build output wholesale (`cargo clean`), which makes the next build take
  minutes.
- Touching the packaged app folder's bundles.
- Removing worktrees themselves.
