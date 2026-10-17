# Spec: 171 — a 2D map of a project's knowledge

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. **Neighbours by meaning.** For every entry, the map's relations are its
   nearest entries by cosine of the stored vectors (at most
   `KNOWLEDGE_MAP_NEIGHBOURS` = 8), and a drawn line joins two entries only when
   one is among the other's `KNOWLEDGE_MAP_EDGES_PER_ENTRY` = 4 nearest and the
   cosine is at least `KNOWLEDGE_MAP_EDGE_MIN_COSINE`. —
   `knowledge_map_joins_only_near_meanings`
2. **Groups.** Entries are grouped by modularity (Louvain) over the neighbour
   graph, weighted by `cosine − KNOWLEDGE_MAP_GROUP_BASELINE`; a group of fewer
   than `KNOWLEDGE_MAP_MIN_GROUP` = 3 entries is not a group, its entries are
   ungrouped. The same entries and vectors always give the same groups. —
   `knowledge_map_groups_related_entries_and_is_deterministic`
3. **Group names.** A group is named by the title of its most central member
   (highest summed weight to other members; ties by lower id), shortened to
   `KNOWLEDGE_MAP_GROUP_LABEL_CHARS` = 16 characters with 「…」. —
   `knowledge_map_names_a_group_by_its_most_central_entry`
4. **Similar entries sit close.** After layout, the mean distance between
   entries joined by a line is less than half the mean distance between all
   pairs, on a fixture of three unrelated topics. —
   `knowledge_map_places_related_entries_together`
5. **Stable placement.** Each saved position carries a fingerprint of the
   vector it was computed from (`vector_hash`, 64-bit FNV-1a of the vector's
   bytes). A saved position counts as *placed* only while its entry's current
   vector has that fingerprint. When every entry is placed, the map is drawn
   from the saved positions untouched, without running the layout. When some
   are placed, the layout *settles*: every placed entry moves by less than
   `KNOWLEDGE_MAP_SETTLE_MAX_SHIFT` = 5 % of the map's extent, and each unplaced
   entry starts at its placed neighbours' weighted centre. When none is placed
   (nothing saved, or 157 rebuilt every vector after a model change), the layout
   runs afresh. — `knowledge_map_keeps_placed_entries_where_they_were`,
   `knowledge_map_lays_out_afresh_when_the_vectors_change`
6. **Positions go with their entry.** Deleting an entry, pruning, and clearing
   the knowledge remove its saved position (`ON DELETE CASCADE`, which every
   connection enables). A save racing a deletion skips the ids no longer in
   `entries` instead of failing: the row is inserted only `WHERE EXISTS` its
   entry. — `knowledge_map_positions_go_with_their_entry`,
   `knowledge_map_save_skips_an_entry_deleted_meanwhile`
7. **Every entry is a point.** Entries and vectors are joined by id. An entry
   whose vector cannot be read is still drawn: ungrouped, without lines, on a
   ring outside the other points. — `knowledge_map_places_an_entry_without_a_vector`
8. **Never blocks the window.** Neighbours, groups, layout, group outlines, and
   the order labels are tried in are computed on a background thread from a
   copy of the entries; asking for the map returns at once and the frame reads
   only the last finished result. A result for a snapshot that has since
   changed is never drawn against the new entries. —
   `knowledge_map_is_computed_off_the_frame`,
   `knowledge_map_result_for_an_old_snapshot_is_not_drawn`
9. **Fast enough at the largest base.** 2 000 entries of 768 dimensions are laid
   out afresh in under `KNOWLEDGE_MAP_BUDGET_SECONDS` = 5 s in a release build on
   this Mac (`KNOWLEDGE_MAP_ITERATIONS` = 300 fresh, 60 when settling). The test
   checks correctness at that size in every build and the time only without
   `debug_assertions`. CI runs a debug build, so CI holds correctness only; the
   time is an acceptance check the author runs with `cargo test --release` in
   build, and the measured time is recorded in `plan.md`. —
   `knowledge_map_lays_out_the_largest_base_in_time`
