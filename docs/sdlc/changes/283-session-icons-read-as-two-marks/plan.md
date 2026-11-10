# Plan: a session leads with one mark, its state a dot on it

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/glyphs.rs` | `COLUMN_SESSION_MARK_SIZE`, `STATUS_DOT_RATIO`, `STATUS_DOT_ATTENTION_RATIO`, `STATUS_DOT_RING` |
| `src/ui/widgets.rs` | `session_mark_speaks`, `session_badge`, `paint_status_dot`; `session_mark` removed |
| `src/app/screens.rs` | the tab and the column row draw `session_badge` |
| `src/tests.rs` | five `session_283_` tests; the tab tests of changes 267 and 273 and the tab state tests rewritten for one mark |
| `DESIGN.md` | the session-tab and terminal-mark paragraphs |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | the tab sentence |
| `docs/sdlc/changes/283-session-icons-read-as-two-marks/` | `mutations.py`, `verification.yaml` |

## Order of work

1. The five tests, watched failing.
2. Constants and the widget; the tab; the column row.
3. The earlier tab tests rewritten to find the mark and the dot.
4. The documents.
5. `make q.fast`; `cargo test --locked session_283_ session_273_ session_tab
   workspace_session_tab`; the design, readme and manual guards.
6. `mutations.py`, `verification.yaml`, the rust reviewer.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The dot is lost against the artwork | A state that cannot be seen | The ring in the surface colour; a screenshot of the packaged app |
| A terminal that stopped looks like one that runs | No dot on a dead terminal | `session_283_running_terminal_says_nothing` checks Done and Failed |
| Waiting is told by colour alone | 要対応 missed by someone who cannot use the colour | `session_283_waiting_dot_keeps_a_shape` |
| A custom command loses its state | No mark and no dot | `session_tabs_show_the_cli_mark_after_the_state` keeps the circle for it |
| The surface ring mismatches the row | A visible halo on a selected row | The row passes `row_selected` when selected; screenshot |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_283_` — `5 passed; 0 failed`.
- `python3 docs/sdlc/changes/283-session-icons-read-as-two-marks/mutations.py`
  — every mutation caught.

## Departures from the plan
