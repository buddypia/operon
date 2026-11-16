# Plan: a branch push is refused from the main checkout and from the worktree

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-17
- **Status**: approved

## Root cause

The trunk allowlist was written in change 128 for the landing step, when
every branch push ran from a session sitting in its worktree, so it admitted
only `git push origin main`. The Publishing gate in `.claude/hooks/guard-bash.sh`
admits a push only in its plain form (`git push [-u|--set-upstream|-q|--quiet]*
origin <branch>`, nothing before or after it). A session whose working
directory does not persist between Bash calls stays in the main checkout, so
the plain form is judged by the trunk allowlist and refused, and the forms the
trunk refusal suggests (`cd <worktree> && …`, `git -C <worktree> …`) are not
plain and are refused by the Publishing gate.

## The fix (option A, approved by the person)

Replace the trunk allowlist pattern `git\s+push\s+origin\s+main\s*$` with

    git\s+push(?:\s+(?:-u|--set-upstream|-q|--quiet))*\s+origin\s+[A-Za-z0-9][A-Za-z0-9._/-]*\s*$

the Publishing gate's plain form for any branch, with `plain_push`'s character
set. Nothing else in any guard changes. A refspec (`main:other`, `:main`), a
forced ref (`+main`), `--force`, another remote, a redirect into a file and a
chained command all stay refused on the trunk: the pattern is anchored at both
ends, its branch character set excludes `:` and `+` and cannot start with `-`,
and the guard judges each simple command and refuses a file redirect itself.

## Files that change

| File | Change |
|---|---|
| `.claude/config/worktree-policy.json` | the push pattern above; the allowlist `description` names the branch push |
| `src/tests.rs` | `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here`: `git push origin feature/x` moves to the allowed side, `git push -u origin feature/x` and `git push -q origin main` join it; `git push origin main:other`, `git push origin +main`, `git push origin :main`, `git push upstream feature/x`, `git push origin feature/x > out`, `git push origin feature/x; rm -rf src` join the denied side next to `git push --force origin main` |
| `docs/sdlc/changes/304-…/` | state, plan, approvals, mutations, verification |

Wording that says a branch push must come from its worktree: none found.
`gate-merge.sh`'s "Push the branch (git push -u origin $branch) from its
worktree" still describes a working form and stays.

## Order of work

1. The test assertions, watched failing against the old pattern.
2. The pattern and description in `worktree-policy.json`; the test passes.
3. `mutations.py`: back to main only; `--force` admitted as a flag; `:` or `+`
   admitted in the branch; the end anchor dropped; any remote admitted — each
   turns the test red.
4. Land; no bundle swap (nothing in `src/` outside `src/tests.rs`).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The pattern admits a push that rewrites or deletes a remote ref | `--force`, `+ref`, `:ref` pass on main | the deny assertions; the Publishing gate refuses them independently |
| The pattern admits a second command | `; rm -rf src` passes | the guard's per-simple-command split and the deny assertion |
| A push from main runs local code | a pre-push hook | none exists (`.git/hooks` holds only reference-transaction) |

## Proof of completion

- `cargo test --locked the_trunk_allowlist_and_the_ownership_check_are_switched_on_here` — 1 passed, after failing on the old pattern.
- `mutations.py` catches every mutation.
- CI green on the branch head; this change's own branch push runs as the plain form.

## Departures from the plan
