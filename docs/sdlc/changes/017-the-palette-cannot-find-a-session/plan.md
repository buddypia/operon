# Plan: a palette that finds sessions and projects

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `PALETTE_RESULT_LIMIT`. |
| `src/app.rs` | `PaletteKind`, `PaletteScore`, `PaletteEntry`; `palette_rank`, `palette_entries`, `activate_palette_entry`; keyboard handling beside the existing palette shortcuts; `ui_command_palette` rewritten to draw ranked rows. |
| `src/i18n_tables.rs` | EN and KO rows for the new palette strings. |
| `src/tests.rs` | The five tests in the spec's Acceptance. |
| `README.md`, `README.ko.md`, `README.ja.md` | The `⌘K` bullet. |

## Order of work

1. `PALETTE_RESULT_LIMIT`, the three types, and `palette_rank` with its ranking test. Compiles; green.
2. `palette_entries` and `activate_palette_entry`, with the empty-query, finding, and clamping tests. Compiles; green.
3. The keyboard handling and the redrawn window. Compiles; green; clippy clean.
4. The measurement test over a full store.
5. Translations and the three READMEs.
6. Gates, bands, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Ranking the store every frame stutters the window | The palette feels heavy while typing | `ranking_the_palette_over_a_full_store_stays_inside_a_frame`, which measures rather than asserts |
| The selection points past a shrunken list | `Enter` opens the wrong row, or nothing | `the_palette_selection_is_clamped_when_the_results_shrink` |
| Arrow keys reach the text field instead of the list | The caret moves and the selection does not | The keys are consumed in `update` before the window draws |
| The empty query lists everything | The palette becomes a list of every session | `an_empty_palette_query_lists_only_the_actions` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, N five higher.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no new breach.
- Each new test watched failing by mutation.
- In the running app: `⌘K`, part of a request, `Enter`, and the terminal opens.

## Departures from the plan

- **`palette_rank` folds both sides, not just the haystack.** The first version
  lower-cased the name and the haystack and trusted the caller to have
  lower-cased the query. It worked, because the one caller did — and the test
  that passed a mixed-case query failed. A function that is only correct when
  its caller happened to fold one argument is a function whose next caller gets
  it wrong.
- **One pass, capped where it is drawn.** The plan had `palette_entries` return
  the capped list and a separate count of what was left over, which is two
  passes over the store per frame. It now returns everything ranked and
  `palette_visible_entries` truncates, so the rows and the overflow count come
  out of one pass.
- **The row's presentation is computed only for rows that matched.** Building
  the state view for every session in the store, on every frame the palette is
  open, is the shape of per-frame work `.claude/rules/rust.md` is about. Moving
  it below the match is correct, and — measured — worth three percent, which is
  recorded on the measurement test so nobody reads that test as proof the loop
  holds no waste.
- **The measurement takes the fastest pass, not the mean.** It failed once in a
  full-suite run while a release build and a packaging script were running
  beside it. A mean measures the machine as much as the code; the floor still
  says what one pass costs with nothing in its way, which is the question the
  ceiling asks. Confirmed it still bites: a deliberately wasteful loop inside
  the pass fails it at 5.45 ms.
- **What the measurement guards, stated on it.** It is a budget on the whole
  pass, not a guard on any line in it: 2.2 ms in a debug build against a 5 ms
  ceiling, and the shipped binary is `--release`.
