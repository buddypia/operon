---
name: feature-pilot
description: The entry point for a development request in Operon — adding, changing, or fixing behaviour. Opens the worktree first, runs the change through the pipeline in docs/sdlc/README.md with the sdlc skill (route, state.yaml, intent, spec, gates, plan), builds it in Rust with its tests in src/tests.rs, lands it with git merge --no-ff, and packages it. Use for anything past a one-line fix, and to resume a change under docs/sdlc/changes/.
---

# Feature Pilot

This skill owns the **order** of a change. It owns no record: every fact it
produces is written where the pipeline already keeps it, and nowhere else.

| Fact | Its only home |
|---|---|
| route, stage, status, attempts, questions, readiness, screen, contract | the change's `state.yaml` |
| the problem, whose, why now | `intent.md` |
| requirements, behaviour, design, policy conformance, acceptance | `spec.md` |
| order of work, proof, departures | `plan.md` |
| who approved what | `approvals.log`, as `.claude/skills/sdlc/references/approval.md` decides |
| review verdicts and carried findings | `review.yaml`, checked by `scripts/check-review.sh` |

Do not start a second copy of any of these — no feature folder, no context JSON,
no separate feature spec, no project config. Two records of one position drift,
and the hooks read only the first.

## 0. Before anything is written

1. **Resume or new.** `ls docs/sdlc/changes/`. If a directory matches the
   request, read its `state.yaml` first, find its tree with `git worktree list`,
   and continue from `resume` there. Skip the rest of this step.
2. **Open the worktree.** From the main checkout:
   `make wt.new BR=feature/<NNN-slug>` (`fix/` for a bug), where `NNN` is the next
   free number under `docs/sdlc/changes/` and the slug names the problem, not
   the solution. Then `cd .worktrees/feature/<NNN-slug>` and stay there: every
   write, every `cargo` run, and the commit happen in that directory, which is
   the arrangement `.claude/hooks/gate-commit.sh` expects. Do not use Claude
   Code's EnterWorktree — it exports `GIT_DIR` and `GIT_WORK_TREE` (lessons 048,
   049).
3. On `main` only reads run (`git grep`, `grep`, `cat`, `ls`, `git log`);
   `.cli/hooks/worktree-policy-guard.mjs` refuses edits there. A typo also goes
   through a worktree — it just gets no change directory, and its commit
   message says the stages were skipped.

## 1. Classify

- **Route, risk, autonomy, screen**: `.claude/skills/sdlc/SKILL.md` §1–2, from
  `docs/sdlc/routes.yaml` and `docs/sdlc/risk.yaml`. A request that is two routes
  is two changes.
- **Placement** (routes `feature` and `modify`): `git grep -n` the nouns of the
  request and read the module map in `CLAUDE.md` ("Shape of this codebase").
  Record one verdict, with the evidence, in `spec.md` **Design**:

  | Verdict | Meaning | Then |
  |---|---|---|
  | `DUPLICATE` | the behaviour exists | stop; name the module and the test that pins it |
  | `EXTEND` | an existing module grows | route `modify` |
  | `NEW_IN_MODULE` | new behaviour in a concern the map names | route `feature` |
  | `NEW_MODULE` | a new concern: `src/<name>.rs` | route `feature`; wiring in the reference below |

  Ambiguous candidates are a question to the person, counted against the ceiling
  in `state.yaml`; never guess `NEW_MODULE`.
- **Route `bugfix`** runs `.claude/skills/root-cause/SKILL.md` before anything
  else: the cause named, the failing test watched failing.

## 2. Stages 1–2 — run the pipeline

Follow `.claude/skills/sdlc/SKILL.md` §3–7 in the worktree: `state.yaml`,
`intent.md`, `spec.md`, the readiness gate, the screen, and `plan.md` committed
before any source file is edited. What follows is only what a Rust change must
put in `spec.md` so that stage 3 raises no question:

- **Requirements** — numbered; each names the test function in `src/tests.rs`
  that fails without it.
