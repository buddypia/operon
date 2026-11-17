# Plan: compare a file with a past revision, side by side, and annotate it

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17 (evaluator)
- **Status**: departed from (see below)

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `GIT_BLAME_MAX_BYTES`, `GIT_FILE_HISTORY_MAX_BYTES`, `FILE_HISTORY_MAX_ENTRIES`, `DIFF_SIDE_BY_SIDE_MIN_WIDTH` |
| `src/git.rs` | blame / log parsers, `DiffBase`, argument builder, three git calls |
| `src/models.rs` | `DiffLayout`, `EditorView::Annotate` |
| `src/store.rs` | `diff_layout` field |
| `src/ui/diff.rs` | side-by-side rows, change blocks, `DiffPaneView` / `DiffPaneOutcome`, pair drawing |
| `src/app.rs` | compare state, caches, background keys/results, requests |
| `src/app/screens.rs` | compare bar, picker, 注釈 view, gutter menu, F7, call-site updates |
| `src/glyphs.rs` | three icons |
| `src/i18n_tables.rs` | strings |
| `DESIGN.md`, `README*.md` | icon line; feature and budget lines in three languages |
| `src/tests.rs` | R1–R10 tests |

## Order of work

1. Contracts: R1, R2, R5 parser/argument tests failing, then `src/git.rs`.
2. R6 store round-trip test failing, then `DiffLayout` + store field.
3. R3, R4, R8 pure-layout tests failing, then the functions in `src/ui/diff.rs`.
4. R7, R9 pane tests failing, then `DiffPaneView` and pair drawing; update the
   two call sites (Git screen passes 統合 with actions on, so it is unchanged).
5. R10 test, then `git_blame` outcomes.
6. Wiring: caches, background tasks, compare bar, picker, 注釈 view, F7, strings,
   icons, docs.
7. `make q.fast`, `cargo test --locked diff_307_ i18n icon design`, screenshot
   the running build.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Pairing drops or duplicates a line | missing code in one column | R3 asserts every index exactly once |
| Git screen behaviour shifts with the new pane signature | stage/discard gone there | existing `ide_261_*` tests and the Git screen passing actions on |
| Hunk action on a non-HEAD diff applies a wrong patch | file corrupted | R7: buttons not drawn off HEAD |
| A blame of a huge file blocks or eats memory | stutter | 4 MiB ceiling, background thread, R10 |
| Per-frame cost of pairing | stutter on large diffs | linear pass, same order as today's row build; checked on a 180 KB diff |
| Old store fails to load | data loss | R6 old-file test |

## Proof of completion

- `make q.fast` — fmt and clippy clean.
- `cargo test --locked diff_307_` — 10 passed.
- CI full suite green on the branch head.
- Running app screenshot of 差分 (左右), 注釈, and a commit's change.

## Departures from the plan

- `src/app/compare.rs` is new: the 差分 and 注釈 views moved out of
  `src/app/screens.rs` rather than growing it (module line in `CLAUDE.md`,
  paid for by trimming three other lines under the always-loaded band).
- The side-by-side columns scroll sideways together by one shared offset, not
  by the pane's horizontal scroll bar: one bar over two columns carried the
  right column out of sight.
- Dates come from git (`%ad`) and from blame's `author-tz` through
  `civil_date_time`; the crate has no date library.
- F1 (empirical-verifier): a commit subject holding a unit separator was cut;
  `parse_file_log` now splits into at most five fields.
- F2 (formal-verifier): an unknown `diff_layout` value rejected the whole
  store; `lenient_diff_layout` reads it as the default. Lesson 117.
- Not done: the running app was not driven on screen — the debug build's
  accessibility tree could not be read and physical input was not permitted;
  layout is pinned by `diff_307_side_by_side_pane_draws_both_columns` and the
  person is asked to look after packaging.
