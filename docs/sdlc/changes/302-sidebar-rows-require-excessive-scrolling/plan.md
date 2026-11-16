# Plan: compact vertical spacing and type scale in the session sidebar

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `SIDEBAR_ROW_GAP: f32 = 2.0` after `SIDEBAR_DEFAULT_W` |
| `src/ui/widgets.rs` | Change `status_count_chip` `FontId::proportional(12.5)` to `11.5` |
| `src/app/screens.rs` | Tighten vertical `item_spacing`, `interact_size.y` on non-button rows / `CONTROL_HEIGHT_SMALL` headings, frame margins, `ColumnDrag` drop gap, and `12.0`/`11.5` text sizes in `ui_workspace_projects`, `ui_sidebar_agent_segment`, and `ui_sidebar_filter_summary` |
| `src/tests.rs` | Update `session_column_project_is_a_heading_not_a_card` to `.size(12.0)`; add `sidebar_302_session_rows_and_project_headings_are_compact`, `sidebar_302_column_uses_compact_type_scale`, `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs` |
| `docs/sdlc/changes/302-sidebar-rows-require-excessive-scrolling/` | `mutations.py`, `verification.yaml`, `state.yaml` |

## Order of work

1. Add the three `sidebar_302_*` tests in `src/tests.rs` rendering via `ui_central_panel` so `ui_central_panel`'s `10.0` `item_spacing` and `apply_interface_metrics` are active. Watch `sidebar_302_` fail on the current layout/sizes.
2. Add `SIDEBAR_ROW_GAP` in `src/config.rs`, update `status_count_chip` in `src/ui/widgets.rs`, update `ui_workspace_projects` / `ui_sidebar_agent_segment` / `ui_sidebar_filter_summary` in `src/app/screens.rs`, and update `session_column_project_is_a_heading_not_a_card` in `src/tests.rs`.
3. Run `sidebar_302_` and existing sidebar/design-system suites (`session_column_project_`, `sidebar_251_`, `sidebar_252_`, `session_285_`, `workspace_project_`, `every_button_keeps_a_documented_height`, `every_text_size_is_a_level_on_the_type_scale`, `design_md_documents_exactly_what_the_app_paints`, `every_theme_stays_readable`).
4. Write `mutations.py`, run the mutation sweep, run `make q.fast` and `bash scripts/check-bands.sh`, have an independent verifier review, commit, push, wait for CI, merge to `main`, clean up worktree, and ship `/Applications/Operon.app` via `.claude/skills/ship/SKILL.md`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| `ui.horizontal` inherits `28.0` `interact_size.y` and keeps rows tall despite tighter `item_spacing` | Row center-to-center pitch stays `> 24.0px` | `sidebar_302_session_rows_and_project_headings_are_compact` via `ui_central_panel` |
| `sidebar_251_short_stacked_column_can_scroll_to_project_rows` fails if `"first"` moves above `120.0px` | Existing test fails because `"first"` is visible before scrolling | `cargo test --locked sidebar_251_` |
| `ColumnDrag::Session` drop line uses stale `item_spacing.y` or wrong gap, or dragging breaks at `2px` gap | Dragging fails to reorder sidebar and top tabs | `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs` and `session_285_dragging_a_row_reorders_its_tabs` |
| Font size or button height violates `DESIGN.md` gates | `every_text_size_is_a_level_on_the_type_scale` or `every_button_keeps_a_documented_height` fails | `make q.fast` and targeted tests |
| `src/app/screens.rs` crosses `10000` lines (`bands.yaml` `propose` ceiling) | `check-bands.sh` reports `largest_module_lines` at `propose` | `wc -l src/app/screens.rs` and `bash scripts/check-bands.sh` |

## Proof of completion

- `cargo fmt --check` — exits 0 with no output.
- `cargo clippy --locked -- -D warnings` — exits 0 cleanly.
- `cargo test --locked sidebar_302_` — 3 passed (watched failing before edits).
- Targeted existing suites (`sidebar_251_`, `sidebar_252_`, `session_285_`, `workspace_project_`, `session_column_project_`, `every_button_keeps_a_documented_height`, `every_text_size_is_a_level_on_the_type_scale`) — all pass.
- `python3 docs/sdlc/changes/302-sidebar-rows-require-excessive-scrolling/mutations.py` — all mutations caught.
- CI passes on branch before merging to `main`, and `/Applications/Operon.app` is packaged, installed, verified (`check-installed-build.sh` CURRENT), and running instead of the stale `Operon Stable.app`.

## Departures from the plan

- Extracted `session_status_icon_sized(..., height: f32)` in `src/ui/widgets.rs` and called it with `COLUMN_SESSION_MARK_SIZE` in `src/app/screens.rs` for custom/fallback rows after the independent verifier caught that `session_status_icon` kept `CONTROL_HEIGHT_SMALL` (`24.0px`) row height on custom commands.
- Kept session row vertical `inner_margin` at `2px` (`Margin::symmetric(SPACE_XS as i8, 2)`) alongside `SIDEBAR_ROW_GAP = 2.0` (`20px` row height, `22px` pitch, `4.0px` badge-ring clearance below `mark.bottom()` vs `3.55px` outer ring) and restored the selected accent bar to `+/- 2.0`, after the verifier caught a `0.55px` ring overlap at `1px` margin.
- Added `F1` (`sidebar_302_custom_rows_and_badge_rings_stay_compact_without_overlap`), added unselected project `"q"` coverage in `sidebar_302_column_uses_compact_type_scale` and `session_column_project_is_a_heading_not_a_card`, added `ColumnDrag::Project` drop-line verification in `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`, updated `sidebar_302_session_rows_and_project_headings_are_compact` to verify `ui_sidebar_agent_segment`'s `28.0px` `widget_rect` (`24px + 2*1px margin + 2*1px stroke`), and tuned `mutations.py` to 12 orthogonal single-hit mutations so CI's `CARGO_INCREMENTAL=0` obligation sweep finishes well within `OBLIGATION_SECONDS=900`.
