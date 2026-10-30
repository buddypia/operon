# Plan: one tooltip line per truncated label

- **Spec**: `./bug.md` (bugfix route: the bug report is the intent)
- **Approved**: 2026-10-30, independent evaluator
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| src/app/screens.rs | Drop hovers that repeat a truncated label's own text: session tab, project name (project page, session column), terminal header title without a goal. |
| src/tests.rs | `a_truncated_session_tab_names_its_title_once_on_hover`. |

## Order of work

1. Write the test; watch it fail on the current code.
2. Make `workspace_tab_title` return no hover for a session tab and add the hover only when present; remove the three sibling repeats.
3. `make q.fast`, the new test and the workspace tab tests; independent review; CI on the branch, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A tab that fits loses its only tooltip | Hover shows nothing | Intended: the whole title is visible; egui adds it back once elided. |
| Document tab loses its path | No path on hover | Document hover kept; the review reads the diff. |

## Proof of completion

The new test passes and failed before the fix; `make q.fast` and CI pass.

## Departures from the plan
