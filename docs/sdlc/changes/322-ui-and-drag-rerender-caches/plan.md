# Plan: Zero-rerender drag pipeline and derived-state render caching across Operon

- **Spec**: `./spec.md`
- **Approved**: 2026-11-26
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `MAX_CACHED_DIFF_ROW_GALLEYS` (`4096`) and `MARKDOWN_VIEWPORT_CULL_THRESHOLD` (`32`). |
| `src/ui/session_tree.rs` | Add zero-group-clone `WorkspaceTabs::can_move_tab(&self, tab, target, drop, area) -> bool`. |
| `src/app/session_order.rs` | Relocate sidebar filter/count/column-drag/attention/workspace-tab helpers (`~650` lines) from `src/app.rs` and `src/app/screens.rs`; add `SidebarViewCache`, content-fingerprinted `sync_workspace_live_sessions`, O(N log N) rank-map `arrange`, and `O(S + T)` group count/attention passes. |
| `src/app/terminal_output.rs` | Relocate terminal search UI/key handlers (`~253` lines) from `src/app/screens.rs` and `queue_terminal_input` (`22` lines) from `src/app.rs`; add `dock_resize_drag` to `is_layout_dragging()`, add `is_any_drag_active()`, defer `request_terminal_resize` / `flush_terminal_resize` during layout drags with one-shot `finish_layout_drag()` flush, implement `sync_visible_sessions_and_streams`, and avoid cloning strings in steady `sync_terminal_streams`. |
| `src/ui/diff.rs` | Add `DiffRenderCache` (cached unified/side-by-side `DiffRow` table, `change_rows`, `offsets`, `widest`, and bounded `Arc<egui::Galley>` row cache shaped under a single `fonts_mut` lock per viewport miss) and `diff_pane_with_cache`. |
| `src/ui/syntax.rs` | Cache static `ScopeSelectors` via `OnceLock`, retain `syntax_galley` and `gutter_galley` on `CachedEditorTextLayout`, and retain `galley` on `ViewerRowLayout`. |
| `src/ui/markdown_view.rs` | Retain per-block width/DPI-keyed `Arc<egui::Galley>`s and `measured_height` inside `RenderedBlock`, cull off-screen blocks past `MARKDOWN_VIEWPORT_CULL_THRESHOLD` (`32`), and add `galley_builds()` / `galley_hits()` counters. |
| `src/ui/widgets.rs` | Cache home dir lookup in `friendly_path` via `OnceLock` and decoded bundled PNG `ColorImage` data via `OnceLock<Mutex<HashMap<...>>>`. |
| `src/ui/files.rs` | Replace per-node `PathBuf::clone` in `file_tree_rows` with borrowed `&Path` lookup over a pre-filtered project set and add fast cached-row rendering helper. |
| `src/app.rs` | Wire `dock_resize_drag`, `last_visible_sessions`, `last_workspace_sync_fingerprint`, `SidebarViewCache`, `FileTreeRowCache`, `CachedPaletteView`, drag-gated polling, and remove relocated methods (`~580+` lines) so `src/app.rs` shrinks below `9500` lines. |
| `src/app/screens.rs` | Wire `dock_resize_drag`, `can_move_tab`, `SidebarViewCache`, cached editor/viewer galleys & zero-alloc cursor line tracking, `FileTreeRowCache`, and `CachedPaletteView` while removing relocated methods (`~427` lines) so `src/app/screens.rs` shrinks below `9500` lines and source pins stay intact. |
| `src/tests.rs` | Add `perf_322_r1_*` through `perf_322_r6_*` empirical tests and `model_check_perf_322_cache_equivalence_and_can_move_tab` exhaustive `exhaustive_check` test. |
| `docs/features/session-column/spec.md` | Update moved symbol locations and sidebar/workspace sync caching notes. |
| `docs/features/terminal-pane/spec.md` | Update moved terminal search symbol locations and unified drag resize deferral notes. |
| `docs/features/git-changes/spec.md` | Note `DiffRenderCache` virtualized row table and row `Galley` cache. |
| `docs/features/files-editor/spec.md` | Note cached editor/viewer/Markdown `Galley`s, `ScopeSelectors` cache, and zero-clone tree cache. |
| `docs/features/palette/spec.md` | Note `CachedPaletteView` single-pass cache. |

## Order of work

1. **Module line-budget relocations (`src/app/terminal_output.rs`, `src/app/session_order.rs`, `src/app.rs`, `src/app/screens.rs`)**:
   - Move terminal search methods (`253` lines) from `src/app/screens.rs` and `queue_terminal_input` (`22` lines) from `src/app.rs` into `src/app/terminal_output.rs`.
   - Move column drag / sidebar filter helpers (`174` lines) from `src/app/screens.rs` and session filter/count/attention/workspace-tab helpers (`~560` lines) from `src/app.rs` into `src/app/session_order.rs`, preserving `src/tests.rs` source-pin section markers (`ui_workspace_projects..toggle_project_fold`, `ui_terminal_session_tabs..ui_terminal_panel`, `ui_terminal_pane..workspace_tab_title`). Tree compiles cleanly with both files well below `9500` lines.
2. **TDD failing tests first (`src/tests.rs`)**:
   - Add `perf_322_r1_*`..`perf_322_r6_*` and `model_check_perf_322_cache_equivalence_and_can_move_tab` against the public/crate APIs designed in `spec.md`, and verify they fail before implementation.
