# Spec: open generated files from terminal via Cmd+click, make session tree icons interactive, and preview images in editor

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Terminal Path Cmd+click Dynamic Resolution**:
   When a user clicks on an underlined terminal path or uses Command/Option+click, `open_terminal_target` must dynamically attempt resolving the path against the session worktree/project root if it is not currently resolved in `resolved_paths` (or if it was previously cached as `None` before file creation).
2. **Session and Project File Tree Icon Clickability**:
   In `file_tree_rows`, clicking on disclosure icons (`ICON_DISCLOSURE_OPEN`, `ICON_DISCLOSURE_CLOSED`) or folder icons (`ICON_FOLDER_OPEN`, `ICON_FOLDER_CLOSED`) must dispatch `FileTreeAction::Toggle`. Clicking on file icons (`file_tree_icon`) must dispatch `FileTreeAction::Open`.
3. **In-Editor Image Preview**:
   `read_file_for_editing` must distinguish decodable image files (PNG, JPEG, GIF, WebP, BMP, ICO) from arbitrary unopenable binary files, returning a decodable image outcome or loading image bytes. The editor (`ui_editor`) must render an image preview card showing image dimensions, file size, scaled responsive preview with dark/light mode checkerboard/card contrast, and an external opening button.
4. **Image Dependency Support**:
   Enable `image` crate features for `"jpeg"`, `"gif"`, and `"webp"` in `Cargo.toml`.
5. **Session File Tree Navigation to Editor**:
   Opening a file from the session file tree must reliably open the document in the project Files tab, activating the tab and displaying its content (code, markdown preview, or image preview).

## Behaviour

- **Terminal Click**:
  - User sees a file path printed in terminal output (e.g., `<assets/output.png>` or `<src/lib.rs>`).
  - When holding ⌘ (or ⌥) and hovering, the cursor turns to a pointing hand.
  - Clicking opens the target file in the built-in editor. If the file was created after the text appeared, it dynamically resolves instead of failing.
  - If the path genuinely does not exist or falls outside the project/worktree boundary, a clear notice is shown: `ファイルが見つかりません: <path>` or `<path> はプロジェクトの外にあるため、エディタで開けません。`.
- **File Tree Interaction**:
  - Clicking the chevron or folder icon expands or collapses that folder level.
  - Clicking the file icon or filename opens the file in the editor.
- **Image Preview in Editor**:
  - Image files open with the `Preview` tab selected by default.
  - The editor header displays the file name, external app button (`既定のアプリで開く`), reload button (`ディスクから読み直す`), and image metadata (dimensions in px, file size in KiB).
  - The image preview renders within the available space while preserving aspect ratio.
  - Non-image binary files continue to display: `バイナリファイルはエディタで開けません。`.

## Design

1. **Terminal Link Resolution (`src/ui/terminal.rs`, `src/app.rs`)**:
   - `open_terminal_target`:
     When `TerminalTargetKind::Path { path, line }` is triggered, if `self.resolved_paths.get(&path)` is `None` or `Some(None)`, synchronously invoke `resolve_terminal_path(&worktree, &path)`. If found, insert into `resolved_paths` and proceed to open the file.
   - `terminal_row_links`:
     Allow re-probing of paths if they were cached as `None` when new terminal activity occurs or periodically, avoiding permanent cache starvation.
2. **File Tree Interactive Rows (`src/ui/files.rs`)**:
   - In `file_tree_rows`, wrap folder icons (disclosure chevron and folder glyph) with click sensing (`ui.add(egui::Label::new(...).sense(egui::Sense::click()))`). If clicked, push `FileTreeAction::Toggle(node.path.clone())`.
   - Wrap file icon with click sensing. If clicked, push `FileTreeAction::Open(node.path.clone())`.
3. **Image Loading & Outcome Model (`src/git.rs`, `src/models.rs`, `src/app.rs`)**:
   - Extend `FileOpenOutcome` in `src/models.rs`:
     ```rust
     pub(crate) enum FileOpenOutcome {
         Text(String),
         Image {
             bytes: Vec<u8>,
             width: u32,
             height: u32,
         },
         Unopenable(String),
     }
     ```
   - In `read_file_for_editing`:
     If file is binary (or extension matches known image formats), attempt decoding image dimensions via `image::load_from_memory`. If successful, return `FileOpenOutcome::Image { bytes, width, height }`. If unsuccessful, return `FileOpenOutcome::Unopenable("バイナリファイルはエディタで開けません。")`.
   - In `OpenDocument`:
     Store `image_data: Option<ImageDocumentData>` where `ImageDocumentData` holds `width`, `height`, and cached texture or bytes.
4. **Editor Screen Image View (`src/app/screens.rs`)**:
   - In `ui_editor`:
     If the document is an image, display an image preview pane showing:
     - Dimension and size info chip: `tf!("{w} × {h} px · {size} KiB", ...)`
     - Centered image view using egui texture / `egui::Image` scaled to fit available height/width with aspect ratio preservation.
     - Toggle view shows `Preview` (disabled `Edit` and `Diff` for binary images).

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | Yes | Uses existing palette colours (`palette.card`, `palette.raised`, `palette.border_subtle`, `palette.text_muted`). No new palette keys needed. |
| Icons | Yes | Uses existing `ICON_PREVIEW`, `ICON_FOLDER_OPEN`, `ICON_FOLDER_CLOSED`, `ICON_DISCLOSURE_OPEN`, `ICON_DISCLOSURE_CLOSED`, `ICON_OPEN_EXTERNAL`, `ICON_REFRESH`. |
| Identifier SSOT | No | No shared string identifiers across components introduced. |
| Durability | Yes | In-memory document and UI state only. SQLite schema untouched; `STORE_SCHEMA_VERSION` unchanged. |
| Subprocess safety | Yes | Uses existing `resolve_project_file`, `resolve_terminal_path`, and `open_path`. |
| Documentation | Yes | Japanese UI copy added in `src/i18n_tables.rs`. |
| Local-first | Yes | Entirely local decoding via `image` crate; zero network or external telemetry. |
| Budgets | Yes | Adheres to `EDITOR_FILE_MAX_BYTES` (500 KiB default or bounded image memory ceiling). |

## Flagged concerns

None. All interactions conform to existing patterns and user confirmation was obtained for image format coverage and navigation flow.

## Acceptance

- `cargo test --locked` passes, including new tests:
  - `test_session_file_tree_icon_clicks_toggle_and_open`
  - `test_open_terminal_target_dynamic_fallback_resolution`
  - `test_read_file_for_editing_image_outcome`
  - `test_in_editor_image_preview_rendering`
- In running app:
  - Creating a file via CLI in terminal and Command+clicking path immediately opens in editor.
  - Clicking folder chevron/icon in session file tree expands/collapses folder.
  - Clicking file icon in session file tree opens file in editor.
  - Opening PNG/JPEG/GIF/WebP image renders visual image preview with dimensions and file size.

## Rejected alternatives

- Separate top-level "Editor" page in main navbar: Rejected because it fragments navigation from project context, worktree association, and git diff tracking.
- Video/audio player support: Rejected to keep binary footprint and memory usage lean.
