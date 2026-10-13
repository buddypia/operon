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
| `src/app/screens.rs` | `ui_terminal_workspace` lays out edges + centre; `ui_dock_edge`, `ui_dock_drop`, `ui_dock_menu`, `finish_dock_drag`; `ui_terminal_panel` drops its side panel; list heading grip |
| `src/i18n_tables.rs` | new ids in EN and KO; remove the four 「サイドバーを…」 ids |
| `src/tests.rs` | requirement tests; migrate side-panel tests to the dock model |
| `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | one sentence on docking |

## Order of work

Nothing compiles between removing `SidebarSide` and migrating its tests, so
steps 1–5 land as one commit and no commit has a broken suite.

1. Constants in `src/config.rs`; the dock types in `src/ui/session_tree.rs`
   with stub bodies that return nothing (no `todo!`); app state `dock` and
   `dock_drag` replacing the six fields; a stub `finish_dock_drag`.
2. Drawing against that model: workspace layout, edges, splitters, drop
   overlay, menus, list grip; `ui_terminal_panel` loses its side panel. With
   the stubs every panel lands nowhere.
3. Migrate the existing side-panel tests (122–144) and the inline
   `session_tree.rs` tests to the dock model so the test binary compiles; the
   `#[test]` count does not fall. The tests that used `render_panel_row_shapes`
   (the terminal panel alone) move to the workspace harness
   (`render_terminal_panel_shapes`, which calls `ui_terminal_workspace`).
4. Requirement tests 1–9. Run `cargo test --locked dock` and watch them fail
   against the stubs.
5. Fill in the model, geometry, and `finish_dock_drag`; 1–9 and the migrated
   tests pass. Strings in both tables; MANUAL sentence ×3.
6. `make q.fast`; `cargo test --locked dock`, `sidebar`, `stacked`,
   `session_file_tree`, `i18n`, `icon`; `bash scripts/check-bands.sh`.
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
- `cargo test --locked dock` — the nine requirement tests pass; each was watched
  failing in step 4.
- CI on the branch: `test result: ok. N passed; 0 failed; 6 ignored`.
- Running app: files to the bottom, list to the top, middle release and Escape
  do nothing.

## Departures from the plan

- `cli_restore_is_in_history_not_in_the_overview` read `screens.rs` for an
  exact indented line; the history now draws in the dock's centre child, so the
  assertion reads the source without whitespace and names `&mut center`.
- The new empty-state line says 「一覧からセッションを選んでください。」 instead of
  「左の一覧から…」: the list need not be on the left any more.
- Four tests fail locally and not in the change: two hook tests read this
  session's `CLAUDE_PROJECT_DIR`, and one socket test exceeds the Unix path limit
  under this session's long `$TMPDIR`. CI on the branch is the judge.
- The running-app check could not be made by this session: computer-use
  onboarding is unfinished and screen capture is refused on this checkout. The
  layout is proved by the workspace render tests; the eye on the screen is the
  person's after the install.
