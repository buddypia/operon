# Spec: the product map shows neither where work stands nor what to do next

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Words used below. The *model* is what `python3 scripts/change-map.py --json`
prints. A change is *open* when its group is `needs` or `flight` (the grouping
of change 312). A requirement line and *pinned* are change 315's. A feature is
*tied* to a change by a `recorded` or `inferred` link (change 313), never by an
`estimated` one. The *page logic* is the object `scripts/change-map/page.js`
exports to `node`, which touches no document.

1. The model's `status` says where things stand: `features` counts the
   catalogue by badge (`steady`, `changing`, `waiting`, `total`); `phases`
   holds one entry per pipeline stage, in order, with the names of the open
   changes whose `state.yaml` gives that stage in its `stage:` field, and
   `off_track` the open changes whose field is absent or names none of the
   six (a record that cannot be read is among them);
   `attention` holds four cards in this order — `decide` (changes of group
   `needs`), `confirm` (open changes with a `human` contract line still
   `pending`, one item per line), `carried` (findings a landed change's
   `review.yaml` carried), `gaps` (features with a problem or no pinned line)
   — each with its `total` and at most five `items`; `next` holds at most
   eight changes of group `flight` with their stage, next step and how many
   contract lines have passed of how many, and `next_total`; `branches` groups
   the open changes by the working copy that holds them; `domains` gives, per
   domain, its features by badge, its lines and pinned lines, and how many of
   its features wait, most waiting first. Fails without it:
   `map_status_counts_features_changes_and_what_waits_for_a_person`.
2. Every feature of the model carries `cover` (`full`, `partial`, `none`,
   `empty` — all, some, none of its lines pinned, or no line), `pinned`, and a
   `rail` of six steps in this order — `brief`, `spec`, `screens`,
   `requirements`, `tests`, `live` — each `done`, `partial` or `missing`:
   a document that exists; every, some or none of the feature's screens
   photographed; at least one line; `cover`; and a `## Seen in the running
   app` section with a line in it. It carries `code`, the files its catalogue
   entry (`paths`, `shared`) and its spec's `## Code and tests` section name
   in backticks, each once, under the first layer whose rule its path meets,
   with what the module map says of it: `test` is `src/tests.rs`; `draw` is
   `src/app.rs` and anything under `src/app/` or `src/ui/`; `text` is
   `src/i18n.rs`, `src/i18n_tables.rs`, `src/theme.rs` and `src/glyphs.rs`;
   `core` is any other path under `src/`; `doc` is a path under `docs/` or
   ending `.md`; `script` is everything else; `next`, the newest open
   change tied to it with that change's next step, or nothing; and `evidence`,
   the three newest landed changes tied to it, each with how many obligations
   its `verification.yaml` lists, how many findings it carried and how many
   approvals it holds. Fails without it:
   `map_features_carry_cover_rail_code_by_layer_and_evidence`.
3. The model's `work.cards` puts every open change, the eight newest landed
   changes and every backlog item read on a board of five columns: `backlog`
   (an item nobody holds), `held` (an item a session holds), `flight`, `you`
   (group `needs`), `done`. A card has one `id` (`change:<name>` or
   `issue:<number>`) and one column; its `owner` is `person` in `you`, `ai` in
   `held` and `flight`, and `none` elsewhere; backlog cards keep the order the
   backlog gives (best score first) and carry kind, score and link. An item
   whose holder the backlog could not read (`held_known` false) stays in
   `backlog` and is marked `held: "unknown"`. Fails
   without it: `map_work_board_puts_every_card_in_exactly_one_column`.
4. Without `--backlog` the map starts no process but git's and the model's
   `backlog` says it was not asked. With `--backlog` it runs
   `<root>/scripts/backlog.py --timeout <s> list --json` once, for at most
   `--backlog-seconds` (default 30) and 1 MiB of output, keeps at most 200
   items and says in `backlog.total` how many there were and in
   `backlog.notice` that some are not shown; when that process is missing, exits non-zero, runs over its time,
   prints more than the ceiling or prints something that is not a list, the
   model's `backlog.error` says which in Japanese, the board shows the other
   columns, and the map is still written with exit 0. Fails without it:
   `map_reads_the_backlog_only_when_asked_and_survives_its_failure`.
