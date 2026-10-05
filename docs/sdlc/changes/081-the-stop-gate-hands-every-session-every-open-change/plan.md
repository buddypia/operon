# Plan: the Stop gate asks only about the changes this tree is working on

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-09-25
- **Status**: done — departed from (see below)

## Named cause

`open_contract` in `.claude/hooks/gate-stop.sh` is a glob over every
`docs/sdlc/changes/*/state.yaml` whose status is not `done` or `archived`.
Thirty changes are open in the tree today, most of them parked for months, and
every session is handed their pending items as if they were its own.

## The rule it becomes

A change is **this tree's** when the tree is changing it:

1. its directory has uncommitted or untracked files (`git status`), or
2. a commit on this branch since it left `main` touches it
   (`git diff $(git merge-base main HEAD) HEAD`).

Only those changes' pending `machine` items are reported. On `main` with a
clean tree that is none. A branch working on change N is still held to N's
contract at every stop — which is what the section was for.

Alternatives weighed:
- *Skip `blocked` / `awaiting-user` / `reviewing` statuses.* Would still name
  049 and 077 to everyone, and silence a change's own session once it parks
  it. Rejected.
- *Archive the stale changes.* Treats today's list, not the cause; the next
  parked change reappears in every session. Rejected — and it is the owners'
  call, not this change's.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/gate-stop.sh` | `in_play`, and the contract loop skips a change not in it |
| `src/tests.rs` | `the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on` |
| `docs/sdlc/lessons.md` | entry, with its Guard |

## Order of work

1. Commit this plan and `state.yaml`.
2. The test: a fixture repository whose `main` carries an open change with a
   pending item (901), and a branch that commits to a second one (902).
   - on `main`, clean: exit 0, 901 not named;
   - on the branch: 902 named, 901 not;
   - on `main` with 901's `state.yaml` edited and uncommitted: 901 named.
   Watch it fail on the unedited hook.
3. The scope in the hook, bash 3.2-safe (no `case` inside `$( )`, lesson 040).
4. Mutation: drop each half of `in_play` in turn, and drop the filter.
5. Gates, bands, lesson, review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A session resuming a change without touching it is not reminded | the stop passes | accepted: the first edit to its `state.yaml` brings it back in; the `resume` line is still what a session reads first |
| No `main` ref (another repository) | `merge-base` fails | falls back to uncommitted only |
| bash 3.2 parse failure | every stop exits 2 | the tests run the hook under `/bin/bash` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new test fails on the unedited hook and passes after.

## Departures from the plan

- **`grep -qxF` became `grep -xF … >/dev/null`.** The first draft used `-q`,
  and `every_gate_script_reads_its_pipes_to_the_end` refused it: under
  `pipefail` a `grep -q` that stops at the first match can make the writer's
  SIGPIPE the pipeline's status and read a match as a miss. The guard that
  exists caught it; nothing new was needed.
