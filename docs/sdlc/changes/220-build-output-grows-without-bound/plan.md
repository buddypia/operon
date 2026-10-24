# Plan: build output keeps growing until a person notices the disk is full

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `scripts/prune-target.sh` | trim loops become a function; repeatable `--target-dir`; `--all-worktrees`; `--if-idle` with `PRUNE_PGREP` |
| `scripts/install-prune-schedule.sh` | new: per-user launchd agent, `--print`, `--uninstall` |
| `Makefile` | `prune.schedule`, `prune.unschedule` |
| `src/tests.rs` | the six tests named in the spec |
| `docs/sdlc/changes/220-build-output-grows-without-bound/` | this change's record |

## Order of work

1. Write the six tests, each running a copy of the script inside a throwaway git
   repo; run them and watch each fail for its stated reason (the script lacks the
   flags, the installer and the targets do not exist). Every test clears `CARGO_TARGET_DIR` unless it sets it. The no-flags test passes
   already and is the pin that must stay green through step 2.
2. Refactor `prune-target.sh` into a per-directory function with no behaviour change
   (the missing-directory case becomes `return`); the no-flags test and
   `prune_target_script_runs_dry_run_cleanly` stay green.
3. Add repeatable `--target-dir`, then `--all-worktrees`, then `--if-idle`.
4. Add the installer and the Makefile targets.
5. `make q.fast`, then `cargo test --locked prune_`.
6. Mutation sweep, review, push, CI, land, then run `make prune.schedule` once.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Refactor changes what the default run trims | `seed-target.sh` seeds a different set | `prune_target_without_flags_trims_one_directory`, which asserts the sibling worktree is untouched and `CARGO_TARGET_DIR` is honoured |
| A pass trims a worktree mid-build | a build fails with a missing test program | `--if-idle`, and the newest program per directory is kept |
| launchd runs with a bare `PATH` | the weekly pass logs `command not found` | the plist test asserts absolute `/bin/bash` and script path; `pgrep`, `git`, `find`, `stat`, `du` are in `/usr/bin` |
| One worktree has no `target/` | the weekly pass aborts at the first such worktree | the all-worktrees fixture includes one without `target/` |
| A build starts mid-pass | an older test program is rebuilt, nothing breaks | `--if-idle` is re-checked before each directory |
| Installer leaves a stale agent pointing at a moved checkout | weekly pass logs a missing script | the installer replaces a loaded copy each run |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked prune_` — six new tests plus the existing one pass.
- `bash scripts/prune-target.sh --all-worktrees --dry-run` — lists items from every worktree.
- `launchctl print gui/$(id -u)/com.buddypia.operon-prune-target` — shows the agent.

## Departures from the plan

- The all-worktrees test uses three linked worktrees, two without a `target/`
  (named so that one sorts before and one after the worktree that has it), not
  one. The first sweep let "a worktree without target/ ends the whole pass" survive
  because the only such worktree was listed last; the mutation sweep is what found it.
- The weekly agent's own run on an already-trimmed machine reclaims nothing: `make
  wt.new` trims main's `target/` each time, so main held one build generation when
  measured (2.9G, all live). The change prevents regrowth in the other worktrees;
  it does not shrink what is live.