5. The model's `process` draws the pipeline from its three tables: `stages`;
   every route of `docs/sdlc/routes.yaml` with its stages, attempt ceiling,
   closing gate, required document, the methods
   `docs/sdlc/verification-methods.yaml` asks of it, and how many changes took
   it; every surface of `docs/sdlc/risk.yaml` with its tier, autonomy, whether
   it needs a screen approval, its paths and the methods asked of it; and
   every method with its kind and command. A table that cannot be read leaves
   its part empty and `process.error` says which. Fails without it:
   `map_process_shows_every_route_surface_and_method_the_pipeline_defines`.
6. The model's `catalog` lists what the application defines: for each of the
   three palettes of `src/theme.rs`, every colour role in declaration order
   with its colour as CSS (`#rrggbb` for `Color32::from_rgb`, `rgba(0,0,0,a)`
   with `a` the alpha over 255 for `Color32::from_black_alpha`; an array role
   such as `ansi` becomes `ansi.0`, `ansi.1`, …; the `dark` flag is not a
   role); every entry of `ICON_VOCABULARY` with the Phosphor name on the
   right of its `ICON_*` constant (`icons::NAME`) and how many times that
   identifier appears as a whole word in the rest of `src/` (outside
   `src/glyphs.rs` and `src/tests.rs`); and every action of
   `KEYMAP_ACTIONS` with its label and chord. Against this repository the
   palettes hold the same roles and no role or mark is missing. Fails without
   it: `map_catalog_lists_every_palette_role_icon_and_key`.
7. The page logic reads and writes a link with a choice in it:
   `parseRoute("#features?state=changing,waiting&cover=none")` gives the view
   and the choice, `formatRoute` writes the same link back, and a link without
   a choice reads as before. `lensToggle` replaces a lens's choice, or adds to
   and removes from it when asked to combine; `featureMatches` keeps a feature
   that meets every lens (any chosen value within one lens);
   `lensCounts` gives, for every value of every lens, how many features would
   remain if that value were chosen with the other lenses as they are. Fails
   without it: `map_page_logic_parses_lenses_and_counts_them_faceted`.
8. The page logic writes the two texts a person hands to a session.
   `workOrder` takes what is in the basket, a kind of work (`fix`, `add`,
   `review`, `document`, `test`) and a sentence, and gives a text that names
   feature-pilot, the sentence, every item, and under "先に読むファイル" each
   file to read once — a feature's brief, spec and code, a change's record
   folder, a screen's picture record. `reviewPacket` takes the verdicts
   (`ok`, `fix`, `hold`, each with a note) and gives one text with the fix
   verdicts first, then hold, then a count of ok, and nothing for a feature
   without a verdict; both say how many characters they are. `pruneBasket`
   drops the entries whose record the model no longer holds, and `keep`
   wraps the browser's storage so that a read or a write it refuses returns
   the fallback or `false` instead of throwing. Fails without it:
   `map_page_logic_builds_the_work_order_and_the_review_packet`.
9. The page logic narrows the board and the requirement board and knows its
   keys: `boardFilter` keeps the person's cards, the sessions' cards or all;
   `requirementMatches` keeps a line by domain, feature, pinned or not, and
   words; `queueMatches` keeps a feature of the left column by words and by
   verdict given or not, and `verdictCounts` counts all, judged and unjudged;
   `sectionOf` gives the text under one heading of a spec and nothing for a
   heading that is not there; `themeNext` steps `auto` → `light` → `dark` →
   `auto`; `KEYS` names a view for each of
   `s f g w r c p d` and none of them twice. Fails without it:
   `map_page_logic_filters_board_and_requirements_and_knows_its_keys`.
