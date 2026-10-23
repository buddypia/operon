# Intent: Switching between a battle-test build and a stable build is manual

- **Status**: approved
- **Opened**: 2026-10-24

## Problem

> 「実際にバトルテストで確認と検証を繰り返して使う用とリリースしてあまり変えず安定的に利用する用途で分けて利用したいが、どのようにビルドして利用すれば良い？」
> 「CLIとして作っておいて」

A person who both develops Operon and relies on it every day to supervise live
coding agents wants two separate ways to run it: a battle-test build where new
changes are repeatedly built and verified against real sessions, and a pinned
stable build that stays untouched for dependable daily use.

Today, every step of that separation has to be remembered and typed by hand:
packaging a candidate bundle without overwriting the stable one, quitting whichever
Operon process currently holds the single-instance lock before launching the
other, promoting a battle-tested candidate into the pinned stable bundle while
keeping the previous stable bundle for rollback, and checking which commit each
bundle was built from.

## Who feels it, and when

- A person who finishes a change or checks out a branch to battle-test it: they
  must manually quit the running app, wait for the single-instance lock to
  release, build or package the candidate, and open the right bundle.
- A person who hits an issue during battle-testing or finishes verifying a
  candidate: they must manually copy or swap bundles to return to their stable
  app or promote the verified candidate into the stable slot.

## Desired outcome

- One command shows which Operon process is running and which commit each
  channel bundle (battle-test canary, pinned stable, previous stable rollback,
  and main installed app) was built from.
- One command builds and launches the battle-test canary bundle (or runs from
  source for quick iteration), cleanly quitting any running Operon process
  first so the single-instance lock is handed over while live `tmux` sessions
  keep running.
- One command promotes the battle-tested canary bundle into the pinned stable
  bundle atomically while retaining the prior stable bundle, one command
  switches back to running the stable bundle, and one command rolls the stable
  bundle back to the retained prior stable bundle.

## Constraints this change inherits

- macOS only.
- Local-first: no telemetry, no accounts, no cloud calls.
- Only one Operon process may hold the local session index lock at a time;
  switching channels must quit the running Operon cleanly and wait for exit
  before launching the target channel, leaving detached `tmux` sessions alive.

## Systems likely affected

Packaging and bundle-management scripts under `scripts/`, the test suite in
`src/tests.rs`, and the packaging documentation in `README.md`, `README.ja.md`,
and `README.ko.md`.

## Open questions

None: the person asked for the battle-test vs. stable workflow to be provided
as a CLI, and every operation in it is recoverable.

## Not in scope

- Running two Operon GUI windows simultaneously against two separate session
  stores: battle-testing is meant to verify candidate builds against the real
  projects and live `tmux` sessions, then switch back to stable seamlessly.
- Apple Developer ID notarization for external distribution.
