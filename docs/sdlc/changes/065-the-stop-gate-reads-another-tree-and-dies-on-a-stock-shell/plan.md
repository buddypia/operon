# Plan: the stop gate reads another tree, and on a stock macOS it reads nothing

- **Spec**: none — `bugfix` route. The two measurements below are the intent.
- **Approved**: 2026-09-21
- **Status**: done — see **Departures from the plan**

`.claude/hooks/gate-stop.sh` has two defects, in the same file, on the same
`paused` surface. Both are measured, not inferred.

## The two causes, named

**One — it resolves the wrong checkout.** Line 31 is
`cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0`. `CLAUDE_PROJECT_DIR` is unset in this
environment, so the hook lands in `.`: the working directory of the Claude Code
process, which for a session working in a worktree is the *main* checkout.
Measured, not guessed — the hook reported contract items from changes 039 and
049 to this session, and those two change directories do not exist in this
worktree at all. Eleven stops in this session were spent reading items that
could not be settled from here, because they belong to somebody else's tree.

Two consequences past the wrong report. `git diff HEAD -- '*.rs'` is answered
from whatever `GIT_DIR`/`GIT_WORK_TREE` say, so the *file list* can come from
this worktree while the *contract scan* — a filesystem glob over
`docs/sdlc/changes/*/state.yaml`, relative to the working directory — comes from
the main checkout. One refusal, assembled out of two different repositories. And
the stamp at `target/.operon-stop-gate`, which is what makes this gate refuse a
position once and never twice, is written into the main checkout's `target/`, so
every worktree on this machine shares one stamp file.

This is change 057's defect, in the sibling file. 057 fixed it in
`.claude/hooks/gate-commit.sh`, wrote the reasoning into a 35-line comment
there, and guarded it with
`the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in`. Nothing swept
the fix across to the other hook that asks the same question.

**Two — it does not parse on a stock macOS.** `/bin/bash -n` on the hook:

```
line 63: syntax error near unexpected token `;;'
line 63: `    case "$status" in done|archived) continue ;; esac'
```

bash 3.2 scans a command substitution by counting parentheses, and the `case`
at line 63 is inside the `$( )` that opens at line 59 — so the unbalanced `)`
in the pattern `done|archived)` closes the substitution early. The shebang is
`#!/usr/bin/env bash`, which finds the 3.2 at `/bin/bash` on any machine without
a newer bash ahead of it on `PATH`. There, this Stop hook exits 2 with a shell
syntax error before it reads a single contract — and a Stop hook that exits 2
blocks the stop. So on a stock macOS the gate is not weak, it is an unconditional
wall with an error message about a `;;`.

