# Spec: a change map generated from the records — everything at a glance, the documents on demand

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. Every directory under `docs/sdlc/changes/` is on the map with the route,
   stage, and status its `state.yaml` records; none is skipped —
   `change_map_carries_every_change_with_its_recorded_position` (run against
   this repository; the position is read from the `state:` block only, never
   from the `sets:` block that most state files also carry).
2. Changes whose status is `awaiting-user`, `blocked`, or `failed` are listed
   first, in the page and in the text summary, each with its `resume` line —
   `change_map_puts_what_waits_for_the_person_first`.
3. A change that exists only in another working copy of the repository appears
   with the position recorded there and the branch it is on —
   `change_map_reads_changes_under_way_in_other_worktrees`.
4. A change's `intent.md`, `spec.md`, `plan.md`, `screen.md`, and other text
   records are carried in the page byte for byte (up to the ceilings of
   requirement 7), and every finding left open in
   `review.yaml` appears with its `person:` line —
   `change_map_embeds_the_documents_and_open_findings_as_written`.
5. Each change is filed under the areas its plan names, and
   `--find WORD` lists the changes of an area or with the word in their name or
   title — `change_map_groups_changes_by_the_area_their_plan_names`.
6. The page loads nothing from outside itself, and a document's text cannot
   end the page's data block or run as markup —
   `change_map_page_loads_nothing_from_outside`.
7. A change whose record cannot be read is still shown, first, with the reason;
   a document past the size ceiling is cut and marked —
   `change_map_shows_a_change_whose_record_cannot_be_read`.
8. The map knows every status the pipeline defines and nothing else, read
   against `docs/sdlc/templates/state.yaml` —
   `change_map_knows_every_status_the_pipeline_defines`.
9. The pipeline sends a session to the map where a request starts and where
   placement is judged, and `docs/sdlc/README.md` records what was built and
   why the earlier decision was narrowed —
   `change_map_is_named_where_a_request_starts`.

## Behaviour

One command, three outputs.

python3 scripts/change-map.py writes the page to `.tmp/change-map.html`
(ignored by git; `--out PATH` chooses another place) and prints a summary a
session or a person can read in the terminal:

```
変更 308 件 — 要対応 1 · 進行中 1 · 完了 302 · 保管 4
要対応
  312-changes-cannot-be-surveyed-at-a-glance  awaiting-user  build  feature/312-…
    次にすること: <resume>
進行中
  313-…  in-progress  build  feature/313-…
    次にすること: <resume>
地図: /…/.tmp/change-map.html
```

python3 scripts/change-map.py --find tmux prints one line per matching
change (`名前  status  stage  領域  — 題`) and writes nothing; no match prints
「該当する変更はありません」 and exits 0. `--json` prints the model without
document bodies, for tests and tools.

The page, at 1280px (narrower: the detail covers the whole width):

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Operon 変更地図   308 件 · 要対応 1 · 進行中 1 · 完了 302 · 保管 4        │
│ [ 検索: 番号・題・領域 ]                       生成 2026-11-18 21:40 · eb1109a │
├──────────────────────────────────────────────────────────────────────────┤
│ あなたの判断を待っています (1)                                            │
│ ┌ 312 変更を一目で見渡せない ─ 判断待ち ─ build ─ feature/312-… ───────┐ │
│ │ 次にすること: …resume…                                               │ │
│ └──────────────────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────┤
│ 進行中 (1)     計画 │ 設計 │ 実装 │ テスト │ 出荷 │ 保守                 │
│                     │      │ ●313 │        │      │                      │
├──────────────────────────────────────────────────────────────────────────┤
│ 領域ごとの変更                                                            │
│ app        画面と状態            ■■■■■■■■■■■■ 96   [311] [310] [308] … さらに 84 │
│ tmux       セッションの起動と停止 ■■■■ 31          [309] [303] [288] …           │
│ 開発プロセス                      ■■■■■■■ 71      [312] [304] [300] …           │
│ 未分類                            ■ 9             [128] …                       │
├──────────────────────────────────────────────────────────────────────────┤
│ すべての変更   [状態 ▾] [経路 ▾]                                         │
│ 312  変更を一目で見渡せない   判断待ち  build  feature  開発プロセス      │
│ 311  …                        完了      test   modify   app, i18n         │
└──────────────────────────────────────────────────────────────────────────┘

