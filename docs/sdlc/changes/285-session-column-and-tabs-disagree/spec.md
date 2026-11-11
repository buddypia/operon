# Spec: the session column shows exactly the project's tabs, in one order, and a session can be pinned

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. R1: on the session page, a project's rows in the left column are exactly
   the session tabs of that project's workspace (every tab group, the shown
   project's and every other project's), whatever their state: a closed tab
   has no row, a finished session with a tab has a row, a session opened from
   history has a row. Only the column's own kind/state filters and folds hide
   rows. `session_285_column_lists_exactly_the_tabs`.
2. R2: a session of any project that is Queued, Starting, Active, or Unknown
   and has never had a tab gets one — in its own project's workspace, whether
   or not that project is shown — once; a closed one is not reopened by this.
   `session_285_a_new_session_in_another_project_gets_a_tab_and_a_row`.
3. R3: the column's rows and every tab group's session tabs follow one order,
   computed by one function: pinned sessions first, then the order file's
   order (sessions it does not list first, newest first, as today). A group's
   session tabs take the slots session tabs already hold there, so a document
   tab keeps its place. The column no longer sorts rows by state.
   `session_285_rows_and_tabs_share_one_order`.
4. R4: dragging a session row in the column and releasing it among its own
   project's rows of the same pin state moves it there, writes the order file,
   and the project's tabs follow. `session_285_dragging_a_row_reorders_its_tabs`.
5. R5: dropping a tab on a tab writes the order its group then shows into the
   order file, so the rows follow. `session_285_dragging_a_tab_reorders_the_rows`.
6. R6: the row's and the tab's context menus offer 「ピン留め」, or
   「ピン留めを外す」 on a pinned session. Pinning puts the session first in
   both places with a pin mark (hover 「ピン留め中」), and is written to
   `session-order.json` (`pinned`); a file written before the field existed
   reads as nothing pinned, and pins survive a relaunch.
   `session_285_pin_from_row_or_tab_moves_it_first_in_both`,
   `session_285_pins_survive_a_relaunch`.
7. R7: a pinned session's tab has no close button, and ⌘W, 「画面から外す」, and
   every other close leave it open with the brief notice
   「ピン留め中のセッションは閉じられません。ピン留めを外してから閉じてください。」.
   Deleting the session still removes its tab — in every project's
   workspace, shown or not — and its pin, so no row outlives its session.
   `session_285_a_pinned_tab_does_not_close`.

## Behaviour

- Open three sessions in project p: three tabs, three rows, same order.
  Close one tab: two of each. Let one end (`/exit`): its tab and row stay,
  the mark shows 完了; × closes both.
- A session started in project q while p is shown: q's header counts it and
  q's rows list it at once; switching to q shows its tab.
- New sessions appear first among the unpinned, in the column (as today) and
  in the tab strip (left, after the pinned tabs) — the strip used to add them
  at its right end.
- Right-click a row or a tab → 「ピン留め」: it moves first in both, a pin mark
  sits before its title in the row and in place of the tab's ×. Right-click →
  「ピン留めを外す」: it returns to its place in the order, and gets its × back.
- ⌘W on a pinned tab: nothing closes; a brief notice says
  「ピン留め中のセッションは閉じられません。ピン留めを外してから閉じてください。」.
- Drag a row (hover adds 「ドラッグで並べ替え」): an accent line shows where it
  lands among its project's rows of the same pin state; release moves the row
  and the tabs. A release outside, or in its own gap, moves nothing; Esc
  cancels.
- Empty: a project with no tabs shows 「開いているセッションはありません」
  (was 「起動中のセッションはありません」).
- Error: a failed write of the order file shows the existing
  「並び順を保存できませんでした: {error}」; the move or pin holds for this run.
- Too large / damaged file: read as today (1 MiB ceiling; unreadable means no
  order and no pins).
- Interrupted: the file is replaced atomically, as today.

## Design

Placement: `EXTEND` — the column (`ui_workspace_projects` in
`src/app/screens.rs`), the tab model (`WorkspaceTabs` in
`src/ui/session_tree.rs`), the tab lifecycle (`sync_workspace_live_sessions`,
`close_workspace_tab` in `src/app.rs`), and the order file
(`SessionArrangement`, change 181) all exist. Pinning is new behaviour inside
them, hence route `feature`; it needs the single list, so it is one change.

- `src/ui/session_tree.rs`
  - `SessionArrangement.pinned: Vec<Uuid>` (`#[serde(default)]`).
  - `workspace_session_rank(&Session, &SessionArrangement)` — the one key:
    (not pinned, unlisted-first, listed position, newest first). New
    `sort_workspace_sessions` sorts by it; the column and the tabs both use it.
  - `WorkspaceTabs::arrange(&mut self, key)`: within each group, reorders the
    session tabs by `key` among the slots session tabs hold (an insertion sort
    over those slots), so document tabs stay where they are; the front tab
    stays the front tab. No heap allocation; each comparison looks the session
    up in the store and its id up in the order file, so a frame costs
    O(tabs² · sessions) lookups — tens of tabs, a few hundred sessions.
  - `WorkspaceTabs::session_ids()` — the session tabs, every group.
  - `follow(order: &mut [Uuid], sequence: &[Uuid])`: writes `sequence`'s
    members of `order` into the slots they hold in `order`, in `sequence`'s
    order; ids `order` does not hold are ignored (R5). The caller passes the
    project's full level, sorted by `sort_workspace_sessions`, so every
    session has a slot.
