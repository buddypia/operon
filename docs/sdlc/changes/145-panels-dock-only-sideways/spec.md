# Spec: each workspace panel docks on any of the four edges

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `test_dock_layout_moves_one_panel_and_leaves_the_rest` — moving one panel
   (the session list, or one of files / conversation / changes) to an edge
   changes that panel's edge only; a view moved to an edge becomes that edge's
   showing tab; the edge it left falls back to its first remaining view.
2. `test_dock_drop_edge_partitions_the_workspace` — a release point inside the
   workspace and within 30% of an edge's span from that edge picks the nearest
   edge; the middle and anything outside the workspace pick nothing.
3. `test_dock_rects_keep_the_terminal_its_minimum` — the rectangles for the
   occupied edges: top and bottom span the full width, left and right fill the
   band between them, and the centre keeps `DOCK_CENTER_MIN_W` ×
   `DOCK_CENTER_MIN_H`. Each axis is fitted on its own: a shortage of height
   shrinks top and bottom to `DOCK_MIN_H`, then drops bottom, then top; a
   shortage of width shrinks left and right to `SIDEBAR_MIN_W`, then drops
   right, then left. The edge holding the session list is never dropped. A
   dropped edge's views stay docked there and are drawn again once the window
   has room. Sizes that are not finite fall back to the defaults.
4. `test_dock_split_lengths_keep_both_minimums` — two sections sharing an edge
   split its length by the stored share, each keeping its minimum, halving when
   the edge is too short for both.
5. `test_every_panel_docks_on_every_edge_in_the_workspace` — rendering the
   workspace with each panel on each edge draws that panel's content inside that
   edge's rectangle (session list heading, file tree row, conversation turn,
   changes tab).
6. `test_dock_same_edge_list_and_views_share_it` — the session list and a view on the
   same edge are drawn together, list first (above on left/right, left of it on
   top/bottom); views on one edge are tabs of one group showing one at a time.
7. `test_dock_session_list_has_its_own_grip` — the session list heading leads
   with `ICON_DRAG_HANDLE` like the view tabs, so it can be dragged.
8. `test_dock_layout_moves_a_group_together` — moving the views on one edge as a
   group (the 「パネルの配置を変更」 menu) moves every one of them and keeps the
   one that was showing; the session list on that edge stays.
9. `test_dock_drop_moves_and_tells_the_person` — finishing a drag on an edge the
   panel is not on moves it and leaves the brief notice
   「{panel}を{edge}に移動しました」; finishing in the middle or on its own edge
   moves nothing and leaves no notice; the drag ends either way.
10. The existing side-panel tests (changes 122–144) are rewritten against the
   dock model rather than deleted: defaults, fold and reopen, the selected tab's
   content, Escape cancelling a drag, and the stacked list-over-files layout.

## Behaviour

- **Grips.** Each view tab (`⠿ ファイル`, `⠿ 会話 N`, `⠿ 変更 N`) and the session
  list heading (`⠿ プロジェクト別セッション`) has a grip. Hover shows the open-hand
  cursor and the tooltip 「ドラッグして移動」. A click on a view tab still selects it.
- **Dragging.** While a panel is dragged the cursor is the closed hand, the four
  edges of the workspace show drop zones (a faint frame each), and the zone under
  the pointer is highlighted in the accent colour. A tooltip beside the pointer
  says 「{panel}を{edge}に移動」 (e.g. 「ファイルを下部に移動」) over a zone, and
  「中央で離すとキャンセル」 in the middle. Edge words: 左側, 右側, 上部, 下部.
  Panel words: セッション一覧, ファイル, 会話, 変更.
- **Drop.** Releasing over a zone moves that one panel there and shows the brief
  notice 「{panel}を{edge}に移動しました」. Releasing in the middle, outside the
  workspace, or on the edge it is already on changes nothing and shows nothing.
  Escape cancels the drag.
