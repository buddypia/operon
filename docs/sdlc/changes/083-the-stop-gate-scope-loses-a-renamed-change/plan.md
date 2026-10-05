# Plan: the Stop gate's scope keeps a renamed change, a quoted path, and a repository with no main

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-09-25
- **Status**: done — departed from (see below)

## Named cause

`in_play` in `.claude/hooks/gate-stop.sh` (change 081) reads the uncommitted
half from `git status --porcelain` v1 text. A staged rename prints
`R  old -> new` and the pattern captures `old`; a path git quotes starts
`"docs/…` and the pattern misses it. And a failed
`git merge-base refs/heads/main HEAD` is read as "no committed work".

## Fix

- Uncommitted half: `git diff --name-only HEAD` (tracked, staged or not, with
  the new side of a rename) plus `git ls-files --others --exclude-standard`
  (untracked) — the same two questions lines 127-128 already ask for `changed`
  — both with `-z`, so no path is quoted, and split on NUL with `tr`.
- Committed half: when there is no fork point to diff from, every open change
  is in play, as before 081. Failing closed there costs a list in a
  repository that has no `main`; failing open costs an unread contract.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/gate-stop.sh` | `in_play` as above |
| `src/tests.rs` | three more runs in `the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on`: an untracked new change, a staged `git mv` of a change, a change whose only edit is a file with a space in its name; and a fixture with no `main` |
| `docs/sdlc/lessons.md` | lesson 050's Guard extended |

## Order of work

1. Commit this plan and `state.yaml`.
2. The runs; watch the rename, the space, and no-`main` runs fail on the
   unedited hook. (The untracked run is expected to pass already: it pins a
   path 081 had but did not test.)
3. The fix. Everything green.
4. Mutation: drop `ls-files`, drop the `diff HEAD`, drop the no-fork fallback.
5. Gates, bands, review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `tr '\0' '\n'` then `sed` mangles a path with a newline | a change dir with a newline in its name drops out | accepted: change dirs are slugs |
| bash 3.2 / pipefail | every stop exits 2, or a match reads as a miss | tests run `/bin/bash`; `every_gate_script_reads_its_pipes_to_the_end` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new runs fail on the unedited hook and pass after.

## Departures from the plan

- **The quoted run passed on the broken hook at first.** This machine's
  `~/.gitconfig` sets `core.quotepath=false`, so git quoted nothing and the
  `-z` mutation survived. The test now hands the hook git's default through
  `GIT_CONFIG_COUNT`, uses a non-ASCII name, and runs it once untracked and
  once tracked-and-edited, so each `-z` has a run that needs it.
- The test collects every miss before asserting, so a red run names all the
  ways in that are lost, not the first.
- **Review round 1 found the fix reintroduced a loss 081 did not have.**
  `git diff` pairs a move between two change directories as a rename and
  prints the new path alone, so the change a file left dropped out; 081's
  porcelain reading kept the old side. Both diffs now pass `--no-renames`, and
  a run moves a file from 901 into 906 with both still open. Watched red with
  the flag removed from the uncommitted diff. The branch diff's copy of the
  flag has no run of its own (carried).