- `src/app.rs`
  - `sync_workspace_live_sessions`: new live sessions (adds `Queued`) of every
    project open in that project's workspace (the shown one, or the entry in
    `project_workspaces`, created when absent); every workspace in
    `project_workspaces` drops session tabs whose session the store no longer
    holds (the shown one already does, `screens.rs` after the sync); then
    every workspace is `arrange`d by `workspace_session_rank`. Runs per frame as today: lookups,
    no allocation beyond today's.
  - `project_tab_sessions(project) -> Vec<Uuid>`: the column's source (R1).
  - `close_workspace_tab`: refuses a pinned session (R7) with
    `notice_briefly`. Removing a deleted session's tab stays a direct
    `workspace.retain`, and prunes the pin.
  - `toggle_session_pin(id)`: flips, prunes to stored sessions, saves.
  - `follow_tab_drop(group)`: after `insert_tab`, take the project's sessions
    sorted by `sort_workspace_sessions` as the full level, `follow` the
    group's session sequence into it, and write the whole level into the
    order file as `finish_column_drag` does (pruned, saved, the same error
    notice). The pin order is then re-imposed by the rank, so a tab dropped
    across the pin boundary lands at the boundary.
  - `workspace_project_drag: Option<Uuid>` becomes
    `workspace_column_drag: Option<ColumnDrag>`.
- `src/app/screens.rs`
  - `ui_workspace_projects`: rows from `project_tab_sessions` instead of every
    store session minus finished; no state sort; rows sense drag and collect
    `ColumnTargets` for the dragged row's project and pin state; the drop goes
    through the existing `ui_column_drop` → `finish_column_drag`. Pin mark
    (`ICON_PIN`, `chip` 11.5px, `text_muted`) before the title; menu entry.
  - Tab strip: `ICON_PIN` instead of the × for a pinned session; menu entry;
    `follow_tab_drop` after a successful `insert_tab`.
- `src/i18n_tables.rs`: English and Korean rows for the four new strings
  「ピン留め」「ピン留めを外す」「ピン留め中」 and the R7 notice, and for
  「開いているセッションはありません」; 「ドラッグで並べ替え」 already has rows.
- `README.md`, `README.ja.md`, `README.ko.md`: the `session-order.json`
  bullet says pins live there too, and the column lists the open tabs.
- Persisted shape: the store is untouched. `session-order.json` gains an
  optional field; old files read as before; nothing migrates.
- External commands: none. Background work: none. Draw path: no I/O; the order
  file is written only on a drag release or a pin toggle.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | The pin mark reads `palette.text_muted`, the drop line `palette.accent`; no new role. |
| Icons | Yes | `ICON_PIN` exists in `src/glyphs.rs` and `ICON_VOCABULARY`; no new glyph. |
| Identifier SSOT | Yes | The file name stays `SESSION_ARRANGEMENT_FILE_NAME`; the order is one function used by both views, which is what R3's test pins. |
| Durability | Yes | Not the store. The order file keeps `write_file_atomically`, its 1 MiB ceiling read on the open handle, and "unreadable is empty"; the new field is `serde(default)`. Pins are pruned to stored sessions, so the file stays bounded by the store. No cancellation, migration, or tmux path changes. |
| Subprocess safety | No | No child process, no launch path. |
| Documentation | Yes | The README bullet in all three languages together. |
| Local-first | Yes | A local file; no network, telemetry, or `unsafe`. |
| Budgets | Yes | No new scan or output path; per-frame work is bounded by the open tabs. |
| Draw path — `.claude/rules/rust.md` | Yes | No I/O per frame; `arrange` sorts in place without allocating (cost stated in Design); the column's per-frame list replaces the one it builds today. |
| Buttons and sizes — `.claude/rules/palette-and-glyphs.md` | Yes | Menu entries use `quiet_button`; the pin mark's size is the `chip` level. |

## Flagged concerns

- **The column stops grouping rows by state** (change 237's order): a
  re-arranged layout, so the screen goes to the person (`screen.md`).

## Acceptance

- R1–R7 tests pass, each watched failing first; existing tests that asserted
  the old membership or state order are updated, not deleted.
- A mutation sweep catches: the column drawing every live session again, a
  finished tab dropped from the column, a deleted session's tab kept in a
  project not shown, a document tab moved by the sort, another project's new session left
  without a tab, the tabs keeping their own order, a row drag not written, a
  tab drop not followed, a pin not first, a pin not saved, and a pinned tab
  closing.
- `make q.fast` passes; CI passes before the merge into `main`; packaged from
  `main` after landing per `.claude/skills/ship/SKILL.md`.

## Rejected alternatives

- Column rules (live and unread sessions; closing a tab hides the row): the
  person chose tab rules.
- Keeping two lists and syncing them on each event: the drift the person
  described comes from exactly that; one list cannot drift.
- New tabs at the strip's right end, with the column also newest-last: changes
  the column's long-standing newest-first order and its file semantics
  (unlisted first); one rule had to win, and the column's is the persisted one.
- Pins in the store: a paused surface and a schema bump for a per-machine
  preference that already has a file.
