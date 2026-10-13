# Spec: a docked panel can be moved somewhere it is not drawn, and the app says it moved

- **Status**: approved

## Requirements

1. A move that would leave a panel without room is not made. This holds for a
   drag, for a grip's right-click menu, and for the placement menu. The check is
   made with every view open: views that are folded, or that a project-less
   session does not offer, are judged where they would be drawn once shown, so a
   move made while they are hidden cannot hide them later. Every panel drawn
   before a made move is drawn after it, and every panel the move sends to
   another edge is given room there, at the current window size.
   `test_dock_move_keeps_every_shown_panel_on_screen`
2. A refused move says why. The notice is 「ウィンドウが小さいため{edge}には置けません」
   and the layout is unchanged, for the placement menu's group move too.
   `test_dock_refused_move_says_why`
3. A drag driven by pointer events — press on a grip, move, release on an edge —
   moves the panel.
   - With the button still held, Escape ends it and moves nothing.
   - A release on the panel's own edge moves nothing and says nothing.

   `test_dock_drag_by_pointer_events`. This closes a coverage gap rather than a
   bug: these assertions already pass on main, and the test is watched failing
   by mutation (the Escape branch deleted; the drag never armed). The Escape
   test in change 125 is made to hold the button down for the same reason.
4. The drag hint names what a release does: the move, the refusal, or
   「ここで離すとキャンセル」 — in the middle and on the panel's own edge alike.
   `test_dock_drop_hint_names_what_a_release_does`

## Behaviour

- **Dragging.** A zone the panel cannot go to is outlined in `palette.warning`,
  not the accent, and the hint reads 「ウィンドウが小さいため{edge}には置けません」.
  Releasing there leaves the layout as it is and shows the same text as a notice.
- **Menus.** In the right-click menu and the placement menu, an entry that would
  be refused is greyed, with the same text on hover.
- **Own edge or middle.** The hint is 「ここで離すとキャンセル」. It was
  「中央で離すとキャンセル」, which is wrong on the own edge.
- **Before the first draw.** Until the workspace has been drawn there is nothing
  on screen to lose, so a move is made as before.
- **Shrinking the window.** A window that shrinks after a move still gives up
  edges as it did, and growing it brings them back. Not changed here.
- **English.** Inside a sentence the session list reads "the session list".

## Design

### Placement verdict: `EXTEND`

`DockLayout` in `src/ui/session_tree.rs` (change 145) grows; nothing new is a
concern of its own.

- `OperonApp::dock_area: Option<egui::Rect>` is the rect the workspace was last
  drawn into; `ui_terminal_workspace` writes it each frame.
- `DockLayout::rects(area, views: Option<bool>)` is the one place occupancy is
  decided (`views` is `Some(has_project)` while views are open). The workspace
  draws from it and the check reads it, so they cannot disagree.
- `DockLayout::keeps_on_screen(&after, area)`: every panel `rects(area,
  Some(true))` gives room to in `self` it also gives room to in `after`, and
  every panel whose edge differs between the two is given room in `after`.
- `OperonApp::dock_fits(&after)` and `dock_commit(after, edge, done)` are the
  single gate that `dock_panel_to` and the new `dock_views_to` (the placement
  menu) both pass through. The layout a move would produce is a copy of
  `self.dock` with `move_panel` / `move_views` applied: `DockLayout` is `Copy`.
- `dock_drop_hint(panel, from, at, fits) -> String` is a pure function, for
  requirement 4.
- The placement menu's button is drawn as `icon_button` draws its button (28px,
  no stroke, focus ring) through `egui::menu::menu_custom_button`, at the call
  site; no new helper.

There is no persisted shape, no external command, and no background work. The
check runs four `dock_rects` calls per frame while a drag is in progress, and one
per menu entry while a menu is open: arithmetic on eight floats.

## Policy conformance

- **`.claude/rules/rust.md`:** the only allocation in the draw path is the hint
  string, which each drag frame already builds.
- **`.claude/rules/palette-and-glyphs.md`:** `palette.warning` is an existing
  role; the menu button takes `CONTROL_HEIGHT` and the `icon_button` font size.
- **`CLAUDE.md`:** every new string goes through `tr`/`tf!` with EN and KO rows.

## Acceptance

- The four tests above pass. Requirements 1 and 2 fail on main's
  `dock_panel_to` (a panel hidden; no refusal notice); requirement 4 fails on
  main's hint text; requirement 3 fails under the two mutations it names.
- The 145 dock tests still pass.
- fmt and clippy are clean.

## Rejected alternatives

- **Fold a given-up edge's views into the session list's edge.** This keeps them
  reachable after any resize. But a panel the person put on the bottom would
  appear on the left, which is a second rule to learn and a layout the person
  did not choose. Refusing the move is honest at the moment it matters.
- **Let the terminal go below its minimum.** `DOCK_CENTER_MIN_W` (300) and
  `DOCK_CENTER_MIN_H` (360, the pane header plus the terminal's 320px floor)
  exist because the terminal is unusable below them.
- **Judge only what is on screen now.** Simpler, but a move made with the views
  folded or a project-less session selected would hide them the moment they
  open.
