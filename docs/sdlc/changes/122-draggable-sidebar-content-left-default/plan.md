# Plan: draggable sidebar contents and left sidebar file list default

- **Spec**: `./spec.md`
- **Approved**: 2026-10-06
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `SIDEBAR_MIN_W`, `SIDEBAR_MAX_W`, `SIDEBAR_DEFAULT_W` constants |
| `src/ui/session_tree.rs` | Extend `InspectorTab` with `Sessions`, add `SidebarSide`, tab header layout & draggable tab helper |
| `src/app.rs` | Add sidebar state fields to `OperonApp`, default tabs initialization, tab moving/reordering helpers |
| `src/app/screens.rs` | Update `ui_terminal_workspace` to render left sidebar with active tab, draggable splitters, right sidebar with active tab |
| `src/i18n_tables.rs` | Register translations for tab headers and tooltips |
| `src/tests.rs` | Unit tests for defaults, tab dragging between sides, reordering, rendering, splitter resizing, and empty sidebar collapse |

## Order of work

1. Constants in `src/config.rs`: `SIDEBAR_MIN_W`, `SIDEBAR_MAX_W`, `SIDEBAR_DEFAULT_W`.
2. Types in `src/ui/session_tree.rs`: `InspectorTab::Sessions`, `SidebarSide`, pure helper functions for tab management.
3. App state fields and methods in `src/app.rs`: `left_sidebar_tabs`, `right_sidebar_tabs`, `left_sidebar_tab`, `right_sidebar_tab`, `move_sidebar_tab`, `reorder_sidebar_tab`.
4. Tests in `src/tests.rs`: write tests and watch them fail.
5. Workspace layout in `src/app/screens.rs`: render left and right sidebars dynamically with draggable tab pills and resizable splitters.
6. Verification: `make q.check`, `cargo test --locked`, formatting, clippy.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Tab drag accidentally triggers tab selection or terminal click | Click event swallowed or fires unexpectedly | `Sense::click_and_drag()` distinguishes click from drag; unit tests for click selection vs drag drop |
| All tabs moved to one side leaving empty blank area | Awkward empty gutter taking terminal space | `test_sidebar_collapses_when_empty` checks width collapses to 0 |
| Sidebar width exceeds window bounds | Terminal squeezed off screen | Width clamped to `SIDEBAR_MIN_W..=SIDEBAR_MAX_W` and constrained by available window width |
| Missing translation string | i18n test failure | `test_i18n_tables_complete` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- The new tests in `src/tests.rs` pass:
  - `test_sidebar_defaults_left_files_and_right_conversation`
  - `test_sidebar_tab_drag_between_sides`
  - `test_sidebar_tab_reorder_within_side`
  - `test_sidebar_renders_selected_tab_content`
  - `test_sidebar_splitter_resize_clamp`
  - `test_sidebar_collapses_when_empty`
- In the running app, opening the session workspace presents the file list in the left sidebar by default, with draggable tabs and draggable splitters.

## Departures from the plan

None yet.
