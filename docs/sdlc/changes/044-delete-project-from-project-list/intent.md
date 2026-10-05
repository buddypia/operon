# Intent: Delete project from the projects list view

- **Status**: approved
- **Opened**: 2026-09-14

## Problem

Operon provides project removal only from inside an opened project's detail header (`削除…`). When viewing the main project list (`Page::Projects`), there is no way to remove or delete a project directly from its list card. A user with multiple registered projects or projects imported accidentally via workspace scanning must navigate into each project individually to remove it.

## Who feels it, and when

Anyone organizing their project collection on the Projects screen:
- After scanning a large workspace that brought in unwanted repositories.
- When managing projects that have been moved, deleted, or are no longer relevant.
- When cleaning up projects without wanting to enter their overview tabs.

## Desired outcome

A user viewing the project list on the Projects page can initiate project removal directly from each project's card (via a dedicated delete button on the row and via right-click context menu). The action presents an in-place confirmation card stating that only Operon records are removed while files on disk remain untouched. Confirming removes the project immediately, while canceling restores the normal card without entering the project.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Deleting a project removes Operon's local records (sessions, handoffs, native references) but leaves working directory files intact on disk.
- Active sessions running in the project block deletion until stopped.
- The project card row must use `clickable_card` to ensure inner action buttons receive clicks cleanly without triggering project navigation.

## Systems likely affected

- `src/app/screens.rs` (`ui_projects` list view)
- `src/ui/widgets.rs` (if any widget tweaks are needed)
- `src/i18n_tables.rs` (translation entries for new copy)
- `src/tests.rs` (UI behavior and regression tests)

## Open questions

None. The backend project removal logic `remove_project` already exists and is well-tested; only the list-level UI entry point and inline confirmation card are missing.

## Not in scope

- Deleting actual repository files or git worktrees on disk.
- Batch deletion of multiple projects simultaneously.
