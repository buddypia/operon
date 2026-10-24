# Spec: build output keeps growing until a person notices the disk is full

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Every test below clears `CARGO_TARGET_DIR` (except where it sets it itself) and
copies `scripts/prune-target.sh` into a throwaway git repository
and runs that copy, so the script's own "repo it lives in" is the fixture and never
the real checkout.

1. `--target-dir <dir>` may be given more than once and trims each directory given.
   Test: `prune_target_trims_every_target_dir_it_is_given` — in two fixture target
   dirs the older test program and its `.dSYM` go and the newest stays, in both.
2. `--all-worktrees` trims the `target/` of every worktree in
   `git worktree list --porcelain` (main checkout included; paths with spaces
   work). A worktree without a `target/` is skipped, not fatal. `CARGO_TARGET_DIR`
   is ignored under this flag, and combining it with `--target-dir` is refused
   with exit 1. Test: `prune_target_all_worktrees_trims_each_worktrees_target` — a
   fixture repo with two linked worktrees, one without a `target/`; the main
   checkout and the other linked worktree hold stale test programs: both are
   trimmed and the run exits 0.
3. Without `--all-worktrees` the script trims exactly one directory as before
   (`--target-dir`, else `CARGO_TARGET_DIR`, else `<repo>/target`), so
   `scripts/seed-target.sh` is unchanged. Test:
   `prune_target_without_flags_trims_one_directory` — with no flags only
   `<repo>/target` is trimmed and a linked worktree's stale program survives; with
   `CARGO_TARGET_DIR` set that directory is trimmed instead.
4. `--if-idle` makes the script exit 0 and delete nothing further from the moment
   it sees `cargo` or `rustc` running, and print `prune-target: build in progress, skipped`. The
   check runs before each target directory. The script asks with
   `$PRUNE_PGREP -x cargo` and `$PRUNE_PGREP -x rustc` (default `pgrep`, a single
   command, no arguments); exit status 0 means a match. Test:
   `prune_target_if_idle_skips_while_a_build_runs` — with a stub that exits 0
   nothing is deleted and the line is printed; with a stub that exits 1 the stale
   test program is deleted.
5. `scripts/install-prune-schedule.sh` installs a per-user launchd agent that runs
   `/bin/bash <main checkout>/scripts/prune-target.sh --all-worktrees --if-idle`
   every Monday at 10:00, with stdout and stderr in
   `~/Library/Logs/operon-prune-target.log`; `--uninstall` removes it; `--print`
   writes the plist to stdout and touches nothing. Test:
   `prune_schedule_plist_is_valid_and_weekly` — `--print` output passes
   `plutil -lint`, has `Weekday` 1, `Hour` 10 and `Minute` 0 in
   `StartCalendarInterval` (an omitted `Minute` would fire every minute of that
   hour), both log keys, and the argument list above.
6. `make prune.schedule` and `make prune.unschedule` call the installer with no
   flag and with `--uninstall`. Test: `prune_make_targets_exist_for_the_schedule`
   — the Makefile names both and the installer script they call.

## Behaviour

Each pass first prints `prune-target: <date>`, then the per-item lines and the
summary line the script already prints (`Prune complete: N item(s) pruned,
reclaimed approximately X.`); all of it lands in the log. Skipped runs
print `prune-target: build in progress, skipped` and exit 0 so launchd does not
treat a busy week as a failure. The installer prints, in Japanese, `毎週月曜 10:00
に target/ を掃除するよう登録しました。` or `登録を解除しました。`. No window opens
and no app text changes.

## Design

Placement verdict: `EXTEND` — `scripts/prune-target.sh` already owns trimming build
output; the installer is a new script beside the other install-style scripts, no new
Rust module. The trimming loops move into a function called once per target dir; a missing
directory makes it `return`, never `exit`.
Process list reads go through `PRUNE_PGREP` (default `pgrep`) so a test can answer
without starting a build. The installer resolves the main checkout with
`git rev-parse --path-format=absolute --git-common-dir`, writes the plist to
`~/Library/LaunchAgents/com.buddypia.operon-prune-target.plist`, and loads it with
`launchctl bootstrap gui/$UID`, replacing a loaded copy first (`bootout`). Budgets:
the script lists at most the files under each `deps/` and `incremental/` it already
scanned; the worktree list is bounded by `git worktree list`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | no UI |
| Icons | No | no UI |
| Identifier SSOT | No | the agent label appears in the installer only, and the test reads it from `--print` |
| Durability | No | does not touch the store, cancellation, tmux, or a scan limit; it deletes only regenerable build output |
| Subprocess safety | No | shell scripts for build output, no new child of the app |
| Documentation | No | no user-facing docs; script headers explain usage |
| Local-first | Yes | nothing leaves the machine; no network, no `unsafe` |
| Budgets | Yes | see Design: scans stay inside the directories the script already scans |

## Flagged concerns

- **Trimming a worktree whose owner is mid-test** — a stale-by-mtime test program
  is not the one being run (the newest is kept), and `--if-idle` skips the whole
  pass while `cargo` or `rustc` runs; resolved.
- **Installing on the person's Mac** — the agent is per-user, reversible with
  `make prune.unschedule`, and the person chose weekly automatic trimming; resolved.

## Acceptance

- `cargo test --locked prune_` passes, including the six tests named above.
- `bash scripts/prune-target.sh --all-worktrees --dry-run` lists items from more
  than one worktree when more than one holds stale test programs.
- `launchctl print gui/$(id -u)/com.buddypia.operon-prune-target` shows the agent
  after `make prune.schedule`, and fails after `make prune.unschedule`.

## Rejected alternatives

- Make the default trim every worktree: `seed-target.sh` calls the script while
  sibling worktrees may be building; an explicit flag keeps that path unchanged.
- `cargo clean` weekly: the next build compiles every dependency from nothing.
- A cron entry: launchd runs a missed job when the Mac wakes; cron does not.
- Deleting `dist/`'s previous bundle: it is the last-good copy the atomic swap keeps.
