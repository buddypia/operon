# Plan: Compact, counted sidebar filters

- **Spec**: `./spec.md`
- **Approved**: 2026-11-07
- **Status**: done

## Files that change

| File | Change |
|---|---|
| src/app.rs | Remove waiting filter; add CLI-aware group counts, matching count, waiting count. |
| src/app/screens.rs | Redraw sidebar filters; history chip and pinning stop reading the waiting filter. |
| src/ui/widgets.rs | `status_filter_tile`, `agent_filter_toggle`; remove `session_filter_button`. |
| src/i18n_tables.rs | Add new strings, drop rows left unused, in all tables. |
| DESIGN.md | Controls row for the sidebar filters names the new helpers. |
| src/tests.rs | Four 252 tests; 251 tests follow the removed words and waiting dimension. |

## Order of work

1. Commit intent, screen, spec, plan after evaluation.
2. Write the 252 tests and adjust 251 tests; watch the 252 tests fail.
3. State changes in `src/app.rs`, helpers in widgets, sidebar drawing, strings, DESIGN.md.
4. Named tests, the design and i18n guards, `make q.fast`, mutation sweep.
5. Native screenshot at default and narrow widths; publish the branch, CI, merge, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Removing waiting_filter changes history pinning | 要対応 queue missing in history | Existing history tests; pinning reads status_filter only. |
| Custom-painted tile loses keyboard or accessible name | Screen readers hear nothing | Widget info assertion in the 252 tests. |
| Text sizes or colours off-scale | Design guards fail | `every_text_size_is_a_level_on_the_type_scale`, `design_md_documents_exactly_what_the_app_paints`. |
| Tile text clipped at 180px (`SIDEBAR_MIN_W`) | Overlapping words | Narrow-column test and screenshot; tiles stack below two `STATUS_TILE_MIN_W` widths. |

## Proof of completion

Named tests pass; `make q.fast` clean; mutation sweep catches every mutation; CI green at the final head; screenshot shows the tiles; installed build matches after packaging.

## Departures from the plan

The first render drew each tile's mark as the circled vector status mark that workspace tabs use, while the sidebar's own rows and project counts use the group glyphs; the tiles now paint `SessionStatusGroup::icon()` so a tile and the rows it leaves carry the same mark, and no `status_mark()` was added to the group. 「クリア」 is the quiet link alone, without the close mark: the quiet button takes one string, and joining a glyph to the word would have needed a new message id for a decoration. The native check used a throwaway software rasteriser over egui's own tessellation instead of a framebuffer capture.