10. The page has the views 現況, 機能, 画面, ドメイン, 用語集, 要件, 作業,
    変更, 進め方 and カタログ in that order in its bar, opens on 現況 (the
    view change 316 called 概要, under the same link), and
    draws a list of every feature down the left of every view; it keeps the
    basket, the verdicts and the theme in the browser's storage under names
    that begin `operon-map-` (the theme's is change 316's), and still loads nothing from outside itself
    (one data block, one script, no link, frame, import or request). Fails
    without it: `map_page_has_status_tiles_work_process_and_catalog_views`.
11. `python3 scripts/change-map.py --serve` listens on `127.0.0.1` only, on
    `--port` (default 8765; 0 for any free one), prints the address as its
    first line, and answers `GET /` with a page built from the documents as
    they are at that moment; any other path is 404 and any other method 405,
    each request is cut at 8 KiB and 5 seconds, and nothing it is sent is
    written anywhere. It stops with exit 0 on `SIGTERM` or `SIGINT`, or after
    `--requests N` answers; a port already in use is exit 2 with
    「ポート N は使用中です」 and nothing served. Fails without it:
    `map_serves_a_fresh_page_on_loopback_and_stops`.
12. `docs/features/README.md` names every view of the page and the `--backlog`
    and `--serve` options. Fails without it:
    `map_readme_names_every_view_and_option`.

## Behaviour

The page keeps its bar (brand, search, the ten views, the basket, the theme
button) and gains a column down the left: every feature under its domain's
name, each with its badge dot and its tested share as a thin bar, a box to
narrow the list by words, and three counts (すべて / 未判定 / 判定済み) that
narrow it by verdict. Pressing a feature opens its page; `j` and `k` step to
the next and the previous feature of the list as narrowed. Below 900 px the
column folds away behind a button labelled 「機能一覧」.

**現況** opens first. Four counts (機能 with 安定 / 変更中 / 判断待ち; 要件
with テストあり share; 画面 with 写真あり; 変更 with 要対応 / 進行中 / 完了).
「どこまで進んだか」 is a row of the six stages 計画 → 設計 → 実装 → テスト →
出荷 → 保守, each with the number of open changes and their numbers as links;
changes at no known stage are listed under 「段階不明」. 「あなたが対応するもの」
is the four cards 「判断・再開が必要」「人の確認待ち」「残した指摘」「文書・写真・
テストの欠け」, each with its count, up to five rows that open the record, and
「ほか N 件」 when there are more; a card with nothing says 「ありません」.
「進行中と次の一手」 lists the changes under way with stage, the next step in
the record's own words, and a bar of contract lines passed. 「ドメイン別」 and
「作業中のブランチ」 follow. With no change open the stage row shows zeros and
the list says 「進行中の変更はありません」.

**機能** is tiles (picture, title, badge, tested share, the rail as six dots)
under three rows of chips: 状態 (安定 / 変更中 / 判断待ち), テスト (すべてあり /
一部 / なし), ドメイン. Each chip shows the count it would leave. A press
chooses one value of the row; with ⌘ or ⇧ held it adds or removes. The choice
is in the link (`#features?state=changing&cover=none`), so a reload or a copied
link shows the same tiles. 「絞り込みを解除」 clears it. Nothing matching:
「条件に合う機能はありません」. A feature without a picture shows its title on a
plain tile.

**A feature's page** gains, above what it has: the rail as six labelled steps
(概要 / 仕様 / 画面写真 / 要件 / テスト / 実機確認) each 済 / 一部 / なし, and
「次の一手」 when a change is open on it. Its sections, reachable from a row of
links at the top: 画面, 概要, 要件 (one card per line with its number, its
tests, and 「テストなし」 on a bare line), 状態と不変条件 (the spec's States and
Invariants), コード (the files by layer — 描画 / 文言・色 / 処理 / テスト /
スクリプト / 文書 — each a row that can be put in the basket), 根拠 (the
landed changes' obligations, carried findings and approvals), 決定 (the spec's
inherited decisions), 変更, 原文. A bar at the foot holds the verdict: 「OK」
「要修正」「保留」 and a note; it is kept in the browser and shown as a mark in
the left column. 「レビュー結果をコピー」 in the bar copies every verdict as one
text; with none it says 「判定がまだありません」.

