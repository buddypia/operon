# Intent: sidebar tab button rounded consistency, drag-and-drop affordance, and smooth rendering

- **Status**: approved
- **Opened**: 2026-10-11

## Problem

1. **Button rounded corner inconsistency**: In the session workspace, the sidebar tab buttons (「ファイル」, 「会話」, 「変更」) appear with sharp, square 90-degree corners when selected, while other buttons and filter chips throughout Operon use `RADIUS_CONTROL` (6px) rounded corners. In `egui 0.31`, `egui::Button::selected(true)` explicitly defaults to `CornerRadius::ZERO` unless overridden, breaking the visual consistency of the design system.
2. **Lack of drag-and-drop & resize affordances**: The session workspace supports dragging sidebar tabs between the left column and right dock, as well as dragging the horizontal and vertical splitters. However, the UI has no visual cues (such as grip icons or resize handles) indicating that the panel or tabs can be dragged, leaving users uncertain about whether panels can be manipulated.
3. **Sidebar opening lag and inefficient rendering**: When a session is newly opened, opening the file/conversation/changes sidebar causes noticeable stuttering (カクカクする) and dropped frames. The draw path executes recursive tree building (`build_file_tree`) and partition operations synchronously on the UI thread, repeatedly re-evaluates rebuilding every frame while background scans run, allocates new badge HashMaps every frame, and renders hundreds of unculled file rows inside the scroll area.

## Who feels it, and when

Anyone using Operon to supervise coding agent sessions:
- On selecting or opening a session, the workspace stutters as the sidebar loads.
- Looking at the sidebar, the selected tab button looks blocky, square, and out of place next to rounded controls.
- Attempting to adjust layout, users cannot discover that tabs can be dragged to dock on either side, nor that the hairline divider between sessions and inspector can be dragged to resize.

## Desired outcome

1. **Rounded button consistency**: All buttons, including inspector tabs when selected, consistently adopt `RADIUS_CONTROL` (6px) rounded corners, adhering strictly to `DESIGN.md` controls and shapes specification.
2. **Clear drag-and-drop affordance**: Add a recognizable grip icon (`ICON_DRAG_HANDLE`, Phosphor `DOTS_SIX_VERTICAL`) to draggable tabs, set `CursorIcon::Grab` on hover, provide descriptive tooltips, and draw visible centered grip handles on horizontal and vertical splitters so draggable elements are immediately obvious.
3. **Smooth, lag-free sidebar rendering**:
   - Cache file tree construction once per scan and avoid per-frame rebuild attempts while background scans are in flight.
   - Cull off-screen file rows in `file_tree_rows` so that large repositories with hundreds or thousands of files render smoothly at 60 FPS without layout overhead.
   - Eliminate per-frame heap allocations for git status badges and touched file queries.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese (via `tr()` and tables in `src/i18n_tables.rs`); code, comments, and docs are English.
- Conformance to `DESIGN.md` control sizes (`CONTROL_HEIGHT` = 28px, `CONTROL_HEIGHT_SMALL` = 24px) and corner radii (`RADIUS_CONTROL` = 6px).
- Any new icon must come from Phosphor regular weight and be registered in `ICON_VOCABULARY`.
- No change to the persisted store schema (`STORE_SCHEMA_VERSION` remains intact).

## Systems likely affected

- `src/glyphs.rs` (drag handle icon definition and vocabulary)
- `src/ui/widgets.rs` (splitter grip rendering helper, tab button styling)
- `src/ui/session_tree.rs` (rounded tabs, drag handle display, tree caching and splitters)
- `src/ui/files.rs` (viewport culling for file tree rows)
- `src/app/screens.rs` (splitter grip rendering and cursor feedback)
- `src/tests.rs` (regression tests for button rounding, icon resolution, drag affordance, and tree caching)

## Not in scope

- Adding new docking positions (e.g. floating separate OS windows).
- Mutating filesystem files via drag-and-drop.
