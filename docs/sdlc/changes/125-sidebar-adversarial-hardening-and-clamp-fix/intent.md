# Intent: adversarial hardening for sidebar splitters, drag drop bounds, and ID isolation

- **Status**: approved
- **Opened**: 2026-10-06

## Problem

Following the implementation of change 122 (left-default file list and draggable sidebar contents), a critical adversarial review identified 5 vulnerabilities and bottlenecks:
1. **Right-Splitter Clamp Collapse**: In `src/app/screens.rs`, dragging the splitter when the inspector is docked to the right computes `ui.available_width() - 320.0` *after* the terminal viewport is laid out, leaving only ~274px and causing the upper clamp bound to collapse to `180.0` (`SIDEBAR_MIN_W`). Once dragged, the right sidebar cannot be expanded past 180px.
2. **Unconstrained Pointer Release**: `pointer.any_released()` terminates drag on secondary/auxiliary clicks instead of requiring `Primary` mouse button release.
3. **Missing Cancellation & Boundary Chattering**: Lack of `Escape` key cancellation, no deadzone around the center dividing line, and lack of viewport containment check causing edge-case jitter or accidental drops.
4. **Immediate-Mode Heap Allocation Overhead**: Unnecessary per-frame string allocation (`format!`) and duplicate `tr()` dictionary lookups in the drag rendering loop.
5. **Widget ID Collision**: Potential widget ID clash if `InspectorTab::Sessions` is active concurrently with workspace session tabs.

## Who feels it, and when

Users who dock the sidebar to the right side and attempt to resize it with the splitter, or users dragging tabs near the screen boundaries or attempting to cancel a drag with Escape.

## Desired outcome

1. Fix the right-side splitter width clamping so both left and right splitters calculate the maximum width from the total available row width before child allocations, smoothly clamping between `SIDEBAR_MIN_W` (180px) and `SIDEBAR_MAX_W` (480px) without collapsing.
2. Restrict drop handling strictly to `PointerButton::Primary` release.
3. Add `Escape` key drag cancellation and a ±24px deadzone around the center line with viewport containment check (`panel_rect.contains(pos)`).
4. Eliminate per-frame heap allocations during drag-and-drop tooltips.
5. Scope `InspectorTab::Sessions` rendering with `ui.push_id` to guarantee zero widget ID collisions.
6. Retain 100% backward compatibility with existing tests and `store.json` schema.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text in Japanese; code and comments in English.
- No schema change to `store.json`.
- All 707 existing tests must pass.

## Systems likely affected

- `src/app/screens.rs` (pre-capture `total_row_w`, symmetric splitter clamp, drag safety guards)
- `src/ui/session_tree.rs` (ID scope isolation)
- `src/tests.rs` (regression tests for right-side splitter resizing and drag cancellation)

## Open questions

None.

## Not in scope

- Adding arbitrary detachable floating windows outside the Operon application window.
