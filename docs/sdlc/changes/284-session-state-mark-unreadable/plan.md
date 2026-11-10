# Plan: a session's state reads as a shape on its mark

- **Spec**: `./spec.md`
- **Approved**: 2026-11-11
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/glyphs.rs` | `StatusMark::paint_glyph`, `StatusMark::paint_badge`; `STATUS_DOT_RATIO` 0.64; `STATUS_DOT_ATTENTION_RATIO` removed; `STATUS_COUNT_MARK_SIZE` |
| `src/ui/widgets.rs` | `paint_status_dot` draws the mark; `SessionStatusGroup::status_mark`; `status_count_chip` |
| `src/app/screens.rs` | project counts call `status_count_chip`; the markless row calls `session_status_icon` |
| `src/tests.rs` | three `session_284_` tests; 283's dot tests and the two project-count tests rewritten for the badge and the generated mark |
| `DESIGN.md`, READMEs, manuals | the sentence describing the dot, where it exists |
| `docs/sdlc/changes/284-session-state-mark-unreadable/` | `mutations.py`, `verification.yaml` |

## Order of work

1. The three new tests and the rewritten count tests, watched failing.
2. `StatusMark` split; badge; group mark; the count chip; the screens calls.
3. 283's tests rewritten (one ratio, the glyph instead of the hand-drawn stem).
4. Build and screenshot the app; adjust glyph scale or stroke if unreadable.
5. Documents.
6. `make q.fast`; `cargo test --locked session_284_ session_283_ workspace_project_`.
7. `mutations.py`, `verification.yaml`, the rust reviewer.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Glyph unreadable at badge size | A blur instead of a shape | Screenshot of the built app, step 4 |
| Badge hides too much of the CLI mark | Vendor mark unrecognisable | Screenshot; the badge overhangs the corner as the dot did |
| Count chip loses its click | Clicking a count does nothing | `workspace_project_count_click_opens_first_session_of_its_group` |
| A font glyph survives in the column | Mixed styles | `session_284_markless_row_draws_the_generated_mark`, the count test |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_284_` and the rewritten tests — pass.
- `python3 docs/sdlc/changes/284-session-state-mark-unreadable/mutations.py`
  — every mutation caught.

## Departures from the plan

- The badge was checked in a vector preview of the same geometry at retina
  scale on all four CLI marks before the tests were written; every glyph read
  at 0.64 of the mark with the glyph at 0.6 of the badge's radius, so no
  stroke was thickened beyond the `2 × scale` the plan's step 4 allowed for.
- The rust reviewer found the count chip's accessible name dropped the count
  the old label's text carried. It now reads 「要対応 1 件」 through a new
  `tf!` row, `"{p0} {p1} 件"`, so `src/i18n_tables.rs` changes too.
- `workspace_project_picker_keeps_saved_order_counts_and_per_project_launch`
  also found the count by its font glyph; it now finds the generated mark.
