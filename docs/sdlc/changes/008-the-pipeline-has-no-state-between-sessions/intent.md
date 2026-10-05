# Intent: the pipeline has six stages and no state, so every session has to re-derive where the work stands

- **Status**: approved
- **Opened**: 2026-09-01

## Problem

Change 007 added the routing half of an orchestrator — kind
of request, mandatory stages, attempt ceiling — and stopped there, on the grounds
that the rest depended on machinery Operon does not have. That was true and it
was also the easy half. What was left behind is the part that makes an
orchestrator an orchestrator rather than a table: it holds **state**.

Four consequences, all live in this repository today:

- **A change directory does not say where it stands.** `docs/sdlc/changes/007-…/`
  holds three documents, each with a `Status:` line an agent wrote by hand, and
  nothing machine-readable. A session resuming that work reads three files and
  guesses. There is no equivalent of "grasp the situation in three seconds".
- **Nothing checks that a spec is implementable before implementation starts.**
  `docs/sdlc/templates/spec.md` says "never write n/a to a row you did not check"
  and "flagged concerns are answered before stage 3". Both are prose. A spec with
  an empty Policy conformance cell, an unanswered flagged concern, a surviving
  template placeholder, or a requirement no acceptance criterion covers passes
  into stage 3 exactly as readily as a complete one.
- **Operon paints, and no gate looks at the painting before the code exists.**
  Lesson 005 is a widget written for a bug and never called, found on screen
  rather than in review. A screen described and agreed before implementation is
  the cheapest place to catch that, and the pipeline has no such step.
- **"Done" is not defined per change.** `.claude/hooks/gate-stop.sh` asks whether
  the three gates ran. It cannot ask whether *this* change's own acceptance
  criteria were met, because they are prose in a section heading.

The attempt ceiling from change 007 makes the gap sharper rather than smaller: it
says stop after three failures and write a handoff, and the handoff has to
reconstruct from three prose documents what a state file would have recorded as
it happened.

## Who feels it, and when

- A session that comes back to a change after a context window ended, or after
  `/clear`, or the next day. It reads `plan.md`, finds `Status: in progress`, and
  cannot tell which of the numbered steps in **Order of work** are done. Change
  007 itself was written across a single session for exactly this reason — there
  was no way to hand it over.
- A session that starts stage 3 from a spec that is not ready. The cost lands
  later, as a rewrite, which is the failure the design stage exists to prevent.
  It happened inside change 007: five backticked rule paths in its own `spec.md`
  and `plan.md` were stale, and a test caught them — after the documents were
  written and committed as approved.
- Whoever reads a diff that changed a screen and finds the layout wrong. There is
  no artifact recording what the screen was *supposed* to look like.

## Desired outcome

- A change directory carries one machine-readable state file. A session reads it
  first and knows, without inference: which route, which stage, how many attempts
  are spent, what it is waiting for, and one line saying what to do next.
- The states a change can be in are named and closed, including the ones that are
  not progress: waiting on a person, blocked on a question, failed. A session
  that cannot proceed says which of those it is in rather than stopping silently.
- A spec is checked before stage 3 by a command, not by a reader's diligence.
  Observable: a spec with an unanswered flagged concern cannot reach a Go verdict.
- A change that paints has its screen agreed before its code exists.
- Risk decides autonomy. The surfaces where a mistake is unrecoverable here are
  already written down in `REVIEW.md`; they should decide whether a session
  proceeds, proceeds with care, or stops and asks — mechanically, not by feel.
- "Done" is a contract this change wrote for itself, and the stop gate checks it.
- Every one of the above is counted by `scripts/harness-metrics.sh`, banded, and
  guarded by a test — and none of it lands in `CLAUDE.md`, which is at its
  budget.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- No Node toolchain, and no Python in the harness. Flat data files, bash, and
  Rust tests are the three things this repository runs.
- `steering_bytes` is already breaching at `diagnose` (127 176 against a 125 000
  threshold). This change may not simply add prose on top of that.
- User-facing text is Japanese; the harness is English.
- A mechanism without a guard is not finished work.

## Systems likely affected

`docs/sdlc/` (a new state template, a risk table, the README),
`.claude/skills/sdlc/` (becomes the orchestrator, with references),
`scripts/` (a readiness checker, the metrics script), `.claude/hooks/gate-stop.sh`,
`docs/sdlc/bands.yaml`, `src/tests.rs`, `CLAUDE.md` and `AGENTS.md` by one line
each. No application behaviour: `src/` gains tests only.

## Open questions

- **Does a second entry point compete with the `sdlc` skill?** — Answered before
  the spec: a second, differently named orchestrator beside `sdlc` would work only
  if the two had different domains. Here they would be rivals for the same job,
  and two entry points for one job is the failure mode. The `sdlc` skill becomes the orchestrator.
- **JSON with a schema, or a flat file like `bands.yaml`?** — A JSON state file
  with a 300-line schema needs a repair skill for when it corrupts. That repair
  skill exists *because* the format needs one.
  A flat positional file cannot drift from its parser, which is the property
  `docs/sdlc/bands.yaml` and `docs/sdlc/routes.yaml` were both written for.
- **How much of the readiness gate applies?** — Its Phase 2.5 is Hook Model,
  JTBD, AARRR, and paywall design. Operon is a free local tool with no funnel.
  Phases 0, 1, 1.5, and 3 map; 2.5 does not and is dropped, not stubbed.
- **Where does the detail live, given the budget?** — A skill's `SKILL.md` is
  injected when the skill is invoked; a file under its `references/` is read only
  if the skill decides it needs it. That is a real tier and it is not currently
  measured. It must be measured as part of this change, or additions hide their
  own cost.

## Not in scope

- Auxiliary repair skills, status-sync skills, or redundant sub-skills. The six
  stages already provide clear decomposition; a sub-skill per stage would be
  new documents restating what the stage does.
- Model routing and evidence caching with TTLs. One developer, one machine, and
  gates that take under thirty seconds.
- The HTML review decks and `review-deck.mjs`. The checkpoint *gating* is implemented;
  the browser rendering is not.
- `project-config.json` path indirection. Operon owns its layout, and the module
  map in `CLAUDE.md` is already checked in both directions.
- A per-change lock for concurrent sessions. Plausible here — Operon exists to
  run several agent terminals — but no incident has happened, and a lock with an
  expiry is a mechanism that fails in ways nothing would notice yet.
- Any change to what the application does.
