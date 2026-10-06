# Plan: one left sidebar — sessions above, files / conversation / changes below

- **Spec**: `./spec.md`
- **Approved**: 2026-10-06
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `SIDEBAR_SPLIT_DEFAULT`, `SIDEBAR_SECTION_MIN_H` |
| `src/ui/session_tree.rs` | `stacked_sidebar_heights`; drop `InspectorTab::Sessions` |
| `src/app.rs` | `session_list_split` field and default |
| `src/app/screens.rs` | stacked left column in `ui_terminal_workspace`; panel only docked right in `ui_terminal_panel`; drag handler lifted to the workspace |
| `src/tests.rs` | new tests; panel-render helper draws the whole workspace; Sessions-tab assertions replaced |

## Order of work

1. Write `test_stacked_sidebar_heights_clamp`,
   `test_stacked_sidebar_shows_sessions_and_files_together`,
   `test_stacked_sidebar_returns_column_width_to_terminal`; watch them fail.
2. Constants, helper, field.
3. Restructure the workspace and panel drawing; move the drag handler.
4. Remove `InspectorTab::Sessions`; update the two tests that named it.
5. `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked`.
6. Review (`rust-reviewer`), commit, land, package, swap.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Tests that render only `ui_terminal_panel` lose the left panel | ~45 render assertions fail | helper renders `ui_terminal_workspace` with the session selected |
| egui id clash between session list and panel scroll areas in one column | scroll state shared / id warning | each half drawn under its own `push_id` |
| Drop target changes break 125's dead zone | panel flips on tiny drags | dead zone kept; rect is the workspace |

## Proof of completion

- `cargo test --locked`: `0 failed; 6 ignored`.
- The three new tests pass and were watched failing.
- Packaged app screenshot matches `screen.md` After.

## Departures from the plan

- **Panel minimum.** Review found 120px too small for the lower half: the tab
  row and frame take ~56px and the panel body floors at 100px. The panel now has
  its own `SIDEBAR_PANEL_MIN_H` (180px); the list keeps `SIDEBAR_SECTION_MIN_H`.
  A rule drag in a column too short for both is ignored.
- **Allocation.** The spec's "no allocation" was wrong: the stacked column
  clones the selected `Session` and its `Project` per frame, as
  `ui_terminal_workspace` and `ui_terminal_panel` already did.
- **Responsive tests.** The 640px rule now applies only docked right, so
  `test_session_file_tree_responsive_width_thresholds` and S3 of
  `test_session_file_tree_real_world_workload_scenarios` render the panel row
  alone (`render_panel_row_shapes`) docked right; F7 also pins that docked left
  the panel stays at 600px.
- **Screen.** Not verified in the running app: synthetic clicks did not reach the
  dev window. Positions are pinned by the headless render tests instead.
- `the_trunk_allowlist_and_the_ownership_check_are_switched_on_here` failed once
  under the full parallel run (60s node timeout) and passed alone; unrelated.
