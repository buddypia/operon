# Native workspace rendering evidence

## Isolation and command

The native harness and its synthetic project, source file, store, Cargo manifest,
and lockfile live entirely in a directory created by `mktemp -d`:

`/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.HOoCwHQyWI`

The harness symlinks the worktree source modules and assets, reuses its dependency
build cache, and renders the production `ui_terminal_workspace` inside an eframe
window named **Operon Workspace Preview**. Its own main replaces normal startup:
no live Operon store is opened, no instance is closed, and no account or real
agent session is operated. Synthetic sessions use unique preview tmux names and
Exited status. Notifications are disabled. The wrapper calls the workspace
renderer, rather than `OperonApp::update`, so the real app's polling and launch
loop is not started.

From the feature worktree:

```sh
cargo run --locked \
  --manifest-path /var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.HOoCwHQyWI/Cargo.toml \
  --bin operon-workspace-preview --target-dir target
```

The latest run rebuilt the harness from the current worktree source, printed
both screenshot paths, and exited 0. The wrapper requested native pixels with
`ViewportCommand::Screenshot`, saved the returned `Event::Screenshot` image,
then closed its own viewport. No browser or desktop screenshot automation was
used, and no preview process was left running.

## Actual rendered images

- Horizontal: `/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.HOoCwHQyWI/workspace-horizontal.png`
- Vertical: `/var/folders/29/1fj7pwg51hb3g8s6lyy_rldw945t4y/T/tmp.HOoCwHQyWI/workspace-vertical.png`

Both images were opened with `view_image`, not judged from the save result.
The window uses a 1440 by 900 logical-pixel viewport. The left column shows
プロジェクト, its project entry, and navigation actions. Two session tabs occupy
the central session group; workspace.rs is a document tab in another group.
The right inspector remains visible with ファイル, 会話 and 変更 controls.

Horizontal: the session and source editor divide the center into equal left and
right halves, with a visible vertical divider handle. The compact session header
separates agent/status/branch from the menu and close controls. The source editor
shows line numbers, highlighted Rust and edit/diff/save controls without crossing
the group boundary. An initial overlapping full session header was found in
native pixels and corrected before the latest run.

Vertical: session tabs and terminal occupy the upper half, with the file tab and
source editor below. The horizontal divider and its handle are visible. Tab
strips, terminal content and editor controls remain within their respective
group rectangles. The compact header's visible agent/status icons and action
controls are separated; this image does not establish all metadata labels.

## Limits of this evidence

The harness seeded tab state and moved the document through the production
WorkspaceTabs model to make both split arrangements. It did not simulate a
pointer drag, type into a live agent, or prove save/dirty-close behavior. Pointer
interaction, navigation, unread visibility and buffer protection are checked by
the separate named egui/integration tests recorded with the change. Native
screenshots establish the drawn geometry and readability of these representative
states, not every interaction or window size.

## Independent review regressions

The initial review identified parser-cache eviction between visible files, an untargeted line jump, and Escape/release ordering. `workspace_visible_documents_do_not_reparse_unchanged_frames` was observed failing with source syntax builds 8 vs 4 and Markdown builds 10 vs 6 before cache loans. It passes after per-group cache loans. `workspace_line_jump_waits_for_its_target_document` verifies that an earlier editor cannot spend a loading target's pending jump, then that the loaded target consumes it. Actual pointer Escape/release was observed reordering before its condition was fixed. The independent reviewer confirmed all three fixes against the staged index before final formatting.

Final verification: `make q.fast` passed. The workspace-filter run passed 17 tests, targeted file-open and branch-tab checks passed, 28 existing file/navigation tests passed, and the official bounded sweep caught 13/13 mutations across model, geometry, cache, jump, cancellation and user-facing claims. Test counts grew from 1061 to 1075; ignore count remains 8. Independent final approval binds staged digest 4024105707b46a6b. CI and installation evidence follows after source landing.
