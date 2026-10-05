# Plan: Word-level marks inside a changed diff row

- **Spec**: `./spec.md`
- **Approved**: 2026-08-30
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/theme.rs` | Add `diff_added_emphasis` and `diff_removed_emphasis` to `Palette`, to all three `*_PALETTE` tables, and to `tokens()`. |
| `DESIGN.md` | Six new front-matter values, the two roles in the Diff list, and the paragraph that says why a row now carries two bands. |
| `src/models.rs` | `DiffLine` gains `highlights: Vec<Range<usize>>`. |
| `src/git.rs` | The token split, the bounded comparison, the replacement-block pairing, and the similarity gate; run once at the end of `parse_diff`. |
| `src/ui/diff.rs` | Draw a marked span as a section background inside the row's existing galley. |
| `src/tests.rs` | The four new comparison tests, the contrast checks for the two roles, and the raised token-count tripwire. |
| `README.md`, `README.ja.md`, `README.ko.md` | Say that a rewritten line marks the words that changed. |

## Order of work

1. `src/theme.rs` and `DESIGN.md` together — a role added to one and not the
   other fails `design_md_documents_exactly_what_the_app_paints`, so they are
   one step, not two.
2. `src/models.rs`: add the field. The tree does not compile between this step
   and the next, because every `DiffLine` literal in `src/git.rs` and
   `src/tests.rs` is now missing a field. That is deliberate — the compiler
   enumerating the construction sites is cheaper than grepping for them.
3. `src/git.rs`: construct the field empty at each site, then add the tokenizer,
   the bounded comparison, the pairing walk, and the gate.
4. `src/ui/diff.rs`: split the line append on the marked spans.
5. `src/tests.rs`: the four comparison tests, the two contrast checks, the
   tripwire.
6. The three READMEs.
7. The three Rust gates, the band check, packaging, and the installed-app
   replacement `AGENTS.md` requires.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A byte range lands inside a multi-byte character | egui panics while laying out a Japanese or emoji diff line | The tokenizer walks `char_indices`, and a test compares two lines of Japanese text. |
| A mispairing marks unrelated words | The pane confidently points at the wrong change | The similarity gate, and `a_replacement_block_pairs_its_lines_in_order_and_leaves_the_surplus_alone`. |
| A long line makes parsing quadratic | A large diff takes visible time to open | `DIFF_WORD_TOKEN_LIMIT`, and `a_line_past_the_word_comparison_ceiling_still_marks_its_middle`. |
| A mark is invisible inside its own wash | The feature silently does nothing on one theme | The 1.5:1 check against the row wash in `every_theme_stays_readable`. |
| A mark makes the line unreadable | Code on a marked span cannot be read | The 4.5:1 check of `text` over each mark. |
| Spans computed against the wrong string | Marks drift sideways from the words they mean | The tests assert the marked substrings by value, not the ranges by number. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no new breach beyond the recorded
  `always_loaded_bytes` warn.
- The four new tests fail before this change (the field does not exist) and pass
  after.
- In the installed app, a rewritten line shows marks on the changed words only.

## Departures from the plan

Two, both found by a test rather than by review.

**The tokenizer needed a rule for scripts written without spaces.** The plan's
tokenizer was "a run of word characters, a run of whitespace, or one other
character", which makes `セッションを復元しました。` a single token — so a
Japanese line could only be marked entirely or not at all, and the similarity
gate then chose "not at all". This app's own interface is Japanese and one of
its three READMEs is Korean, so that is not an edge case. `writes_without_spaces`
now makes each CJK, kana, and Hangul character its own token, and adjacent
marked characters merge back into one band. Korean and Japanese cases were added
to `a_rewritten_line_marks_only_the_words_that_changed`.

**Spec requirement 4 contradicted requirement 3, and the ceiling test was not
testing the ceiling.** Written up as lesson 009 and corrected in `spec.md`: the
over-ceiling fallback produces one very wide band, requirement 3 exists to throw
very wide bands away, and the gate wins. `docs/sdlc/lessons.md` also records why
the first version of that test passed on a build with no ceiling at all.

Each of the five guards this change adds was watched failing under a mutation of
the code it guards before it was trusted — the renderer dropping the bands, the
similarity gate disabled, CJK back to runs, and the ceiling branch removed.
