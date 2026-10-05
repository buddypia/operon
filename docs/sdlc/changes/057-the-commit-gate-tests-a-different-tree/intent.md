# The commit gate ran the three gates over a tree nobody was committing

## What happened

Change 056 was reviewed, staged, and green on all three gates. `git commit`
was refused by `.claude/hooks/gate-commit.sh` with
`Commit blocked by the gate: cargo test --locked` and 25 failing tests.

Three of those 25 do not exist in the tree being committed:

```
the_review_gate_refuses_what_it_says_it_refuses                  mine=0 main=1
the_commit_gate_installs_the_trampoline_and_refuses_an_empty_one mine=0 main=1
the_stop_gate_refuses_a_branch_carrying_an_unjudged_commit       mine=0 main=1
```

The gate had run `cargo test --locked` in `/Users/me/dev/buddypia/operon`
— the main checkout, carrying another session's uncommitted work, a 22238-line
`src/tests.rs` against the committing tree's 19754 — while the commit itself was
landing in `.worktrees/fix/trust-prompt-and-readiness-gate`.

## Who feels it, and when

Every session that follows `AGENTS.md`. That file says work lands from a
worktree — `make wt.new BR=feature/<task>` — and this session was in one. The
gate reads `CLAUDE_PROJECT_DIR` to decide where to run, and that variable names
the project the *session* belongs to, which for a worktree session is the main
checkout. So the prescribed arrangement is exactly the arrangement the gate gets
wrong. A session that works in the same directory its project points at never
sees this.

## Why it is worse than a refusal

The refusal is unfixable by the person who receives it: the failures are in
code they have not touched and cannot see from the tree they are in. The
documented response — "a failing test is evidence; do not quiet it" — sends them
to debug someone else's work in progress.

And it is not only a refusal. `GIT_DIR` and `GIT_WORK_TREE` are exported into a
worktree session, so the foreign suite that the gate started inherited a pointer
to the committing tree, and its own git tests committed change 056's staged
files onto that branch as `initial`, author
`Operon test <test@example.invalid>`. Twice — `822f1ae` and `34619a7`, both
recovered with `git reset --soft`. That half is fixed in change 056; this change
is about the gate choosing the tree.

## What would be observably different

The three gates run in the tree the commit will land in, so a refusal names a
test the committer can run, see fail, and fix. A commit redirected at another
checkout is refused with that reason rather than gated against the wrong
sources.

## Not in scope

The running hook in this machine's main checkout is a 503-line file that has
never been committed; the tracked one is 94 lines and carries the same defect.
Fixing the tracked file makes every fresh clone and every future session
correct, and does not change what gates a session whose project directory is
that main checkout. Whether to adopt the uncommitted 503-line installer is the
paused decision change 054 left open, and it is still open.
