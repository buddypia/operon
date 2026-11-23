# Spec: domains, a glossary, the screen flow and a requirement board on the product map, kept true by the suite

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. `docs/features/domains.yaml` lists between 3 and 16 domains, each with a
   unique kebab-case `id`, a Japanese `title`, `summary`, `owns` and `excludes`,
   `adjacent` ids that exist and are mutual, and `modules` that exist; every
   `src/` module the module map of `CLAUDE.md` names is in exactly one domain's
   `modules`; every feature of `docs/features/index.yaml` carries a `domain:`
   that exists, and every domain has at least one feature —
   `domains_cover_every_module_once_and_every_feature`. The module map is
   read as every backticked token beginning `src/` between the
   `## Shape of this codebase` and `## Policies` headings, the way
   `feature_catalogue_claims_every_module_of_the_map` reads it; a token ending
   in `/` (`src/ui/`) stands for each `.rs` file directly in it, and a domain
   claims those by listing the directory or the file; `src/app.rs` and
   `src/app/screens.rs` are separate tokens; `src/tests.rs` is excluded (it is
   the suite, owned by no domain).
2. `docs/features/glossary.yaml` lists terms, each with a unique kebab-case
   `id`, a Japanese `term`, a Japanese `meaning`, a `feature` of the catalogue,
   and optionally `en`, `ui`, `code`, `avoid`, `see`. Every `ui` word is part of
   a Japanese message id in `src/i18n_tables.rs`; every `code` name occurs as a
   whole word in a `src/` file other than `src/tests.rs`; every `see` id is a
   term; no two terms share a `term` —
   `glossary_terms_resolve_to_features_screen_words_and_code`.
3. No word a term lists under `avoid` occurs in a Japanese message id of
   `src/i18n_tables.rs` or in a feature brief, and no `avoid` word is another
   term's `term` — `no_screen_word_uses_a_word_the_glossary_avoids`. An
   `avoid` word is at least two characters and is matched as a substring
   (Japanese has no word boundary to match on), so it is chosen specific
   enough that today's tables and briefs pass.
