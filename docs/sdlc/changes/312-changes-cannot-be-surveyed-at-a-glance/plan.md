# Plan: a change map generated from the records

- **Spec**: `./spec.md`
- **Approved**: 2026-11-18
- **Status**: approved

Interrogated before acceptance. What breaks: nothing in the application — no
`src/` file but `src/tests.rs` changes. What was assumed: every change under
way has a working copy (the pipeline opens one), so branch tips are not read.
The alternative was a hand-kept map; the spec rejects it. What proves it done:
the nine tests, the sweep, and the page opened in a browser.

## Files that change

| File | Change |
|---|---|
| scripts/change-map.py | new: reads the records, prints the summary, `--find`, `--json`, writes the page |
| `src/tests.rs` | nine `change_map_*` tests and their fixture helpers |
| `.claude/skills/feature-pilot/SKILL.md` | step 0 runs the map instead of `ls`; placement runs `--find` |
| `docs/sdlc/README.md` | a row under "Built", a sentence beside the two "Not built" rows |
| `docs/sdlc/changes/312-changes-cannot-be-surveyed-at-a-glance/` | `mutations.py`, `verification.yaml`, `approvals.log`, `state.yaml` |

## Order of work

1. The tests for requirements 1–8 in `src/tests.rs`, with a fixture repository
   holding a done change, an `awaiting-user` one, a broken one, and one that
   exists only in a second worktree. Watched failing: the script does not exist.
2. scripts/change-map.py: the model (parsing, working copies, areas, groups)
   and `--json`; then the summary and `--find`; then the page.
3. Run it on this repository; read the 未分類 count and the page size, and
   record both under Departures if they differ from the spec's estimate.
4. The skill and `docs/sdlc/README.md`, with the test for requirement 9
   watched failing first.
5. Open the page in a headless browser: the first section, a detail, each tab,
   the address link, the search. Fix what is seen.
6. `mutations.py`, `verification.yaml`, both verifiers, `state.yaml`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A document's text ends the data block or runs as markup | the page is blank or shows stray markup | requirement 6's test puts `</script>` and `<!--` in a fixture document and decodes the block |
| An old `state.yaml` has a shape the parser does not expect | a finished change listed as unreadable | requirement 1's test runs on every record present and allows no unreadable one |
| The skill grows past a steering band | `scripts/check-bands.sh` names a new tier | the bands contract item; the wording replaces lines rather than adding |
| A stale agent worktree shows a landed change as under way | a change listed twice or at an old stage | a name is read from this checkout first and branch-bearing copies before detached ones; requirement 3's test has both |
| The page's JavaScript fails on real data | tabs or search do nothing | step 5, on the real page, with the console read |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked change_map_` — `test result: ok. 9 passed; 0 failed`.
- `python3 docs/sdlc/changes/312-changes-cannot-be-surveyed-at-a-glance/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — the same five breaches as before the change,
  at the same tiers.
- The page opened headless: 312 under 「あなたの判断を待っています」, its spec
  readable, no console error.

## Departures from the plan
