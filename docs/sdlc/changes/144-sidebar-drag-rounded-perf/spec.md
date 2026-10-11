# Spec: sidebar tab button rounded consistency, drag-and-drop affordance, and smooth rendering

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `test_sidebar_tab_buttons_use_control_radius`: Inspector tab buttons (`ファイル`, `会話`, `変更`) in `src/ui/session_tree.rs` explicitly specify `.corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))`, ensuring that when selected (`Button::selected(true)`), they render with standard 6px rounded corners instead of falling back to `CornerRadius::ZERO`.
2. `test_drag_handle_icon_resolves_and_vocab_registered`: Add `ICON_DRAG_HANDLE` (`icons::DOTS_SIX_VERTICAL`) to `src/glyphs.rs` and register it in `ICON_VOCABULARY`. Draggable sidebar tabs render with the `ICON_DRAG_HANDLE` icon prefix and display `CursorIcon::Grab` on hover, providing intuitive visual affordance that the tab can be dragged to switch sides.
3. `test_splitter_resize_grip_affordance`: The horizontal splitter between the session list and the side panel in `src/app/screens.rs`, as well as the vertical splitters, render a centered grip handle indicator (pill or grip dots) and highlight on hover with `CursorIcon::ResizeVertical` or `CursorIcon::ResizeHorizontal`, making resize capabilities discoverable at rest.
4. `test_session_file_tree_caching_avoids_redundant_rebuild`: In `src/ui/session_tree.rs`, `session_file_tree_cache` caches both the visible and hidden tree partition without attempting repeated synchronous rebuilds while a background file scan is still pending.
5. `test_file_tree_rows_culls_invisible_nodes`: In `src/ui/files.rs`, `file_tree_rows` checks viewport visibility against `ui.clip_rect()`, skipping layout and label rendering for off-screen rows, eliminating stuttering and frame drops on large directories.
6. `test_git_status_badges_allocation_free_in_render`: Optimize git status badge access in the file tree rendering so that no per-frame `HashMap` clones occur on each paint pass.

## Behaviour

- In the session workspace inspector tab row, tabs display with standard `RADIUS_CONTROL` (6px) rounded corners both at rest and when selected, perfectly matching other control buttons in the application.
- Draggable tabs lead with the Phosphor `DOTS_SIX_VERTICAL` grip icon (⠇) and show `CursorIcon::Grab` when hovered, with tooltip 「ドラッグして移動」.
- When dragged, the cursor turns into `CursorIcon::Grabbing`, and the destination half of the window displays tooltip guidance indicating the drop action.
- The splitter divider between the session list and the side panel features a centered grip indicator and highlights subtly on hover, setting `CursorIcon::ResizeVertical` and tooltip 「ドラッグして高さを調整」.
- When opening a new session or switching between sessions, the file tree displays instantly without dropped frames or stuttering, as file tree construction is cached and off-screen nodes are culled from immediate-mode layout.

## Design

### Placement verdict: `EXTEND`

The changes extend existing layout and widget modules:
- `src/glyphs.rs`: Add `ICON_DRAG_HANDLE` constant and add to `ICON_VOCABULARY`.
- `src/ui/widgets.rs`: Add helper for rendering splitter grip affordance `splitter_grip_h` and `splitter_grip_v`.
- `src/ui/session_tree.rs`: Update `ui_session_inspector` tab buttons to include `ICON_DRAG_HANDLE`, apply `RADIUS_CONTROL`, and optimize file tree caching.
- `src/ui/files.rs`: Add viewport visibility culling in `file_tree_rows`.
- `src/app/screens.rs`: Render splitter grip affordances in `ui_session_column` and `ui_terminal_panel`.
- `src/tests.rs`: Add tests verifying tab button corner radius, icon vocabulary, splitter response, and viewport culling.

No new module or store schema change is required.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Uses existing semantic palette roles (`control`, `border_subtle`, `border`, `text_strong`, `text_muted`); no new colour roles added. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Added `ICON_DRAG_HANDLE: &str = icons::DOTS_SIX_VERTICAL;` from bundled Phosphor font and registered in `ICON_VOCABULARY`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No shared or persisted configuration identifiers modified. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Pure UI and rendering performance enhancement; persistence schema unchanged. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No subprocess execution modified. |
| Documentation — user-facing docs change in all three languages together | no | Reuses existing localized strings (`tr("ドラッグして移動")`, `tr("ドラッグして高さを調整")`); no documentation divergence. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | No network calls, no telemetry, no unsafe code introduced. |
| Budgets — any new scan or output path states its byte and item ceiling | no | Reuses existing `FILE_TREE_LIMIT` and `FILE_FILTER_RESULT_LIMIT` budgets; no new scans introduced. |

## Flagged concerns

- **Visual density of grip icon in tab bar**: Placing `ICON_DRAG_HANDLE` on every tab could crowd the tab bar if spacing is too large. We resolve this by sizing the grip icon at 12px with 2px padding, ensuring the label and counts fit comfortably without wrapping at standard sidebar widths.
- **Tree viewport culling accuracy with variable node heights**: In `file_tree_rows`, each row has a predictable height (`CONTROL_HEIGHT_SMALL` = 24px). Using `ui.is_rect_visible` with the row's allocated bounds ensures perfect culling without missing partially visible rows or breaking scroll geometry.
- **Cache invalidation when switching worktrees**: When switching sessions with different worktrees, `session_file_tree_cache` checks `cached_root == &root`. If different, it immediately swaps to the target worktree's tree or initiates a background scan without stalling the UI thread.

## Acceptance

- `cargo fmt --check` succeeds.
- `cargo clippy --locked -- -D warnings` succeeds.
- `cargo test --locked` passes for all existing and new tests.
- Sidebar tab buttons (`ファイル`, `会話`, `変更`) exhibit 6px rounded corners when selected, consistent with `DESIGN.md`.
- Draggable tabs show `ICON_DRAG_HANDLE` (⠇) and hover `Grab` cursor.
- Horizontal and vertical splitters display visible centered drag grip handles and highlight on hover.
- Opening and viewing session sidebars operates smoothly at 60 FPS without UI-thread frame stalls.

## Rejected alternatives

- **Floating window detachable sidebars**: Adding multi-window OS docking support would require complex window lifecycle management and native OS windowing, introducing high complexity and instability for a focused cockpit app.
- **Static text label indicating drag ("ドラッグ可能")**: Adding prose labels clutters precious horizontal sidebar space. Standard iconography (⠇ grip handle) and hover tooltips provide universal, clean affordance without text bloat.
