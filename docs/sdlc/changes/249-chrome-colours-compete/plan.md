# Plan: neutral dark application chrome

- **Spec**: `./spec.md`
- **Status**: done

## Steps

1. Approve intent/spec/screen/plan via independent evaluator and readiness gate; commit before source changes.
2. Add named neutral chrome tests, observe first requirement fail against current palette; change DARK_PALETTE and DESIGN.md together; add the fine selected nav underline and neutral wordmark.
3. Run named palette/readability/ANSI/documentation tests, q.fast, bands and mutation sweep; independent empirical verifier attacks evidence.
4. Record evidence, commit and push, wait for CI, merge from main, clean up branch/worktree, package and install per ship.

## Proof of completion

Existing contrast tests protect readability across themes. Semantic-preservation test compares untouched dark role values and other palettes against main's baseline. Mutation restores blue selection, changes orange accent to blue, and removes the selected nav underline; each requires its guard to fail. Render actual egui controls using current palette and inspect their output. No runtime work added. Main may move concurrently; refresh and recheck landing diff if needed.

## Departures from the plan

User revised the initial all-grey proposal to grey chrome with orange accents and approved it with 「進めて」; specification and screen were updated before implementation.


Implementation details and evidence:

- Active session/file workspace tabs also get the same 2px orange bottom rule, exercising both selected sessions with `active_workspace_tab_has_an_orange_rule`; this applies the approved selected-tab cue to the existing workspace surface.
- `dark_chrome_uses_neutral_roles` failed on the old blue row-selected value (0 != 92); the navigation and workspace rule tests failed because no rule existed. All passed after implementation. Ten targeted palette/geometry/focus/ANSI tests passed, and the actual topbar wordmark/primary-action paint test passed.
- `make q.fast` passed. Test count rose from 1163 to 1168; ignore count remained 8. Restore/archive code is untouched, so live-CLI ignored tests do not apply.
- Bands: unchanged src/app.rs is 9529 lines on main and this branch, above its pre-existing 9400 diagnose threshold. Steering bytes and approval escapes are warning-level observations. No gate or threshold changed.
- Native agent-desktop window capture was refused because Accessibility is denied. Native macOS screencapture succeeded using the observed Operon window ID, allowing rendered visual comparison without changing permissions.
- Light and High Contrast palette tables remain byte-identical; only their shared navigation drawing gains the same selected-tab indicator. No formal method is required for the low-risk drawing surface; the empirical verifier and rerunnable obligations cover the finite palette and emitted controls.

- Independent empirical verifier re-ran all 10 obligations in its isolated worktree in 248 seconds: all held, including seven mutations caught and restored. Additional paint attacks across all three themes confirmed no accent rule under the inactive workspace sibling and confirmed focused nav ring paints after the accent line. No reproduced defect or surviving mutation. Author final q.fast and 8 existing/new regression tests passed after restoration.
- The release-binary contract answers the template's classification question (yes, source changes require packaging); packaging and installation execute from main after landing, with the installed-build guard providing fresh runtime delivery evidence. This record does not claim the currently running app has already been replaced.
