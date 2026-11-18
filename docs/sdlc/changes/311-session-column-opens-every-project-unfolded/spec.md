# Spec: the session column is a list of projects, each folded until opened

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. On the session page a project in the left column is folded unless the
   person unfolded it: with no order file, every project draws its heading
   (the closed disclosure, its name, its 要対応 / 実行中 / アイドル counts, its
   「セッションを開始」) and no session row. Clicking the disclosure lists the
   project's sessions; clicking it again hides them. The unfolded projects are
   written to the order file as `unfolded`, are still unfolded after a
   relaunch, and a project added later starts folded.
   Test: `column_311_projects_start_folded_and_unfold_on_click`.
2. An order file written by an earlier build — with `folded` and without
   `unfolded` — still loads: its project order, session order and pins are
   kept, and every project is folded. A removed project's id leaves
   `unfolded` on the next save.
   Test: `column_311_an_older_order_file_keeps_its_order_and_folds_every_project`.
3. Folded projects sit closer together: the top of one folded project's name
   to the top of the next is at most `28.0px` (it is `37.0px` today: a `24px` heading, the `4px` row gap, a `1px` rule, `2px + 2px` of frame margin and the column's `4px`), and no
   rule is drawn under a folded heading. An unfolded project keeps the rule
   between its heading and its rows.
   Test: `column_311_folded_projects_sit_closer_together`.
4. While a filter of the column is on (a state tile or a CLI), a project with
   a matching session lists the matching rows whether or not it is unfolded,
   its disclosure shows the open mark, and the fold the person chose is
   neither changed nor saved by it; clearing the filter folds it back.
   Test: `column_311_a_filter_lists_matches_of_a_folded_project`.
5. In an unfolded project the frame's `2px` margin above the heading and below the last row is gone with the folded one's, and nothing else changes: rows and the centre-top tabs
   list the same sessions in one order, a row or a tab can be pinned, and
   dragging either moves both.
   Tests (existing, their fixtures unfolding the project first):
   `sidebar_302_compact_rows_still_pin_reorder_and_sync_with_top_tabs`,
   `session_285_dragging_a_tab_reorders_the_rows`.

## Behaviour

- First open, or after updating from the current build: the column reads as a
  list of projects, all folded. Counts stay on the heading, and clicking a
  count still opens the first session of that state in its tab; the project
  stays folded.
- Disclosure tooltip: 「展開する」 folded, 「折りたたむ」 unfolded (both exist).
- Empty project, unfolded: 「開いているセッションはありません」 as today.
  Folded: the heading only.
- No projects: 「プロジェクトがありません」 as today.
- Filter on, nothing matches: 「一致するセッションはありません。」 as today;
  projects without a match are not drawn, as today.
- Filter on, matches: every drawn project lists its matches; the disclosure
  reads open and clicking it does nothing while the filter is on (tooltip
  「フィルタ中はすべて表示されます」).
- Too many: sixty projects folded scroll in the column as today; nothing is
  drawn for a folded project's rows.
- Interrupted or failed save: the fold is in effect for this run and the
  notice 「並び順を保存できませんでした: {error}」 says so, as today.
- A session opened from elsewhere (a count, ⌘J, 履歴) opens its tab; its
  project is not unfolded.
- The centre top keeps the tab strip with each session's name; pin and drag
  are unchanged.

## Design

Placement: `EXTEND` (route `modify`). `git grep -n folded -- src` finds the
fold in `src/ui/session_tree.rs` (`SessionArrangement.folded`),
`src/app/screens.rs` (`ui_workspace_projects`, `toggle_project_fold`) and
`src/app/session_order.rs` (`commit_session_arrangement`); the behaviour
exists and its default changes.

- `src/ui/session_tree.rs`: `SessionArrangement.folded: Vec<Uuid>` is replaced
  by `unfolded: Vec<Uuid>` (`#[serde(default)]`). The struct does not deny
  unknown fields, so an older file's `folded` key is ignored and a file
  without `unfolded` folds everything — which is requirement 2 with no
  migration code. An older build reading a newer file ignores `unfolded` and
  unfolds everything, its own default.
- `src/app/screens.rs`:
  - `toggle_project_fold` adds or removes the project in `unfolded`, retains
    only projects the store holds, and saves as today.
  - `ui_workspace_projects`: `let unfolded = filtering ||
    self.session_arrangement.unfolded.contains(&project_id);` decides the
    mark, the tooltip and whether rows draw. While `filtering` the disclosure
    click is not taken and its tooltip is the filter one. The project frame's
    vertical margin goes from `2` to `0`, and `hairline(ui, palette)` is
    drawn only when unfolded: a folded project is its `CONTROL_HEIGHT_SMALL`
    (`24px`) heading, and the column's `SPACE_XS` (`4px`) spacing puts the
    next one `28px` below.
- `src/app/session_order.rs`: `commit_session_arrangement` retains `unfolded`
  as it retained `folded`.
- `src/i18n_tables.rs`: one row per table for
  「フィルタ中はすべて表示されます」.
- `README.md`, `README.ja.md`, `README.ko.md`: one sentence each — projects
  start folded, the disclosure opens one, the choice is kept in
  `session-order.json`. `CHANGELOG.md`: one line under Changed.
- `src/tests.rs`: the four `column_311_*` tests; fixtures of tests that read
  session rows unfold their project; `workspace_project_fold_hides_rows_and_survives_reload`
  and `session_arrangement_without_folded_projects_still_loads` are rewritten
  to the new default without being removed.
- Persisted shape: `session-order.json` gains `unfolded` and stops writing
  `folded`. It is a per-machine preference beside the store, bounded by
  `SESSION_ARRANGEMENT_FILE_MAX_BYTES`, read once at start; not the store, no
  `STORE_SCHEMA_VERSION`.
- External commands, background work: none. The draw path reads one `Vec`
  membership per project, as it did.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | No colour is added; read `.claude/rules/palette-and-glyphs.md`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | Reuses `ICON_DISCLOSURE_OPEN` and `ICON_DISCLOSURE_CLOSED`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | The file name stays `SESSION_ARRANGEMENT_FILE_NAME`; the key `unfolded` is the serde field name, written and read by the one struct, and requirement 1's test round-trips it through a relaunch. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | The store, cancellation, tmux and migration are untouched; the order file keeps its atomic write and its byte ceiling, and requirement 2 holds an older file. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process. |
| Documentation — user-facing docs change in all three languages together | yes | The three READMEs gain the same sentence; `CHANGELOG.md` one line. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | Drawing and one local file only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new scan; `unfolded` is retained to the store's projects, inside the file's existing ceiling. |

## Flagged concerns

None.

## Acceptance

- `cargo test --locked column_311_` passes (4 tests, each watched failing first).
- `cargo test --locked` for `sidebar_302_`, `session_285_`, `workspace_project_`, `sidebar_251_`, `sidebar_252_`, `session_column_project_`, `test_change_181_` passes; CI runs the whole suite.
- `python3 docs/sdlc/changes/311-session-column-opens-every-project-unfolded/mutations.py` catches every mutation.
- `make q.fast` and `bash scripts/check-bands.sh` pass.
- In the shipped `/Applications/Operon.app`: the session page opens with every project folded; unfolding one lists its sessions and survives a relaunch.

## Rejected alternatives

- Keep `folded` and fold every project once at first start: needs a marker in the file and code for a project added later; a set of unfolded projects is the default itself.
- Unfold the project of the session being opened: the person asked for folded by default, and a count or ⌘J would unfold the column again one project at a time.
- Hide the counts on a folded heading: they are what makes a folded project worth reading.
- Smaller heading text or a heading under `CONTROL_HEIGHT_SMALL`: off `DESIGN.md`'s scale.
