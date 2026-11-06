# Plan: compact session status badges

- **Status**: done

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

The first geometry guard assumed a badge within 35px of the painted title. Observed egui output placed the second badge at x=533 and its short title at x=610.4 because the existing title is centred in a 160px cell. Corrected the guard to the existing cell width without changing product layout. The changed no-permanent-word guard was observed failing before implementation, then passing.

Inspected the generated geometry in a monochrome sheet at native 18px and 3x size. All eleven central marks and their common enclosure are readable. Actual workspace primitives and hover behavior are covered by the named tests; no running user application was terminated.

Band investigation: `largest_module_lines` is 9529 in src/app.rs on both main and this branch. This change does not touch that module. Existing steering/escape warnings are recorded; no new band policy is introduced.

Tooltip-test investigation: pinned egui 0.31.1 permits only one tooltip per layer based on the previous pass and defaults to showing only when the pointer is still. The test now disables only that timing behavior and allows a handoff frame; production hover timing is unchanged. R1, R2, R3 and workspace pointer tests passed with CARGO_INCREMENTAL=0. A disk-full compilation was avoided with this build setting, then the user explicitly authorized removal of this worktree's generated incremental cache.

Independent empirical verification: empirical_verify_status applied the branch snapshot in an isolated scratch worktree and ran all six recorded obligations with CARGO_INCREMENTAL=0. Exit0 in186s: all five test obligations passed and all six deliberate mutations were caught. The wrapper retained the success result, not individual panic text; no claim is made about unavailable detailed logs. No production code changes were requested by verification.

CI42262424249 passed static checks and all six verification obligations. Full-suite result was1162 passed,1 failed,6 ignored: the documentation path guard interpreted the combined file-and-symbol reference in this spec as a nonexistent path. Corrected the reference to the actual ui_tab_workspace symbol and a separate existing file path; no production code changed.

Added the existing documentation path guard as I4 so it runs with the focused obligations as well as the full CI suite. The first local invocation reused a binary compiled in the now-removed verifier checkout because of the shared build cache; forcing a rebuild by updating the source timestamp corrected its embedded manifest root without changing source bytes.
