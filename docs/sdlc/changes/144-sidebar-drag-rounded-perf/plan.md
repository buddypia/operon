# Plan: sidebar tab button rounded consistency, drag-and-drop affordance, and smooth rendering

- **Spec**: `./spec.md`
- **Status**: approved

## Files that change

- `src/glyphs.rs`:
  - Add `ICON_DRAG_HANDLE` (`icons::DOTS_SIX_VERTICAL`) and register in `ICON_VOCABULARY`.
- `src/ui/widgets.rs`:
  - Add `splitter_grip_h` and `splitter_grip_v` helper functions for drawing splitter drag handles and hover states.
- `src/ui/session_tree.rs`:
  - Apply `ICON_DRAG_HANDLE`, `RADIUS_CONTROL`, and `CursorIcon::Grab` to draggable inspector tabs.
  - Optimize `session_file_tree_cache` so that scanning state does not trigger redundant frame evaluations.
- `src/ui/files.rs`:
  - Add viewport visibility culling in `file_tree_rows` using `ui.is_rect_visible`.
- `src/app/screens.rs`:
  - Draw splitter grip affordances in `ui_session_column` and `ui_terminal_panel`.
- `src/tests.rs`:
  - Add unit tests for tab corner radius, icon vocabulary, splitter response, and viewport culling.
- `docs/sdlc/changes/144-sidebar-drag-rounded-perf/state.yaml`:
  - Pipeline state tracking.

## Order of work

1. **Icon and widgets foundation**:
   - Add `ICON_DRAG_HANDLE` to `src/glyphs.rs` and update `ICON_VOCABULARY`.
   - Add `splitter_grip_h` and `splitter_grip_v` in `src/ui/widgets.rs`.
2. **Tab rounded corners and drag affordance**:
   - In `src/ui/session_tree.rs`, update tab buttons to use `RADIUS_CONTROL`, `ICON_DRAG_HANDLE`, and `CursorIcon::Grab` on hover.
3. **Splitter drag handle rendering**:
   - In `src/app/screens.rs`, use `splitter_grip_h` and `splitter_grip_v` on vertical and horizontal splitters.
4. **Rendering performance optimization**:
   - In `src/ui/session_tree.rs`, refine file tree caching to prevent per-frame churn while scanning.
   - In `src/ui/files.rs`, add visibility culling in `file_tree_rows`.
5. **Tests and Verification**:
   - Add targeted tests in `src/tests.rs`.
   - Verify `cargo fmt --check` and `cargo clippy --locked -- -D warnings`.
   - Verify `cargo test --locked` for new and existing tests.
6. **Review, land, and package**:
   - Check review with `scripts/check-review.sh`.
   - Land changes onto `main` and package with `scripts/package-macos.sh`.

## Proof of completion

- `test_sidebar_tab_buttons_use_control_radius` passes, proving tabs render with `RADIUS_CONTROL` when selected.
- `test_drag_handle_icon_resolves_and_vocab_registered` passes, proving `ICON_DRAG_HANDLE` resolves from the icon font and satisfies font tests.
- `test_file_tree_rows_culls_invisible_nodes` passes, verifying that off-screen rows skip widget layout.
- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` pass cleanly.
- `bash scripts/check-readiness.sh docs/sdlc/changes/144-sidebar-drag-rounded-perf` exits 0 (Go).

## Departures from the plan

None so far.
