# Spec: sessions dropped on a terminal split it, as editor groups do

- **Intent**: `./intent.md`
- **Screen**: `./screen.md`
- **Status**: approved

## Requirements

1. A drop on a terminal is judged by where the pointer is: within 30% of an
   edge (`DOCK_DROP_BAND`) it is that edge, the nearer one when two qualify;
   otherwise the middle. — `pane_drop_follows_the_pointer_like_an_editor_group`
2. Dropping a session on a terminal's edge puts the two beside each other on
   that side — left/right side by side, top/bottom stacked — each taking half.
   — `dropping_on_an_edge_splits_the_terminal_on_that_side`
3. A session is never on screen twice: dropping one already shown moves it, and
   its old place closes. Dropping a shown terminal on another's middle swaps
   the two; dropping a session not on screen there replaces the target.
   — `a_session_dropped_twice_moves_rather_than_copies`
4. At most four terminals (`PANE_MAX`), none narrower than `PANE_MIN_W` (320px)
   or shorter than `PANE_MIN_H` (200px): a drop that would break either is
   refused and changes nothing. — `a_split_past_four_or_too_small_is_refused`
5. Taking a terminal off the screen, or the session being removed from the
   store, gives its room to its neighbour and leaves the session running.
   — `closing_a_terminal_gives_its_room_to_the_neighbour`
6. Selecting a session (list click, ⌘J, the attention queue) focuses the
   terminal already showing it, or else shows it in the focused terminal.
   — `selecting_a_session_replaces_the_focused_terminal_only`
7. With two terminals in the layout the workspace draws both, each with its own
   title, side by side. — `two_split_sessions_are_both_drawn`
8. The two-second session poll captures the output of every terminal on screen,
   not only the focused one, and every terminal on screen counts as read.
   — `every_terminal_on_screen_is_polled_and_read`
9. During a drag, the zone a release would fill is painted, and the hint says
   which: 「左に分割」「右に分割」「上に分割」「下に分割」「ここに表示」, or
   「これ以上分割できません」 when refused. — `pane_drop_hint_names_the_zone`

## Behaviour

- **Starting a drag**: a session row in the list (the existing reorder drag),
  or the `⠿` grip in a terminal header, shown only while two or more terminals
  are on screen. Hover on the grip: 「ドラッグして分割位置を変更」.
- **While dragging over a terminal**: the zone (a half, or the whole for the
  middle) is filled `palette.accent` at 0.12 with a 2px `palette.accent` frame;
  refused: `palette.warning` frame. No zone on the terminal already showing the
  dragged session. A tooltip beside the pointer says the zone's word.
- **Release**: over a zone, the layout changes and the dropped session takes the
  keyboard. Anywhere else, or Escape, nothing changes; the list still reorders
  when released among its own rows.
- **Focus**: the focused terminal (`selected_session`) has a 2px
  `palette.accent` frame while two or more are shown; a primary press anywhere
  in another terminal focuses it. Only the focused terminal forwards keys,
  owns IME, polls for its echo, and resolves path links; the side views follow
  it.
- **Closing**: `✕` (「画面から外す」) on each header while two or more are
  shown. When one terminal is left the workspace is exactly today's.
- **Empty / error**: a terminal whose output has not arrived says
  「ターミナル出力を待っています…」 as today. A stopped session stays in its
  terminal with its 「開始」 verb, as today.
- **Too long**: titles truncate in each header as today.
- **Not kept** across restarts, like `DockLayout`.

## Design

**Placement: `EXTEND`.** `src/ui/session_tree.rs` already holds the workspace's
in-memory arrangement (`DockLayout`, `ColumnDrag`, `dock_drop_edge`); the pane
tree joins it. Drawing stays in `src/app/screens.rs` (`ui_terminal_workspace`,
`ui_terminal_panel`).

- `src/ui/session_tree.rs`: `PaneDrop { Left, Right, Top, Bottom, Center }`,
  `PaneNode` (a leaf of one `Uuid`, or a split with an axis, a share, and two
  boxed children), `PaneLayout { root: Option<PaneNode> }` with `sessions`,
  `contains`, `show`, `place`, `remove`, `retain`, `rects(area)` (pane rects
  plus each rule's rect and split index), `set_share`, and `fits(area)`; free
  functions `pane_drop(pos, rect)`, `pane_drop_zone(rect, drop)`, and
  `pane_drop_hint(drop, refused)`. All pure.
- `src/app.rs`: `OperonApp.panes: PaneLayout` and `pane_drag: Option<Uuid>`;
  `select_session` calls `panes.show`; `is_session_on_screen` reads the panes;
  `request_session_poll` captures every visible session;
  `BackgroundResult::SessionPoll.output` becomes a `Vec`.
- `src/app/screens.rs`: `ui_terminal_workspace` draws one `ui_terminal_panel`
  per pane rect with a rule between splits (`paint_splitter_v`/`_h`), then
  `ui_pane_drop` handles a held `column_drag` session or `pane_drag`.
  `ui_terminal_panel` gains a `split: bool` argument for the grip, `✕`, and
  frame, and confines echo polling, link resolution, and keyboard focus to the
  focused session.
- `src/config.rs`: `PANE_MAX`, `PANE_MIN_W`, `PANE_MIN_H`.
- Nothing persisted, no external command, no background thread added: the poll
  that already runs every two seconds captures up to four panes instead of one,
  each through the same bounded `tmux_capture_with_cursor`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes | only existing roles: `accent`, `warning`, `raised`, `border_subtle`, as `ui_dock_drop` uses them |
| Icons | yes | existing `ICON_DRAG_HANDLE`, `ICON_CLOSE`; no new constant |
| Identifier SSOT | no | no string crosses a boundary; the three limits are constants in `src/config.rs` |
| Durability | no | nothing persisted; `DockLayout` precedent; store untouched |
| Subprocess safety | yes | no new child: the poll reuses `tmux_capture_with_cursor`, bounded at four panes |
| Documentation | yes | strings through `tr` with rows in both tables of `src/i18n_tables.rs`; README has no workspace section to change (checked) |
| Local-first | no | nothing leaves the machine; no `unsafe` |
| Budgets | yes | four captures per poll at most (`PANE_MAX`), each under the existing capture ceiling |

## Flagged concerns

- **Screen approval** — a re-arranged layout is a person's call
  (`approval.md`). The question went unanswered; the work proceeds in the
  worktree on the recommended screen and does not land until the person
  approves the screen or the screenshot.

## Acceptance

- `cargo test --locked` passes, including the nine tests named above.
- In the running app: drag a session row onto the right third of the terminal;
  the right half lights with 「右に分割」, and on release two terminals sit side
  by side, both updating. Click the left one: its frame turns accent and typing
  goes there. `✕` on one returns to a single terminal.

## Rejected alternatives

- A fixed 2-up/4-up grid toggle: not what was asked; position-by-drop is.
- Persisting the arrangement: the docked panels it sits beside are not kept, and
  a stored shape is a paused surface for a recoverable preference.
- Tabs inside a split: a separate change if wanted.
- Making `selected_session` a set: every consumer of the focused session (views,
  search, ⌘J, path menu) would need to choose one again; one focus plus a layout
  keeps them as they are.
