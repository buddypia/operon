# Intent: file list sidebar should be on the left by default and sidebar contents draggable to move

- **Status**: approved
- **Opened**: 2026-10-06

## Problem

In Operon's session workspace, the file list (`ファイル` tab) is currently tucked into the right-side inspector panel, while the left sidebar is dedicated exclusively to the session list (`履歴`).
Standard modern IDE conventions (VS Code, Cursor, Zed) place the primary file tree on the left sidebar by default, which makes Operon's current layout feel inverted to developers who naturally look to the left for project files.
Furthermore, the sidebar panels and tabs are fixed to their designated sides: users cannot drag tabs or panel contents to move them between the left and right sidebars or reorder them, nor can they drag sidebar splitters to adjust panel widths freely.

## Who feels it, and when

Every developer supervising autonomous agents in Operon who wants an intuitive, familiar workspace layout where files are immediately visible on the left, and who wants the freedom to customize the workspace by dragging sidebar contents to their preferred positions.

## Desired outcome

1. In the session workspace, the file list (`ファイル`) is placed in the left sidebar by default, showing the active session's worktree/project file tree upon opening the workspace.
2. The session list (`セッション` / `履歴`) is also available as a tab in the left sidebar by default, allowing quick switching between the file explorer and session navigation.
3. The right sidebar contains the conversation timeline (`会話`) and git diff (`変更`) tabs by default, collapsible via the side panel toggle.
4. Users can drag any sidebar tab (`ファイル`, `セッション`, `会話`, `変更`) to move its contents between the Left and Right sidebars, or reorder tabs within a sidebar.
5. Users can drag the vertical splitters to resize the left and right sidebar widths smoothly.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese (via `tr()` and tables in `src/i18n_tables.rs`); code, comments, and docs are English.
- No change to the persisted store schema; layout states are managed cleanly in `OperonApp`.
- Maintain complete backwards compatibility with existing sessions, project configurations, and keyboard shortcuts.

## Systems likely affected

- `src/app.rs` (app state for sidebar layout, active tabs, widths, and drag tracking)
- `src/app/screens.rs` (terminal workspace layout, left/right sidebar rendering, splitter dragging)
- `src/ui/session_tree.rs` (tab rendering, tab dragging, drop targets, inspector layout)
- `src/i18n_tables.rs` (Japanese, English, Korean translations for tabs and hints)
- `src/tests.rs` (layout, drag-drop tab migration, default tab selection tests)

## Open questions

None. The user explicitly requested: "ファイル一覧のサイドバーはやはり左のサイドバーがデフォルトが良い。そしてドラッグしてサイドバーの内容が移動できると良い。"

## Not in scope

- Persisting complex multi-window dock layouts to disk across app reboots (in-memory session state is sufficient and avoids schema migrations).
- Modifying project file tree semantics or mutating filesystem files via drag-and-drop.