- **Keyboard and menu alternative.** Right-clicking a grip opens a menu with
  「左側に移動」「右側に移動」「上部に移動」「下部に移動」; the entry for the
  current edge is disabled. The view group's former 「サイドバーを左/右側に移動」
  button becomes a menu button 「パネルの配置を変更」 with the same four entries,
  moving every view in that group together.
- **Shared edge.** Views on one edge are tabs of one group; the group shows its
  last selected view. The session list and a group on one edge are split by a
  draggable rule — stacked on left/right (list above), side by side on top/bottom
  (list on the left).
- **Sizes.** The rule between each edge and the terminal resizes that edge:
  left/right width within `SIDEBAR_MIN_W`–`SIDEBAR_MAX_W`, top/bottom height
  within `DOCK_MIN_H`–`DOCK_MAX_H`. Defaults: left and right `SIDEBAR_DEFAULT_W`
  (268), top and bottom `DOCK_DEFAULT_H` (240).
- **Defaults.** Session list left; files, conversation, and changes on the right,
  files showing — what the workspace looks like today (change 139).
- **Empty and folded states.** 「サイドパネルを畳む」 folds every view group
  (wherever they are docked) and the terminal header's 「サイドパネルを表示」
  brings them back; the session list is not folded. Without a selected session,
  or with the history open, the view groups are not drawn. A session that
  belongs to no project has only the conversation view; a group holding only
  files and changes is then not drawn.
- **Too narrow / too short.** The terminal keeps at least `DOCK_CENTER_MIN_W` × `DOCK_CENTER_MIN_H` (300 × 360). Edges shrink
  to their minimum first; then the edges that do not hold the session list are
  dropped, bottom before top and right before left. The session list's edge is
  never dropped (as today: the list column is always drawn).
- **Not remembered.** The arrangement lasts until the app quits, as the side did.

## Design

### Placement verdict: `EXTEND`

The side panel's layout types live in `src/ui/session_tree.rs`
(`SidebarSide`, `SessionColumnsLayout`, `sidebar_drop_side`,
`stacked_sidebar_heights`) and the workspace in `src/app/screens.rs`
(`ui_terminal_workspace`, `ui_session_column`, `ui_sidebar_tab_drop`,
`ui_terminal_panel`). Those grow; no new module.

- `src/ui/session_tree.rs`
  - Remove `SidebarSide`, `SessionColumnsLayout`, `sidebar_drop_side` (and the
    inline `mod tests` there, which `CLAUDE.md` forbids; its two tests become
    dock-rect tests in `src/tests.rs`).
  - Add `DockEdge { Left, Right, Top, Bottom }` with `ALL`, `index`, `label`
    (message id), `is_horizontal`.
  - Add `DockPanel { Sessions, View(InspectorTab) }` with `ALL`, `label`.
  - Add `DockLayout { edges: [DockEdge; 4], showing: [Option<InspectorTab>; 4],
    sizes: [f32; 4], splits: [f32; 4] }` with `Default`, `edge_of`, `move_panel`,
    `move_views(from, to)`, `select`, `views_at(edge) -> impl Iterator` (no allocation), `shown_view(edge,
    has_project)`.
  - Add `dock_drop_edge(pos, area) -> Option<DockEdge>`, `dock_rects(area,
    occupied, sizes, keep) -> DockRects { edges: [Option<Rect>; 4], center: Rect }`,
    and `split_lengths(total, share, min_first, min_second)`;
    `stacked_sidebar_heights` becomes a call to `split_lengths`.
  - `ui_session_inspector` takes the `DockEdge` it is drawing and draws only that
    edge's views; tab grips start a drag of `DockPanel::View(tab)`; the dock
    button becomes the 「パネルの配置を変更」 menu.