Already known and deliberately deferred: `KNOWN_UNPARSED_UNDER_BASH_3_2` in
`src/tests.rs` holds this exact entry, with the fix written out and the reason it
was not applied — `.claude/hooks` is `paused`. That deferral is what is being
discharged here.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/gate-stop.sh` | resolve the working tree from the payload's `cwd` and `git rev-parse --show-toplevel`, as `gate-commit.sh` does; write the `case` pattern as `(done\|archived)` |
| `src/tests.rs` | new guard `the_stop_gate_reads_the_tree_the_session_is_working_in`; `KNOWN_UNPARSED_UNDER_BASH_3_2` emptied |
| `docs/sdlc/lessons.md` | entry 040 |

## Order of work

The parse fix first, because it is one character and because every later
measurement of the hook is worth less while the hook cannot be read by the shell
its shebang finds.

1. `.claude/hooks/gate-stop.sh` line 63: `done|archived)` → `(done|archived)`.
   Confirm with `/bin/bash -n`.
2. Empty `KNOWN_UNPARSED_UNDER_BASH_3_2` to `[(&str, &str); 0] = []`. Both tests
   that read it already handle an empty list; one of them fails loudly if an
   entry is left behind after its script is fixed, which is the direction that
   matters here.
3. Write the guard, and watch it fail against the unfixed resolution.
4. `.claude/hooks/gate-stop.sh` line 31: the four-line resolution.
5. Lesson 040, then the four mutations.

## The guard, and what makes it discriminate

Two repositories in a fixture. `working` holds an untracked `.rs` file and a
change directory whose `state.yaml` has an open `machine pending` item;
`elsewhere` is clean and has neither. The hook refuses with `exit 2` only if it
looked at `working`.

One case is not enough, and the reason is specific to this hook. When
`GIT_DIR`/`GIT_WORK_TREE` are set, every `git` call in the hook answers from
`working` **whichever directory the hook cd'd into** — so the Rust-file half of
the refusal fires even with the bug present, and a case asserting only the exit
status would pass against the mutation. What separates them is the contract
scan, which is a filesystem glob relative to the working directory and therefore
finds nothing when the hook is standing in the wrong tree. So the pointers case
asserts on the *text*: the refusal must name the contract item. That is the
same finding lessons entry 037 records against 057's own guard — enumerate every
input the decision reads, and vary every one of them — applied before the fact
rather than after.

Four cases, one per input the hook can resolve a tree from:

| Arrangement | Due |
|---|---|
| the reported `cwd` is the only thing naming `working` | `exit 2` |
| the pointers are the only thing naming `working` | `exit 2`, and the refusal names the contract item |
| nothing resolves, so `CLAUDE_PROJECT_DIR` is reached | `exit 2` |
| nothing names `working` at all — the negative control | `exit 0` |

The stamp is removed before each run. Without that, two cases that resolve the
same tree in the same state are indistinguishable: the second is answered by the
stamp rather than by the resolution, and it would go green against a mutation.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Honouring `GIT_DIR`/`GIT_WORK_TREE` is wrong for *this* hook | the gate reports a tree the session is not in | it is the same question `gate-commit.sh` settled: the pointers are written into a worktree's own `settings.local.json` by `.claude/scripts/worktree-init.mjs`, so they name the session's tree. The pointers case in the guard is the assertion |
| The fixture's `elsewhere` is a repository under `$TMPDIR`, and a repository *above* the fixture would make `--show-toplevel` answer from outside it | the two cases that need "nothing resolves" would silently stop testing the fallback | `$TMPDIR` on macOS is `/var/folders`, outside any checkout; asserted by the negative-control case, which turns red if resolution starts answering from somewhere else |
| The fix makes the gate start reporting change 061's seven pending machine items | every stop in this worktree gains a refusal that was not there yesterday | that is the gate working. 061 is open and those items are genuinely unsettled; the previous silence was the defect |
| `(done\|archived)` parses under 3.2 but changes behaviour | a done or archived change stops being skipped | measured: `/bin/bash -n` on a reduced copy of the same `case` inside the same `$( )` returns 0, and the parenthesised form is what bash documents for this |

## Proof of completion

- `/bin/bash -n .claude/hooks/gate-stop.sh` — no output, exit 0.
- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, with
  N one higher than 488.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — `bands: N metrics within their bands`.
- `bash scripts/pipeline-indicators.sh --lessons` — 39 of 39.
- `the_stop_gate_reads_the_tree_the_session_is_working_in` watched failing
  against the unfixed line 31, and against each of the four mutations.
- The hook, run by hand from this worktree, reporting 061's contract items and
  not 039's or 049's.

## Departures from the plan

**The parse defect already had a change, and this plan did not know it.** Change
055, `the-stop-gate-dies-before-it-reads-a-contract`, was opened by change 050
for exactly that defect and had been sitting at `status: awaiting-user` with one
question: *may a session edit `.claude/hooks/gate-stop.sh`?* That is the question
this change was authorised to answer. Found by running the fixed hook by hand —
055 was in the list of open contracts it printed, which is the gate doing its
job on the first run after being pointed at the right tree.

Nothing about the work changes; the paper trail does. 055 keeps the parse defect
and is closed by this landing, 065 keeps the tree-resolution defect, and they
land in one commit because they are one file. The 055 closure goes in a second,
paper-trail-only commit, because `scripts/check-review.sh` refuses two change
directories in one commit.

Opening 065 without first reading the change list is the mistake underneath
this. `.claude/skills/sdlc/SKILL.md` §0 says to check whether the work is already
in flight, and the check was run against the *slug* of the tree-resolution
defect, which is not the slug 055 has.

**One more defect seen, not fixed here.** The contract scan's `awk` prints only
the first line of a `machine pending` item, so a wrapped one arrives truncated
— `vocabulary-complete: bash` and `restore-e2e: cargo test --locked -- --ignored,
because this`. Visible in the run above. It is a display bug in the same file and
on the same paused surface; folding it in would widen a change whose two defects
are already measured and guarded. Recorded for a follow-up rather than carried
silently.

**Five mutations, not four.** The fifth is the parenthesised `case` pattern
reverted, which the guard catches because it runs the hook under `/bin/bash`.
That was not foreseen when the plan said four; running the guard under the oldest
shell turned the parse fix into something the suite measures behaviourally.

**This change could not be committed, and change 066 is why.** The first attempt
to land it was refused with `route bugfix is not in docs/sdlc/routes.yaml on
either side of this diff` — for a route the table plainly has. Change 064 had
added a sixth column to that table, and the gate which actually judges a commit
is `refs/heads/main:scripts/check-review.sh`, which reads a route row as exactly
five values. So 066 was written and landed first, and this work waited in the
working tree while it did. The lesson this change was going to be 039 became 040,
because 066's took the number it was next in line for.
