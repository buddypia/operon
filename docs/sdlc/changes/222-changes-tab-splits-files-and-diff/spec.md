# Spec: the project changes tab shows the changed file list and commit box beside the live diff

- **Intent**: `./intent.md`
- **Screen**: `./screen.md`
- **Status**: approved

## Requirements

1. When `self.git_view` is `GitView::Changes`, `OperonApp::ui_git` lays out two side-by-side columns with `ui.horizontal_top`: a `280.0` px left column containing the changed file list (`tr("変更されたファイル")`), stage checkboxes, status badges, selectable file path labels, `ICON_EDIT` editor buttons, `ui_commit_box` (when `snapshot.files` is non-empty), and `ui_push_row`; a vertical `ui.separator()`; and a right column calling `self.ui_git_diff_pane(ui, project, palette)`. Pinned by `the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane`.
2. Clicking a file's selectable label in `GitView::Changes` sets `self.git_selected_file = Some(path)` and keeps `self.git_view` at `GitView::Changes` (instead of switching `self.git_view` to `GitView::Diff`), so the file list, stage checkboxes, and commit box remain drawn on the left while the right pane filters to that file's diff. Pinned by `clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab`.
3. Clicking the refresh button (`ICON_REFRESH`) while `self.git_view == GitView::Changes` calls both `self.request_git_changes(project)` and `self.request_git_diff(project, self.git_selected_file.clone())`. Pinned by `clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab`.

## Behaviour

- Opening a project's 「変更」 tab on `GitView::Changes` draws both the changed files column on the left and the working-tree diff pane (`作業ツリーの差分`) on the right.
- Clicking a file row in the left column selects `self.git_selected_file = Some(path)` and updates the right diff pane to show that file's diff plus the 「すべて表示」 button, while `self.git_view` remains `GitView::Changes` and the left column stays visible.
- Clicking 「すべて表示」 in the right diff pane clears `self.git_selected_file = None` so the right pane returns to the full working-tree diff.
- Clicking `ICON_REFRESH` (`更新`) on `GitView::Changes` refreshes both the changed file snapshot and the diff snapshot.
- `GitView::Diff` continues to render `self.ui_git_diff_pane(ui, project, palette)` at full width, and `GitView::History` continues to render the commit history.
- Empty, loading, and error states for both `git_changes_cache` and `git_diff_cache` render in their respective columns without change to their wording.

## Design

Placement: `EXTEND` — `OperonApp::ui_git` and a new helper `OperonApp::ui_git_diff_pane` in `src/app/screens.rs`.

- `src/app/screens.rs`:
  - Extract the `GitView::Diff` body (`src/app/screens.rs:3141-3237`) into `pub(crate) fn ui_git_diff_pane(&mut self, ui: &mut egui::Ui, project: &Project, palette: &Palette)`.
  - In `ui_git`, when `ICON_REFRESH` is clicked on `GitView::Changes`, call both `self.request_git_changes(project)` and `self.request_git_diff(project, self.git_selected_file.clone())`.
  - In `GitView::Changes`, wrap the file list / commit box / push row and `ui_git_diff_pane` in `ui.horizontal_top(|ui| { ... })` with a `280.0` px left column, `ui.separator()`, and a right column taking `ui.available_width().max(240.0)`.
  - Remove `self.git_view = GitView::Diff;` when a file path is clicked in `GitView::Changes`, leaving `self.git_selected_file = Some(path)`.
  - In `GitView::Diff`, call `self.ui_git_diff_pane(ui, project, palette)`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No new role | Uses existing `palette` roles (`text`, `text_muted`, `accent`, diff roles) already drawn in `ui_git`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No new mark | Uses existing `ICON_REFRESH` and `ICON_EDIT`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new identifier or persisted string. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing persisted changes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | Uses existing `request_git_changes` and `request_git_diff` background tasks. |
| Documentation — user-facing docs change in all three languages together | No | The manual and READMEs already describe reviewing changed files, diffs, and committing in the 「変更」 tab without prescribing a tab switch on click. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | Pure UI layout change in `src/app/screens.rs`. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No new scan or output path. |

## Flagged concerns

None. `every_diff_pane_has_a_comment_footer_below_it` (`src/tests.rs:34027`) counts `diff_pane(` and `ui_diff_comment_footer(` calls in `src/app/screens.rs`; keeping a single pair inside `ui_git_diff_pane` preserves that 1-to-1 invariant.

## Acceptance

- `cargo test --locked the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab` passes, and both fail on `main`.
- `cargo test --locked every_diff_pane_has_a_comment_footer_below_it the_git_sub_tabs_are_translated_message_ids` passes.
- `make q.fast` and `bash scripts/check-bands.sh` pass.

## Rejected alternatives

- Remove `GitView::Diff` entirely: keeping `GitView::Diff` as a full-width diff sub-tab costs one match arm (`GitView::Diff => self.ui_git_diff_pane(ui, project, palette)`) and preserves a full-width view for wide diffs as well as existing sub-tab contracts.
- Duplicate the `diff_pane` call in both `GitView::Changes` and `GitView::Diff`: extracting `ui_git_diff_pane` avoids code duplication and keeps `diff_pane(` / `ui_diff_comment_footer(` at one call site in `ui_git`.
