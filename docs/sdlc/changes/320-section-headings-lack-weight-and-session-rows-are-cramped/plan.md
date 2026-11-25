# Plan: headings that read as headings, and session rows with room

- **Spec**: `./spec.md`
- **Approved**: 2026-11-25
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `DESIGN.md` | `title` level (24px) in typography; prose says thirteen levels and what `title` is for |
| `src/config.rs` | `SIDEBAR_ROW_PAD_Y`, `SIDEBAR_ROW_INDENT` |
| `src/ui/widgets.rs` | `page_header`: 24px title, `SPACE_SM` above it |
| `src/app/screens.rs` | column heading 15px strong `text_strong`; session row margin and selected bar |
| `src/tests.rs` | `heading_320_*`, `column_320_*`; `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs` reads the new heading size and padding |
| `CHANGELOG.md` | one line under Changed |
| `docs/features/session-column/spec.md`, `docs/features/app-shell/spec.md` | the sizes and spacing, and the tests that pin them |
| `docs/sdlc/changes/320-…/` | `mutations.py`, `verification.yaml`, `state.yaml`, `approvals.log` |

## Order of work

1. `DESIGN.md` level and the two constants (the tree compiles; nothing uses them).
2. The four new tests, watched failing.
3. The drawing changes; measure the project name's offset and set
   `SIDEBAR_ROW_INDENT` to it.
4. Adjust `sidebar_302_*`'s heading size and drop-line arithmetic; run the
   named tests and the type-scale and design tests.
5. Feature specs, changelog, `mutations.py`, `make q.fast`,
   `bash scripts/check-bands.sh`, `verification.yaml`, verify-app pictures,
   commit, publish the branch, CI, land, ship.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Other tests measure row positions or the title's top | failures in `sidebar_`, `session_285_`, `column_311_`, and the drop-line arithmetic of `sidebar_302_*` that hard-codes the 2px row padding | CI's full run; locally the named families |
| The indent drifts from the name when the heading changes | rows no longer line up under the name | `column_320_session_rows_sit_under_the_project_name` |
| The selected bar leaves the row | bar drawn in the indent gap or outside | the verify-app picture; the bar's code reads the row rect |
| `src/app/screens.rs` crosses its band | `largest_module_lines` at `propose` | `bash scripts/check-bands.sh` |

## Proof of completion

- `cargo test --locked heading_320_ column_320_ sidebar_302_` — passes, the new ones watched failing.
- `cargo test --locked every_text_size_is_a_level_on_the_type_scale` — passes.
- `python3 docs/sdlc/changes/320-section-headings-lack-weight-and-session-rows-are-cramped/mutations.py` — every mutation caught.
- `make q.fast`, `bash scripts/check-bands.sh` — pass; CI green on the branch.
- verify-app pictures of the session column and the Projects page.

## Departures from the plan

- Earlier versions of intent, spec and plan also raised the 「ナレッジ」 heading
  to 24px. It is a tab's heading under a project's 24px name, so it stays 20px;
  `knowledge` left the features. Re-approved before landing.
- Five tests of changes 302 and 319 pinned the column as it was (rows at most
  24px apart, the heading 13.5px and muted). They now read the new constants
  and sizes; none was removed.
- `SIDEBAR_ROW_INDENT` measured at 42px: the project name starts 38px into its
  block, plus the block's 4px margin.
