# Plan: Delete project from the projects list view

- **Spec**: ./spec.md
- **Approved**: 2026-09-14
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | Update `ui_projects` to use `clickable_card`, render `ICON_CLOSE` delete button, context menu, and danger confirmation card when `pending_project_removal == Some(project.id)`. |
| `src/i18n_tables.rs` | Add Japanese string keys and English/Korean translations to `EN_TABLE` and `KO_TABLE` in strict alphabetical sort order. |
| `src/tests.rs` | Add regression test verifying the project list deletion button and in-place confirmation card behavior. |
| `docs/sdlc/changes/044-delete-project-from-project-list/state.yaml` | Advance stage to `build` / `test` / `done`. |

## Order of work

1. Update `src/i18n_tables.rs` with new translation keys for project deletion in `EN_TABLE` and `KO_TABLE` in sorted order.
2. In `src/app/screens.rs`, refactor project rows in `ui_projects`:
   - If `self.pending_project_removal == Some(project.id)`, render the danger-bordered in-place confirmation card.
   - If not pending, render `clickable_card` with `ICON_CLOSE` button and context menu.
3. In `src/tests.rs`, add a test checking project removal from the project list.
4. Run `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings`.
5. Package the app bundle with `scripts/package-macos.sh` and replace `/Applications/Operon.app`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| i18n table sorting drift | `every_translation_table_is_sorted_by_message_id` fails | `cargo test --locked` |
| i18n table key mismatch | `every_message_id_has_a_row_in_every_table` fails | `cargo test --locked` |
| Click event collision between row navigation and delete button | Clicking delete button opens project overview | `clickable_card` architecture and click flag checking |
| Accidental project deletion without confirmation | Project removed immediately on row click | Confirmation state requirement in UI flow and test |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 483 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `the_project_list_offers_inline_deletion_and_confirmation` unit test passes.
- Project list shows `[✕ 削除]` button, clicking shows confirmation card, confirming removes the project.

## Departures from the plan

None.