**作業** is five columns — バックログ / 着手中 / 進行中 / あなたの対応待ち /
最近の完了 — or, with 「一覧」, one table. 「自分」「AI」「すべて」 narrow it. A card
opens its change, or for a backlog item shows its text with 「指示をコピー」.
Without `--backlog`: the two backlog columns say 「バックログは読み込んでいません。
`--backlog` を付けて作り直すと表示されます」. When reading failed: 「バックログを
読めませんでした: 理由」 (the reason in words) with the other three columns intact. More than 200
items: 「バックログが多すぎます。N 件のうち 200 件を表示しています」.

**要件** gains chips (ドメイン, テストあり / なし) with counts and draws lines
as cards grouped by feature; the table stays behind 「表」.

**進め方** draws the pipeline: the six stages across, one row per kind of
request with the stages it passes filled, its attempt ceiling, its closing
check, the evidence asked of it and how many changes took it; below, the
files that make a session stop and ask (止まって確認 / 見守りつき / 任せる)
with their paths and evidence; and the pinned order intent → spec → readiness
→ plan → build → evidence → land. A table that cannot be read:
「進め方の表を読めません: 」 and the file's path in its place.

**カタログ** shows the three palettes side by side, one row per role with its
swatch and value; the marks by name with their use count (「描画には
アプリ内蔵のアイコンフォントが必要です」 in place of a drawing); the keys with
their chords. A source that cannot be read leaves its part saying
「読めませんでした: 」 and the file's path.

**The basket** survives a reload. It holds features, changes, screens,
requirement lines, files and backlog items; takes a sentence (「何をしてほしい
ですか」), a kind of work (修正 / 追加 / レビュー / 文書 / テスト), shows the
text and its length before copying, and copies with the button or ⌘↩. An
entry whose record has gone since it was put in is dropped on load. Storage
that cannot be read or written (private mode) leaves the page working without
memory and says 「この画面では選んだ内容を保存できません」 once.

**The theme** button steps 自動 → ライト → ダーク.

**Keys**: `s` 現況, `f` 機能, `g` 画面, `w` 作業, `r` 要件, `c` 変更, `p`
進め方, `d` ドメイン, `/` search, `b` basket, `j`/`k` next and previous
feature, `1`–`3` verdict on a feature's page; never while typing in a field.

**Served**: `--serve` prints 「地図を配信しています: http://127.0.0.1:8765/
（止めるには Ctrl-C）」. A port in use: 「ポート N は使用中です」, exit 2.

## Design

`scripts/change-map.py` gains, each a function over what `build` already holds
unless said:

- `status_of(changes, features, domains, requirements)`; `feature_depth(...)`
  for `cover`, `rail`, `code`, `next`, `evidence` (obligations counted from the
  `verification.yaml` text `read_change` already holds); `work_of(changes,
  backlog)`.
- `read_backlog(root, seconds)` — one child, `subprocess.run` with a timeout,
  `stdout` read to `BACKLOG_BYTES + 1`, `GIT_*` dropped as `other_worktrees`
  does. The script run is the mapped root's own, so a fixture supplies a
  stand-in and no environment variable names a command.
- `read_process(root)` over the three tables with `block`; `read_catalog(root)`
  over `src/theme.rs`, `src/glyphs.rs`, `src/app/keymap.rs` and a walk of
  `src/` (at most 400 files of 2 MiB) for the mark counts.
- `serve(root, port, requests, backlog)` — `http.server.HTTPServer` bound to
  `127.0.0.1`, a handler that rebuilds the model per `GET /`, a socket timeout
  and a request-line ceiling; `build` gains a `backlog` argument.
