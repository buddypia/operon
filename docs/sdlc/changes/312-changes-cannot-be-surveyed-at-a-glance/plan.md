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

- **Areas for a change with no plan table.** The spec filed such a change as
  未分類. Measured on this repository that was 93 of 308, most of them bug
  fixes, whose route writes no intent or spec. A third source was added: the
  repository paths named in the comment at the head of `state.yaml` (the bug
  report), in `bug.md`, `plan.md`, `spec.md`, `verification.yaml`, and
  `mutations.py` — minus the template pointer every state file carries and the
  sweep runner every sweep names. 42 remain 未分類; their records name no
  path, and the map says so rather than guessing.
- **Page size.** 6.6 MB, not the 5 MB estimated: the page also carries
  `state.yaml`, `review.yaml`, `verification.yaml`, and `approvals.log`.
- **No commit id in the header.** The spec's sketch showed one; reading it is
  a second `git` call, and the Design allows one. The header shows when the
  page was generated.
- **Three guards were weaker than their requirement**, found by the sweep and
  tightened before the first commit: the unquoted table path was also reached
  by the new third source (the fixture now uses a path only the table rule
  reads); a change with no state file was still found through a sibling
  worktree (the test now requires it to be read from this checkout); and the
  README check accepted the change number in either of two rows (it now reads
  the domain-map row).
- **Seen in a browser** (headless Chrome, 1440×900, this repository's page):
  308 changes; 312 in the 実装 column; its detail opened with 概要, 意図, 仕様,
  計画, 承認の記録, 状態の記録; `#312/spec` opened the spec rendered and
  `#295/findings` the three open findings in Japanese; search `tmux` left 22
  rows, a word with no match showed 「該当する変更はありません」; all 716
  Markdown documents rendered without an exception or an injected element; no
  console error.
