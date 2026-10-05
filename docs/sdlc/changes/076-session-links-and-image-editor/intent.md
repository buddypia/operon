# Intent: open generated files from terminal via Cmd+click, make session tree icons interactive, and preview images in editor

- **Status**: approved
- **Opened**: 2026-09-24

## Problem

1. **Terminal file link resolution failure for generated files**:
   When coding agents (Claude, Codex, Antigravity) print file paths or generate new files in the terminal, clicking the path via ⌘ (Command) + click often fails with a "file not found" notice. This occurs because the terminal scanner probes file paths on appearance; if probed before generation completes, `None` is permanently cached and never invalidated or re-checked upon click.
2. **Session file tree icons cannot be clicked to toggle or open**:
   In the session view's right-hand file tree, folder disclosure triangles, folder icons, and file icons are passive labels rather than clickable interactive targets. Users tapping directly on folder or file icons cannot collapse/expand directories or open files, violating expected GUI tree affordances.
3. **Binary rejection for images in built-in editor**:
   When image files (such as generated charts, icons, UI mockups, or screenshots in PNG, JPEG, GIF, WebP) are opened via Cmd+click or selected from the file tree, the editor displays an error stating "バイナリファイルはエディタで開けません。" rather than presenting an in-editor visual image preview.
4. **Session file tree navigation into editor**:
   Clicking a file row or icon in the session tree should reliably navigate to the editor in the project view, opening the requested file in a new editor tab with full preview capability.

## Who feels it, and when

- Anyone using coding agents in Operon sessions who asks the agent to create or modify files (code, documents, or graphics) and expects to inspect the generated output by Command+clicking the path printed in the terminal or selecting it from the session file tree.
- Anyone navigating nested project structures in the session file tree who expects clicking the folder icon or disclosure chevron to expand or collapse directory levels.

## Desired outcome

1. **Reliable Cmd+click file opening**:
   Command+clicking (or Option+clicking) any valid path in terminal output immediately resolves and opens the target file in the built-in editor, even if the file was created after the text was printed to the terminal.
2. **Interactive file tree icons**:
   Clicking disclosure chevrons or folder icons toggles directory expansion; clicking file icons opens the document in the editor.
3. **In-editor image preview**:
   Image files (PNG, JPEG, GIF, WebP) render inside the built-in editor in Preview mode, showing visual image dimensions, file size, aspect-ratio-scaled rendering with zoom/scrolling, and external system opener button.
4. **Seamless navigation to editor**:
   Clicking a file in the session tree opens the file in the project's Files/Editor view in an active tab without losing worktree context.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted store schema changes; store schema remains unchanged.
- Backward compatibility with existing editor modes (Edit, Markdown Preview, Git Diff).

## Systems likely affected

- `src/ui/terminal.rs` (terminal target extraction, link overlay probing & fallback resolution)
- `src/ui/files.rs` (tree node icon click interaction)
- `src/ui/session_tree.rs` (session file tree actions)
- `src/git.rs` (file opening outcome, image detection, byte loading)
- `src/models.rs` (file open outcome model if needed or extended)
- `src/app.rs` (open terminal target fallback resolution, document representation)
- `src/app/screens.rs` (in-editor image preview rendering)
- `Cargo.toml` (image crate features for JPEG, GIF, WebP)

## Open questions

All answered during initial alignment:
- Navigation destination: Navigates to project's Files tab containing the full built-in editor and file tabs.
- Image format support: Supports PNG as well as JPEG, GIF, and WebP.

## Not in scope

- Editing pixel data or writing modified image bytes from within Operon.
- Video or audio media playback.
- Converting Operon to an arbitrary multi-window application.