choosing any change opens, on the same page:
┌ 312 changes cannot be surveyed at a glance ──────────── [前へ][次へ][閉じる] ┐
│ 判断待ち · feature · リスク low · feature/312-…                            │
│ 計画 ─ 設計 ─ ●実装 ─ テスト ─ 出荷       試行 0/3 · 質問 1/7             │
│ 次にすること: …                                                          │
│ [概要] [意図] [仕様] [計画] [画面案] [未解決の指摘 2] [承認の記録] [その他] │
│ ……the chosen document, rendered; [原文] shows it exactly as written……     │
└──────────────────────────────────────────────────────────────────────────┘
```

- 概要 shows the completion contract as a checklist (済 / 未 / 免除 with the
  text that decides each), the areas, and where the change lives.
- The address carries the choice (`#312/spec`), so a link opens one document of
  one change directly; Esc closes the detail.
- Status words a person reads: `planning` 計画中, `ready` 着手可, `in-progress`
  実装中, `reviewing` 検証中, `done` 完了, `awaiting-user` 判断待ち, `blocked`
  停止中, `failed` 失敗, `archived` 保管, unreadable 読めない記録. The raw value
  stays visible beside it in the detail.
- Empty states: no change waits → 「判断を待っている変更はありません」; nothing
  under way → 「進行中の変更はありません」; a tab whose document does not exist
  is not shown; a search with no match → 「該当する変更はありません」.
- Errors: a `state.yaml` that is missing, has no `status`, or a status outside
  the closed set puts the change first with 「記録を読めません: <理由>」. If the
  list of working copies cannot be read (no git, a timeout), the map is built
  from this checkout alone and says 「他の作業コピーを読めませんでした」 in the
  header and the summary. A root with no `docs/sdlc/changes/` exits 2 with
  「docs/sdlc/changes がありません」.
- Too large: a document over 256 KiB is cut at the ceiling and marked
  「長いため途中まで表示しています」; a change directory's records past the
  fortieth are named, not carried.
- Interrupted: the page is written to a temporary file beside the target and
  renamed, so a half-written page never replaces a whole one.

## Design

Placement verdict: **`NEW_MODULE` outside the crate** — a new script,
scripts/change-map.py. Evidence: `git grep -n -i "feature map\|change-map\|機能地図"`
finds nothing outside this change's own directory, and the one branch with
the name, `feature/feature-map-review-surface`, has no commit beyond `main`;
`scripts/pipeline-indicators.sh` computes elapsed-time
indicators from git history and prints JSON (a different question: how fast,
not where); `scripts/change-receipt.py` reconstructs one change at one commit.
Neither lists positions or carries documents. No `src/` module changes, so the
module map in `CLAUDE.md` is untouched.

- **Sources, and nothing else.** For each change directory: `state.yaml`
  (the `state:` block alone for the position — 174 state files repeat
  `status:` inside `sets:` — plus `resume` and `contract:`), every `*.md`, `review.yaml`
  (`carried:` items and their `person:` lines), `approvals.log`,
  `verification.yaml`. Parsed by line shape, as `scripts/check-review.sh` and
  the tests do. `docs/sdlc/routes.yaml` gives each route's stages;
  `docs/sdlc/templates/state.yaml` gives the status set; the top-level
  `` `src/<name>.rs` — … `` lines of `CLAUDE.md` give each area its
  description. Nothing is written back and no ledger is added.
- **Other working copies.** One `git worktree list --porcelain` (GIT_* removed
  from the environment, ten-second deadline, 64 KiB of output, at most 64
  working copies). A change name absent from this checkout is taken from the
  first working copy that has it, those on a branch before detached ones, and
  carries that path and branch. A name present here is read here.
- **Areas.** Paths are the backticked tokens containing `/` or ending in a
  file extension, and the first cell of each table row (a plan names a file
  that does not exist yet without backticks, because the path guard reads
  them), under `## Files that change` in `plan.md`; without that
  section, under `## Systems likely affected` in `intent.md`; without either,
  the change is 未分類. `src/tests.rs` and the change's own directory are not
  areas. `src/<name>.rs` and `src/<name>/…` are the area `<name>`;
  `scripts/`, `.claude/`, `.cli/`, `.githooks/`, `.github/`, `.codex/`,
  `docs/sdlc/`, `evals/`, `Makefile`, `AGENTS.md`, `CLAUDE.md`, `REVIEW.md` are
  開発プロセス; `Cargo.toml` and `Cargo.lock` are 依存関係; anything else is
  文書. A change belongs to every area it names, most-named first.
- **Groups.** `awaiting-user blocked failed` and unreadable → 要対応;
  `planning ready in-progress reviewing` → 進行中; `done` → 完了; `archived` →
  保管. The table is in the script once, and requirement 8 holds its keys to
  the template's set.