10. **Encoding.** A point's shape is its kind (決定 ◆, 原因 ●, 手順 ■, 注意 ▲,
    事実 ○ hollow); its radius grows with √(references + recurrences) from 3 to
    9 px; its opacity is its freshness, floored at 25 %; below 0.3 it is drawn in
    `warning`, as the list's bar is; a pinned entry has a ring. —
    `knowledge_map_point_style_follows_the_entry`
11. **Groups are visible as groups.** Each group is outlined by the convex hull
    of its members (padded by the largest radius), filled with
    `accent` at 8 % and stroked `border_subtle`, with its name beside it. —
    `knowledge_map_hull_encloses_its_group`
12. **Hover.** Pointing at a point shows its title and
    「{kind} ・ 鮮度 {p0}% ・ 参照 {n}」. — `knowledge_map_tooltip_names_the_entry`
13. **Selection is shared with the list.** Choosing a point, or a title in a
    group's member list, opens the same detail the list shows (body, sources
    with 「会話を開く」, 固定 / 削除) and counts as a reference exactly as opening
    the row does; switching to the list shows that row open, and the reverse. —
    `knowledge_map_and_list_share_the_open_entry`
14. **Search and kind filter.** With a search result or a kind filter, entries
    outside it are dimmed to 15 % and the rest keep full style; the panel says
    how many match. — `knowledge_map_dims_what_the_search_did_not_find`
15. **View.** Drag pans; scroll or pinch zooms keeping the point under the
    pointer where it is; 「全体を表示」 fits every point inside the map. —
    `knowledge_map_zoom_keeps_the_point_under_the_pointer`,
    `knowledge_map_fit_shows_every_point`
16. **Labels.** Group names are shown at every zoom; entry titles appear only
    zoomed in, at most `KNOWLEDGE_MAP_MAX_LABELS` = 60, most-used first, and
    never overlapping one another or a group name. Placement is recomputed only
    when the result, the zoom, the pan, or the map's size changes, not every
    frame. — `knowledge_map_labels_never_overlap`,
    `knowledge_map_labels_are_placed_only_when_the_view_changes`

## Behaviour

Strings are Japanese; the screen is `screen.md`.

- The knowledge tab gains 「表示: [一覧] [マップ]」 beside the search box once the
  project has at least one entry. 「一覧」 is the default and is exactly 157's list;
  the choice holds until the app quits, as the sort does.
- **Map, nothing chosen:** the map on the left, and on the right 「グループ」 with
  each group's name, 「{n} 件」, and a freshness bar of its mean; choosing a group
  highlights and frames its members and lists their titles. Ungrouped entries
  are listed last as 「その他 ({n} 件)」.
- **Map, an entry chosen:** the right panel shows the entry's detail (157's
  detail block), with 「← グループ一覧」 to go back.
- **Hover:** a tooltip with the title, 「{kind} ・ 鮮度 {p0}% ・ 参照 {n}」.
- **Computing:** the first time, 「配置を計算しています…」 with a spinner in the map
  area. When a layout exists and entries changed, the old layout stays drawn
  and new points appear when the settle finishes.
- **Failure:** a `danger` banner 「マップを作れませんでした: {error}」 with
  [ 再試行 ]; the list still works.
- **One or two entries:** points are drawn, no lines or groups, and the panel
  says 「グループはまだありません」.
- **Search with no hit:** every point dimmed and 「一致するナレッジはありません。」.
- **Largest base (2 000 entries):** all are drawn; lines are drawn only for
  the hovered or chosen entry's neighbours plus the base lines, which are thin
  `border_subtle`.
- **Collection running:** the map updates when the run's snapshot arrives, the
  same moment the list does.

## Design

Placement: `NEW_IN_MODULE` — the map is a view of 157's knowledge
(`src/knowledge.rs` holds entries and vectors; `src/app/knowledge.rs` holds the
tab). Evidence: `git grep -n "entry_vectors\|KnowledgeView"` finds the only
readers of knowledge vectors and the tab state.

