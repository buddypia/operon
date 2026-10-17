# Plan: 171 — a 2D map of a project's knowledge

- **Spec**: `./spec.md`
- **Approved**: 2026-10-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/knowledge/map.rs` | New. `neighbours`, `edges`, `groups` (Louvain), `group_names`, `principal_plane` (PCA by power iteration), `layout` (fresh / settle), `hull` (monotone chain), `label_order`, `vector_hash` (FNV-1a), `compute_map` joining entries and vectors by id, `KnowledgeMap`, `MapPoint`, `MapGroup`. Pure; no I/O. |
| `src/knowledge.rs` | `SavedPosition { entry_id, x, y, vector_hash }` (defined here, consumed by `map.rs`). `layout` table in `KNOWLEDGE_SCHEMA`; `saved_layout() -> Vec<SavedPosition>`, `save_layout(&[SavedPosition])` through `write_if_enabled`, each row `INSERT … SELECT … WHERE EXISTS`. `pub(crate) mod map; pub(crate) use map::*;` added in step 2, as `embed` is. `knowledge_map(root, project_id, entries)`: open existing, read `entry_vectors` and `saved_layout`, `compute_map`, save when the layout ran. |
| `src/ui/knowledge_map.rs` | New. `MapView` (pan, zoom, `zoom_at`, `fit`, `to_screen`, `to_map`), `point_style(entry, now, dimmed)`, `point_tooltip(entry, now)`, `place_labels(map, view, rect, …)`, `draw_map(ui, palette, map, entries, view, highlight) -> MapResponse` (hovered, clicked id). |
| `src/ui/mod.rs` | `mod knowledge_map;` and `pub(crate) use knowledge_map::*;`, as the other modules are. |
| `src/app/knowledge.rs` | `KnowledgeLayoutMode { List, Map }` on `KnowledgeState`; `KnowledgeView` gains `map`, `map_for` (the snapshot `Arc` it was made from), `map_error`, `map_view`, `map_group`, cached label placement. `request_knowledge_map`, `apply_knowledge_map`. The spec's `map_task` is `spawn_background`'s channel and its `snapshot_key` is `map_for`, the snapshot `Arc` compared with `Arc::ptr_eq` — the mechanism the tab's loads already use. `request_knowledge_map` returns before cloning anything when `background_tasks` holds `KnowledgeMap(id)`, when `map_error` is set (only 再試行 clears it), or when `map_for` is the current snapshot. The 表示 switch; 並び hidden in マップ; `ui_knowledge_map` (map + right panel: groups, members, or 157's detail block; narrow layout below 700px). |
| `src/app.rs` | `BackgroundKey::KnowledgeMap(Uuid)`, `BackgroundResult::KnowledgeMapped { project_id, from, result }` (`KnowledgeMap`, `MapPoint`, `MapGroup` derive `Debug`, as `BackgroundResult` requires), its arm in `process_background_results`: removes `BackgroundKey::KnowledgeMap(project_id)` first, then applies or, when stale, drops the result and leaves `map_for` unchanged so the next frame asks again. |
| `src/config.rs` | `KNOWLEDGE_MAP_NEIGHBOURS` 8, `KNOWLEDGE_MAP_EDGES_PER_ENTRY` 4, `KNOWLEDGE_MAP_EDGE_MIN_COSINE` 0.78, `KNOWLEDGE_MAP_GROUP_BASELINE` 0.66, `KNOWLEDGE_MAP_MIN_GROUP` 3, `KNOWLEDGE_MAP_GROUP_LABEL_CHARS` 16, `KNOWLEDGE_MAP_ITERATIONS` 300, `KNOWLEDGE_MAP_SETTLE_ITERATIONS` 60, `KNOWLEDGE_MAP_SETTLE_MAX_SHIFT` 0.05, `KNOWLEDGE_MAP_MAX_LABELS` 60, `KNOWLEDGE_MAP_BUDGET_SECONDS` 5, `KNOWLEDGE_MAP_NARROW_WIDTH` 700, radius and opacity bounds. |
| `src/glyphs.rs` | `ICON_FIT` (Phosphor `arrows-out` or the closest bundled glyph) and its `ICON_VOCABULARY` entry, if no existing constant reads as "fit". |
| `src/i18n_tables.rs` | Rows for every new string in every table, sorted. |
| `src/tests.rs` | The tests named in spec.md Requirements, plus synthetic-vector helpers. |
| `README.md`, `README.ja.md`, `README.ko.md` | One paragraph on the map in 「Project knowledge」. |
| `CLAUDE.md` | Module map: `src/knowledge/map.rs` under the knowledge line; `knowledge_map.rs` added to the existing `src/ui/` list. |

## Order of work

The tree compiles after each step.

1. **Constants and contracts.** The `KNOWLEDGE_MAP_*` constants in
   `src/config.rs`; `SavedPosition`, the `layout` table, `saved_layout` /
   `save_layout`; tests: positions round-trip, cascade on delete / prune /
   clear (and a later entry reusing a deleted id has no saved row), the
   `WHERE EXISTS` skip, a database created without `layout` opens and gains it,
   and `KNOWLEDGE_SCHEMA_VERSION == 1` (Req 6). Watched failing first.
2. **Pure map, test first per requirement.** `pub(crate) mod map;` with
   `vector_hash`, `neighbours`, `edges` (Req 1); `groups` + determinism (Req 2);
   `group_names` (Req 3); `principal_plane` and `layout` fresh (Req 4), settle
   and reuse, including a saved row whose fingerprint no longer matches counting
   as unplaced (Req 5); identical vectors stay finite; no-vector ring (Req 7);
   `hull` (Req 11); `label_order`. Each test run red before its function.
3. **Measure.** Embed entry-shaped texts (`title: … | text: …`, 3 topics × 4 +
   6 unrelated) through the shipped runtime with a throwaway `#[test]` behind
   the model check, read the cosine split, set
   `KNOWLEDGE_MAP_EDGE_MIN_COSINE` / `KNOWLEDGE_MAP_GROUP_BASELINE`, record the
   numbers in Departures, delete the throwaway. Then
   `cargo test --release --locked knowledge_map_lays_out_the_largest_base_in_time`
   (Req 9) and record the time.
