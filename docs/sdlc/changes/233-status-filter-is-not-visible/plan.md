# Plan: Preserve history access after central tab migration

- **Spec**: `./bug.md` (bugfix route skips intent/spec)
- **Approved**: pending independent evaluation
- **Status**: draft

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

None yet.
