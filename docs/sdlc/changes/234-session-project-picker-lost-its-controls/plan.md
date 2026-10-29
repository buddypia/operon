# Plan: restore project navigation around central tabs

- **Spec**: `./bug.md` (bugfix skips intent and spec)
- **Approved**: 2026-10-30, independent rust_review evaluator
- **Status**: done

## Files that change

| File | Change |
|---|---|
| src/app.rs | Project selection/central focus consistency and live-session tab population, preserving opened files and layout. |
| src/app/screens.rs | Restore project cards, counts, per-project launch and existing project drag ordering while retaining central tab renderer. |
| src/tests.rs | Pointer regressions for real project controls, live-only population, focused selection and preserved document/split state. |
| docs/sdlc/lessons.md | Record the UI scope regression with a guard watched failing. |

## Order of work

1. Use the user-confirmed project-scoped tabs: save/restore each project's existing WorkspaceTabs in memory, retaining buffers globally.
2. Commit plan/screen/bug records before source edits; run readiness.
3. Add focused integration/pointer guards and observe them fail for the named causes.
4. Restore existing project navigation affordances and synchronize tab/project context; reuse WorkspaceTabs and existing arrangement operations.
5. Run named workspace/history guards and q.fast, bounded mutations, native rendered inspection, independent review and branch CI.
6. Merge/push main, package and install using the existing authorized update procedure, verify CURRENT, close records and clean worktree/branches.

## Risks and proof

Opening historical sessions is caught by a mixed-status fixture. Losing project
controls is caught through actual whole-workspace rendered coordinates and
pointer actions. Losing file buffers/layout on project switches is caught by
round-trip state assertions. Selection synchronization is tested for both tab
kinds. No guard should demand that left-side session rows return: only project
navigation belongs there, while running sessions and files occupy central tabs.

## Departures from the plan

Whole-workspace pointer checks required separate project-navigation and History
drag payloads, preserving both callers' release handling. Startup initializes
workspace context without closing History. Independent review added multi-frame
launcher-target and hidden-History unread guards; both were observed failing,
and corrected without changing the agreed layout. A first order mutation was
not a defect because reversing fixture order coincidentally matched its saved
order, so it now removes ordering rather than accidentally keeping it.
