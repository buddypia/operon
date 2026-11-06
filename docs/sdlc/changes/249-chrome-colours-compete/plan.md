# Plan: neutral dark application chrome

- **Spec**: `./spec.md`
- **Status**: approved

## Steps

1. Approve intent/spec/screen/plan via independent evaluator and readiness gate; commit before source changes.
2. Add named neutral chrome tests, observe first requirement fail against current palette; change DARK_PALETTE and DESIGN.md together.
3. Run named palette/readability/ANSI/documentation tests, q.fast, bands and mutation sweep; independent empirical verifier attacks evidence.
4. Record evidence, commit and push, wait for CI, merge from main, clean up branch/worktree, package and install per ship.

## Proof of completion

Existing contrast tests protect readability across themes. Semantic-preservation test compares untouched dark role values and other palettes against main's baseline. Mutation restores orange/blue chrome and requires neutral test failure. Render actual egui controls using current palette and inspect their output. No runtime work added. Main may move concurrently; refresh and recheck landing diff if needed.

## Departures from the plan

None.