**What the reference does** (read from `~/dev/buddypia/personal-knowledge`,
`viewer/server.py` and `viewer/static/app.js`, 2026-10-17): no projection at
all. Each item's top-4 neighbours with cosine ≥ 0.74 become edges; a k-NN graph
(k = 8, weight cosine − 0.5) is grouped by Louvain (resolution 1.6, seeded);
groups under 3 are dropped; a force simulation (charge, link springs whose
length falls with similarity, a pull to the group centre) places the points in
the browser on every load, unseeded and unsaved; newcomers start at a placed
neighbour. Groups are drawn as translucent regions with a label (keywords, or an
LLM name). 171 keeps the graph, Louvain, the group pull, and newcomers at their
neighbours; it adds determinism and saved positions (Req 2, 5), and names groups
locally (Rejected alternatives). Its thresholds are re-measured for
EmbeddingGemma 2's 768-dimension vectors (below).

- `src/knowledge/map.rs` (new, under the knowledge module): pure functions
  `neighbours` (each entry's nearest indices and cosines), `groups` (a group
  index or none per entry; Louvain, nodes visited in id order, no RNG),
  `group_names`, `hulls`, `label_order`, and `layout` (an x, y per entry; force
  simulation: pairwise repulsion, a spring on each neighbour edge, a pull to the
  group centre; `KNOWLEDGE_MAP_ITERATIONS`; initial positions from the two
  leading principal components when nothing is saved, from saved positions
  otherwise, with new entries at their neighbours' weighted centre plus a jitter
  derived from the id, and already-placed entries moving at a tenth of the step
  while settling). `compute_map(entries, vectors, saved) -> KnowledgeMap {
  snapshot_key, points, edges, groups, hulls, label_order }` joins
  entries and vectors by id. At ≤ 2 000 entries × 768 dimensions the neighbours
  are O(n²·d) ≈ 1.5 × 10⁹ multiply-adds and repulsion O(n²) per iteration; both
  run on the background thread.
- `src/knowledge.rs`: a `layout` table
  `(entry_id INTEGER PRIMARY KEY REFERENCES entries(id) ON DELETE CASCADE, x REAL
  NOT NULL, y REAL NOT NULL, vector_hash INTEGER NOT NULL)` created with the
  others. `saved_layout()` returns the rows; `save_layout(points)` replaces the
  table's rows in one transaction through
  `write_if_enabled`, each row `INSERT … SELECT … WHERE EXISTS (SELECT 1 FROM
  entries WHERE id = ?1)` so an entry deleted meanwhile is skipped (Req 6).
  Additive: a 157 build runs the same `CREATE TABLE IF NOT EXISTS` list without
  this table, never reads it, and its `DELETE FROM entries` still cascades into
  it because every connection sets `foreign_keys=ON` (`KnowledgeDatabase::open`).
  So `KNOWLEDGE_SCHEMA_VERSION` stays 1: raising it would make a 157 build
  refuse the whole knowledge base for a table it can ignore.
- `src/app/knowledge.rs`: `KnowledgeView` gains `map: Option<Arc<KnowledgeMap>>`,
  `map_task` (a `Receiver`, as the snapshot's load is), `map_error`, and the
  view state (pan, zoom, chosen group, hovered, the cached label placement and
  the view key it was placed for). `KnowledgeState` gains `layout_mode`. When the
  map is shown and the view's snapshot `Arc` is not the one the map was made
  from, a task is spawned with a copy of the entries; it opens its own
  connection, reads vectors and the saved layout, and decides by Req 5 whether
  to reuse, settle, or lay out afresh; it saves only when it ran the layout.
  A partial save (Req 6) or `clear()` leaves no row that claims an entry it
  does not describe, so neither needs a separate key.
  Selection uses the existing `open_entry` and `KnowledgeEdit::Reference`.
- `src/ui/knowledge_map.rs` (new): draws hulls, lines, points, labels, legend.
  Per frame it transforms the precomputed points, hulls, and label anchors by
  the view's pan and zoom — 2 000 points and ≤ 8 000 line segments, which egui's
  painter tessellates without a per-frame allocation beyond its own — and reads
  the cached label placement, which is redone (≤ 60 labels against ≤ 2 000
  candidates in the precomputed order, rectangle-collision rejection) only when
  the view key changes. `MapView` holds pan and zoom with `zoom_at` and `fit`.
- `src/config.rs`: the `KNOWLEDGE_MAP_*` constants. `KNOWLEDGE_MAP_EDGE_MIN_COSINE`
  starts at 0.78 and `KNOWLEDGE_MAP_GROUP_BASELINE` at 0.66, read off
  `tests/fixtures/embedding-golden.json`'s reference vectors (pairs about one
  topic 0.84–0.93; median of all pairs 0.66; 95th percentile 0.77); re-measured
  on entry-shaped texts in build and recorded in `plan.md`.
