# Plan: the project changes tab shows the changed file list and commit box beside the live diff

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | two new tests (`the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane`, `clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab`) |
| `src/app/screens.rs` | `ui_git` renders the 2-pane split in `GitView::Changes`, keeps `self.git_view == GitView::Changes` on file selection, refreshes both changes and diff from `GitView::Changes`, and shares `ui_git_diff_pane` between `GitView::Changes` and `GitView::Diff` |
| `docs/sdlc/changes/222-…/` | this change's record |

## Order of work

1. Write the two tests in `src/tests.rs` and run them by name to watch each fail for its stated reason (`作業ツリーの差分` is not drawn on `GitView::Changes`, and clicking a file switches `app.git_view` to `GitView::Diff`).
2. Extract `ui_git_diff_pane` in `src/app/screens.rs`, lay out `GitView::Changes` as a side-by-side 2-pane split (`ui.horizontal_top`) with the `280.0` px left column and the right diff pane, keep `self.git_view` on `GitView::Changes` when a file row is clicked, and refresh both changes and diff when `ICON_REFRESH` is clicked on `GitView::Changes`.
3. Re-run the two new tests plus `every_diff_pane_has_a_comment_footer_below_it` and `the_git_sub_tabs_are_translated_message_ids`.
4. Run `make q.fast` (`cargo fmt --check` and `cargo clippy --locked -- -D warnings`) and `bash scripts/check-bands.sh`.
5. Run the mutation sweep (`mutations.py`), write `review.yaml`, commit, push, and merge after CI passes.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Duplicating `diff_pane(` without a matching `ui_diff_comment_footer(` | `every_diff_pane_has_a_comment_footer_below_it` fails | `ui_git_diff_pane` holds the single `diff_pane(` and `ui_diff_comment_footer(` pair |
| Right diff pane gets zero width inside `horizontal_top` | diff header overlaps or clips against the left column | `the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane` checks `diff_rect.left() > files_rect.right()` |
| Clicking a file switches away from `GitView::Changes` | file list and commit box disappear | `clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab` — `2 passed`, each watched failing first.
- `cargo test --locked every_diff_pane_has_a_comment_footer_below_it` — `1 passed`.
- `cargo clippy --locked -- -D warnings` — clean.
- `python3 docs/sdlc/changes/222-changes-tab-splits-files-and-diff/mutations.py` — all mutations caught.

## Departures from the plan

- None yet.
