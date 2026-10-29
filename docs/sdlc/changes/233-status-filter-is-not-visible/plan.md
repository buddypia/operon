# Plan: Preserve history access after central tab migration

- **Spec**: `./bug.md` (bugfix route skips intent/spec)
- **Approved**: 2026-10-30, independent rust_review evaluator
- **Status**: done

## Files that change

| File | Change |
|---|---|
| src/app.rs | Home status entry opens the filter's history/list home. |
| src/app/screens.rs | History renders one chosen list/grid with accessible existing session controls, without duplicate full-height content. |
| src/tests.rs | Migrate old sidebar and global-jump assertions to the requested layout while preserving their behavioral guarantees; pin Home entry and new sidebar bounds. |

## Order of work

1. Reproduce the named CI-failed tests and the missing Home filter entry.
2. Restore Home filtered-list entry and one history list/grid renderer; preserve advanced row operations and search.
3. Update tests whose old placement assumptions contradict the explicit project-only-left request, keeping operation/identity/bounds assertions.
4. Run all 28 named failures plus workspace guards and q.fast; independently review, push and wait for full CI before landing the stacked changes.
5. Package/install only after the corrected source lands; preserve 232 release tracking until installation.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Test adaptation hides regression | A previously accessible action vanishes. | Retain pointer actions and visible content checks in history, new project-only check. |
| Duplicate history content clips grid | Controls draw outside center. | Existing grid bounds and empty-project pointer tests. |
| Jump loses target identity | Other file scrolls. | Updated target-map assertions plus 232 multi-group jump guard. |

## Proof of completion

All 28 named CI failures pass with their behavior guarantees preserved; Home status entry exposes relevant sessions and filter controls; q.fast passes; independent review and full CI pass before merge. No source build is installed with the known regression.

## Departures from the plan

Native rendering exposed an additional grid-header width defect for long project
names, so the name is bounded/truncated while keeping add/open controls visible.
Project selection now exits history to reveal the newly opened tabs, and the
project-column new-session action explicitly uses the selected project rather
than the project of an older active session tab. The old simple list/card were
removed after tracing their controls to the advanced history rows and Home's
existing details menu. Preserving both title navigation and double-click rename
required a small egui-time pending-click state; no worker or durable state was
added. These are recoverable corrections to the intended navigation and existing
control reachability, with focused interaction tests.

The pre-merge gate refused the original pending release command although ship
requires the main checkout after merge. Records now waive pre-merge execution
only, with its reason, while retaining installed-app as a mandatory pending
human contract. Actual main-checkout packaging, authorized replacement, launch
and installed-build CURRENT remain required before goal completion.
