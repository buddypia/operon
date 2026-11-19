# Plan: feature documents, a feature-first page, and the rules that keep them true

- **Spec**: `./spec.md`
- **Approved**: 2026-11-19
- **Status**: approved

Interrogated before acceptance. What breaks: nothing in the application — no
`src/` file but `src/tests.rs` changes. What was assumed: the catalogue's `paths`
are a good enough proxy for which change touched which feature (shared files are
ignored, older links are labelled estimates). The alternative was to type
progress into the feature documents; the spec rejects it. What proves it done:
the nine tests, the sweep, the documents written from the code, and the page
opened in a browser.

## Files that change

| File | Change |
|---|---|
| `docs/features/index.yaml` | new: the catalogue |
| `docs/features/<id>/brief.md` | new, 21 of them: Japanese, for the reviewer |
| `docs/features/<id>/spec.md` | new, 21 of them: English, for the AI |
| `scripts/change-map.py` | the feature layer, the screens, and the feature-first page |
| `src/tests.rs` | the nine tests of the spec and their fixture helpers |
| `.claude/skills/feature-pilot/SKILL.md` | the reversal, intake and close read and update the feature documents |
| `.claude/skills/sdlc/SKILL.md` | close (§10) updates the documents; `state.yaml` names features |
| `docs/sdlc/README.md` | the narrowed "Not built" entry and a row under "Built" |
| `docs/sdlc/templates/state.yaml` | the `features:` field |
| `docs/sdlc/changes/313-the-product-cannot-be-seen-feature-by-feature/` | `mutations.py`, `verification.yaml`, `approvals.log`, `state.yaml` |

## Order of work

1. This plan and the catalogue, committed before any source file is edited.
2. The tests of requirements 1–5 and 9 in `src/tests.rs`, watched failing (no
   brief or spec exists; the field and the skill text are absent).
3. The 21 briefs and specs, written from the code and the manual by one writer
   per screen, then read and checked here against the tests and the code.
4. `features:` in the template and 313's state; the skill, sdlc and README text.
5. The tests of requirements 6–8 (the degraded states: a missing document, an
   unreadable catalogue, no image, over the ceilings, over 64 entries and the
   `unmapped` count among them), watched failing; then `scripts/change-map.py`:
   catalogue parse, derivation, model, page.
6. The page opened in a headless browser, three screenshots, fixes for what
   reads badly.
7. `mutations.py`, `verification.yaml`, both verifiers, `state.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A brief states behaviour the code does not have | the reviewer is told something false | the writers cite code and tests; rule 3's test fails on a path or test that does not exist; I read each pair against the manual |
| A shared file makes every change touch every feature | the 進捗 tab is noise | paths claimed by more than two features are ignored; requirement 6's test pins it |
| The page grows past what a browser opens quickly | slow open | images are the only new weight (about 1.5 MB); measured in step 6 |
| The new test rule 5 fires on an unrelated change later | a landing is refused for the wrong reason | it checks route `feature` / `modify` only, and only paths claimed by one or two features |
| The skill grows past a steering band | `scripts/check-bands.sh` names a new tier | the bands contract item; wording replaces lines |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked` on each new test name — `ok. 1 passed`.
- `python3 docs/sdlc/changes/313-the-product-cannot-be-seen-feature-by-feature/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — the same breaches at the same tiers.
- The page in headless Chrome: product view, a feature's four tabs, the change
  list, no console error.

## Departures from the plan

- **Catalogue field added.** `index.yaml` gained `shared:` for files that serve the
  whole product (strings, colours, drawing parts, the suite): claimed so no module
  is nobody's, never used to tie a change to a feature. Without it every change
  that touched `src/i18n_tables.rs` showed as a change to 設定.
- **Order of the document tests.** The tests of requirements 1–5 and 9 were written
  after the 21 briefs and specs, so they passed on their first run; their red is
  shown by the sweep instead (each mutation removes a heading, a claim, a test
  name, a path or a sentence and a test goes red). Tests 6–8 were watched failing.
  The first sweep caught 22 of 24: two page sentences were asserted against the
  whole page, where the documents' own text satisfied them, and are now asserted
  against the page without its data block.
- **Screenshots live once.** A screen carries an image file name and the model holds
  one `images` map of data URIs, so a screenshot shown on a screen and again under
  a feature's 画面 tab is stored once. The page is about 10 MB (the seven images are
  3.7 MB of it, not the 1.5 MB the spec estimated).
- **The first commit carried the paper trail only**: a commit that lands
  `docs/features/` is judged as code by the review gate and needs
  `verification.yaml`, so `index.yaml` went with the commit that holds the evidence.
- **Not done:** the leftover scripts that expect a domain-map.json are untouched;
  `cargo test --locked -- --ignored` was not run (restore and the shared-session
  archive did not move).
- **F2, found after the push, by the coordinator rather than a verifier.** One line of
  the page script opened a string with a double quote and went on with single
  quotes, so the script failed to parse and the product tab rendered empty at head
  `fd528a7`. Every test and CI stayed green because nothing parsed the script, and
  the screenshots had been taken before that line was written. Fixed, and
  `change_map_page_script_parses` now runs `node --check` on the page's script (it
  fails loudly where node is absent); it was watched failing on the broken line
  first. The page is re-checked in a browser after every edit to its script.
