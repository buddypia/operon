# Plan: the session column shows exactly the project's tabs, in one order, and a session can be pinned

- **Spec**: `./spec.md`
- **Approved**: 2026-11-11
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/ui/session_tree.rs` | `SessionArrangement.pinned`; `workspace_session_rank`, `sort_workspace_sessions`; `WorkspaceTabs::arrange`, `WorkspaceTabs::session_ids`; `follow` |
| `src/app.rs` | `sync_workspace_live_sessions` over every project, `Queued` added, deleted sessions' tabs dropped from every `project_workspaces` entry, then `arrange` every workspace; `project_tab_sessions`; `close_workspace_tab` refuses a pinned session; `toggle_session_pin`; `follow_tab_drop`; `workspace_project_drag` → `workspace_column_drag: Option<ColumnDrag>`; pins pruned where the order is pruned |
| `src/app/screens.rs` | `ui_workspace_projects`: rows from `project_tab_sessions`, no state sort, row drag and targets, pin mark, menu entry, empty text; tab strip: pin mark instead of ×, menu entry, `follow_tab_drop` after `insert_tab`; `finish_column_drag` prunes pins |
| `src/i18n_tables.rs` | English and Korean rows for the five new strings (「ドラッグで並べ替え」 exists) |
| `README.md`, `README.ja.md`, `README.ko.md` | the `session-order.json` bullet |
| `src/tests.rs` | the R1–R7 tests; existing tests that asserted the old membership (queued without a tab) or the state order updated |
| `docs/sdlc/changes/285-…/` | state, plan, verification, mutations, approvals |

## Order of work

1. Failing tests first, each run by name and watched failing on its assertion
   (stubs only where a name must exist to compile: `pinned` field,
   `toggle_session_pin` that does nothing, `follow` that does nothing):
   `session_285_column_lists_exactly_the_tabs`,
   `session_285_a_new_session_in_another_project_gets_a_tab_and_a_row`,
   `session_285_rows_and_tabs_share_one_order`,
   `session_285_dragging_a_row_reorders_its_tabs`,
   `session_285_dragging_a_tab_reorders_the_rows`,
   `session_285_pin_from_row_or_tab_moves_it_first_in_both`,
   `session_285_pins_survive_a_relaunch`,
   `session_285_a_pinned_tab_does_not_close`.
2. `src/ui/session_tree.rs` helpers (pure, unit-testable).
3. `src/app.rs` lifecycle: sync, arrange, close gate, pin toggle, tab-drop follow.
4. `src/app/screens.rs` column and tab strip.
5. Strings and READMEs.
6. Run the existing `workspace_project_`, `session_column`, `arranged`,
   `workspace_layout`, `relaunch` tests by filter; update the ones whose
   assertion was the old behaviour, saying so in the commit.
7. `make q.fast`; mutation sweep; verification.yaml.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Per-frame `arrange` fights a tab drag (the sort undoes the drop) | the dropped tab jumps back | R5 test drops a tab and checks both orders |
| Sorting moves the front tab | another terminal gets the keyboard | `arrange` keeps the front tab; R3 test asserts `active()` unchanged |
| Opening tabs in other projects' workspaces changes the relaunch layout or the closed-tab memory | closed tabs reappear after a relaunch | existing `workspace_layout`/`relaunch` tests run by filter; `workspace_seen_sessions` still records every placed session |
| A queued tab draws a terminal for a session without tmux | blank or error pane | the existing pane drawing for non-live sessions (Exited tabs exist today); checked in the running app |
| A deleted session keeps a tab in a project not shown | a row for a session that no longer exists | R7 test deletes a session of a project not shown and checks its rows |
| `arrange` moves a document tab | a file tab jumps among session tabs | R3 test holds a document tab between two session tabs |
| The pinned refusal blocks an internal close (pane move, delete) | a tab cannot move between groups, or a deleted session's tab stays | refusal only in `close_workspace_tab`; `WorkspaceTabs::close` (used by `insert_tab`/`move_tab`) and `workspace.retain` untouched; R7 asserts delete removes it |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_285_` — 8 passed.
- `cargo test --locked workspace_` — passes.
- `python3 docs/sdlc/changes/285-session-column-and-tabs-disagree/mutations.py` — every mutation caught.
- Running app: column rows equal the tabs; pin from a row and from a tab; ⌘W on a pinned tab; drag a row and a tab.

## Departures from the plan

- **Order of work.** The code was written before the tests rather than after
  each one was watched failing. What stands in for that: every guard was
  watched failing through the mutation sweep (17 of 17 caught), each mutation
  aimed at the claim its test makes.
- **The filter tiles.** The column's status tiles counted every stored
  session, so after this change they promised rows the column no longer
  draws — the same mismatch the person reported, one level up.
  `status_group_counts_for_chosen_agents` now counts only sessions with a tab
  (`session_has_a_tab`, `src/app.rs`). `sidebar_251_filters_drive_real_rows`
  and `sidebar_252_status_tiles_count_and_toggle` open their finished session
  from history so it has a row; R1's test asserts the tiles' total.
- **One more test.** `session_285_arrange_keeps_documents_and_the_front_tab`
  pins `WorkspaceTabs::arrange` alone (documents keep their slots, the front
  tab stays), which the drawn tests cannot reach without a document buffer.
- **`commit_session_arrangement`** was taken out of `finish_column_drag` in
  `src/app/screens.rs` so the row drag, the tab drop, and the pin toggle write
  the order file through one function; it also prunes pins.
- **String tables.** The new rows sit in id order, and the long notice is one
  line: `every_message_row_is_one_the_code_shows` reads only one-line rows.
- **A module of its own.** The new methods put `src/app.rs` past the
  `largest_module_lines` band (10,094 > 10,000). The tab sync, the column's
  source, the pin toggle, the tab-drop follow, `session_has_a_tab`, and
  `commit_session_arrangement` live in `src/app/session_order.rs`, mapped in
  `CLAUDE.md`; `src/app.rs` ends at 9,901 lines, below where it started.
- **Two tests from the verifier.** `empirical-verifier` reproduced nothing;
  two of its probes stay as tests, a project round trip with a pin and a
  closed tab, and ⇧⌘T plus a split group
  (`session_285_a_project_round_trip_keeps_rows_and_tabs_equal`,
  `session_285_reopen_and_a_split_group_keep_one_order`).
