# Intent: <one line, the problem not the solution>

- **Status**: draft | approved | superseded
- **Opened**: YYYY-MM-DD

Only when this change exists because a control band reached `propose`, one line
at the left margin naming the metric it answers — `scripts/check-bands.sh` reads
it, so that a breach can say whether its prescribed response exists and what
state that response is in. Delete the line otherwise; a change that does not
answer a band must not claim to.

**Answers band**: <metric name from docs/sdlc/bands.yaml>

## Problem

What is wrong, or missing, today. Written from the position of whoever noticed
it. No file names, no API shapes — if this section names a module, it has drifted
into `plan.md`.

## Who feels it, and when

The situation that produces the problem. "Every time a session is restored from
Codex to Claude" is useful; "users" is not.

## Desired outcome

What would be true afterwards, stated so that it can be observed. Not the
mechanism.

## Constraints this change inherits

Only the ones that actually bind. Delete the rest.

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is on real machines — a shape change needs a migration and
  a `STORE_SCHEMA_VERSION` bump.

## Systems likely affected

A guess, to be corrected by `spec.md`. Name modules from the map in `CLAUDE.md`.

## Open questions

The things that must be answered before a spec can be written. Each one names
who or what answers it.

## Not in scope

What a reader might reasonably assume is included, and is not.
