# Plan: compact session status badges

- **Status**: approved

## Steps

1. Update actual workspace tests to require icon-only state and live tooltip text, observe the changed behavior guard fail on the current UI.
2. Generate a finite vector badge vocabulary in glyphs.rs, add its status widget helper, replace the tab glyph/word pair, and document it in DESIGN.md.
3. Exercise all fourteen kinds across three themes, run hover/interaction tests, mutation sweep and make q.fast. Inspect a rendered icon sheet and actual egui output for enclosure and readable geometry.
4. Record executable evidence, obtain isolated empirical verification, commit and push. Wait for CI, merge into main, push and clean up, package from main. Install if no user process needs terminating; otherwise report the precise remaining step.

## Risks and alternatives

Small inner marks can blur: use an 18px circle with 1.5px strokes and inspect at native size and enlarged. Avoid changing state classification or replacing shared status chips. Pointer hover must cover the full badge cell and retain the current label/hint. No per-frame parsing or I/O.

## Proof of completion

The R1–R3 tests in spec.md inspect real egui shapes and tooltips. Workspace pointer tests retain tab interactions. The mutation sweep falsifies each requested behavior and must catch every mutation. Full suite runs in CI.

## Departures from the plan

None.
