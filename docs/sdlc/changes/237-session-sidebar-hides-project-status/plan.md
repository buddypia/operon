# Plan: project column groups each project's session states and folds

- **Spec**: `./spec.md`
- **Approved**: 2026-10-30
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/ui/session_tree.rs` | `SessionArrangement.folded`, serde default |
| `src/app/screens.rs` | `ui_workspace_projects` rewritten as fold/counts/rows; header button removed; tab glyph; fold toggle saves |
| `src/i18n_tables.rs` | 「起動中のセッションはありません」「プロジェクトがありません」「折りたたむ」「展開する」 in EN and KO |
| `src/tests.rs` | new tests per spec; two existing tests updated for the removed button and the replaced count |

## Order of work

1. Contract: `session_arrangement_without_folded_projects_still_loads` and the
   round trip; add the field.
2. Failing UI tests for requirements 1–5 and 7, then the column and tab code.
3. Strings in both tables.
4. Update the two existing tests whose assertions the spec supersedes; no test removed.
5. `make q.fast`, then `cargo test --locked workspace_` and the arrangement tests.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Count/row click opens a tab in the wrong project | wrong terminal in front | count-click test asserts active tab and selected project |
| Fold save fails on every frame | notice spam / I/O per frame | save only inside the click branch; review |
| Header drag stops reordering after the layout change | order not saved | kept real-drag test |
| Status pass is slow with many sessions | frame stutter | allocation-free kind; one pass per frame |

## Proof of completion

- `make q.fast` — clean.
- `cargo test --locked workspace_` and `session_arrangement` — all pass.
- Native screenshot of the Sessions page showing counts, rows, a fold, and tab glyphs.

## Departures from the plan

- Eight more existing tests looked a session title up as the first or
  leftmost match on screen; the column now lists those titles too, so each
  looks right of the column via `project_column_right` (no assertion weakened):
  `the_terminal_starts_directly_under_the_session_header`,
  `clicking_a_pinned_rows_title_opens_the_session`,
  `a_session_rows_corner_stays_inside_its_block`,
  `dragging_a_central_tab_onto_a_terminal_splits_it`,
  `history_renders_each_session_once_in_the_chosen_view`,
  `history_title_single_click_opens_after_the_double_click_window`,
  `workspace_pointer_switch_reorder_split_and_close_document`,
  `workspace_escape_and_release_over_tab_cancel_reordering`.
- 「起動中のセッション」 lost its last caller with the 「N 件」 count, so its
  rows left both tables (`every_message_row_is_one_the_code_shows`).
- Tab state words appear on hovering the tab's glyph; the title's own hover is
  unchanged.
- `finish_column_drag` also prunes `folded` to existing projects.
- Count chips drop the space and use the 12.5 body size: a native screenshot showed "operon" cut to "op…" beside three counts (review round 2).
