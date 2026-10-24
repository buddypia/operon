# Spec: the command palette opens centered near the top of the window

- **Intent**: `./intent.md`
- **Screen**: `./screen.md`
- **Status**: approved

## Requirements

1. `OperonApp::ui_command_palette` (`src/app/screens.rs:451-456`) configures `egui::Window::new(tr("検索・操作"))` with `.anchor(egui::Align2::CENTER_TOP, [0.0, 68.0])` and `.default_width(520.0)`. Pinned by `the_command_palette_sits_centered_near_the_top_of_the_window`.
2. When rendered in a `1280.0 x 800.0` viewport, the `検索・操作` window sits horizontally centered (its horizontal center within `24.0` px of `640.0`), clear of the left navigation rail (`left > 300.0`), and near the top of the viewport (`top < 160.0`). Pinned by `the_command_palette_sits_centered_near_the_top_of_the_window`.

## Behaviour

- Opening `ui_command_palette` draws `egui::Window::new(tr("検索・操作"))` anchored at `egui::Align2::CENTER_TOP` with offset `[0.0, 68.0]` and default width `520.0`.
- Closing the palette via the window's close button (`open = false`) or `Escape` continues to set `self.command_palette_open = false`.
- Empty-query action listing (`an_empty_palette_query_lists_only_the_actions`), search ranking, and keyboard selection clamping (`the_palette_selection_is_clamped_when_the_results_shrink`) remain unchanged.

## Design

Placement: `EXTEND` — `OperonApp::ui_command_palette` in `src/app/screens.rs`.

- `src/app/screens.rs`:
  - Add `.anchor(egui::Align2::CENTER_TOP, [0.0, 68.0])` and update `.default_width(420.0)` to `.default_width(520.0)` on `egui::Window::new(tr("検索・操作"))`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No new role | Uses existing `palette` roles (`text`, `text_muted`) already drawn in `ui_command_palette`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No new mark | No icons added or changed. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new identifier or persisted string. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing persisted changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | Pure UI layout change. |
| Documentation — user-facing docs change in all three languages together | No | The manual and READMEs describe opening the palette with `⌘K` without prescribing top-left coordinates. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | Pure UI layout change in `src/app/screens.rs`. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan or output path. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked the_command_palette_sits_centered_near_the_top_of_the_window` passes, and fails on `main`.
- `cargo test --locked an_empty_palette_query_lists_only_the_actions the_palette_selection_is_clamped_when_the_results_shrink` passes.
- `make q.fast` and `bash scripts/check-bands.sh` pass.

## Rejected alternatives

- `egui::Align2::CENTER_CENTER`: A command palette expands downward as results change; anchoring at `CENTER_TOP` keeps the search input stationary at `[0.0, 68.0]` while the result list grows or shrinks below it.