4. `docs/features/flow.yaml` lists screens (`id`, Japanese `title`, `kind` of
   `page tab sheet overlay panel cli`, `screen` of the catalogue's closed set,
   `features` of the catalogue, optional `parent` screen, `page` (a `Page` variant), `tab` (a `ProjectTab` variant) and `image` that exists)
   and ways (`from`, `to` screens, and at least one of `key` — an action id of
   `src/app/keymap.rs` — and `label` — part of a Japanese message id). Every
   `Page` variant of `src/app.rs` and every `ProjectTab` variant is named by a
   screen's `page` or `tab`; every screen but the CLI is reached from `home` by
   ways; every feature appears on at least one screen —
   `screen_flow_reaches_every_page_and_every_way_resolves`. Reached means: `home`
   is reached, and the `to` of a way whose `from` is reached is reached; a
   `parent` is a grouping for drawing and is not a way, so a tab needs a way of
   its own (the tab's button). `screen` is the catalogue screen id the box is
   drawn under (`session`), and `page` is the `Page` variant's name
   (`Sessions`); the two are separate fields and neither is derived from the
   other.
5. Every action id `src/app/keymap.rs` defines whose name begins with `page.`,
   and `palette.open`, `session.new` and `attention.next`, is the `key` of at
   least one way — `every_navigation_key_is_a_way_in_the_flow`.
6. A behaviour line of a feature spec may end with `Pinned by` and test names;
   every name given exists as a `#[test]` function in `src/tests.rs`, and every
   spec whose "Code and tests" names a test pins at least one behaviour line
   with one. No share of lines is required: which line a test pins is a
   reading of the test, and the board shows the lines nothing pins —
   `feature_behaviours_name_tests_that_exist`.
7. The change map model carries `domains` (with their features, modules and the
   changes that touched any of their features), `terms` (with the features
   whose brief uses them), `flow` (screens with their ways in and out and
   their terms) and `requirements` (every numbered behaviour: feature, number,
   text, tests), and `--find WORD` matches a domain, a term and a screen —
   `change_map_carries_domains_terms_flow_and_requirements`.
8. The page has the views 製品, ドメイン, 画面の流れ, 用語集, 要件 and 変更一覧,
   addressed by `#domain/<id>`, `#screen/<id>`, `#term/<id>`, `#requirements`;
   the flow is drawn as SVG inside the page; a feature view shows its domain
   and gains a 用語 tab; a 作業かご collects features, screens, behaviours and
   terms and produces one instruction; and the page loads nothing from outside —
   `change_map_page_has_domain_flow_glossary_and_requirement_views`.
9. A missing or unreadable `domains.yaml`, `glossary.yaml` or `flow.yaml`, a
   file past its ceiling, and entries past their count ceilings still produce a
   page with the other views working and the reason shown in the affected view —
   `change_map_survives_a_missing_domain_glossary_or_flow`. The test exercises:
   no `domains.yaml` (the ドメイン view reads 「ドメインの定義を読めません」 and
   the product and change views still render); a `glossary.yaml` over 256 KiB
   (「用語集を読めません」 with 「256 KiB を超えています」); a `flow.yaml` that is
   not the flat format (「画面の流れを読めません」); and 17 domains, of which 16
   are carried and the page says 「ドメインが多すぎます。17 件のうち 16 件を表示しています」.
10. `feature-pilot` intake names `docs/features/domains.yaml` and
    `docs/features/glossary.yaml`, and close (sdlc §10) names updating the
    glossary and the flow when a change adds a term, a screen or a way —
    `the_pipeline_reads_the_domain_the_glossary_and_the_flow`.

## Behaviour

`python3 scripts/change-map.py` still writes `.tmp/change-map.html` and prints
the summary; `--json` gains `domains`, `terms`, `flow`, `requirements`. The
header gains four views.

```
┌ Operon 製品地図 [製品][ドメイン][画面の流れ][用語集][要件][変更一覧] [検索] [作業かご 3] ┐
```

- **ドメイン** — one card per domain: title, summary, number of features and
  how many are changing. `#domain/<id>`: 担当すること, 担当しないこと, 隣のドメイン
  (links), 機能 (cards with badges), コード (module paths and the module map's
  line for each), 用語 (terms of its features), 変更 (changes touching its
  features, newest first).
- **画面の流れ** — an SVG diagram: screens as boxes grouped in bands by
  catalogue screen, ways as arrows labelled with the key (⌘1) or button word.
  Choosing a box highlights its ways and opens `#screen/<id>` beside the
  diagram: picture, kind, 機能, 入ってくる道, 出ていく道, 用語. A screen with no
  picture shows its name only.
- **用語集** — terms grouped by domain (of their feature), searchable; each shows
  meaning, 英語名, 画面での表記, コード, 使わない言葉, 関連語, 機能. `#term/<id>`
  scrolls to and marks one. A feature's 用語 tab lists its terms and the terms
  its brief uses.
- **要件** — a table of every behaviour line: 機能, 番号, 内容, テスト. Filters:
  domain, feature, 「テストの記録あり / なし」, and text. The count of lines with
  no test is in the heading.
- **作業かご** — a check box on a feature card, a screen, a behaviour row and a
  term adds it. The basket lists them with a type (変更 / 不具合 / 文書 /
  テスト) and a 「指示をコピー」 button that copies a Japanese instruction naming
  each item with its documents (`docs/features/<id>/brief.md`, `spec.md`, the
  behaviour number, the term's entry, the screen's entry) and asking the session
  to start through feature-pilot. The basket lives in the page's memory; when
  the copy is refused the text is selected for a manual copy
  (「コピーできませんでした。選択された文を手でコピーしてください」).
- Empty and error states: a missing or unreadable file empties its view with
  「ドメインの定義を読めません」「用語集を読めません」「画面の流れを読めません」;
  a domain with no change says 「このドメインの機能に触れた変更の記録はありません」;
  a term search with no hit says 「該当する用語はありません」; an empty basket
  says 「まだ何も入っていません」.

## Design

Placement verdict: **`EXTEND`** — `scripts/change-map.py` (changes 312, 313)
grows three source files and four views; `docs/features/` (change 313) gains
the files. Evidence: `python3 scripts/change-map.py --find 312` and `--find 313` list the two changes that built the map, and
`git grep -n -i "glossary\|domains.yaml\|flow.yaml"` finds only leftover `.claude/scripts/lib/` code that expects a `data/registry/glossary.json` which does not exist; no product glossary, domain or flow file is in place.
No `src/` module changes; only `src/tests.rs`. Features named: none (the map
is the development process, not something Operon's user sees, per 313).

What is ported from learn_korean, and what is not:

| learn_korean | Here |
|---|---|
| owner domains with boundary and adjacent domains | `domains.yaml` |
| `glossary-terms.json` with trace to feature, UI words, code, forbidden terms | `glossary.yaml` (`feature`, `ui`, `code`, `avoid`) |
| interaction graph and route inventory extracted from Dart | `flow.yaml`, written by hand and checked against `Page`, `ProjectTab`, keymap ids and message ids |
| widget mock-ups from a component catalogue | the real screenshots in `docs/images/` |
| FR board joining SPEC to progress | the requirement board over spec Behaviour lines and `Pinned by` |
| work basket copying a CLI instruction | 作業かご |
| CONTEXT.json state and progress | not ported: derived from change records (313) |
| spec-hash review packets | not ported: approvals stay per change |

- **Files.** All three are flat like `index.yaml` (`- id:` or `- from:` opens an
  entry, `key: value` lines belong to it), read by one parser in Python and one
  in `src/tests.rs`. `index.yaml` gains `domain:` per feature.
- **Domains own code, features own behaviour.** A module is in exactly one
  domain; a feature is in exactly one domain; a shared file such as
  `src/app.rs` is in the domain of the application shell.
- **Terms.** A term belongs to one feature (and through it to a domain).
  A feature's brief "uses" a term when the term's word occurs in it. `avoid`
  words are checked against the screen's words and the briefs, which is what
  makes the glossary the product's vocabulary rather than a list beside it.
- **Flow.** Screens are finer than the catalogue's eight screens: the four
  pages, the project tabs, the side panel, the new-session sheet, ⌘K, the
  broadcast sheet, settings, the CLI. Layout is computed in Python: bands by
  catalogue screen in a fixed order, boxes in a band by a breadth-first order
  from `home`, arrows as cubic curves; the SVG is written into the page.
- **Pinned by.** `1. … Pinned by `a_test`, `b_test`.` at the end of a
  behaviour line. Writing these for the 21 specs is part of this change, from
  what each spec's "Code and tests" already says a test pins; a line no test
  pins is left without one and shows as such.
- **No external command** is added; the page stays one self-contained file.
- **Ceilings.** 256 KiB per file; 16 domains, 400 terms, 80 screens, 300 ways;
  the surplus is named and not carried.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | Nothing in `src/` paints; the page's CSS keeps text at 4.5:1 on its backgrounds. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | no | No glyph in the application. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | Screen words are checked against `src/i18n_tables.rs`, keys against `src/app/keymap.rs`, pages and tabs against their enums, rather than restated; the closed `kind` set lives once in `flow.yaml`'s header. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | The store is not touched; the page is still written temp-then-rename. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No new child; tests run the script as the existing map tests do. |
| Documentation — user-facing docs change in all three languages together | no | The application and manuals do not change; the glossary and domains are internal documents for the reviewer, Japanese like the briefs. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | Local files only; requirement 8's test fails on any reference outside the page. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | The ceilings above; the page grows by text and one SVG. |
| Approval — `.claude/skills/sdlc/references/approval.md` | yes | No paused surface is touched. The direction is the person's request, quoted in the intent; the two readings the session decided are recorded in `state.yaml`. |

## Flagged concerns

- **The flow is written by hand.** Operon has no router table to extract it
  from. Requirements 4 and 5 catch a page, tab or key that has no way and a way
  whose word or key does not exist; a button that moves between screens but has
  no message id of its own, or a way that no longer happens, is the reviewer's
  to catch. Recorded, not hidden.
- **`avoid` can be over-broad.** A word avoided for one concept may be right for
  another. Each `avoid` word is chosen so that today's tables and briefs pass;
  a later legitimate use is resolved by narrowing the word, not by an exception
  list.
- **Pinned by is a claim.** The test checks the named test exists, not that it
  pins that line; the names come from the specs' own "Code and tests" sentences.

## Acceptance

- Each of the ten tests passes (`cargo test --locked <name>`), and every test
  of changes 312 and 313 still passes.
- `python3 docs/sdlc/changes/315-the-feature-map-holds-no-domain-terms-or-screen-flow/mutations.py`
  — every mutation caught.
- The page in headless Chrome: each view, a domain, a term, a screen of the
  flow, the board filtered, the basket copied; no console error.
- `bash scripts/check-bands.sh` — the same breaches as before at the same tiers.

## Rejected alternatives

- **Copy learn_korean's viewer and generator**: 7,800 lines of JavaScript built
  around Flutter, ARB and Supabase; Operon's map is one script with its own
  model, and the ideas port, not the code.
- **A JSON per feature holding state (CONTEXT.json)**: a second record of
  position, which 312 and 313 exist to avoid.
- **Extract the flow from the source**: egui has no route table; parsing
  `self.page = Page::…` assignments would find where, not by which control, and
  would still need the words a person reads.
- **Drawn mock-ups of screens**: real screenshots already exist and do not drift
  from the product's look.
- **Terms inside each feature's brief**: a term shared by features would be
  defined twice; one file, linked both ways.
