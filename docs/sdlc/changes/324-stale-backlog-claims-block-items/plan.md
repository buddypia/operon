# Plan: claim gives back what a gone session held, and the backlog command alone takes the next item

- **Spec**: `./spec.md`
- **Approved**: 2026-11-27
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `scripts/backlog.py` | `STALE_HOURS`, `git()`, `name_of()`, `working_copies()`, `left_behind()`, `give_back()`, `move_here()`; `claim()` moves a main-copy claim into the working copy that names it, and gives back a left-behind item before deciding to skip it |
| `.claude/skills/backlog/SKILL.md` | the command alone is the Run use; `list` only for "what is left"; `claim N` again inside the new working copy; automatic give-back replaces the manual rule |
| `src/tests.rs` | `backlog_claim_gives_back_an_item_a_gone_session_held`, `backlog_claim_keeps_an_item_someone_may_still_work_on`, `backlog_claim_moves_to_the_working_copy_that_took_it_up`, `backlog_skill_takes_the_next_item_when_called_alone`, and a repository helper |
| `docs/sdlc/changes/324-stale-backlog-claims-block-items/` | intent, spec, plan, state, approvals, verification, mutations |

## Order of work

1. The four tests, watched failing against the script and skill as they are.
2. `scripts/backlog.py`: the git helper, `name_of` and `working_copies`, then
   `left_behind`, `give_back` and `move_here`, then the calls in `claim()`.
3. The skill text.
4. `cargo test --locked backlog_`, `make q.fast`, the mutation sweep.
5. Land; then `python3 scripts/backlog.py done 2 --ref <merge>`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| An item still worked on is given back and two sessions do it | a second claim on an item with a working copy or a commit | requirement 2's cases, one per condition, and the sweep removing each |
| git failure read as "nobody is working" | items given back on a machine where git fails | requirement 2's outside-a-repository case; the sweep flipping the `None` branch |
| The holder's name compared unnormalised | a working copy whose name has a space is not recognised | `name_of` shared by both sides; the sweep replacing it on the compare side |
| A session claims on the main copy and works in a working copy of another name | its item is given back a day later while it works | `move_here` in the skill's step 3, and requirement 3's test: moved, then aged a day, then not given back |
| Another session's working copy takes over a main-copy claim | `claim N` from a copy made before the claim moves it | requirement 3's older-copy case, exit 3, and the sweep removing `born_after` |
| A git hook's `GIT_DIR` reaches the script's git child (the commit gate runs tests from a hook) | the fixture's checks read this repository's refs | the script removes the pointer variables from its git children; the tests set `GIT_DIR` to a wrong path on the script's environment in one case and still pass |
| The test's own git setup inherits the worktree's `GIT_DIR` | the fixture is made in this repository | the tests run git through `git_command`, which scrubs those variables |

## Proof of completion

- `cargo test --locked backlog_` — every backlog test passes, the four new
  ones included.
- `make q.fast` — fmt and clippy clean.
- `python3 docs/sdlc/changes/324-stale-backlog-claims-block-items/mutations.py`
  — every mutation caught.

## Departures from the plan

- **A claim under this copy's own name is shown only in a linked copy.** The
  spec's requirement 3 says `claim NUMBER` on an item held under this copy's
  name prints it and exits 0. Every session on the main copy carries the main
  copy's name, so on the main copy that would hand one item to two sessions —
  change 323's `backlog_claim_hands_out_each_item_once` failed on it. On the
  main copy such an item is refused with exit 3, as before; the sweep's
  "the main copy hands a held item to a second session of the same name"
  removes the limit and that test goes red.
- **The tests pass environment variables through `/usr/bin/env`.** The
  `GIT_DIR` case of the risk table never reached the script:
  `run_command_with_output_limit` removes git's repository pointers from every
  child, so the sweep's mutation that stops the script removing them survived.
  `backlog_run_in` now puts the test's variables on `env`'s command line.
- **The kept-item test's working copy has a space in its name**, so that
  comparing the holder unnormalised (a row of the risk table) is caught.