- `src/i18n_tables.rs`: the strings in Behaviour, every table.
- `CLAUDE.md`: the module map gains `src/knowledge/map.rs` and
  `src/ui/knowledge_map.rs` (`claude_md_maps_every_module_that_exists`).
- No new dependency: Louvain, PCA (power iteration), the force simulation, and
  the convex hull are a few hundred lines; `egui`'s painter draws the map.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes, no new role | Points `accent` / `warning`, pinned ring `text_strong`, lines `border_subtle` and `accent` when highlighted, hulls `accent.gamma_multiply(0.08)` fill with a `border_subtle` stroke (as `src/app/screens.rs` washes `accent` at 0.12), labels `text_muted` / `text`. All exist in `src/theme.rs`; kind is told by shape, so no categorical palette is added. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | Point shapes and the legend's shapes are painted, not glyphs. 「全体を表示」's icon is an `ICON_*` constant; if none fits, a new one with its `ICON_VOCABULARY` entry. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The `KNOWLEDGE_MAP_*` values live once in `src/config.rs`. SQL text stays inline in `KNOWLEDGE_SCHEMA` and the queries, as 157's does, rather than being assembled with `format!`. `knowledge_map_keeps_placed_entries_where_they_were` round-trips saved positions and fingerprints. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes, the knowledge database, not the store | The store (`src/store.rs`) is untouched. The knowledge database gains one additive table, written in one transaction through `write_if_enabled`, so a cleared or switched-off project is not written back. `KNOWLEDGE_SCHEMA_VERSION` stays 1 (Design). Positions are derived: a lost table means a fresh layout, never lost knowledge. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No process is started; group names come from entry titles, not a CLI. |
| Documentation — user-facing docs change in all three languages together | Yes | README's 「Project knowledge」 section gains one paragraph on the map in `README.md`, `README.ja.md`, `README.ko.md`; `CLAUDE.md` maps the two new modules. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Computed from vectors already on the Mac; no network, no `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | ≤ `KNOWLEDGE_MAX_ENTRIES` (2 000) points, 8 neighbours and ≤ 4 lines each, `KNOWLEDGE_MAP_ITERATIONS` fixed, ≤ `KNOWLEDGE_MAP_MAX_LABELS` labels, 5 s layout budget (Req 9). |

## Flagged concerns

- **Thresholds measured on mixed-format texts.** The fixture mixes query and
  document prompts; entries are embedded as documents. Answer: re-measure in
  build on entry-shaped texts through the shipped runtime and record the numbers
  in `plan.md`; the requirement tests use synthetic vectors and do not depend on
  the value.
- **Layout time at 2 000 entries.** Answer: Req 9, measured with
  `cargo test --release` in build; the window stays responsive by Req 8 either
  way.

## Acceptance

- `cargo test --locked` passes, including the tests named in Requirements, and
  `cargo test --release --locked knowledge_map_lays_out_the_largest_base_in_time`
  passes on this Mac.
- `bash .claude/hooks/gate-commit.sh` and `bash scripts/check-bands.sh` pass.
- In the running app, on a project with knowledge: 「マップ」 shows grouped points,
  hovering shows a title, choosing a point shows its detail and the list shows
  the same entry open, a search dims the rest, and reopening the tab shows the
  same layout.

## Rejected alternatives

- UMAP / t-SNE crate: a new dependency for a projection the reference project
  does not use; a neighbour-graph force layout matches the reference and keeps
  related entries joined by visible lines.
- Colour by group: needs a categorical palette (new roles × three themes, each
  checked against WCAG) for a distinction the hull and label already make.
- Naming groups with the judge CLI (as the reference's LLM labels do): spends
  the daily judge budget and sends titles to a CLI for a cosmetic label; the
  central entry's title is local and readable.
- Recomputing the layout on every visit without saving: the same entries would
  give the same map, but one new entry could reshuffle everything (Req 5).
- Raising `KNOWLEDGE_SCHEMA_VERSION`: makes a 157 build refuse the whole
  knowledge base for an additive table it can ignore.