- **Behaviour** — the Japanese strings a person reads, and the empty, error,
  too-large, and interrupted states; how a failure reaches the person (`notice`,
  `notice_briefly`, or a row's own state).
- **Design** — the placement verdict; the modules and symbols that change; any
  persisted-shape change; any external command (program, arguments, timeout,
  output ceiling); what runs on a background thread and what the draw path reads.
- **Policy conformance** — every row read against its owner document, not assumed.

## 3. Stages 3–4 — build

Detail is in `.claude/skills/feature-pilot/references/rust-build-chain.md`. The
order is fixed:

1. **Contracts first**, when the spec changes a persisted shape or parses another
   program's output: the round-trip, old-file, and parser-fixture tests.
2. **One failing test per requirement**, watched failing, then the code.
3. **Wiring**: module, strings, keys, background task, constants.
4. **Tidy** the diff.
5. **Verify**: `make q.fast` and `cargo test --locked <filter>` for the tests the
   spec names — the whole suite runs in CI on the push (change 128);
   `cargo test --locked -- --ignored` when restore or the shared-session archive
   moved, or say that you could not.

Move `state.yaml` (stage, status, contract verdicts) as each one closes. A
departure from the plan goes under **Departures from the plan** in `plan.md`;
the earlier sections stay as they were.

## 4. Review, land, package

- **Review**: the route's `review` column in `docs/sdlc/routes.yaml` names the
  reviewers in `.claude/agents/`; `REVIEW.md` is the policy and
  `scripts/check-review.sh` the gate.
- **Commit** in the worktree directory — never `git -C` from elsewhere.
- **Push** the branch from the worktree, `git push -u origin <branch>`, and wait
  for CI with `gh run watch` (in the background; about five minutes). A red run
  is fixed on the branch and pushed again. When `main` has moved, merge it into
  the branch first: the merge gate wants the branch to contain `main`.
- **Land** from the main checkout with `git merge --no-ff --no-edit <branch>` —
  `.claude/hooks/gate-merge.sh` admits it once CI passed on the branch head —
  then `git push origin main` and
  `node .claude/scripts/create-pr/ops.mjs cleanup-worktree --worktree <path>`.
  No pull request.
- **Package** when the `release-binary` contract item says so:
  `bash scripts/package-macos.sh`, then the swap in `.claude/skills/ship/SKILL.md`.
- **Close** per `.claude/skills/sdlc/SKILL.md` §10: `status: done`, and a lesson
  with its guard when the mistake could recur.

## Stopping

Stopping is a state. The attempt ceiling comes from the route and the question
ceiling is seven; at either, record it in `state.yaml` — `failed` with
`handoff.md`, or `awaiting-user` with the adopted defaults in `resume`. Ask a
person only for a decision that would be unrecoverable if wrong; who approves an
artifact is `.claude/skills/sdlc/references/approval.md`'s call, not this file's.

## Under Codex CLI and Antigravity CLI

Both load this file from `.agents/skills/feature-pilot/SKILL.md` (`.agents` is
a symlink to `.claude`; measured 2026-10-04, lesson 060). Their hook files carry only the `.cli/hooks` guards, so
`.claude/hooks/gate-commit.sh`, `.claude/hooks/guard-stage.sh`, and
`.claude/hooks/gate-stop.sh` do not run for them. Do by hand what those refuse:

- before stage 3, `bash scripts/check-readiness.sh <change-dir>` reads Go;
- before the commit, `make q.fast` passes, and `grep -c '#\[test\]' src/tests.rs`
  has not fallen nor `grep -c '#\[ignore' src/tests.rs` risen against `main`;
- before the merge, `bash scripts/ci-verified.sh <branch>` exits 0 (their hooks
  do not carry `.claude/hooks/gate-merge.sh` either);
- before saying done, no `machine` item in the `state.yaml` contract is `pending`.

`scripts/check-review.sh` runs from git itself, so it holds under every CLI.