3. **Implement R1 & R2 (`src/ui/session_tree.rs`, `src/app/terminal_output.rs`, `src/app/session_order.rs`, `src/app.rs`, `src/app/screens.rs`)**:
   - Add `dock_resize_drag`, unified `is_layout_dragging` / `is_any_drag_active`, deferred tmux resize + `finish_layout_drag` flush, zero-clone `can_move_tab`, drag-gated polling, fingerprinted `sync_workspace_live_sessions`, deduplicated `sync_visible_sessions_and_streams` + lean `sync_terminal_streams`, indexed group counts / `track_attention`, and `SidebarViewCache`.
4. **Implement R3, R4, R5, R6 (`src/config.rs`, `src/ui/diff.rs`, `src/ui/syntax.rs`, `src/ui/markdown_view.rs`, `src/ui/widgets.rs`, `src/ui/files.rs`, `src/app.rs`, `src/app/screens.rs`)**:
   - Implement `DiffRenderCache`, `ScopeSelectors` `OnceLock`, `CachedEditorTextLayout` / `ViewerRowLayout` `Arc<egui::Galley>` retention + zero-alloc cursor line count, `MarkdownPreviewCache` per-block `Arc<egui::Galley>` + large-doc viewport culling, `friendly_path` `OnceLock`, PNG `ColorImage` cache, `FileTreeRowCache`, and `CachedPaletteView`.
5. **Update feature specs & run full verification (`perf_322_*`, `model_check_perf_322_*`, `mutations.py`, `make q.fast`, `check-bands.sh`, `check-review.sh --index`, `verify-app.py`)**.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Stale cache across direct field mutations in tests or runtime (e.g. `store.sessions` status flip, theme switch, DPI change, diff toggle) | Old title/count/galley/color painted after state changed | Content-addressed `u64` fingerprints, `SyntaxColours`/`ppp_bits`/width keys, `perf_322_*` invalidation checks, `model_check_perf_322_cache_equivalence_and_can_move_tab` |
| Missed drag release leaving `dock_resize_drag` stuck or pending terminal resize unflushed | Terminal stays at pre-drag columns/rows after dragging dock rule | `!any_dock_rule_dragged` + Escape handling + `finish_layout_drag()`, `perf_322_r1_*` |
| Shared `ColorImage` or `Galley` cache leaking across distinct `egui::Context` instances in unit tests | Dark/garbled textures or panics in `kittest`/headless tests | `TextureHandle` stays per-`ctx.data`, `ColorImage` is CPU-only immutable decoded pixels, `Galley`s cache per `DiffRenderCache`/`EditorTextLayoutCache`/`MarkdownPreviewCache` keyed by `ppp_bits` + `SyntaxColours` |
| Relocating helpers trips source-pin tests or leaves `src/app.rs` / `src/app/screens.rs` near 10,000 lines | `session_column_*` or `check-bands.sh` fails | Keeping `ui_workspace_projects..toggle_project_fold` markers intact, running `cargo test --locked session_column_`, asserting `< 9500` lines in `perf_322_r6_*` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no warnings beyond compile lines.
- `cargo test --locked perf_322_` — all 6 requirement tests pass.
- `cargo test --locked model_check_perf_322_cache_equivalence_and_can_move_tab` — passes exhaustive state walk.
- Existing UI/dock/workspace/diff/editor/markdown/palette/harness tests pass (`0 failed`).
- `python3 docs/sdlc/changes/322-ui-and-drag-rerender-caches/mutations.py` — catches every mutation (`N of N caught`).
- `bash scripts/check-bands.sh` — `0` `propose` breaches, `unsafe_blocks` unchanged, `unwrapped_spawns` unchanged, `src/app.rs` < `9500` lines, `src/app/screens.rs` < `9500` lines.
- `bash scripts/check-review.sh --index` — exits `0`.
- `python3 scripts/verify-app.py` and `/Applications/Operon.app` launch/drag/render identically and responsively.

## Departures from the plan

- **Post-verification hardening (`F1`)**: Fixed empirical/formal verifier findings before commit — added language/origin/seen-sessions/project-workspace-tabs fingerprint inputs in `src/app/session_order.rs`, palette language/keymap/branch/hook/unread/full-scan/open-doc hashing + open/close invalidation in `src/app/screens.rs` & `src/app.rs`, recursive node hashing + sorted badge hashing in `src/ui/files.rs`, diff `line.highlights` + sorted `note_heights` hashing + `>4096` overflow paint fallback in `src/ui/diff.rs`, `SyntaxColours` extended palette fields + `galley_for` finite `wrap_width` + collision-free `gutter_galley_for` key in `src/ui/syntax.rs`, width-independent `code_galleys` cache in `src/ui/markdown_view.rs`, Escape/page-switch/session-library/tab-label-drop drag cleanup in `src/app/screens.rs` & `src/app.rs`, non-collision `probe` id on `can_move_tab` in `src/ui/session_tree.rs`, `perf_322_f1_stale_fingerprint_and_drag_cancel_fixes`, and 4-pane + Document tab coverage in `model_check_perf_322_cache_equivalence_and_can_move_tab` (`src/tests.rs`).
