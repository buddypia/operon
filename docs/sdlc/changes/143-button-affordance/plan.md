# Plan: clear visual affordance on buttons and interactive controls

- **Spec**: `./spec.md`
- **Status**: approved

## Files that change

- `src/ui/widgets.rs`:
  - Update `quiet_button` to render with `palette.link` text color.
  - Update `restore_progress_footer` and `launch_progress_footer` to use `secondary_button` for 「バックグラウンドで続ける」.
- `src/app/screens.rs`:
  - Update modal/sheet actions (attention button, cancel in project removal, custom command choice, reset worktree, re-pin settings, terminal only, tool setup steps) to use `secondary_button`.
- `src/tests.rs`:
  - Add test asserting `quiet_button` and updated footers render properly, maintain documented heights, and preserve focus ring behavior.
- `docs/sdlc/changes/143-button-affordance/state.yaml`:
  - Progress tracking through the pipeline.

## Order of work

1. **Test first**: Add unit test in `src/tests.rs` verifying button rendering and affordance expectations.
2. **Widgets update**:
   - Update `quiet_button` in `src/ui/widgets.rs` to use `palette.link`.
   - Update `restore_progress_footer` and `launch_progress_footer` in `src/ui/widgets.rs` to call `secondary_button`.
3. **Screen call sites update**:
   - Update `src/app/screens.rs` call sites replacing `quiet_button` with `secondary_button` where appropriate.
4. **Verification**:
   - Run `cargo fmt --check` and `cargo clippy --locked -- -D warnings`.
   - Run relevant tests in `src/tests.rs`.
   - Run `bash scripts/check-bands.sh` and `bash scripts/check-transcript-vocabulary.sh`.
5. **Commit & land**:
   - Stage and commit in the worktree.
   - Run gates, push branch, merge into `main`, and package.

## Proof of completion

- The new tests `quiet_button_renders_with_link_color_for_affordance` and `progress_footers_use_secondary_button_affordance` fail before the widgets update and pass after it.
- `every_button_keeps_a_documented_height` and `focus_ring_on_a_button_is_the_strong_outline_at_control_radius` pass without regression.
- Footer geometry tests `the_launch_footer_never_paints_status_under_its_action` and `the_restore_footer_never_paints_status_under_its_action` pass.
- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` pass cleanly.

## Departures from the plan

None so far.
