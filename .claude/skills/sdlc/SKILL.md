---
name: sdlc
description: The pipeline that the feature-pilot skill, the entry point, runs a development request through. Classifies it into a route, holds its state in state.yaml, gates it at readiness and at the screen, and produces intent.md, spec.md, and plan.md under docs/sdlc/changes/. Read it when feature-pilot reaches stages 1-2, and when resuming work on a change somebody else — or an earlier session — started. Do not use for a typo, a translation fix, or a change confined to one function with no user-visible effect.
---

# Run a change through the pipeline

`docs/sdlc/README.md` owns the pipeline and its rationale. This skill runs it, and
adds only what that document does not say: the order of operations, the gates
between stages, and what to do when a stage cannot be completed.

**Read `docs/sdlc/README.md` first if you have not**, in particular "When to skip
stages" — the pipeline is for changes worth a paper trail, and running it on a
typo is the failure mode that gets pipelines abandoned.

Detail lives beside this file and loads only when it is needed:

| Reference | Read it when |
|---|---|
| `.claude/skills/sdlc/references/state-and-resume.md` | writing or updating `state.yaml`, or arriving on a change cold |
| `.claude/skills/sdlc/references/readiness-gate.md` | before entering stage 3, or interpreting a No-Go |
| `.claude/skills/sdlc/references/screen-approval.md` | the change paints |
| `.claude/skills/sdlc/references/approval.md` | anything is about to be called approved — it decides person or evaluator |

## 0. Is this already in flight?

Before anything else:

```sh
ls docs/sdlc/changes/
```

If a directory exists for this work, **read its `state.yaml` first** — before the
prose. It says the route, the stage, what is spent, and one line on what to do
next. Continue from there rather than re-deriving it; re-deriving is what that
file exists to stop.

If nothing exists, this is new work. Continue.

## 1. Does it belong here, and on which route?

Not every request needs a paper trail. If none of these is true, stop and do the
work directly:

- it adds behaviour a user can see,
- it touches more than one module,
- it changes a persisted shape, a byte budget, or a validation gate,
- it adds a dependency,
- it changes a policy, a hook, a skill, or a document an agent is steered by.

Say which one applies. If the answer is none, say that instead of producing
artifacts nobody will read.

Then name exactly one route from `docs/sdlc/routes.yaml`, which decides the
mandatory stages, the attempt ceiling, and whether a person signs it off:

| Route | The request is | Skips |
|---|---|---|
| `feature` | behaviour that does not exist yet | nothing |
| `modify` | behaviour that exists, changed | its own deploy |
| `bugfix` | something is wrong and can be reproduced | intent and spec — the bug report is the intent |
| `refactor` | the same behaviour, differently expressed | intent and spec |
| `security` | a validation gate, a launch path, `unsafe`, or the store | nothing |
| `docs` | documents, translations, comments | everything but build |
| `trivial` | <=2 files, <=20 LOC, minor internal or single-test fixes | plan, design, deploy |

- **`bugfix` runs `.claude/skills/root-cause/SKILL.md` first.** The cause is named
  and the failing test watched failing before any fix.
- A route's `review` column — `full`, `craft`, `none` — names the reviewers whose
  verdict `scripts/check-review.sh` requires before the commit is allowed.

If the request is two routes at once, it is two changes. Split it, or say which
one you are doing and what you are leaving.

## 2. Classify the risk, and take the autonomy it allows

`docs/sdlc/risk.yaml` maps the files a change will touch to a tier and an autonomy
level. Take the **highest** tier among the surfaces matched.

- `paused` — decide before editing. The store, `unsafe` and the `PATH` setup,
  the bundle swap, the gate configuration, and a new dependency are the five;
  `references/approval.md` splits each by what the edit does, and the person's
  side means `status: awaiting-user` and ask.
- `supervised` — proceed, and surface the decisions rather than only the result.
- `full` — proceed.

If any matched surface says `screen yes`, the change paints: read
`references/screen-approval.md` and set `screen: pending`.

## 3. Open the change

In the worktree, never on `main`: `make wt.new BR=feature/NNN-slug` from the main
checkout, then work from that directory (`.claude/skills/feature-pilot/SKILL.md`
step 0). `.cli/hooks/worktree-policy-guard.mjs` refuses these writes on `main`.

```sh
mkdir -p docs/sdlc/changes/NNN-slug
cp docs/sdlc/templates/state.yaml docs/sdlc/changes/NNN-slug/state.yaml
```

Next free three-digit number, then a hyphenated slug **from the problem, not the
solution**: `002-restore-loses-first-turn`, not `002-add-retry-loop`. Fill
`state.yaml` with the route, the risk, and the autonomy just decided.

Keep it current from here on. A stale `state.yaml` is worse than none — an absent
one is obviously absent, and a wrong one is read and believed.

## 4. `intent.md`

Copy `docs/sdlc/templates/intent.md` and fill it by interviewing the person, not
by guessing. Two failure modes:

- **Solution smuggled into the problem.** If the intent names a module or a
  function, it is a plan wearing the wrong hat. Ask what would be *observably*
  different instead.
- **A problem nobody has.** If "who feels it, and when" cannot be answered with a
  concrete situation, the intent is not ready.

Leave open questions open. Count each question you put to the person against the
ceiling in `state.yaml`; at seven, stop asking and adopt recommended defaults —
`references/state-and-resume.md` says how.

Commit it alone. That commit is the audit record for stage 1.

