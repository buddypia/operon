# Plan: the person arranges the projects and sessions of the session column

- **Spec**: `./spec.md`
- **Approved**: 2026-10-20
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/ui/session_tree.rs` | `SessionArrangement`, `load_session_arrangement`, `save_session_arrangement`, `arrange_by`, `place`, `drop_slot`, `ColumnDrag` |
| `src/config.rs` | `SESSION_ARRANGEMENT_FILE_NAME`, `SESSION_ARRANGEMENT_FILE_MAX_BYTES`, `COLUMN_DRAG_EDGE`, `COLUMN_DRAG_SCROLL_STEP` |
| `src/app.rs` | `session_arrangement` (loaded in `from_state`), `column_drag`, `commit_session_arrangement` |
| `src/app/screens.rs` | `ui_terminal_session_tabs`: `arrange_by` after both sorts; row senses `click_and_drag`; the project name senses `drag`; during a drag, rects recorded, the line painted, edge scroll, release → `place` + commit; Escape / lost release end it |
| `src/i18n_tables.rs` | row hover replaced in both tables; 「ドラッグで並べ替え」 and 「並び順を保存できませんでした: {error}」 added |
| `README.md`, `README.ja.md`, `README.ko.md` | one sentence each beside the waiting-sessions bullet |
| `src/tests.rs` | the nine tests in the spec, and one helper |

Test helper: `arranged_column_fixture(label)` — an app from `test_app` on a
temp data folder with two projects 「alpha」 and 「beta」; 「alpha」 holds three
Exited sessions created 3, 2, and 1 hours ago (titles 「一番目」 「二番目」
「三番目」 by `name`), 「beta」 one. A `run(app, ctx, events, height)` closure
renders `ui_terminal_session_tabs` alone in a 300px-wide `CentralPanel`, as
`test_dock_drag_by_pointer_events` renders the workspace, and returns
`text_positions`; a drag is press → four moves → release, positions from
`painted_at`.

## Order of work

1. Write the nine tests against main and watch each fail for its reason:
   - `…keeps_todays_order_until_arranged`, `…place_moves_relative…`,
     `…arrangement_file_is_bounded…`: do not compile (no `arrange_by`,
     `place`, `SessionArrangement`) — then, stubbed, fail on their asserts;
   - `…session_row_drag_reorders…`: 「一番目」 still painted below 「三番目」
     after the drag (the row senses click only);
   - `…project_name_drag_reorders…`: 「beta」 still painted below 「alpha」;
   - `…escape_or_a_release_in_place…`: passes on main for the wrong reason
     (nothing ever moves) — so it also asserts `column_drag.is_some()` after
     the press and moves, which fails on main;
   - `…save_failure_is_said`: no notice after the drag;
   - `…drag_near_the_edge_scrolls…`: in a 220px-tall column, the first
     title's painted y unchanged after holding at the bottom edge;
   - `…hovers_say_rows_and_projects…`: the hover text lacks
     「ドラッグで並べ替え」 (tooltip delay set to zero in the test's style).
2. Constants in `src/config.rs`; `SessionArrangement`, load and save, and the
   three pure functions in `src/ui/session_tree.rs`. A move writes the whole
   level's order — every project, or every session of the one project, hidden
   ones included — so that on the first move from an empty arrangement the
   items not moved do not become "unlisted" and jump above the moved one;
   requirement 2's test starts from an empty arrangement for that reason.
   Requirements 1, 2, 6 pass.
3. `OperonApp` fields, loading in `from_state`, `commit_session_arrangement`.
4. The column: `arrange_by` after both sorts (the column reads exactly as
   before with an empty arrangement — every existing column test still
   passes); then the session drag; then the project drag; then edge scroll;
   then the hovers and strings. Requirements 3, 4, 5, 7, 8, 9 pass in that
   order.
5. READMEs.
6. `make q.fast`; `cargo test --locked change_181`, `every_`, and the column
   tests of changes 163 and 166, which are named by behaviour, by name:
   `each_project_and_its_sessions_share_one_frame`,
   `a_session_rows_menus_offer_stop_and_remove`,
   `a_column_title_double_click_renames_its_session`,
   `an_empty_project_in_the_session_column_starts_a_session`; a mutation sweep from
   `docs/sdlc/templates/mutations.py`; a dev-build screenshot against a
   copied store.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The row now senses drag, so a click with a small jitter no longer opens the terminal | A click that does nothing | egui counts a click until the pointer moves past its drag threshold; `…session_row_drag_reorders…` also clicks a row and asserts it opened |
| The title's double click (rename) or right click stops reaching the title | Rename or the menu never opens | change 163's `a_column_title_double_click…` and the menu tests stay in the run |
| Dragging the project name swallows the ＋ beside it | ＋ starts nothing | `…project_name_drag_reorders…` presses ＋ after the move and asserts the setup opened |
| `arrange_by` per frame costs more than it looks | A stutter with many sessions | No allocation; O(n log n · m) over store-sized lists; the rust-reviewer reads the draw path |
| A save on every frame of the release | Disk churn | `place` returns false for no change and the commit runs only then; requirement 5's test asserts the file is absent after a release in place |
| A stale drag after focus is lost | The next click starts a move | The lost-release check, mirrored from `ui_dock_drop`, in requirement 5's test |
| Edge scroll stops while the pointer is held still | egui draws no frame without input | `request_repaint` on every frame the drag sits in an edge band; requirement 8's test holds the pointer still across frames |
| Mutating the store's order by accident | Other lists reorder | The store's vectors are read, never sorted in place; `arrange_by` sorts the frame's clone |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked change_181` — `9 passed; 0 failed`.
- `cargo test --locked every_` and the four 163/166 column tests named in
  step 6, each run by its name — all pass, with a non-zero count.
- CI's `cargo test --locked` on the pushed branch — green.
- Dev build, copied store: a session dragged up stays after quit and reopen; a
  project dragged to the top stays; Escape mid-drag moves nothing.

## Departures from the plan
