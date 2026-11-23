# Plan: domains, a glossary, the screen flow and a requirement board on the product map

- **Spec**: `./spec.md`
- **Approved**: 2026-11-23
- **Status**: draft

Interrogated before acceptance. What breaks: nothing in the application — no
`src/` file but `src/tests.rs` changes; the risk is to the page script and to
later changes, which the new tests can refuse. What was assumed: the flow can
be written by hand and kept true by checking it against `Page`, `ProjectTab`,
the keymap ids and the message ids; `avoid` words can be chosen so today's
text passes. The alternative was to extract the flow from the source; the spec
rejects it. What proves it done: the ten tests, the sweep, the documents read
against the code, and the page opened in a browser.

## Files that change

| File | Change |
|---|---|
| `docs/features/domains.yaml` | new: the domains, their boundaries, neighbours and modules |
| `docs/features/glossary.yaml` | new: the product's terms |
| `docs/features/flow.yaml` | new: the screens and the ways between them |
| `docs/features/index.yaml` | `domain:` per feature; the header names the three new files |
| `docs/features/<id>/spec.md` | `Pinned by` on the behaviour lines a named test pins (the 21 specs) |
| `scripts/change-map.py` | parse the three files; model `domains`, `terms`, `flow`, `requirements`; the four views, the 用語 tab and the 作業かご |
| `src/tests.rs` | the ten tests of the spec and their fixture helpers |
| `.claude/skills/feature-pilot/SKILL.md` | intake reads the domain and the glossary; close keeps the glossary and flow true |
| `.claude/skills/sdlc/SKILL.md` | §10 names the glossary and the flow |
| `docs/sdlc/README.md` | a row under "Built" for the three files |
| `docs/sdlc/changes/315-the-feature-map-holds-no-domain-terms-or-screen-flow/` | `mutations.py`, `verification.yaml`, `approvals.log`, `state.yaml` |

## Order of work

1. This plan, committed with the spec before any source file is edited.
2. The tests of requirements 1–6 and 10 in `src/tests.rs`, watched failing (no
   file exists, no field, no skill text).
3. `domains.yaml` and `domain:` in `index.yaml`, written from the module map and
   the 21 feature specs.
4. `flow.yaml`, written by reading `src/app.rs`, `src/app/screens.rs`,
   `src/app/keymap.rs` and the manual's chapters 2 and 8, every label looked up
   in `src/i18n_tables.rs`.
5. `glossary.yaml`, drafted per domain by writers who read the code, the briefs
   and the manual, then read here entry by entry against the tests and the code.
6. `Pinned by` on the specs' behaviour lines, from what each spec's "Code and
   tests" says its tests pin, each name checked by requirement 6's test.
7. The skill, sdlc and README text.
8. The tests of requirements 7–9, watched failing; then `scripts/change-map.py`:
   parsers, model, the views, the basket.
9. The page in headless Chrome: every view, a domain, a term, a screen, the
   board filtered, the basket; screenshots; fixes for what reads badly.
10. `mutations.py`, `verification.yaml`, both verifiers, `state.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A term, a domain boundary or a way states what the code does not do | the person and later sessions are told something false | the writers cite code; requirements 2, 4 and 5 fail on a word, key, symbol or screen that does not exist; I read every entry against the code |
| An `avoid` word refuses a legitimate future label | a later change is refused for the wrong reason | words are specific phrases, at least two characters, checked against today's text; the spec says to narrow a word rather than list an exception |
| The page script breaks and the views render empty | a blank view while the tests stay green (313's F2) | `change_map_page_script_parses` runs `node --check`; the page is opened in a browser after every script edit |
| The SVG flow is unreadable with ~30 boxes and ~70 arrows | crossing arrows, overlapping labels | bands by screen, breadth-first order, arrows highlighted only for the chosen box; judged in screenshots |
| `Pinned by` names a test that pins something else | the board claims a line is pinned when it is not | names come from the specs' own sentences; the flagged concern says the test checks existence only |
| The skill grows past a steering band | `scripts/check-bands.sh` names a new tier | the bands contract item; wording replaces lines |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked <name>` for each of the ten tests and the 312/313 map and
  feature tests — `ok`.
- `python3 docs/sdlc/changes/315-the-feature-map-holds-no-domain-terms-or-screen-flow/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — the same breaches at the same tiers.
- The page in headless Chrome: the six views, a domain, a term, a screen of the
  flow, the board filtered to lines with no test, the basket's instruction;
  no console error.

## Departures from the plan

- **A way may be a click.** `flow.yaml` gained `click:` beside `key:` and
  `label:`: some ways are a click on a row that has no word of its own (a project
  row opens its overview). The Japanese text says what is clicked; requirement 4
  requires one of the three.
- **Floors in the tests.** The glossary and spec tests also hold lower bounds
  (40 terms, 30 screen words, 30 code names, 10 avoided words, 50 pinned lines)
  so a file emptied by mistake does not pass by having nothing to check.
- **Arrowheads drawn by hand.** An SVG `<marker>` is referenced through `url(`,
  which the page's no-external-load check refuses; each arrowhead is a small
  polygon instead.
- **どこからでも.** A key way that leaves every page (⌘K and the page keys) would
  draw an arrow from each of 27 boxes. Those ways are listed once as a tag, and
  only the chosen screen's ways are drawn as arrows.
- **The data files ride the landing commit.** The plan commit could not carry
  `domains.yaml`, `glossary.yaml` and `flow.yaml` before the tests existed; the
  catalogue test reads the working tree, so they were written then and committed
  with the code.
- **Written by agents, read by me.** The glossary (three drafts merged, one
  duplicate replaced by 前提), the flow and the 224 `Pinned by` lines were
  drafted by subagents from the code; every entry is held by requirements 2, 4,
  5 and 6, and a 順番待ち entry duplicated across drafts was the one fix the tests
  did not force.
- **The sweep.** 26 mutations, 26 caught on the first run.
