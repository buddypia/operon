# Plan: add a session or a project from the session page's column, and give its project names their own size

- **Spec**: `./spec.md`
- **Approved**: 2026-11-25
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `column_add.rs`, new under `src/app/` | `ColumnAdd`, `COLUMN_ADD_CHOICES`, `apply_column_add`, `ui_column_add_items`, `ui_column_add_project_item`, `apply_column_project_folder` |
| `src/app.rs` | `mod column_add;` |
| `src/app/folder_picker.rs` | `FolderPickerTarget::SessionColumn`, routed in both places a target is matched |
| `src/app/screens.rs` | `ui_workspace_projects`: heading row, empty state, heading menu, name sizes, empty-space menu |
| `src/i18n_tables.rs` | 「新しいセッション…」, 「プロジェクトを追加…」 in every table |
| `src/tests.rs` | the `column_319_` tests |
| `CLAUDE.md` | the module map line for the new `column_add.rs` |
| `docs/features/session-column/spec.md`, `brief.md` | the heading +, the two menus, the sizes |
| `docs/features/project-list/spec.md`, `brief.md` | the column as a third place to add a project |
| `docs/features/flow.yaml` | ways from `sessions` to `folder-picker` and `new-session` through the column's + |
| `docs/features/index.yaml` | the new `column_add.rs` under `session-column` |
| `docs/MANUAL.ja.md`, `docs/MANUAL.md`, `docs/MANUAL.ko.md` | one sentence each on the column's + and right-click |

## Order of work

1. Tests for R1–R9 in `src/tests.rs`, against names that do not exist yet:
   the tree does not compile until step 2.
2. `column_add.rs`, the picker target, the `mod` line: R3–R5 pass.
3. The column's drawing in `screens.rs` and the two strings: R1, R2, R6–R9
   pass.
4. Feature documents, flow, manuals, module map.
5. `make q.fast`; `cargo test --locked column_319_` and the related groups
   named in the spec; the app run per
   `.claude/skills/feature-pilot/references/verify-in-the-app.md`.
6. `verification.yaml`, `mutations.py`, verifiers, close.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The empty-space click area swallows a drop or a row click | rows stop opening, or a drag cannot land below the last project | it is allocated after every row, with `Sense::click()` only; `session_285_`, `test_change_181_` drag tests |
| 13.5px names break the folded spacing | headings sit more than 28px apart | `column_311_folded_projects_sit_closer_together` |
| A new target missed in a `match` | compile error | the compiler; `folder_317_finder_is_still_reachable` lists targets |
| `src/app.rs` crosses its limit | the size assert in `tui_292_*` | that test |
| A size off the scale | the type-scale test | `every_text_size_is_a_level_on_the_type_scale` |
| A project added while a filter is on looks lost | it is not listed though added | the spec says so; `column_319_a_project_added_from_the_column_stays_on_the_session_page` asserts the saved `unfolded` holds it with a filter on |
| The order file cannot be written after registration | the project is added but the fold is lost on relaunch | the existing notice; `column_319_a_registered_folder_is_unfolded_and_chosen` points the order file at an unwritable path and asks for the notice and the project |
| The empty-space menu opens over a row's menu | right-clicking a row shows the column items | `column_319_a_right_click_on_empty_space_offers_the_same_items` right-clicks a row too and asks for 「名前を変更」 without 「プロジェクトを追加…」 |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked column_319_` — 8 passed, each having been watched
  failing.
- The debug build shows `screen.md`'s sections 1–3.

## Departures from the plan

None yet.
