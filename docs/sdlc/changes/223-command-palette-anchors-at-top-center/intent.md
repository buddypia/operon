# Intent: the command palette opens centered near the top of the window

- **Date**: 2026-10-24
- **Route**: `modify`
- **Risk tier**: `low` (`autonomy: full`, `screen: yes`)
- **Status**: approved

## Problem

`OperonApp::ui_command_palette` (`src/app/screens.rs:451-456`) constructs `egui::Window::new(tr("検索・操作"))` without an `.anchor(...)` call and with `.default_width(420.0)`. Because `egui::Window` defaults to placing unanchored windows near the top-left corner `(16.0, 32.0)`, pressing `⌘K` opens the command palette directly over the left navigation rail and macOS window controls instead of centered near the top of the viewport where VS Code, Cursor, and every other modal in `src/app/screens.rs` place their overlays.

## Desired outcome

- Pressing `⌘K` (or opening `ui_command_palette`) anchors the `検索・操作` window at `egui::Align2::CENTER_TOP` with offset `[0.0, 68.0]` and `.default_width(520.0)`, so the command palette opens horizontally centered near the top of the viewport.
- All existing command palette behaviours — empty-query action listing (`an_empty_palette_query_lists_only_the_actions`), search ranking, keyboard selection clamping, and `Escape` / close-button dismissal — remain unchanged.

## Non-goals

- Changing which items appear when `command_search` is empty (pinned by `an_empty_palette_query_lists_only_the_actions`).
- Changing command palette keyboard shortcuts or ranking weights.
