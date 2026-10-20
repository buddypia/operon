# Spec: the person arranges the projects and sessions of the session column

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. **Today's order until something is moved.** With no arrangement, the column
   draws projects by name (case-insensitive) and each project's sessions
   newest first, exactly as on main. Once an arrangement exists, a project or
   session it does not list — added after the last move — is drawn ahead of
   those it lists, in today's order, and the listed ones follow in the
   arranged order. —
   `test_change_181_column_keeps_todays_order_until_arranged`
2. **A move is relative to what is on screen.** A release lands before the row
   it is above, or after the last row drawn. Rows not drawn — under a status
   filter, or lifted into the 要対応 queue — keep their places relative to
   each other, and a release that would leave the order as it was reports no
   change. — `test_change_181_place_moves_relative_to_visible_neighbours`
3. **A session row is dragged within its own project.** Pressing on a session
   row, dragging, and releasing between two of that project's rows puts the
   session there; the column draws it there at once, and an app started again
   from the same data folder draws it there too. A release over another
   project's card, or outside the column, moves nothing: a session never
   changes project. A click on the row still opens its terminal, and a double
   click on its title still renames it. —
   `test_change_181_session_row_drag_reorders_within_its_project`
4. **A project card is dragged by its name.** Pressing on a project's name,
   dragging, and releasing between two cards puts the project there, kept
   across a restart the same way. The ＋ beside the name still starts a
   session. — `test_change_181_project_name_drag_reorders_the_column`
5. **A drag can be abandoned.** Escape with the button held ends the drag and
   moves nothing; so does a release where the drag began, and neither writes
   the file. A drag whose release the window never saw (focus left mid-drag)
   ends on the next frame with no button down. —
   `test_change_181_escape_or_a_release_in_place_moves_nothing`
