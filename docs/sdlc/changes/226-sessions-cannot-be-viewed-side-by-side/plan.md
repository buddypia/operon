# Plan: sessions dropped on a terminal split it, as editor groups do

- **Spec**: `./spec.md`
- **Approved**: 2026-10-26
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `PANE_MAX`, `PANE_MIN_W`, `PANE_MIN_H` |
| `src/ui/session_tree.rs` | `PaneDrop`, `PaneNode`, `PaneLayout`, `pane_drop`, `pane_drop_zone`, `pane_drop_hint` |
| `src/app.rs` | `panes`, `pane_drag` fields; `select_session`, `is_session_on_screen`, `request_session_poll`, `apply_session_poll`, `SessionPoll.output` as a `Vec` |
| `src/app/screens.rs` | `ui_terminal_workspace` draws the pane tree and its rules; `ui_pane_drop`; `ui_terminal_panel(…, split)` header grip, `✕`, focus frame, focus-only echo/link/keyboard |
| `src/i18n_tables.rs` | eight strings in both tables |
| `src/tests.rs` | the nine tests named in the spec |

## Order of work

1. Write the pure-layout tests (requirements 1–5, 9) against `PaneLayout` and
   the drop functions; watch them fail to compile, then fail.
2. Add the constants and the pure types; those tests pass.
3. Write tests 6–8 against `OperonApp`; watch them fail.
4. Wire `app.rs`: fields, `select_session`, `is_session_on_screen`, the poll's
   `Vec` output.
5. Draw: `ui_terminal_workspace` splits the centre by `panes.rects`, rules
   resize, `ui_terminal_panel` takes `split`, `ui_pane_drop` paints and lands.
6. Strings in both tables; `make q.fast`; named tests.
7. Run the app with two sessions and look at it: zones, swap, close, focus,
   typing into the focused pane only.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Keys reach an unfocused pane | typing lands in the wrong agent | the panel surrenders egui focus when it is not the selected session; checked by hand in step 7 |
| Path cache thrashes between panes | links flicker, a filesystem probe per frame | only the focused pane feeds `unresolved_paths` and reads `resolved_paths` |
| Echo poll alternates between panes | slow echo while typing | `poll_terminal_echo` only for the focused pane |
| A removed session's pane lingers | empty terminal with no session | `retain` against the store at the top of the workspace draw; test 5 |
| Release over the list also reorders it | list order changes on a split | session reorder only lands inside its own card (existing); pane drop only when the pointer is over a pane |
| Per-frame allocation in the tree walk | stutter | `rects` returns a small `Vec` of at most four panes and three rules |

## Proof of completion

- `make q.fast` — fmt, clippy clean.
- `cargo test --locked pane_ two_split every_terminal_on_screen selecting_a_session closing_a_terminal a_session_dropped a_split_past` — all pass.
- CI green on the branch head.
- Screenshot of two panes and of a drag zone in the running app.

## Departures from the plan

Filled in during implementation.
