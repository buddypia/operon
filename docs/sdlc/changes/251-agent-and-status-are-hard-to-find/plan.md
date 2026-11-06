# Plan: Recognisable agents and sidebar session filters

- **Spec**: `./spec.md`
- **Approved**: 2026-11-07
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| src/app.rs | Memory-only agent and waiting filters and common predicate/reset methods. |
| src/app/screens.rs | Sidebar controls and matching rows; share filters with history and grid; remove repeated agent words. |
| src/ui/widgets.rs | Accessible vendor image toggle with existing control/focus helpers if needed. |
| src/i18n_tables.rs | All language rows for new labels. |
| src/tests.rs | Behavioral combination and real pointer/render tests. |

## Order of work

1. Commit intent, spec, screen and this plan before source edits after independent evaluation.
2. Write the behavioral tests and observe failure; implement memory predicates and controls.
3. Exercise actual sidebar clicks, shared history/grid filters, ended sessions and reset.
4. Run named tests and q.fast, mutation sweep and independent empirical verification.
5. Push, wait for CI, merge/push main, clean worktree and package/install per ship.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Pinned sessions bypass CLI choice | A foreign CLI still appears | Combined predicate applied before pinned queue; tests. |
| Finished selection remains empty | Sidebar still omits ended rows | Actual sidebar row assertion. |
| Controls overflow column | Mark or label clipped | Wrapped layout render checks at normal width. |
| Home shortcuts retain stale filters | Count differs from opened list | Reset assertion. |

## Proof of completion

`cargo test --locked sidebar_251_`, existing sidebar/status tests, `make q.fast`, and the recorded mutation sweep succeed; CI succeeds at the pushed head. Native render/control verification backs the screen. Installed build stamp matches landed sources after packaging.

## Departures from the plan

None.