Intent, spec, plan, and screen are each approved by whoever
`references/approval.md` names — a person for a significant decision, otherwise
an evaluator that did not write it — and the verdict is a line in the change's
`approvals.log` at the artifact's digest. A `Status: approved` with no line
behind it is refused at stop.

## 5. `spec.md`

Copy `docs/sdlc/templates/spec.md`. Fill the **Policy conformance** table by
actually reading the policy documents each row names. Reading them here is the
point of the stage: a policy applied while the spec is written costs a sentence,
and the same policy discovered in review costs a rewrite.

Never write "n/a" to a row you did not check. An unresolvable row is a **flagged
concern**, and flagged concerns are answered before stage 3 — not carried into it.

## 6. The gates before stage 3

Both must pass. Record both in `state.yaml`.

```sh
bash scripts/check-readiness.sh docs/sdlc/changes/NNN-slug
```

`0` Go, `2` Conditional Go, `1` No-Go. On a No-Go, fix the **spec** — answering a
document problem in the implementation is what this gate exists to prevent. See
`references/readiness-gate.md`, including what the script deliberately cannot
judge, which is the part to do yourself.

If `screen` is `pending`, get it to `approved` now, per
`references/screen-approval.md`. Three revisions maximum, then `awaiting-user`.

## 7. `plan.md`

Start plan mode with `spec.md` in context when a person decides the plan;
otherwise have the evaluator judge it against the spec. Interrogate the plan before accepting
it: what breaks, what was assumed, what the alternative was, and what proves it
done. Then copy `docs/sdlc/templates/plan.md`, fill it, and commit it **before
editing any source file**. A plan committed afterwards is a description.

Write `state.yaml`'s `contract` from the spec's **Acceptance** section. If the two
disagree, the spec is right.

If implementation departs from the plan, fill in **Departures from the plan**. Do
not silently rewrite the earlier sections — the departure is the interesting part.

## 8. Build, verify, review

The existing machinery takes over: `CLAUDE.md` for the conventions and the
verification loop, `.claude/hooks/gate-commit.sh` at the commit boundary,
`REVIEW.md` for the review passes, `.claude/skills/ship/SKILL.md` for a release.

`.claude/hooks/gate-stop.sh` will refuse a finish while a `machine` item in the
contract is still `pending`, so settle them or mark them `waived` with a reason.

**Review has a ceiling too, and it is the one that gets spent without being
noticed.** `approve-with-nits` means approve. Taking a nit moves the digest,
which voids the approvals the nit arrived with, so a taken nit costs a whole
review round and the round produces another nit — change 060 spent nineteen
rounds there. Carry open nits in `review.yaml`'s `carried:` block and land;
only an Important finding may move the digest. `scripts/check-review.sh` prints
`round N of M` every run against `docs/sdlc/review-rounds.yaml`,
and past it the only exit that is not another round is to carry and ship. See
`REVIEW.md` and lesson 038.

A review is requested, not awaited: record who was asked and when, and escalate
to the user rather than waiting twice. A reviewer that is not in `ListAgents` is
not coming back.

## 9. When to stop instead

Every route carries an attempt ceiling — one failure of the closing gate is one
attempt. On reaching it, **stop**; do not try another variation. Set
`status: failed` and write `handoff.md` from `docs/sdlc/templates/handoff.md`. Its
**What was ruled out** table is what makes the next attempt cheaper than this one,
and it is the section that gets left empty.

Stop earlier than the ceiling when the answer is outside the repository: a
question about intent, a trade-off that is the user's to make, or evidence only a
person at the machine can gather. Record `awaiting-user` or `blocked`; stopping
without recording one is indistinguishable from finishing.

## 10. Close the loop

- If anything went wrong in a way that could recur, add an entry to
  `docs/sdlc/lessons.md` — including the **Guard** column. An entry with no guard
  is unfinished.
- If a person found it in something an evaluator approved, it is also an
  **escape**: `bash scripts/check-approvals.sh --list` finds the approval, and an
  `escape` line in this change's `approvals.log` hands that kind back to a person.
- **Watch each new guard fail** before trusting it. Delete or invert what it
  guards and confirm it goes red. A guard nobody has seen fail is a guess about
  what it guards.
- **Copy the sweep from `docs/sdlc/templates/mutations.py`.** It carries the four
  rules earlier sweeps learned by getting them wrong — assert the target is there
  before replacing it, write the mutated file rather than `copy2` it, bound every
  run, and mutate the claim.
- **If the change added or altered a sentence a person reads, mutate the claim**
  — not only the counters behind it. Weaken the sentence, or make it promise
  something the code does not do, and watch a guard go red. Change 061's fifteen
  mutations were all caught and it still went back twice, both times on a claim
  no mutation had asked about. `REVIEW.md` makes this an Important finding, so
  meeting it here costs a mutation and meeting it in review costs a round.
- If the guard can only be a prompt, add an eval under `evals/` and say so —
  `evals/007-mutate-the-claim-not-the-counter.md` is the rule above as one.
- Set `plan.md` to `done` and `state.yaml` to `status: done`, with `resume`
  rewritten to say what outlived the change, if anything.

## Report

The route, the risk tier and the autonomy it allowed, the change directory, the
readiness verdict, which stages produced artifacts and which were skipped and why,
any flagged concern still open, and the contract items still pending. If a ceiling
was reached, the handoff path and the one thing a person has to decide.

Findings left unfixed are reported from their `person:` lines in `review.yaml`
— what goes wrong, when, and whether the person must act — never as a count
of nits or the reviewer's shorthand (sdlc 085).
