# Plan: a step that can be undone is not handed to a person

- **Spec**: none — bugfix route; the bug report in `./state.yaml` is the intent.
- **Approved**: 2026-09-26
- **Status**: done

## The rule

A person is asked for a decision that would be unrecoverable if it turned out
wrong — the person's own criterion. Anything a revert, a re-run, or another
review undoes is decided by the session, recorded in `state.yaml`, and done.
A person is never handed a command to type: a recoverable step a guard
refuses is a gap in the guard, and the fix goes there.

## Files that change

| File | Change |
|---|---|
| `.claude/config/worktree-policy.json` | trunk allowlist gains `git merge --abort` and `git branch -d <branch>…` (never `-D`, never `-f`) |
| `AGENTS.md` | one line: the rule above |
| `REVIEW.md` | the round ceiling: a re-verdict caused only by `main` moving under a reviewed branch is not a round |
| `src/tests.rs` | the trunk allowlist test gains the new allowed and still-denied cases |
| `docs/sdlc/lessons.md` | entry, with its Guard |

Why these two commands and not more: each undoes itself or refuses the
dangerous case. `merge --abort` restores the pre-merge state, and a merge can
be started again. `branch -d` deletes only a branch merged into its upstream
or HEAD, and the tip stays in the reflog. `-D`, `-f`, `reset`, and `checkout`
stay off — each can lose work a person has not seen.

## Order of work

1. Commit this plan and `state.yaml`.
2. Test cases: `git merge --abort`, `git branch -d feature/x`,
   `git branch --delete feature/x feature/y` allowed; `git branch -D feature/x`,
   `git branch -d -f feature/x`, `git branch -d --force feature/x`,
   `git merge --abort --no-ff` denied. Watch the allowed ones fail.
3. The two patterns. Green.
4. The two sentences.
5. Mutations: drop each pattern; widen the branch pattern to accept `-D`.
6. Gates, bands, lesson, review.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The branch pattern admits a force | `-D` or `-f` allowed on trunk | denied cases in the test |
| `git -C <dir> merge --abort` aimed at another session's checkout | an abort of someone else's merge | the session-owner guard judges `-C` targets, unchanged |
| The ceiling sentence becomes a way round the ceiling | rounds never counted | it applies only when the change's own lines are untouched; a reviewer finding still costs a round |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new allowed cases fail on the unedited policy and pass after.

## Departures from the plan

