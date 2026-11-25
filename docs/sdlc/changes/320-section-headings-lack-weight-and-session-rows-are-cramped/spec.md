# Spec: headings that read as headings, and session rows with room

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A page's title — drawn by `page_header` (ホーム, プロジェクト, a project's
   overview, a settings section) — is 24px semi-bold in
   `text_strong`, a new `title` level in `DESIGN.md`'s typography block
   (`heading` stays 20px for what else uses it). `page_header` leaves
   `SPACE_SM` (8px) above the title, not 2px.
   Test: `heading_320_page_titles_are_the_title_level`.
2. The session column's heading 「プロジェクト」 is the `section` level
   (15px), semi-bold, in `text_strong`: one level above the project names
   (13.5px) and no longer muted.
   Test: `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`
   (its size assertion moves from 13.5 to 15) and
   `heading_320_the_column_heading_outranks_the_project_names`.
3. A session row in the column is `CONTROL_HEIGHT` (28px) tall — its 16px mark
   with 6px above and below — and rows keep `SIDEBAR_ROW_GAP` (4px) between
   them, so one row's top to the next is 32px (24px today).
   Test: `column_320_session_rows_have_room`.
4. A session row is indented: its mark's left edge is within 1px of its
   project's name's left edge, instead of at the column's edge.
   Test: `column_320_session_rows_sit_under_the_project_name`.
5. The drop line of a dragged session still lands in the middle of the gap
   between two rows.
   Test (existing, its arithmetic reading the new padding):
   `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`.

## Behaviour

- No string changes. Hover, selection fill and the 2px accent bar of the
  selected row cover the taller row; the bar stays 2px inside its top and
  bottom.
- Rename in place: the name editor sits in the taller row as before.
- Empty project, unfolded: 「開いているセッションはありません」 as today.
- Narrow column (180px): the indent takes space from the session's name, which
  truncates as today.
- Many sessions: the column scrolls as today; each row is 8px taller.

## Design

Placement: `EXTEND` (route `modify`) of `session-column` (rows and heading in
`ui_workspace_projects` in `src/app/screens.rs`) and `app-shell` (`page_header` in
`src/ui/widgets.rs`, used by Home, Projects, a project's overview and
Settings). The 「ナレッジ」 tab's heading in `src/app/knowledge.rs` stays
20px: it sits under a project's 24px name, one level below it.

- `DESIGN.md`: typography gains `title` (Sarasa UI J, 24px, 600, line height
  1.25); the prose counts thirteen levels (it says eleven today, though the block already holds twelve) and names `title` for page titles.
- `page_header` in `src/ui/widgets.rs`: title `.size(24.0)`; top space `SPACE_SM`.
- `ui_workspace_projects` in `src/app/screens.rs`: the column heading
  `.size(15.0).strong().color(palette.text_strong)`; the session row frame's
  margin becomes left `SIDEBAR_ROW_INDENT`, right `SPACE_XS`, top and bottom
  `SIDEBAR_ROW_PAD_Y`; the selected bar is drawn at the frame's left edge plus
  the indent's start so it still marks the row.
- `src/config.rs`: `SIDEBAR_ROW_PAD_Y: f32 = 6.0` (half of `CONTROL_HEIGHT`
  less the 16px mark) and `SIDEBAR_ROW_INDENT`, the measured offset of the
  project name from the frame's left edge.
- Persisted shape, external commands, background work: none. The draw path
  changes constants only.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: the heading moves from `text_muted` to the existing `text_strong`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No icon added or changed. |
| Type scale — every `.size(…)` is a `fontSize` level in `DESIGN.md` (`every_text_size_is_a_level_on_the_type_scale`) | yes | 24px is added as the `title` level before it is used; 15px is `section`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs` | yes | The row padding and indent are constants read by the drawing and by the tests. |
| Durability — store, cancellation, migration | no | Untouched. |
| Subprocess safety | no | No child process. |
| Documentation — user-facing docs in all three languages together | no | The READMEs describe neither sizes nor spacing; `CHANGELOG.md` one line. |
| Local-first | no | Drawing only. |
| Budgets | no | No scan or output path. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked heading_320_ column_320_ sidebar_302_` passes; the new tests watched failing first.
- `cargo test --locked every_text_size_is_a_level_on_the_type_scale design_md_` passes.
- `make q.fast` and `bash scripts/check-bands.sh` pass.
- `python3 scripts/verify-app.py run session-column` and `run project-list` pass, and their pictures show the larger titles.

## Rejected alternatives

- Raise `heading` from 20px to 24px: the overview tiles' values and the project
  row's folder icon also use 20px and would grow with it.
- An all-caps-style small label (the VS Code convention): Japanese has no case,
  and a smaller muted label is what the person found weak.
- Shrinking the gap between rows instead of padding them: the row's hover and
  selection would still hug the text.
