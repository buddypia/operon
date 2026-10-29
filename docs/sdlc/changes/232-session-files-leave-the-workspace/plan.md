# Plan: Keep sessions and files in the central workspace

- **Spec**: `./spec.md`
- **Approved**: pending independent evaluation
- **Status**: draft

Committed before source implementation; independent evaluation follows against
the spec and the explicitly requested screen structure.

## Files that change

| File | Change |
|---|---|
| `src/ui/session_tree.rs` | Workspace tab/group model using existing PaneLayout geometry. |
| `src/app.rs` | App workspace state, tab navigation, visible terminal collection, file opens. |
| `src/app/screens.rs` | Central groups and tab strips, drag/drop, project-only sidebar, editor reuse. |
| `src/tests.rs` | Requirement-focused model and navigation tests. |
| `src/i18n_tables.rs` | All language rows only if new strings are needed. |
| Change directory | Evidence, approvals, mutation sweep and review results. |

## Order of work

1. Independently evaluate spec and plan, record request-authorized screen structure,
   and run readiness before editing source.
2. Write requirement tests, watch them fail, then add workspace groups/tab model
   reusing the existing splitter/drop behavior.
3. Integrate session and file opening, tab close and visible-terminal accounting.
4. Draw the project-only sidebar and central group tab strips; reuse existing
   terminal and file renderers; wire drag reorder, move, edge split and divider.
5. Verify named tests and q.fast, inspect runtime interactions, run bounded
   mutations, obtain independent review, push and watch CI.
6. Merge after CI, push main, clean up with the repository helper, and ship from
   main using the ship skill after the applicable bundle-swap authorization.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Wrong document/root identity | A worktree tab edits the project's copy. | Existing DocumentId and navigation tests. |
| Dirty edits lost while moving/closing | Buffer disappears or is re-read. | Dirty-close test and runtime exercise. |
| Closed tabs reopen automatically | Last tab cannot be closed. | Workspace close test and empty-state inspection. |
| Focus stays on hidden terminal | Typing reaches another session. | Visible-session model test and runtime switching. |
| Tabs orphaned after drag | Content becomes inaccessible. | Move/collapse/retain tests, real drag exercise. |
| Small window clips controls | Content or tabs cannot be reached. | Existing minimum pane rules and narrow-window inspection. |

## Proof of completion

- `make q.fast`: success; fmt/clippy and repository policy checks pass.
- `cargo test --locked workspace_`: all new requirement tests pass.
- New guards observed red before implementation or in bounded mutations.
- Runtime evidence covers all observations in spec Acceptance, including both
  split orientations, mixed session/document groups and unsaved file protection.
- Fresh independent review and successful CI on branch HEAD before merge.
- Installed build identity matches merged source; cleanup checked with real commands.

## Departures from the plan

None yet.