- `main` gains `--backlog`, `--backlog-seconds`, `--serve`, `--port`,
  `--requests`. Nothing else about `--json`, `--find` or the written file
  changes; `--json` gains the new keys.

`scripts/change-map/page.js`: `MapLogic` gains the functions of requirements
7–9 and `parseRoute`/`formatRoute` carry a `query`. `main()` gains the left
column, the 現況, 作業, 進め方 and カタログ views, the tiles and chips of 機能,
the sections and verdict bar of a feature's page, the chips and cards of 要件,
the second basket and the theme. `page.html` gains the bar's entries and the
column; `page.css` the layout and a light theme under
`:root[data-theme="light"]` with `prefers-color-scheme` behind `auto`.

Storage keys: `operon-map-basket`, `operon-map-verdicts`, `operon-map-theme`,
read once at load inside `try`, written on change.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | The application's palettes are read and shown, not changed; the page's own colours are CSS of a generated file outside the application. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | `ICON_VOCABULARY` is read, not extended. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Stages, routes, surfaces, methods, roles, marks and keys are read from the file that defines them, never restated; requirement 5 and 6 compare the model to those files. The storage names live once in `page.js`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | The store is not touched; the page is still written beside its target and renamed. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child in the application. The script's one new child (`backlog.py`) has a time and byte ceiling, an argument list and no shell. |
| Documentation — user-facing docs change in all three languages together | no | `docs/features/README.md` is a developer document in English; no manual changes. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | The page requests nothing; the server binds `127.0.0.1`; GitHub is reached only through change 323's script and only with `--backlog`. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | Backlog: 30 s, 1 MiB, 200 items. Source walk: 400 files, 2 MiB each. Request: 8 KiB, 5 s. Attention cards: 5 rows; next: 8; landed cards: 8; evidence: 3. |

## Flagged concerns

- **A served page can be reached by any local process.** Answered: it is
  read-only, binds the loopback address, serves one path, and shows what the
  repository's files already show to any local process; it accepts no input
  that is stored or run, so there is nothing for a request to change.
- **The backlog needs GitHub.** Answered: only `--backlog` reaches it, through
  the script change 323 landed and with that script's own ceilings; every
  failure is a sentence on the board and never a failed map. The script is not
  changed, so the gate-configuration surface is not touched.
- **Verdicts kept in the browser are not a record.** Answered: intent.md
  decides this; the page says 「この判定はこのブラウザにだけ保存されます」 beside
  the verdict bar, and the copied text is what reaches a session.
- **`page.js` grows to roughly twice its size.** Answered: the logic stays one
  exported object tested from `node` and the drawing one function per view;
  splitting the file would change how the template is assembled and the
  two-block rule that change 316's tests hold, for no behaviour.

## Acceptance

- The twelve tests named above pass, with the earlier map tests:
  `cargo test --locked map_` and `cargo test --locked change_map_`.
- `python3 docs/sdlc/changes/325-the-map-shows-neither-where-work-stands-nor-what-to-do-next/mutations.py`
  watches every new guard fail.
- In headless Chrome, on this repository's own map: each of the ten views
  draws with no console error; a chip narrows the tiles and a reload keeps the
  choice; an item put in the basket and a verdict given are there after a
  reload; the copied text names the files; the theme steps; the served page
  shows a document edited between two loads.

## Rejected alternatives

- A second page for the new views — two maps to keep current, and the left
  column could not span both.
- Feature stages as learn_korean has them — an Operon feature has no stage;
  inventing one would be a second record of position.
- Writing verdicts or basket entries to the repository — the map writes
  nothing back.
- Reading the backlog on every run — a network call in a tool every session
  runs, and a slow or signed-out `gh` would stall it.
- An environment variable naming the backlog command — a command taken from
  the environment; the mapped root's own script needs none.
- A framework or a bundler for the page — the page is one file a person can
  open from disk, and its tests read it as text.