6. **The file is small, and a bad one costs only the arrangement.** The
   arrangement lives in `session-order.json` beside the local index — never in
   the index. A missing, unreadable, unparsable, or over-ceiling file reads as
   no arrangement (requirement 1's order) and is replaced by the next move.
   Each save lists only projects and sessions the index still holds. —
   `test_change_181_arrangement_file_is_bounded_and_fails_towards_default`
7. **A failed save is said.** When the file cannot be written, the move still
   holds for this run and the person reads
   「並び順を保存できませんでした: {error}」. —
   `test_change_181_save_failure_is_said`
8. **A drag reaches rows off screen.** Held within 32px of the column's top
   or bottom edge, a drag scrolls the column towards that edge, so a project
   can be moved past the part of the list that is visible. —
   `test_change_181_drag_near_the_edge_scrolls_the_column`
9. **The gesture is named where a person points.** A session row's hover says
   「クリックでターミナル表示 · ダブルクリックで名前を変更 · ドラッグで並べ替え」;
   a project name's hover adds 「ドラッグで並べ替え」 under its name and path. —
   `test_change_181_hovers_say_rows_and_projects_can_be_dragged`

## Behaviour

- **Idle.** Nothing new is drawn. The pointer over a project's name shows the
  grab cursor.
- **Dragging a session.** The cursor is the grabbing hand. A 2px line in
  `accent` marks the gap the release would land in, across the card's inner
  width. Over another project's card, outside the column, or over the gap the
  row already sits in, no line is drawn — a release there moves nothing.
- **Dragging a project.** The same line, across the column, in the gap
  between two cards (or above the first, below the last).
- **Released.** The column redraws in the new order the same frame; nothing is
  said, since the list itself shows it.
- **Escape / lost release.** The line disappears and nothing moves.
- **Long list.** Within 32px of the column's visible top or bottom, the list
  scrolls 12px a frame towards that edge while the drag is held.
- **Filtered.** With status chips on, only drawn rows and cards are targets;
  hidden ones keep their places (requirement 2).
- **Empty.** A project with no sessions draws its header only; its card is
  still a project target. A column with one project or one session has
  nowhere to move it: no line is drawn.
- **New items.** A session started after a move appears at the top of its
  project's card; a project added after a move appears at the top of the
  column.
- **Error.** Save failure: `notice` 「並び順を保存できませんでした: {error}」; the
  order holds until quit. A bad file at startup: today's order, said nowhere —
  the arrangement is a preference, and the next move rewrites the file.
- **Too large.** The file is read through a 1 MiB ceiling counted on the open
  handle; past it, today's order (requirement 6).

## Design

**Placement**: `NEW_IN_MODULE`. `git grep -n "sort_by_key" src/app/screens.rs`
shows the column's two fixed sorts in `ui_terminal_session_tabs`; nothing
orders it by hand today (no `DUPLICATE`). The column's other per-machine state
— `DockLayout` — lives in `src/ui/session_tree.rs`, the session workspace
module, and the side-file pattern is `src/git/setup.rs`'s `setup-trust.json`.
Route `feature`.

- `src/ui/session_tree.rs` gains:
  - `SessionArrangement { projects: Vec<Uuid>, sessions: Vec<Uuid> }`,
    `Serialize`/`Deserialize` with `#[serde(default)]` fields;
    `load_session_arrangement(data_file)` (reads through
    `SESSION_ARRANGEMENT_FILE_MAX_BYTES` with `Read::take` on the open handle;
    any failure → `Default`) and `save_session_arrangement(data_file, &…)`
    (compact JSON through `write_file_atomically`).
  - `arrange_by(listed: &[Uuid], items: &mut [T], id)`: a stable sort keyed
    by position in `listed`, unlisted first. Position lookups over a list the
    size of the store; no allocation, so it may run per frame.
  - `place(order: &mut Vec<Uuid>, moved, visible: &[Uuid], slot) -> bool`:
    requirement 2.
  - `drop_slot(y, rects: &[Rect]) -> usize`: the index of the first row whose
    centre is below `y`, or `rects.len()`.
  - `ColumnDrag { Project(Uuid), Session { project: Uuid, session: Uuid } }`.
- `src/app.rs`: `OperonApp` gains `session_arrangement: SessionArrangement`
  (loaded in `from_state` beside `load_setup_trust`) and
  `column_drag: Option<ColumnDrag>` (in memory, as `dock_drag` is).
  `commit_session_arrangement()` prunes ids the store no longer holds, saves,
  and on `Err` sets `notice` (requirement 7).
- `src/app/screens.rs`, `ui_terminal_session_tabs`: the two sorts are followed
  by `arrange_by`; the session row's scope senses `click_and_drag`; the
  project name `Label` senses `drag` with `.selectable(false)`; while
  `column_drag` is set, the frame records the drawn rects of the dragged
  item's level (one small `Vec` per frame, only during a drag), paints the
  line, scrolls at the edges, and on the primary release calls `place` then
  `commit_session_arrangement`. Escape and the lost release end it as
  `ui_dock_drop` does.
- `src/config.rs`: `SESSION_ARRANGEMENT_FILE_NAME`,
  `SESSION_ARRANGEMENT_FILE_MAX_BYTES` (1 MiB), `COLUMN_DRAG_EDGE` (32.0),
  `COLUMN_DRAG_SCROLL_STEP` (12.0).
- `src/i18n_tables.rs`: the row hover string replaced in both tables;
  「ドラッグで並べ替え」 and 「並び順を保存できませんでした: {error}」 added.
- No persisted-shape change to the store; no external command; nothing on a
  background thread. One atomic write per completed move, on the frame of the
  release — as the setup approval and diff comments are written.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes | The line is `palette.accent`, the role `ui_dock_drop` already paints for its drop target; no new role, `DESIGN.md` unchanged. |
| Icons | no | No glyph is added; the cursor is egui's `Grab`/`Grabbing`. |
| Identifier SSOT | yes | The file name, ceiling, and two drag constants live once in `src/config.rs`; the round trip through `from_state` is requirement 6's test. |
| Durability | yes | Read `.claude/skills/durability-invariants/SKILL.md`: the store is untouched, `STORE_SCHEMA_VERSION` unchanged. The new byte ceiling is enforced by counting reads on the open handle (rule 3), and an over-ceiling file is reported as no arrangement, never half-read (rule 2). `write_file_atomically`'s `CommittedButNotSynced` is `Ok`: the order landed and is not reported as failed. |
| Subprocess safety | no | No child process. |
| Documentation | yes | `README.md`, `README.ja.md`, `README.ko.md` each gain one sentence beside the waiting-sessions bullet: drag to arrange, kept in `session-order.json`. |
| Local-first | yes | A local file beside the index; no network, no `unsafe`. |
| Budgets | yes | Read ceiling 1 MiB (about 27 000 ids as compact JSON); the file is pruned to the store's ids on every save, so it is bounded by the store. No new scan. |

## Flagged concerns

- **Two places the arrangement could live** — resolved: beside the store, as
  `setup-trust.json` and `diff-comments.json` are. In the store it would be a
  persisted-shape change on a paused surface for a preference nobody else
  would miss.

## Acceptance

- `cargo test --locked` passes in CI, including the nine tests named above;
  locally the nine and `every_` pass.
- In a dev build against a copy of the person's store: drag a session up two
  rows in a project with three, release — it stays; quit and reopen — it is
  still there. Drag a project to the top — same. Escape mid-drag moves nothing.

## Rejected alternatives

- **A field on `Project` and `Session`** — a store shape change and a schema
  bump on a paused surface, for a per-machine preference.
- **Reordering `store.sessions` itself** — changes the index's write order and
  every other list that reads it, and still needs a rule for new sessions.
- **Kept in memory only, like the dock layout** — the intent asks for the
  arrangement to survive a restart.
- **「上へ移動」/「下へ移動」 menu items instead of a drag** — ten clicks to move
  one project ten places; the drag is what was asked for.
- **egui's `dnd_drag_source` / `dnd_drop_zone`** — they wrap the row in a new
  frame and take its click; the row already owns click, double click, and
  context menu, and the dock drag's own state is the pattern this column has.
