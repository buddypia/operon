# Intent: dead interface code nobody can reach

- **Status**: approved
- **Opened**: 2026-09-04

## Problem

A whole settings interface — launch presets, launch option controls, and
several shared widgets — is still in the tree but reachable from nowhere in
the product. It is exercised only by its own tests, and the lint that would
say "nothing calls this" has been silenced at each definition. This is the
shape an earlier incident already recorded: a widget written for a bug that
shipped beside the screen instead of in it, invisible because the test
proved the widget and nothing proved the caller.

## Who feels it, and when

Anyone reading or changing the surrounding code. Every search for where a
launch control is drawn lands on two answers — the live one and the dead
one — and telling them apart takes the same audit that produced this change.
The next person to reuse one of the dead widgets wires a live screen to a
path nothing else walks.

## Desired outcome

The unreachable interface and its helpers are gone, the suite is green with
fewer tests rather than quieter ones, and the persisted preset records stay
exactly as they are — data nobody else has a copy of is not deleted because
its editor went away.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is on real machines — the preset records stay; only the
  unreachable editor goes. No schema change, no migration.
- Behaviour must be provably unchanged: a refactor whose tests changed meaning
  is not a refactor. Tests that pin removed code go with it, openly, through
  the documented removal hatch — not quietly.

## Systems likely affected

A guess, to be corrected by `plan.md`. The app state and its drawing code,
the agent launch controls, the shared widgets, the icon vocabulary, and the
inline suite.

## Open questions

None. The inventory was audited call-site by call-site before this was
written; the plan names every item.

## Not in scope

The persisted preset records themselves, and any new interface to edit them.
Filling the missing translations elsewhere in the tree. The steering-bytes
diagnosis, which is read-only work under a different band.
