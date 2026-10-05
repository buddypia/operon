# Spec: Word-level marks inside a changed diff row

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Where a removed line and an added line are the same line rewritten, the diff
   pane SHALL mark the spans that differ inside each of the two rows, in
   addition to the existing row wash and sign.
2. Pairing SHALL be positional within a replacement block — a run of removed
   lines immediately followed by a run of added lines — and SHALL leave the
   surplus lines of the longer run unpaired.
3. A pair SHALL be marked only where enough of the two lines is common; a pair
   whose marks would cover most of both rows SHALL be left with the row wash
   alone. A wholesale replacement is not a rewrite and marking it says nothing.
4. Comparison SHALL be bounded by a stated token ceiling per side. Past that
   ceiling the pair keeps the common prefix and suffix it can find in linear
   time and marks the remaining middle as one span, rather than running an
   unbounded comparison. That span is subject to requirement 3 like any other,
   so a line whose shared head and tail are only a small part of it keeps no
   marks at all — the fallback is blunter, never louder.
5. The marks SHALL be computed once, while the diff is parsed — never while a
   frame is drawn.
6. Marks SHALL use two new semantic palette roles, one per side, documented in
   `DESIGN.md` with values in all three themes. The line's own ink SHALL remain
   readable at WCAG 2.1 AA over each mark, and each mark SHALL be separable from
   the row wash it sits inside.
7. A line that is not part of a marked pair — context, a hunk header, a note, an
   unpaired addition or removal — SHALL be drawn exactly as it is drawn today.

## Behaviour

The review pane and the editor's 差分 tab look as they do now: a file bar, then
rows, each with two line numbers, a sign, and the line. What changes is a row
belonging to a matched pair. Inside the existing wash, the words that differ
carry a second, stronger band of the same hue, so `-    let ready = false;`
against `+    let ready = true;` shows a mark under `false` and under `true` and
nowhere else.

Where the two sides share almost nothing — a line replaced outright, or a
removed line that happens to sit above an unrelated added one — no marks are
drawn and the row reads as it does today. Where the line is longer than the
comparison ceiling, the common start and end are still found, and everything
between them is marked as a single span.

No new string is shown to a person, so there is nothing to translate.

## Design

`DiffLine` gains `highlights: Vec<Range<usize>>` — byte ranges into its own
`text`, empty for every line that is not part of a marked pair. Nothing about
`DiffLine` is persisted, so no store shape moves.

`src/git.rs` keeps the whole comparison, because turning git's output into what
the pane reads is already its concern and the pane must not compute anything.
After a file's lines are parsed it walks them once, finds each replacement
block, and for each positional pair:

1. Splits both sides into tokens — a run of word characters, a run of
   whitespace, or one other character — so a mark falls on a word rather than
   on a letter.
2. Trims the tokens both sides share at the start and at the end.
3. If either remaining side is longer than `DIFF_WORD_TOKEN_LIMIT` tokens, marks
   the whole remaining middle on each side and stops. Otherwise it runs a
   longest-common-subsequence over the remaining tokens and marks the tokens
   outside it.
4. Merges touching marked tokens into spans, and discards the whole result
   unless both sides keep at least `DIFF_WORD_MIN_COMMON` of their bytes
   unmarked.

`src/ui/diff.rs` draws a marked span as a `TextFormat` background on that
section of the row's existing single galley, so a marked row still costs one
galley and the pane's virtualised scrolling is untouched.

`src/theme.rs` gains `diff_added_emphasis` and `diff_removed_emphasis`, with
`DESIGN.md` rows in all three themes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Yes | Two roles, three themes each, documented in `DESIGN.md`, added to `tokens()` and to the token-count tripwire; `every_theme_stays_readable` gains a check that the line ink clears 4.5:1 over each mark and that each mark clears 1.5:1 against its own row wash. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph is added; the mark is a band of colour behind existing text. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | Nothing here is an identifier two systems exchange. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | `DiffLine` is derived from a `git diff` on demand and never written to the store; it carries no `Serialize`. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No subprocess is added; the git invocation is unchanged. |
| Documentation — user-facing docs change in all three languages together | Yes | The diff description is updated in `README.md`, `README.ja.md`, and `README.ko.md` together. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | A pure in-process comparison over text already in memory. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `DIFF_WORD_TOKEN_LIMIT` bounds the quadratic comparison per side; past it the pair falls back to the linear prefix/suffix trim. The diff itself is already bounded upstream. |

## Flagged concerns

- **A wrong pairing marks the wrong words** — positional pairing is cheap and
  never reorders, but a removal with no counterpart shifts every later pair in
  its block. Answered: the similarity gate in requirement 3 is what makes that
  safe. A mispaired line has almost nothing in common with its partner, so its
  marks are discarded and the row falls back to today's appearance. The failure
  mode is a missing mark, never a misleading one — and the test asserts exactly
  that on a block where the runs are different lengths.
- **A minified bundle on one line** — a single 200 KB line would make an
  unbounded comparison quadratic in something enormous, once per parse.
  Answered by requirement 4: `DIFF_WORD_TOKEN_LIMIT` caps the quadratic part,
  and the prefix/suffix trim that remains is linear and still useful.
- **The fallback band and the similarity gate pull against each other** — the
  over-ceiling path produces one very wide span, and requirement 3 exists to
  throw very wide spans away. Answered while the test for requirement 4 was
  written, which is where the conflict surfaced: the gate wins. A band covering
  most of a row is the highlighter `DESIGN.md` rejects, and it says nothing the
  wash did not. So the fallback is only ever drawn where the trim found a
  substantial shared head and tail, and requirement 4 says so explicitly.

## Acceptance

- `cargo test --locked` passes, including
  `a_rewritten_line_marks_only_the_words_that_changed`,
  `an_unrelated_replacement_keeps_its_row_wash_and_no_word_marks`,
  `a_replacement_block_pairs_its_lines_in_order_and_leaves_the_surplus_alone`,
  `a_line_past_the_word_comparison_ceiling_still_marks_its_middle`, and the
  extended `every_theme_stays_readable` and
  `design_md_documents_exactly_what_the_app_paints`.
- `cargo fmt --check` and `cargo clippy --locked -- -D warnings` pass.
- In the installed app, a file where an agent rewrote one identifier shows the
  old and new identifier marked inside their rows and the rest of the two lines
  unmarked; a file where an agent replaced a block outright shows the rows as
  before.

## Rejected alternatives

- Character-level marks — rejected because a rename produces a scatter of
  single-letter bands that is harder to read than no marks at all.
- Similarity-maximising pairing across the whole block — rejected as more
  machinery than the gate in requirement 3 buys; the gate already turns a bad
  pairing into a missing mark rather than a wrong one.
- Computing marks in the drawing code, on the visible rows only — rejected
  because the pane redraws every frame and the diff does not change between
  them, so it would pay the cost forever to save it once.
- Syntax-highlighting the diff instead — rejected in `intent.md`: it answers a
  different question and would put two meanings on one surface.
