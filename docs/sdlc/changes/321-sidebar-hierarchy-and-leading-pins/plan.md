# Plan: quiet session-list label, 15px project names, and leading pin marks

- **Spec**: `./spec.md`
- **Approved**: 2026-11-25
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | add `SESSION_PIN_SLOT_W: f32 = 14.0` beside `SIDEBAR_ROW_INDENT` |
| `src/i18n_tables.rs` | update `("セッション一覧", "the session list")` to `("セッション一覧", "Session list")` |
| `src/app/screens.rs` | `ui_workspace_projects` (`セッション一覧` 12.5 muted label, 15.0px project names, gutter `ICON_PIN` before mark with `selected_secondary_ink()` on selection); `ui_tab_workspace` (leading `ICON_PIN` before `session_badge` with `selected_secondary_ink()` when active, omit trailing `ICON_PIN` and `×`) |
| `src/tests.rs` | add five `column_321_*` tests; update existing tests that pinned `"プロジェクト"` panel text (`test_sidebar_renders_selected_tab_content`, `test_stacked_sidebar_shows_sessions_and_files_together`, `test_dock_*`) or `13.5px` project name size / `"プロジェクト"` column heading (`session_column_project_is_a_heading_not_a_card`, `heading_320_the_column_heading_outranks_the_project_names`, `sidebar_302_column_uses_compact_type_scale`, `column_319_project_names_read_larger_than_their_rows`) |
| `docs/features/session-column/spec.md`, `brief.md` | document quiet `セッション一覧` heading, `15px` project names, gutter `ICON_PIN`, and `Pinned by` tests |
| `docs/features/terminal-pane/spec.md`, `brief.md` | document leading `ICON_PIN` before `session_badge` on pinned tabs, pinned by `column_321_tab_pin_sits_left_of_agent_mark_without_close_button` |
| `CHANGELOG.md` | one line under `Changed` |
| `docs/sdlc/changes/321-sidebar-hierarchy-and-leading-pins/` | `mutations.py`, `verification.yaml`, `state.yaml`, `approvals.log` |

## Order of work

1. Commit `intent.md` + `state.yaml` first, then evaluate & approve `spec.md`, `screen.md`, and `plan.md` before editing source files.
2. Add the five `column_321_*` tests in `src/tests.rs` and watch them fail (`cargo test --locked column_321_`).
3. Add `SESSION_PIN_SLOT_W` in `src/config.rs`, update `src/i18n_tables.rs`, and apply the UI changes in `src/app/screens.rs`.
4. Update the existing tests in `src/tests.rs` (`test_sidebar_renders_selected_tab_content`, `test_stacked_sidebar_shows_sessions_and_files_together`, `test_dock_*`, `session_column_project_is_a_heading_not_a_card`, `heading_320_the_column_heading_outranks_the_project_names`, `sidebar_302_column_uses_compact_type_scale`, `column_319_project_names_read_larger_than_their_rows`) without removing any test.
5. Update `docs/features/session-column/spec.md`, `docs/features/session-column/brief.md`, `docs/features/terminal-pane/spec.md`, `docs/features/terminal-pane/brief.md`, and `CHANGELOG.md`.
6. Write and run `mutations.py`, invoke an independent verifier subagent to test edge cases, run `python3 scripts/verify-app.py run session-column`, run `make q.fast` and `bash scripts/check-bands.sh`, record `verification.yaml`, commit, push, wait for CI, merge on `main`, clean up worktree, and package/ship `/Applications/Operon.app`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| 15.0px project name makes folded heading taller than 28px | `column_311_folded_projects_sit_closer_together` fails (`pitch <= 28.0`) | `cargo test --locked column_311_` |
| Existing dock / sidebar tests look for `"プロジェクト"` or `13.5px` | `test_dock_*`, `test_sidebar_renders_selected_tab_content`, `test_stacked_sidebar_shows_sessions_and_files_together`, `session_column_project_is_a_heading_not_a_card` fail | Named test run before `make q.fast` |
| Gutter indent math drifts from `SIDEBAR_ROW_INDENT` | Pinned row's AI mark misaligns from unpinned row or project name | `column_321_sidebar_pin_sits_in_gutter_before_aligned_agent_mark` and `column_320_session_rows_sit_under_the_project_name` |
| `src/app/screens.rs` line count grows unnecessarily | `largest_module_lines` band | `bash scripts/check-bands.sh` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past compile lines.
- `cargo test --locked column_321_ heading_320_ column_320_ column_319_ column_311_ sidebar_302_ session_285_ session_286_ session_column_project_is_a_heading_not_a_card test_sidebar_renders_selected_tab_content test_stacked_sidebar_shows_sessions_and_files_together test_dock_ every_text_size_ design_md_ translation feature_behaviours_` — all pass (`0 failed`), with `column_321_*` watched failing first.
- `python3 docs/sdlc/changes/321-sidebar-hierarchy-and-leading-pins/mutations.py` — 100% mutations caught.
- `python3 scripts/verify-app.py run session-column` — exit 0; screenshot verified.

## Departures from the plan

- Empirical verifier `86d10a6d` reproduced F1: `ui.interact(row.rect, ...)` covered the sidebar `ICON_PIN` label's `.on_hover_text(tr("ピン留め中"))` tooltip. Added `if pinned { ui.label(tr("ピン留め中")); }` inside `row.on_hover_ui` in `src/app/screens.rs`, added regression test `column_321_sidebar_pin_hover_shows_tooltip`, and added mutation `sidebar-pin-hover-missing` to `mutations.py` (9 of 9 caught).
