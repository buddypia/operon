# Spec: feature documents, a feature-first page, and the rules that keep them true

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. `docs/features/index.yaml` lists every feature with a unique kebab-case `id`,
   a Japanese `title`, a `screen` of the closed set the file defines, a
   `summary`, `paths` that all exist, a `manual` heading that exists in
   `docs/MANUAL.ja.md`, and an `image` that exists when given; and each feature
   has `docs/features/<id>/brief.md` and `spec.md` with the required headings —
   `feature_catalogue_is_complete_and_every_entry_resolves`.
2. Every `src/` module the module map of `CLAUDE.md` names is claimed by at
   least one feature's `paths` —
   `feature_catalogue_claims_every_module_of_the_map`.
3. A brief is written for a person (Japanese headings and body), a spec for an
   AI (English headings); and a spec's code references are true: every `src/`
   path it names exists, and every test function it names exists in
   `src/tests.rs` — `feature_documents_are_in_their_language_and_name_real_code`.
4. Every change numbered 313 or above carries `features:` in its `state:`
   block, naming ids of the catalogue or `none` —
   `every_change_from_313_names_features_that_exist`; and `docs/sdlc/templates/state.yaml`
   defines the field.
5. A change of route `feature` or `modify`, numbered 313 or above, that names a
   feature lists that feature's `spec.md` in its plan's "Files that change",
   and one whose plan lists a `src/` file claimed by one or two features names
   those features — `a_change_that_alters_a_feature_updates_its_spec`.
6. The change map model carries the features: each with its screen, title,
   summary, the documents as written, the state badge (`steady`, `changing`,
   `waiting`), the stage of the changes in flight, and the timeline of changes
   that touched it newest first (derived from state files, plan file lists
   against `paths`, and the explicit `features:` field), and the screens with
   their images embedded; changes the derivation cannot place are counted in
   `unmapped` — `change_map_carries_the_features_and_their_progress`.
7. The page opens on the product (screens, feature cards with their badge, what
   waits for the person first), the feature view has the tabs BRIEF, SPEC, 進捗,
   画面 and is addressed by `#feature/<id>/<tab>`, the change list is a second
   view, and the page loads nothing from outside —
   `change_map_page_is_feature_first_with_the_change_list_second`.
8. Degraded states hold: a feature whose document is missing, an `index.yaml`
   that cannot be read, a screen without an image, a document or image past its
   ceiling, and a catalogue past 64 entries still produce a page, with the
   reason shown and the surplus named and not carried —
   `change_map_survives_missing_feature_documents_and_an_unreadable_catalogue`.
9. `feature-pilot` names `docs/features/index.yaml` and the touched feature's
   documents at intake and at close, no longer says there is "no feature folder",
   and `docs/sdlc/README.md` records the reversal with its reason —
   `the_pipeline_reads_and_updates_the_feature_documents`.

## Behaviour

`python3 scripts/change-map.py` still writes `.tmp/change-map.html` and prints
the summary; the summary gains one line per feature that is `changing` or
`waiting`. `--json` gains `features` and `screens`. `--find WORD` also matches a
feature's id, title or screen. The page opens on the product.

```
┌ Operon 製品地図        [製品] [変更一覧]            [検索]                  ┐
│ あなたの判断を待っています (n)   ← changes awaiting the person, pinned      │
│ 進行中 計画 設計 実装 テスト 出荷 保守  ← in-flight changes by stage        │
├──────────────────────────────────────────────────────────────────────────┤
│ ホーム          [real screenshot]   ┌機能カード┐ ┌機能カード┐             │
│ プロジェクト     [real screenshot]   │ 安定      │ │ 変更中 実装│             │
│ セッション …                          └──────────┘ └──────────┘             │
└──────────────────────────────────────────────────────────────────────────┘
feature view (#feature/git-changes/brief):
  ← 製品に戻る   変更・コミット・PR   [badge]   画面: プロジェクト
  [BRIEF] [SPEC] [進捗] [画面]
```

- Badges: 安定 when no change in flight touches the feature; 変更中 (with the
  stage word) when one does; 判断待ち when a change that touches it waits for
  the person (it wins over 変更中). A feature touched by an older change only is
  安定. The explicit `features:` field makes a link certain; a link inferred
  from a plan's file list or an older change's title is labelled 推定 in the
  timeline.
- 進捗 lists each touching change, newest first: number, title, status word,
  stage, whether the link is recorded or estimated, and opens the change's
  detail (the 変更一覧 view). A feature with no change says
  「この機能に触れた変更の記録はありません」.
- 画面 shows the screenshot and the manual's section, rendered.
- Empty and error states: a feature whose `brief.md` or `spec.md` is missing
  shows 「文書がありません」 in that tab and the page still opens; an
  `index.yaml` that cannot be read leaves the product view empty with
  「機能の一覧を読めません」 and the change list still works; a screen with no
  image shows its name and features without a picture. Documents and images are
  read once per run, up to 256 KiB per document and 2 MiB per image.
- Search matches a feature's title, id, summary and screen as well as changes.

## Design

Placement verdict: **`EXTEND`** — `scripts/change-map.py` grows a feature layer
over the model it builds from the change records (change 312), and the page
becomes feature-first. Evidence: `python3 scripts/change-map.py --find 312`
shows the one change that built the generator; `git grep -n "domain-map\|features/"`
finds only leftover scripts that expect a domain-map.json under docs/features, which
nothing runs and this change does not touch. No `src/` module changes: the only
Rust edits are tests in `src/tests.rs`, so `CLAUDE.md`'s module map stands.

