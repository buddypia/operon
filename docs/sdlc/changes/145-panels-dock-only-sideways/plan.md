# Plan: each workspace panel docks on any of the four edges

- **Spec**: `./spec.md`
- **Approved**: 2026-10-13
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `DOCK_DEFAULT_H`, `DOCK_MIN_H`, `DOCK_MAX_H`, `DOCK_CENTER_MIN_W`, `DOCK_CENTER_MIN_H`, `DOCK_DROP_BAND`, `DOCK_GAP` |
| `src/ui/session_tree.rs` | `DockEdge`, `DockPanel`, `DockLayout`, `DockRects`, `dock_drop_edge`, `dock_rects`, `split_lengths`; remove `SidebarSide`, `SessionColumnsLayout`, `sidebar_drop_side`, inline `mod tests`; `ui_session_inspector` draws one edge's group |
| `src/app.rs` | replace six side-panel fields with `dock` and `dock_drag` |
| `src/app/screens.rs` | `ui_terminal_workspace` lays out edges + centre; `ui_dock_edge`, `ui_dock_drop`, `ui_dock_menu`; `ui_terminal_panel` drops its side panel; list heading grip |
| `src/i18n_tables.rs` | new ids in EN and KO; remove the four 「サイドバーを…」 ids |
| `src/tests.rs` | requirement tests; migrate side-panel tests to the dock model |
| `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | one sentence on docking |

## Order of work

1. Pure model and geometry in `session_tree.rs` + constants; the requirement
   tests 1–4 written first against stubs and watched failing, then filled in.
   The tree does not compile between removing `SidebarSide` and step 3, so
   steps 1–3 land as one commit.
2. App state: `dock`, `dock_drag`.
3. Drawing: workspace layout, edge drawing, splitters, drop overlay, menus,
   list grip; `ui_terminal_panel` loses the side panel.
4. Strings in both tables; MANUAL sentence ×3.
5. Migrate the existing tests (122–144) to the dock model; add tests 5–7.
6. `make q.fast`; `cargo test --locked dock`, `sidebar`, `stacked`,
   `session_file_tree`, `i18n`, `glyph`; check-bands.
7. Review (craft: rust-reviewer), commit, push, CI, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Scroll/id collisions when a panel changes parent | list or tree jumps to top, or egui id-clash warning | ids keyed by panel (`push_id`); test 5 renders every edge |
| Terminal resize churn while dragging a splitter | tmux resize per frame | existing `request_terminal_resize` dedupe; unchanged path |
| A test count drop from removing inline tests | commit gate refuses | migrate, never delete; count compared before commit |
| Drop overlay not drawn above edges | zones hidden behind panels | painted on a foreground layer after the edges |
| Unused message ids left in tables | i18n table tests fail | remove ids with their last caller |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked dock` — the new tests pass; each was watched failing first.
- CI on the branch: `test result: ok. N passed; 0 failed; 6 ignored`.
- Running app: files to the bottom, list to the top, middle release and Escape
  do nothing.

## Departures from the plan