- `src/app/screens.rs`
  - `ui_terminal_workspace` allocates the workspace rect, computes `dock_rects`,
    draws each occupied edge (`ui_dock_edge`), the edge-to-centre splitters, the
    centre (history, terminal panel, or the empty state), then the drop overlay
    (`ui_dock_drop`, replacing `ui_sidebar_tab_drop`), which ends a drag through
    `finish_dock_drag(target)` — the move and its notice, testable without a
    frame.
  - `ui_session_column` becomes `ui_dock_edge`: list, group, or both with a rule.
  - `ui_terminal_panel` no longer draws a side panel; the terminal takes the
    centre's width.
  - `ui_terminal_session_tabs` heading gains the grip.
- `src/app.rs` — `session_inspector_tab`, `session_inspector_side`,
  `session_inspector_w`, `dragging_sidebar_tab`, `session_list_w`,
  `session_list_split` are replaced by `dock: DockLayout` and
  `dock_drag: Option<DockPanel>`. `show_session_inspector` stays.
- `src/config.rs` — `DOCK_DEFAULT_H` (240), `DOCK_MIN_H` (= `SIDEBAR_PANEL_MIN_H`),
  `DOCK_MAX_H` (560), `DOCK_CENTER_MIN_W` (300), `DOCK_CENTER_MIN_H` (360: the
  pane header plus the terminal's 320px floor), `DOCK_DROP_BAND` (0.3),
  `DOCK_GAP` (6, the rule between an edge and the terminal or two sections).
- `src/i18n_tables.rs` — the new message ids in the English and Korean tables;
  the four 「サイドバーを…」 ids are removed with their last caller.

No persisted shape changes; nothing runs on a background thread; no subprocess.
The draw path reads `DockLayout` (plain `Copy` arrays) and allocates nothing new.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Drop zones reuse `accent`, `border_subtle`, `raised` as the change-144 overlay does; no new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Reuses `ICON_DRAG_HANDLE` (change 144) on the list heading; the placement menu reuses `ICON_RESTORE`. No new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No identifier is shared or persisted; sizes are constants in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Layout is in-memory app state only, as `session_inspector_side` was; the store is untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No process is spawned. |
| Documentation — user-facing docs change in all three languages together | yes | `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` ("The terminal pane") gain one sentence each on dragging a panel by its grip to any edge. `README*` say the side panel opens to the right of the terminal, which stays the default, so they are unchanged. Strings go in both tables. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan or output path. |

## Flagged concerns

- **egui ids when a panel moves edges.** A panel drawn in a different parent
  would lose its scroll state if its id came from the parent. Answer: every edge
  section is drawn under `push_id(("dock", panel-or-group))` keyed by the panel,
  not the edge, so moving it keeps its scroll position.
- **Release detection.** The drop reads the pointer's primary release from input,
  as the change-128 overlay does, so a release over any widget still drops.
- **The 640px rule's tests.** `SessionColumnsLayout`'s 640px threshold is replaced
  by `dock_rects`' minimums; its tests are rewritten to the new rule (right edge
  dropped once the centre would fall under 300px), not deleted.

## Acceptance

- `cargo fmt --check` — nothing.
- `cargo clippy --locked -- -D warnings` — nothing past the compile lines.
- `cargo test --locked dock`, `sidebar`, `stacked`, and `session_file_tree` pass —
  every requirement test above has `dock` in its name; CI runs the suite.
- In the running app: drag 「ファイル」 to the bottom — the tree spans the window
  under the terminal while 会話/変更 stay right; drag 「プロジェクト別セッション」
  to the top — the list runs across the top; release in the middle — nothing
  moves; Escape mid-drag — nothing moves.

## Rejected alternatives

- **All panels as tabs on a shared edge.** Simpler, but the list and the file
  tree on one edge could never be seen together; the person chose the split.
- **`egui_dock` crate.** A new dependency (paused surface) and a tab model that
  owns the whole central area, including the terminal's focus and IME handling.
- **Persisting the layout now.** Changes the store shape; out of scope (intent).
