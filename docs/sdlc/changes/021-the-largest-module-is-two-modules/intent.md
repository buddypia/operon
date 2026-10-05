# Intent: the largest module has been recorded as too large three times running

- **Status**: approved
- **Opened**: 2026-09-05

Written after the fact. The `refactor` route in `docs/sdlc/routes.yaml` runs
`build,test` and does not ask for an intent — the bug report or, here, the band
reading is the intent. `scripts/check-readiness.sh` applies that rule to
`spec.md` and not to `intent.md`, so a route that legitimately has neither is
blocked for missing one. This file is the smaller correction; the checker's
inconsistency is recorded in `./plan.md` for whoever touches it next.

## Problem

`largest_module_lines` had been reported as breached in three consecutive
commits: 8862, then 9078, then 9269, against a warn tier of 8800.
`docs/sdlc/bands.yaml` calls that tier "recorded, no action". Three records in a
row is the point at which a record stops being information and becomes a habit —
the reading is still true, nobody is acting on it, and each new change makes it
slightly less true that anybody will.

`src/app.rs` was also the file every change had to be read in, and three
quarters of it was drawing.

## Who feels it, and when

Anybody opening `src/app.rs` to change anything, which is most changes; and the
next reading of the band, which would have said the same thing a fourth time.

## Desired outcome

The metric is inside its band again, and it got there by the code being in the
right place rather than by the band being moved. Behaviour is provably
unchanged: the same tests pass, in the same number, and no moved line differs
from the line it replaced.

## Constraints this change inherits

- A refactor whose tests changed is not a refactor.
- A new top-level module is a line in `src/main.rs`, which `docs/sdlc/risk.yaml`
  holds at `paused`.

## Systems likely affected

`src/app.rs`, and a new child module beside it.

## Not in scope

- Anything that changes what the application does.
- Splitting the rest of `src/app.rs` further. One seam, taken where the file
  already had one.