**The reversal this change makes.** `feature-pilot` said "no feature folder, no
separate feature spec" because a second record of one position drifts and the
hooks read only the first. That reasoning still binds a change's *position*
(route, stage, status), which stays only in the change's `state.yaml`. What it
did not consider is a different fact: what the product does *now*. A change's
spec is a delta and cannot answer that; nothing else does. So `docs/features/`
holds that second fact, and holds it under three rules that keep it from
drifting the way the forbidden record would: progress is never written into it
(it is derived on every run), a change that alters behaviour updates it in the
same branch (requirement 5), and its references to code are checked
(requirement 3). `docs/sdlc/README.md`, where change 312 narrowed the "Not
built" entry for the domain map, records this second narrowing and its reason.

- **Catalogue.** `docs/features/index.yaml`, flat `- id:` entries, one parser in
  Python for the page and one in `src/tests.rs`. Derived from
  `docs/MANUAL.ja.md` sections 2–11, `docs/images/`, and the module map; the
  development process (`scripts/`, `.claude/`, `docs/sdlc/`) is deliberately not
  a feature, since it is not something Operon's user sees.
- **Documents.** `docs/features/<id>/brief.md` — Japanese, for the reviewer:
  何のための機能か, 誰がいつ使うか, 画面のどこにあるか, 今できること,
  今できないこと・既知の制約, 関連する変更. `spec.md` — English, for the AI:
  Behaviour (numbered, current), States, Invariants, Code and tests, Decisions
  inherited. Both describe main as it is.
- **Derivation.** A change touches a feature when (a) its `state.yaml` names it
  in `features:` (recorded), (b) its plan's file list names a path a feature
  claims, ignoring a path claimed by more than two features, `src/tests.rs`, and
  the change's own directory (inferred), or (c) a feature keyword appears in the
  change's name or title and neither (a) nor (b) found anything (estimated).
- **Process.** `state.yaml` gains `features: "id id"` (or `none`) in the
  `state:` block and the template; intake reads the catalogue and the touched
  feature's two documents before placement and cites the feature in the
  verdict; close (sdlc §10) adds the documents' update to the change's files,
  and the plan lists them. Enforcement is requirements 4 and 5 in the suite CI
  runs on every push; whether the prose of a spec still matches behaviour a
  test cannot see is the reviewer's judgement, and is not claimed as enforced.
- **No external command** is added; the page remains one self-contained file.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Nothing in `src/` paints; the page's colours are its own CSS and keep text at 4.5:1 on its backgrounds. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No glyph in the application. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Screens and feature ids live once in `docs/features/index.yaml`; requirements 1, 4 and 5 read them there rather than restating them. Statuses stay read from the template (change 312). |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | The store is not touched; the page is still written temp-then-rename. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new child process; tests run the script through `run_command_with_output_limit` as the existing map tests do. |
| Documentation — user-facing docs change in all three languages together | no | The application and its manuals do not change; the briefs are a new internal document for the reviewer, Japanese by decision (the person reads Japanese), and the specs are English. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Local files only; requirement 7's test fails on any reference outside the page. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | 256 KiB per feature document, 2 MiB per screenshot, at most 64 features (the surplus is named, not carried); the page was about 5 MB and grows by the seven images (about 1.5 MB). |
| Approval — `.claude/skills/sdlc/references/approval.md` | yes | No paused surface is touched (`docs/sdlc/routes.yaml`, `risk.yaml`, the gate scripts and `.github/workflows` are not edited). The person was asked and did not answer in ten minutes; the three direction decisions are the managing session's and recorded as such in `state.yaml`, with a revert undoing each. |

## Flagged concerns

- **Brief and spec are prose, and prose drifts.** Rules 3 and 5 catch a dead
  path, a vanished test, and a change that forgot to name its feature; they
  cannot judge whether a sentence is still true. The rule that a changing
  behaviour carries the update, and the reviewer reading the diff, are what
  hold the rest. Recorded here, not hidden.
- **Estimated history for the 308 older changes.** Older changes name no
  feature, so their link is inferred from plan file lists and keywords and shown
  as 推定; some will be wrong or missing. Changes the derivation cannot place
  are listed on the 変更一覧 view as 機能未設定 and counted in the final report.

## Acceptance

- `cargo test --locked feature_ change_map_ every_change_from_313 a_change_that_alters the_pipeline_reads`
  passes (each filter run on its own).
- `python3 docs/sdlc/changes/313-the-product-cannot-be-seen-feature-by-feature/mutations.py`
  — every mutation caught.
- The page opened in headless Chrome: the product view with the seven
  screenshots, a feature in each tab, the change list, no console error.
- `bash scripts/check-bands.sh` — the same breaches as before at the same tiers.

## Rejected alternatives

- **Keep progress in the feature documents** (a status line per feature): the
  second record of a position that 312 exists to avoid; derived instead.
- **A feature folder per change** (the leftover domain map's layout): a state
  machine for a layout; the catalogue is one file and the documents one folder.
- **Generate the briefs from the code and manual**: the briefs are the thing a
  person reviews, so a person-readable sentence per fact is written and
  checked, not extracted.
- **Make the page's feature view a separate file**: one generator, one output
  path, one file the person already opens.
