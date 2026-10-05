# Intent: the review gate exists on one machine and nowhere in the repository

- **Status**: approved
- **Opened**: 2026-09-17

## Problem

`cargo test --locked` fails on `main` at 116b8f0, in a fresh worktree of it:

```
存在しないパスを指しているドキュメントがあります:
  docs/sdlc/changes/038-…/intent.md: `scripts/pipeline-indicators.sh`
  docs/sdlc/changes/040-…/intent.md: `scripts/check-review.sh`
  docs/sdlc/changes/051-…/plan.md:   `.claude/state/`
```

Three documents, two different defects.

**`scripts/check-review.sh` and `scripts/pipeline-indicators.sh` were never
committed.** Both exist — 30 KB and 10 KB, dated 12 and 9 September — in the
working tree of the `main` checkout, untracked. `git log --diff-filter=ADR` on
either path returns nothing: they have never been in a commit. So has
`.githooks/`, which holds the `reference-transaction` hook that runs the review
gate.

This is not a cosmetic documentation problem. `scripts/check-review.sh` is
stage 5's gate: `AGENTS.md` and `docs/sdlc/routes.yaml` name it as the thing that
decides whether a change may be committed, and `.githooks/reference-transaction`
refuses a ref update whose commit it has not judged. That hook looks the gate up
as `refs/heads/main:scripts/check-review.sh` first and falls back to the `main`
checkout's working file, and its own refusal message says what the situation is:

```
review gate: scripts/check-review.sh is missing, empty, or cut short in the main
checkout and on the main branch, so the update is refused. Land it there whole.
```

It has never been refused here because the fallback keeps finding the untracked
file on this machine. Everywhere else — a fresh clone, a linked worktree, CI —
the repository describes a review gate it does not contain.

**`.claude/state/` is the opposite defect: the document is right.** It is in
`.gitignore`, added by change 051, which is exactly what 051's plan and spec say
they did. A path the repository deliberately omits is not a dead path, and the
check was reading it as one.

The check had the rule and stated it, in its own doc comment:

> `dist/` and `target/` are deliberately absent from the prefix list: both are
> generated, both are gitignored […]

— and implemented it by hand-listing two of `.gitignore`'s entries in a comment.
`.gitignore` gained a third and the list did not. That is
`.claude/rules/identifiers.md`'s rule about guards, verbatim: a guard that
restates the value it guards is not a guard.

## Who feels it, and when

Anyone who is not working in the `main` checkout of this one machine. The new
`.claude/config/worktree-policy.json` refuses a direct edit of a tracked file on
`main`, so *all* work is now supposed to happen in a worktree — and a worktree is
precisely where the suite is red and the review gate is absent. The policy and
the repository's contents contradict each other today.

## Desired outcome

1. `cargo test --locked` is green in a fresh worktree of `main`.
2. The three files the documents name are in version control, so the gate
   `AGENTS.md` describes is the gate a checkout gets.
3. A document may name a path `.gitignore` excludes without failing a check, and
   the check learns that from `.gitignore` rather than from a list someone keeps
   by hand.

## Systems likely affected

- `scripts/check-review.sh`, `scripts/pipeline-indicators.sh`, `.githooks/` —
  added to version control byte-for-byte as this machine has been running them,
  with one addition: `scripts/check-review.sh` gains a `not-wired:` line in its
  header, because `every_gate_script_is_wired_or_declares_why_not` requires one
  and the honest answer is the one below. Not authored here and not reviewed
  here: the claim is only that the repository should contain the files its own
  documents and hooks require.
- `src/tests.rs` — `harness_documents_only_name_paths_that_exist` reads
  `.gitignore` instead of a hand-kept list, and
  `the_gitignore_exemption_covers_runtime_state_and_nothing_tracked` keeps that
  exemption narrow.

## The gap this change does not close, stated rather than papered over

Adopting the files does not make the gate run. Nothing committed installs
`.githooks/installed/reference-transaction` under `$(git rev-parse --git-path hooks)`:
`core.hooksPath` is unset, and the only installer is a 503-line
`.claude/hooks/gate-commit.sh` that has also never been committed — the tracked
one is 94 lines and does not mention `.githooks`. A fresh clone therefore gets
the gate as a file and not as a gate.

The first attempt here hid that by adding `.githooks` to the wiring test's caller
list. `subprocess-safety-reviewer` returned `do-not-approve` on it, on two counts
that both hold: it asserts wiring a clone does not have, and it would have passed
on a *mention* — the hook runs the gate as `bash "$gate"` through a variable, so
the only literal `scripts/check-review.sh` in that file is path resolution and a
refusal message. That is the `guard-stage.sh` failure the test's own comment
recounts, one directory over. The caller list is unchanged and the gap is written
into the script's header and into lesson 018 instead.

Closing it means adopting the installer, which is a `gate-configuration` surface.
`docs/sdlc/risk.yaml` marks that surface `paused`: it is a decision for a person,
not for the change that noticed it.

## Not in scope

- Reviewing what those scripts do. They are adopted as they stand.
- Installing the git hook — `paused` surface, see above.
- `steering_bytes`, which `scripts/check-bands.sh` reports at its `propose` tier
  (170 868 against a 150 000 threshold). It stood at 167 146 before this change,
  already over, and the 3 722 bytes this change adds are the lesson the pipeline
  requires it to write.
