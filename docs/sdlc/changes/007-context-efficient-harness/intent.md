# Intent: advanced development harness mechanisms are worth adopting, but an unbounded harness would cost more context than it earns

- **Status**: approved
- **Opened**: 2026-08-31

## Problem

Development harnesses, as they scale, solve problems Operon has not
solved: classifying a
request by *kind* before routing it, refusing to fix a bug before the root
cause is named, stopping an agent that has failed the same way three times, and
re-reading an incident ledger at the start of every session. Operon has one
undifferentiated pipeline, an incident ledger nothing reads back, and no ceiling
on how long an agent may thrash.

Adding an elaborate harness infrastructure is not an option, and the reason is the same
thing Operon already measures. Heavy architectures with dozens of skills, hooks, and
scripts, assuming bloated state machines or complex feature-folder
layouts, cost too much context. Most of that is unnecessary discovery overhead for a product
this repository is not. Operon's own control bands say the harness is *already* costing
more than it should: `always_loaded_bytes` reads 12 392 against a 9 716 baseline
and an 11 000 warn threshold. Adding to an over-budget document is how steering
stops being read.

The problem is that the mechanisms
worth having and the context budget to hold them are in direct conflict, and
nothing in this repository currently distinguishes knowledge that must be present
on every turn from knowledge that must be present only when a particular file is
open.

## Who feels it, and when

Three concrete situations, all of which have already happened here:

- A bug arrives and the session starts editing before it can say what the root
  cause is. `docs/sdlc/README.md` says "write the failing test first" for a bug
  fix, but nothing routes a bug fix differently from a feature, so the sentence
  is advice at the moment it is needed.
- A gate fails, the session tries a variation, it fails again, and it keeps
  going. There is no point at which the loop is required to stop and write down
  what it learned. Lesson 008 was found by a person, not by a ceiling.
- `docs/sdlc/lessons.md` holds nine entries, each ending in the guard that now
  catches it. It is 12 KB and loaded by nobody. Lesson 004 — a guard blind to the
  rename it existed to survive — is exactly the class of mistake a session would
  avoid if the lesson were in front of it while it wrote the guard, and it is not.

The person who feels it is whoever reads the resulting diff and finds a mistake
the repository had already written down.

## Desired outcome

- A development request is classified by kind, and the kind decides which stages
  are mandatory, which gate closes it, and how many attempts it gets before the
  session must stop and hand off. Observable: a bug fix cannot reach "done"
  without a regression test that was seen failing first.
- An agent that has failed the same way three times stops, and what it learned
  survives the stop as a committed artifact rather than as a lost context window.
- Knowledge that applies only when a particular file is open is *not* paid for on
  every turn. Observable: `always_loaded_bytes` falls below its 9 716 baseline
  while the policies it used to hold are still applied when the files they govern
  are touched.
- The new mechanisms are visible to the same instruments as the old ones: each
  one is counted by `scripts/harness-metrics.sh`, banded in
  `docs/sdlc/bands.yaml`, and guarded by a test in `src/tests.rs`.
- What was deliberately omitted is written down, so decisions are not re-derived.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- No Node toolchain. Every adopted mechanism is bash, a flat data file, or a Rust
  test — the three things this repository already runs.
- User-facing text is Japanese; code, comments, and docs are English. The harness
  is docs, so the harness is English.
- A mechanism without a guard is not finished work (`docs/sdlc/lessons.md`).

## Systems likely affected

`CLAUDE.md`, `AGENTS.md`, `REVIEW.md`, `docs/sdlc/README.md`,
`docs/sdlc/bands.yaml`, `docs/sdlc/lessons.md`, `docs/sdlc/templates/`,
`.claude/settings.json`, `.claude/hooks/`, `.claude/skills/sdlc/`,
`.claude/agents/`, `scripts/harness-metrics.sh`, `src/tests.rs`. No change to
application behaviour: `src/` gains tests and nothing else.

## Open questions

- Does Claude Code actually load `.claude/rules/` conditionally, or would moving
  policy there quietly turn it off? — Answered before the spec: the mechanism is
  real, rules with a `paths:` frontmatter list load when a matching file is
  **read**, and rules without one load unconditionally. The read-not-write
  distinction is the risk, and the spec must say what covers it.
- Is an elaborate feature orchestrator needed? — Not as a complex multi-script suite.
  The core requirement is the work-type routing table and the auto-stop ceiling, and those are
  two flat tables. The spec defines this lean architecture directly.
- How much steering may this change add before the band moves a tier? — 107 077
  now, 125 000 is the diagnose threshold. The spec states the budget it spends.

## Not in scope

- Generic business discovery: market research, pricing, GTM, betting tables,
  opportunity mapping. Wrong domain for a local terminal manager.
- A JSON state file, a project-config file, a domain map, the code-ownership
  index, and the HTML review decks. All unnecessary boilerplate, all replaceable here by git and the six-stage artifacts.
- Enforced pre-flight/post-flight checklists. Operon's gates already run
  deterministically at the commit boundary; a checklist an agent prints to itself
  is prose with no guard, which is the thing this repository has decided not to
  add.
- Any change to what the application does. This change is entirely harness.