- **The page.** One HTML file: inline CSS, inline JavaScript, and the model as
  JSON in a `<script type="application/json">` block with `<`, `>`, `&`,
  U+2028 and U+2029 written as `\u` escapes, so no document text can close the
  block. Documents are rendered in the browser by a small escape-first
  Markdown renderer (headings, lists, tables, fenced code, inline code,
  emphasis, links); 原文 shows the text in a `<pre>`. No external script,
  style, font, or image. Light and dark follow the system.
- **Process integration.** `.claude/skills/feature-pilot/SKILL.md` step 0
  ("Resume or new") runs the script instead of `ls`, because `ls` on one
  checkout cannot see a change under way in another; step 1 ("Placement") runs
  `--find` on the request's nouns and reads the matching changes' specs before
  choosing a verdict. `docs/sdlc/README.md` gains the row under "Built" and
  the narrowed reason under "Not built".
- **No external command besides** `git worktree list --porcelain`. Nothing
  runs in the application, on a background thread, or in the draw path.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Nothing in `src/` paints. The page's colours are its own CSS; `DESIGN.md` governs the application's palette, and the page keeps text at 4.5:1 or better on its two backgrounds. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No glyph in the application; the page uses text labels. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Statuses come from `docs/sdlc/templates/state.yaml`, stages from `docs/sdlc/routes.yaml`; requirement 8 fails when the script's grouping names a status the template lacks or misses one. The command spelled in the skill is checked against the script's existence by requirement 9. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | The store is not read or written. The page is written temp-then-rename so an interrupted run leaves the earlier page. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | The application spawns nothing new. The tests run the script through `run_command_with_output_limit`; the script's one `git` call has a deadline and an output ceiling and inherits no `GIT_*`. |
| Documentation — user-facing docs change in all three languages together | no | `README.md` and its translations describe the application, which does not change. `docs/sdlc/README.md` is English-only by convention. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | A local file; requirement 6 fails if the page references anything outside itself. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | 256 KiB per document, 40 records per change, 64 working copies, 64 KiB and ten seconds for the `git` call. With the changes present today the page is about 5 MB. |

## Flagged concerns

- **The person's premise is unconfirmed** — they asked about a feature map as
  though one existed, and the working copy of
  `feature/feature-map-review-surface` (no commits; uncommitted content not
  read) is the one place it could still be. Answer: recorded in `intent.md` as an open question with
  its adopted default; the change stops at `awaiting-user` before landing, so
  nothing reaches `main` on the assumption.
- **`docs/sdlc/README.md` lists a domain map and HTML review decks under "Not
  built"** — Answer: those rows rejected a hand-kept ownership ledger and a
  deck rendered per checkpoint by a Node runtime. This is neither: one
  generated file, no ledger, no checkpoint, one Python script with no
  dependency, which passes that section's own rule (a flat data file, a bash
  script, or a Rust test — the script is the bash-script kind). The reason for
  the first rejection also weakened: at 307 changes "git and the artifacts are
  the state" is true and no longer readable by a person. The rows stay; a
  sentence says what change 312 built beside them. Managing-session decision,
  undone by a revert.
- **The script's JavaScript is not run by `cargo test`** — Answer: the tests
  hold the data and the page's structure; the interaction is exercised once in
  a headless browser and recorded as a `human` contract item with what was
  observed.

## Acceptance

- `cargo test --locked change_map_` passes: the nine tests above.
- `python3 docs/sdlc/changes/312-changes-cannot-be-surveyed-at-a-glance/mutations.py`
  catches every mutation.
- `make q.fast` and `bash scripts/check-bands.sh` report no new breach.
- Opened in a browser, the generated page shows change 312 under
  「あなたの判断を待っています」, and choosing it shows this spec as written.

## Rejected alternatives

- **A hand-kept feature-map document or JSON** — a second record of each
  change's position; it drifts from `state.yaml`, which the hooks read.
- **A committed generated page** — stale on the next commit, and 5 MB of diff
  noise per change.
- **One HTML file per change plus an index** — a directory to keep in step and
  dead links when one is regenerated alone; one file opens anywhere and a link
  with `#312/spec` reaches the same place.
- **A tab inside Operon** — the application manages terminals for any project;
  this repository's pipeline records are not a product feature.
- **A strict area × stage grid** — every change but this one is closed, so the stage
  axis would be one full column. Stages are drawn only for work under way, and
  areas carry the history.
- **Reading branch tips that have no working copy** — the pipeline opens a
  worktree for every change, so a branch without one is landed or abandoned.
- **Removing the leftover domain-map scripts here** — gate configuration; a
  removal there is the person's decision and a different route.
