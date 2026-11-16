# Plan: the stop gate runs out of time and checks nothing

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-16
- **Status**: approved

## Root cause

Change 081 narrowed the contract half of `.claude/hooks/gate-stop.sh` to the
changes in play, but did it as a filter inside the old walk: the loop still
globs every `docs/sdlc/changes/*/state.yaml` and, for each one, runs
`grep -xF "$(basename "$(dirname "$state_file")")"` against the in-play list
to decide to skip it. That is three child processes per change in the tree,
whatever the session is working on: about 870 for the 290 changes there now.
Measured on this tree with no change in play, the hook took 6 to 13 seconds
idle; under the load of parallel agents it reaches the 30-second timeout
`.claude/settings.json` gives it, and Claude Code cancels it. A cancelled Stop
hook refuses nothing, so the contract, the approval ledger and the branch audit
are all skipped. The transcripts hold 182 such cancellations in 30 days.

## The fix

Walk the in-play list instead of the tree. When there is a `main` to fork from,
the loop reads `docs/sdlc/changes/<name>/state.yaml` for each name in
`$in_play`, the list the filter already compared against, so the set of state
files read is exactly the set the filter let through before. With no `main`,
every state file is read, as now. The body of the loop does not change.

Nothing is narrowed or removed: the same state files are read and the same
items reported, and the stops that used to be cancelled now finish.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/gate-stop.sh` | the `open_contract` loop reads the in-play changes rather than filtering the glob |
| `src/tests.rs` | `the_stop_gate_does_not_pay_for_changes_it_is_not_working_on` |
| `docs/sdlc/changes/297-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. The test, watched failing on the unedited hook (done: 25 child processes
   with 2 parked changes, 199 with 60).
2. The loop.
3. The existing stop-gate tests (`the_stop_gate_`), which pin what is reported:
   in play, parked on main, untracked, renamed, quoted, moved, and no `main`.
4. `mutations.py`: putting the per-change filter back turns the new test red;
   reading no state file, or the glob without the filter, turns the existing
   scope test red.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| An in-play change is no longer read | a pending item goes unreported | `the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on`, all eight arrangements |
| A parked change is read again | another session's items reported | the same test's `on_main` run, and the new test's `item-800` check |
| A name with a space or non-ASCII byte splits | a change goes unread | the scope test's quoted-name runs; the loop reads lines, not words |
| The cost comes back by another tool | the count grows with parked changes | the new test shims ten text tools, not only the three used today |

## Proof of completion

- `cargo test --locked the_stop_gate_` — every stop-gate test passes, the new one included.
- `python3 docs/sdlc/changes/297-…/mutations.py` — every mutation caught.
- The hook on this tree, no change in play: well under a second, from 6 to 13.

## Departures from the plan

- **`scripts/check-approvals.sh` too.** With the loop fixed the hook still took
  5 to 6 seconds on this tree. A trace put 5.1 of them in the two calls of
  `scripts/check-approvals.sh` the gate makes: its `ledger()` ran an `awk`, a
  `basename` and a `dirname` per change's `approvals.log` (134 ledgers, 402
  child processes, 2.4 seconds a call). Trust is computed from every ledger,
  so every ledger is still read, now in one `awk` over all of them. Its
  `--list` output over the real 558 ledger lines is byte-identical before and
  after; a call takes 0.3 seconds. Test:
  `the_approval_ledger_reads_every_ledger_in_one_pass` (15 child processes with
  2 ledgers, 189 with 60, before the fix). The file is gate configuration like
  the hook, and like it the edit reads the same lines and relaxes nothing.
- **Where "the same set" is not exact.** The empirical verifier, refused a
  shell by the worktree guard in its isolated worktree, read the diff and named
  two trees where the loop reads differently, neither a valid change: a change
  directory whose name starts with a dot is now read when in play (the glob
  skipped it — a check gained, not lost), and on the no-`main` fallback a
  directory name holding a newline is split. `scripts/check-approvals.sh` now
  takes a change's name literally where `awk -v` read backslash escapes in it.
  Old and new `check-approvals.sh` were compared over empty, comment-only,
  unterminated, spaced, non-ASCII, malformed and directory-shaped ledgers, and
  over none, in every mode, under `/bin/bash` 3.2 and bash 5: identical.
- **Measured.** The whole hook on this tree under a load average of 23 to 25:
  1.0 to 1.3 seconds, against 8 to 11 for the old hook with the fixed script
  and 9.5 to 14.4 for the old hook with the old script.
