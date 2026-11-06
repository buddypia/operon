# Plan: readable state at the beginning of session tabs

- **Spec**: `./spec.md`
- **Approved**: 2026-11-07
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| src/glyphs.rs | Recognizable idle and running marks. |
| src/app/screens.rs | Visible translated state before the tab title. |
| src/tests.rs | Icon mapping and rendered geometry/state transition guards. |
| DESIGN.md | Describe pause and circular arrows. |

## Order of work

1. Add guards, observe failures against current implementation.
2. Update two glyph constants and tab state text with existing egui APIs.
3. Run named tests and existing workspace pointer guards, mutation sweep and make q.fast.
4. Record obligations; independent empirical verifier re-runs them in a separate worktree.
5. Commit, push, wait for CI, merge from main, clean branch/worktree, package installed build.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| State text is truncated or on another line | Same-name tabs remain ambiguous | Rendered geometry guard. |
| A cached status stops updating | Busy appears idle | Transition guard in the same app/context. |
| Global marks affect rows | Rows show different shapes | Shared constants and bundled-font test. |

## Proof of completion

R1–R3 named tests pass; mutation sweep catches every removal; make q.fast and CI pass. Installed build matches main. Verify native rendering when installation can safely proceed.

## Departures from the plan

The test guards were authored just before the plan commit; production implementation remained untouched until after the plan was committed.

The first geometry guard incorrectly assumed short titles were left aligned. Drawn text showed the state at x303 and the short title at x432, centered within its existing 160px slot. The guard now allows that slot while still requiring icon, state and title in left-to-right order on the same tab row.
