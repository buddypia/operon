# Spec: a palette that finds sessions and projects

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. With nothing typed the palette lists the five actions it lists today, in the
   order it lists them today, and nothing else.
2. With something typed it lists every action, session, and project whose
   searchable text contains the query, case-insensitively, ranked and capped at
   twelve results.
3. A session's searchable text is its displayed title, its request, its agent
   label, its branch, and its project's name. A project's is its name and its
   path. An action's is its visible label.
4. Ranking, best first: a match at the start of the row's own name beats a match
   at the start of a word in it, which beats a match anywhere in the text. Ties
   break by kind — actions, then sessions, then projects — and then by recency,
   newest first, using a session's launch or creation time and a project's added
   time.
5. The selection starts at the first result and is clamped into range whenever
   the results change, so it never points past the end of a shorter list.
6. While the palette is open, the up and down arrows move the selection, `Enter`
   activates the selected result, and `Escape` closes the palette. The text
   field keeps focus throughout.
7. The selected result is drawn on the palette's own selected-row surface.
8. Activating an action does what it does today. Activating a session opens it
   in the terminal workspace, which also clears its unread mark. Activating a
   project selects it and opens the project page. In every case the palette
   closes and its query is cleared.
9. A session row shows its title, its state token, its agent, and its project. A
   project row shows its name and a shortened path.
10. The results are computed once per frame, from the store as it is, and
    nothing is cached between frames.

## Behaviour

- `⌘K`, type `log`: sessions whose title or request mentions logging appear
  under any action whose label matches, best match first, with the first one
  selected. `Enter` opens it.
- `⌘K`, type a project name: that project's row appears, and its sessions do
  too, because a session carries its project's name in its searchable text.
- `⌘K` with nothing typed: the five actions, exactly as today.
- A query matching nothing: 「一致する項目がありません」.
- More than twelve matches: the best twelve, and a line saying how many more
  there are.

## Design

| Piece | Shape |
|---|---|
| A result | `PaletteEntry { kind: PaletteKind, label: String, detail: String, score: PaletteScore }` where `PaletteKind` is `Action(&'static str)`, `Session(Uuid)`, or `Project(Uuid)`. |
| Ranking | `palette_rank(name: &str, haystack: &str, query: &str) -> Option<PaletteScore>`: `None` when the query is absent from the haystack; otherwise `Name` when the name starts with the query, `Word` when a word in the haystack does, `Anywhere` otherwise. `PaletteScore` orders `Name < Word < Anywhere` so a plain sort puts the best first. |
| Building | `palette_entries(&self) -> Vec<PaletteEntry>`: the actions, then one entry per session, then one per project, each ranked; sorted by `(score, kind rank, Reverse(recency))`; truncated to `PALETTE_RESULT_LIMIT`. |
| Keyboard | handled in `update` beside the existing `⌘K` and `Escape`, before the window draws, so the selection the window paints is the one the keys just moved. |
| Activation | `activate_palette_entry(kind)`, which the click path and the `Enter` path both call. |
| Drawing | `ui_command_palette` draws the field, then one `selectable_label` per entry with its detail beneath, then the empty or overflow line. |

`PALETTE_RESULT_LIMIT` is 12 and lives in `src/config.rs` beside the other
ceilings.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The selected row uses `selectable_label`, which draws on the theme's existing selection surface; no colour is introduced. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new mark; the rows are text. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The action keys stay the English strings they already are, matched in one `match`; the new ceiling is a constant in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is persisted. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process. |
| Documentation — user-facing docs change in all three languages together | Yes | The `⌘K` bullet in `README.md`, `README.ko.md`, and `README.ja.md` is rewritten. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Reads the store in memory. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | One pass over sessions and projects per frame while the palette is open, capped at `PALETTE_RESULT_LIMIT` rows drawn. Both collections are already bounded by the store, and the pass is string matching with no allocation per candidate beyond the entry it keeps. |

## Flagged concerns

- **Work in a draw path.** `.claude/rules/rust.md` forbids per-frame work, and
  this ranks the whole store every frame the palette is open. Resolved by
  measuring rather than by asserting: a test builds the entries against a store
  of 500 sessions and 50 projects and asserts the pass stays under 5 ms, which
  is a fraction of a 60 Hz frame. The palette is open only while a person is
  typing into it, and caching on the query would add a cache to invalidate for a
  cost the measurement says is not there.
- **`command_selection` exists and is read by nothing.** That is lesson 005's
  shape — a widget written for a bug and never called — and this change is what
  makes it reachable. The keyboard test is what proves it.

## Acceptance

- `cargo test --locked` passes, including `an_empty_palette_query_lists_only_the_actions`,
  `the_palette_finds_a_session_by_its_request_and_its_project`,
  `the_palette_ranks_a_name_match_above_a_word_match_above_anywhere`,
  `the_palette_selection_is_clamped_when_the_results_shrink`, and
  `ranking_the_palette_over_a_full_store_stays_inside_a_frame`.
- In the running app: `⌘K`, type part of a session's request, press `Enter`, and
  land in that session's terminal.

## Rejected alternatives

- **A fuzzy subsequence match**, as an editor's file finder uses. It is the
  right algorithm for paths and the wrong one for Japanese: a query of two kana
  matches almost every request as a subsequence, and the ranking then has to
  undo what the matcher did.
- **Caching the results on the query.** The measurement says the pass is cheap,
  and a cache keyed on a query is a cache that has to be invalidated when a
  session's state changes underneath it.
- **Sections with headings.** More layout than this change has agreed, and the
  kind is already visible in each row's detail line.
