# Plan: the session search finds what the row shows, and the history search says "none" only after it looked

- **Spec**: `./spec.md`
- **Approved**: 2026-11-18
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/screens.rs` | New `OperonApp::session_matches_search(&self, &Session, needle: &str) -> bool`; `ui_session_grid` and `ui_terminal_session_tabs` trim and lower-case `self.search` and call it in place of their `format!` filters; `ui_sessions` draws 「履歴を検索しています…」 while `TranscriptSearch` or `KnowledgeSessionSearch` runs and the "none" sentence only when `transcript_searched` equals the trimmed field. |
| `src/app.rs` | Field `transcript_searched: Option<String>` (initialised `None`), set to the query in the `BackgroundResult::TranscriptSearch` arm. |
| `src/i18n_tables.rs` | Sorted EN and KO rows for 「履歴を検索しています…」. |
| `src/tests.rs` | Four `search_310_*` tests. No test removed, no `#[ignore]`. |
| `docs/sdlc/changes/310-…/` | `mutations.py`, `verification.yaml`, `state.yaml`, `approvals.log` as the gates ask. |

## Order of work

1. Commit `spec.md`, `screen.md`, `plan.md`, `approvals.log` after
   `check-readiness.sh` and `check-approvals.sh` pass.
2. Write the four `search_310_*` tests. The tree does not compile until
   `session_matches_search` and `transcript_searched` exist; add them as stubs
   (`true`, never set) and watch each test fail for its stated reason.
3. `src/app/screens.rs` and `src/app.rs`: the real match, the two call sites,
   the history arms.
4. `src/i18n_tables.rs`.
5. `cargo test --locked search_310_` and the tests Acceptance names;
   `make q.fast`; `bash scripts/check-bands.sh`.
6. `mutations.py` (the "none" claim included), `verification.yaml`, the
   `empirical-verifier` pass; commit, push, CI, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| One field is left out of the match | A word on the row still hides it | R1 asserts each field alone; one mutation per field |
| The grid keeps its own filter or skips the trim | Grid and list disagree; a trailing space empties the grid | R2 draws both views with a trailing space; mutation restoring the untrimmed needle |
| "None" shows for text nobody searched | The sentence at the first keystroke again | R3; mutation dropping the `transcript_searched` comparison |
| "None" shows while a search runs | A person stops waiting | R4; mutation dropping the running check |
| "Searching" shows under an emptied field | A line about a query that is no longer there | R4's emptied-field case; mutation dropping the non-empty condition |
| A finished search never reads "none" | The field is never set | R3's finished case; mutation removing the assignment |
| `src/app.rs` or `src/app/screens.rs` crosses 10 000 lines | `check-bands.sh` reports `propose` | Additions held small; `bash scripts/check-bands.sh` in the contract |
| Orphan or unsorted catalog row | Commit gate fails | The three catalog tests |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked search_310_` — `4 passed; 0 failed`.
- The catalog tests and
  `history_partial_results_are_disclosed_even_during_automatic_discovery` — `ok`.
- `bash scripts/check-bands.sh` — nothing at `propose`.
- `python3 docs/sdlc/changes/310-session-search-misses-what-the-row-shows/mutations.py`
  — every mutation caught.
- Packaged app: a word from an unnamed session's listed title keeps its row;
  「ローカル履歴」 shows nothing until Enter, then the searching line, then
  matches or the "none" sentence.

## Departures from the plan

- **`search_local_transcripts` forgets the finished query on an empty submit
  (F1).** The empirical verifier showed that submitting an emptied field
  clears the matches while `transcript_searched` kept the word they were found
  for, so typing that word again read "none". The empty-query branch now sets
  `transcript_searched = None` beside the clear. Guarded by
  `f310_matches_cleared_by_an_empty_submit_are_not_answered_with_none` and one
  more mutation (16 in all).
- **The agent is searched as its chip writes it (F2).** The verifier showed a
  custom-command row reads 「カスタム CLI」 while the match read only the
  picker's 「別のコマンド」, and that the picker's fallback 「不明な AI」 would
  find a plain terminal that shows no such word. The match now reads
  `agent_short_label`, and the picker's product name only for the three CLIs
  the picker lists. Guarded by `f310_the_agent_chip_on_the_row_is_searched`
  and three mutations in place of one (18 in all).
- **`session_search_needle` is its own function.** The plan had each call site
  trim and lower-case; one function is what lets a test and a mutation hold
  both views to the same text.
