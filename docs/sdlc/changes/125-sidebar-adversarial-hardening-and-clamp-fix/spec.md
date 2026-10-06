# Spec: adversarial hardening for sidebar splitters, drag drop bounds, and ID isolation

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. In `src/app/screens.rs`, the maximum inspector width `max_inspector_w` must be calculated from the outer total row width `total_row_w` before children allocations, clamped between `SIDEBAR_MIN_W` (180.0) and `SIDEBAR_MAX_W` (480.0) with respect to `TERMINAL_MIN_W` (300.0).
2. Right-side splitter resizing must use `(inspector_w - delta).clamp(SIDEBAR_MIN_W, max_inspector_w)` so that dragging the right-side splitter can freely expand the sidebar up to 480.0 pixels without collapsing to 180.0 (`test_sidebar_right_splitter_expansion_not_collapsed`).
3. Drag release must strictly require `PointerButton::Primary` release. Right clicks, middle clicks, and outside-window releases must not trigger side switching.
4. Pressing the `Escape` key during an active drag must immediately cancel `dragging_sidebar_tab` with no side switch and no toast notification (`test_sidebar_tab_drag_escape_cancellation`).
5. A deadzone of ±24.0 pixels around the center dividing line and a panel boundary containment check (`panel_rect.contains(pos)`) must prevent accidental switching near the center or outside the window (`test_sidebar_tab_drag_deadzone_and_bounds`).
6. In `src/ui/session_tree.rs`, `InspectorTab::Sessions` rendering must be wrapped in `ui.push_id("inspector_sessions_isolated", ...)` to prevent widget ID collisions with the workspace session sidebar.
7. Tooltip rendering during drag must use pre-computed formatted labels (`tf!`) to avoid per-frame `format!` heap allocations.

## Behaviour

- Users docking the inspector to the right side can drag the splitter to expand or shrink the sidebar across the full 180px–480px range.
- Users dragging a tab across the center line experience a smooth drop with a 24px deadzone preventing jitter.
- Pressing `Escape` while dragging a tab cleanly cancels the drag operation.
- Releasing secondary or middle mouse buttons does not drop the dragged tab.
- Frame rates remain smooth at 60/120Hz with zero per-frame tooltip heap allocations.

## Design

Verdict: `EXTEND` / `HARDEN` — modifies layout calculations and event handling in `src/app/screens.rs` and `src/ui/session_tree.rs`.

## Non-functional requirements

- Latency: zero noticeable frame drops during splitter or tab drag.
- Memory: 0 heap allocations per animation frame during drag.
- Backward compatibility: 100% preservation of all existing 707 test contracts.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | Reuses existing palette tokens (`palette.raised`, `palette.border_subtle`, `palette.text_strong`) without adding new colour literals |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Reuses established icons without adding new icons |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Min and max sidebar widths (`SIDEBAR_MIN_W: f32 = 180.0`, `SIDEBAR_MAX_W: f32 = 480.0`) in `src/config.rs` are strictly respected |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | No persisted store schema change; transient layout state remains in `OperonApp` |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new subprocesses or command executions are introduced |
| Documentation — user-facing docs change in all three languages together | no | Internal UI layout hardening conforming to standard expectations |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Operates strictly locally in-memory with pure immediate-mode egui widgets |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new scanning or output paths |

## Flagged concerns

- **Symmetric splitter resize calculation without layout collapse**: Capturing `total_row_w = ui.available_width()` before child UI layout begins ensures that both left and right splitters have identical, accurate bounds and cannot collapse to `SIDEBAR_MIN_W`.
- **Drag safety and escape hatches**: Restricting drops strictly to Primary button release with a 24px deadzone and adding `Escape` key cancellation guarantees robust and reliable pointer interaction.

## Rejected alternatives

- Computing splitter bounds dynamically from post-allocation `ui.available_width()` — Rejected because right-side child allocations deplete available width and crush the clamp upper bound to 180px.
- Allowing arbitrary mouse button release to trigger drops — Rejected because right-click context menus and auxiliary clicks would accidentally trigger layout side switches.

## Acceptance

- `cargo fmt --check` passes with no output.
- `cargo test --locked` passes with 0 failures.
- `cargo clippy --locked -- -D warnings` passes with 0 warnings.
- The new regression tests pass:
  - `test_sidebar_right_splitter_expansion_not_collapsed`
  - `test_sidebar_tab_drag_escape_cancellation`
  - `test_sidebar_tab_drag_deadzone_and_bounds`
