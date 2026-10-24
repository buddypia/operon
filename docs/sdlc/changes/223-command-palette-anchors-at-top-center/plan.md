# Plan: the command palette opens centered near the top of the window

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | new test `the_command_palette_sits_centered_near_the_top_of_the_window` |
| `src/app/screens.rs` | `ui_command_palette` anchors `egui::Window::new(tr("検索・操作"))` at `egui::Align2::CENTER_TOP, [0.0, 68.0]` with `default_width(520.0)` |
| `docs/sdlc/changes/223-…/` | this change's record |

## Order of work

1. Write `the_command_palette_sits_centered_near_the_top_of_the_window` in `src/tests.rs` and run it by name to watch it fail on unanchored placement.
2. Update `OperonApp::ui_command_palette` in `src/app/screens.rs` to add `.anchor(egui::Align2::CENTER_TOP, [0.0, 68.0])` and `.default_width(520.0)`.
3. Re-run `the_command_palette_sits_centered_near_the_top_of_the_window` plus `an_empty_palette_query_lists_only_the_actions` and `the_palette_selection_is_clamped_when_the_results_shrink`.
4. Run `make q.fast` (`cargo fmt --check` and `cargo clippy --locked -- -D warnings`) and `bash scripts/check-bands.sh`.
5. Run the mutation sweep (`mutations.py`), write `review.yaml`, commit, push, and merge after CI passes.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Anchoring interferes with window close or search focus | Palette fails to focus or close | Existing palette tests and `the_command_palette_sits_centered_near_the_top_of_the_window` render via `ui_command_palette` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked the_command_palette_sits_centered_near_the_top_of_the_window` — `1 passed`, watched failing first.
- `cargo test --locked an_empty_palette_query_lists_only_the_actions the_palette_selection_is_clamped_when_the_results_shrink` — `2 passed`.
- `cargo clippy --locked -- -D warnings` — clean.
- `python3 docs/sdlc/changes/223-command-palette-anchors-at-top-center/mutations.py` — all mutations caught.

## Departures from the plan

- None yet.