4. **Background task.** `knowledge_map()` in `src/knowledge.rs`; the
   `BackgroundKey` / `BackgroundResult` pair and its arm; `request_knowledge_map`
   with its three early returns; `apply_knowledge_map`. Tests: asking returns at
   once and leaves a task in flight (Req 8); a stale result is not drawn and the
   next frame asks again (Req 8); while a task is in flight or after a failure,
   further frames spawn nothing.
5. **View maths and icon.** `ICON_FIT` and its `ICON_VOCABULARY` entry in
   `src/glyphs.rs` unless an existing constant reads as "fit"; `MapView::zoom_at`,
   `fit` (Req 15), `point_style` (Req 10), `point_tooltip` (Req 12),
   `place_labels` with its cache key (Req 16).
6. **Drawing and wiring.** `draw_map` (hulls, lines, points, labels) and the
   legend row (the five shapes with 決定 / 原因 / 手順 / 注意 / 事実, 大きさ: 参照回数,
   濃さ: 鮮度, the pin ring); the 表示 switch; the right panel; choosing a group
   frames its members with `fit` over their bounds and dims the rest;
   selection through `open_entry` + `KnowledgeEdit::Reference` (Req 13);
   dimming for search and kind (Req 14); the states in `screen.md` (computing,
   failure with 再試行, one or two entries, no hit, narrow).
7. **Strings, docs, module map.** `i18n_tables.rs`, READMEs, `CLAUDE.md`.
8. **Look at it.** Release build, a temporary data directory seeded with
   synthetic entries from real embedded texts, screenshot the map, the group
   panel, a chosen group, a chosen entry, a search, and the narrow layout, in
   the dark and light themes; compare with `screen.md`; fix what differs.
9. **Gates.** `make q.fast`, the named tests, `bash scripts/check-bands.sh`,
   the mutation sweep from `docs/sdlc/templates/mutations.py` over the new
   guards, review (rust-reviewer, durability-reviewer per `routes.yaml`).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Layout too slow at 2 000 entries | spinner for many seconds | Req 9 release test; step 3 measurement |
| Force simulation unstable (NaN, explosion) with duplicate vectors | points vanish or fly off | a test with identical vectors; positions clamped and finite-checked before save |
| Settle moves placed entries too far | map reshuffles after one new entry | Req 5 test asserts the 5 % bound |
| Label placement per frame | stutter while hovering | Req 16 test counts placements across frames |
| Stale map drawn against new entries | a click opens the wrong entry | Req 8 stale-result test; map keyed by entry id, not index |
| 157 build opening a database with `layout` | refusal or a crash on downgrade | a test that `KNOWLEDGE_SCHEMA_VERSION == 1`; a test that a database made without `layout` opens and gains it |
| The map task re-spawned or its entries cloned every frame | CPU busy while the tab is open; a failure retried at frame rate | step 4 tests: no new task while one is in flight or after a failure |
| A deleted id reused by a new entry | the new entry drawn at the old one's place | cascade removes the row; step 1 test; `vector_hash` mismatch as a second line (step 2) |
| Thresholds wrong for real knowledge | everything joined, or nothing | step 3 measurement on entry-shaped texts through the shipped model |
| Hull fill unreadable on a theme | groups invisible or garish | step 8 screenshots in the dark and light themes |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --release --locked knowledge_map_lays_out_the_largest_base_in_time` — passes, time recorded below.
- Each test named in spec.md watched failing before its code.
- Screenshots from step 8 match `screen.md`.

## Departures from the plan

Filled in during implementation.
