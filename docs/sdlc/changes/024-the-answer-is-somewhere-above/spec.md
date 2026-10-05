# Spec: find something in a session's scrollback

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `⌘F` opens a one-row find bar under the terminal pane and puts the caret in
   its field. The key is consumed, so the agent never receives it.
2. Matching is case-insensitive substring matching over the text of each line,
   with the escape codes already stripped — the pane's colours are styling and
   nobody searches for them.
3. Every occurrence is a match, including several on one line. The bar shows
   which one of how many.
4. `Enter` moves to the next match and `Shift+Enter` to the previous, both
   wrapping. The pane scrolls so the match is on screen.
5. Matches are washed in place, at the columns they occupy. The current one is
   washed more strongly, so "where am I" is answered without reading.
6. Column arithmetic is by display width, not by characters: a CJK character
   occupies two columns in the monospace face this pane uses, which
   `src/ui/terminal.rs` already states and now has a function for.
7. `Esc` closes the bar and gives focus back to the pane. While the field has
   focus, no keystroke reaches the agent.
8. A query being composed in an IME does not search until it is committed —
   change 014's rule, for the same reason: a preedit is not yet text.
9. Nothing here may make the pane do work proportional to the whole scrollback
   on a frame. Matches are recomputed when the query or the buffer changes;
   drawing touches only the rows the viewport shows.
10. Search state is per session, and closing the bar forgets nothing until the
    session goes away.

## Behaviour

The screen was agreed before this was written:

```
┌─ セッション ────────────────────── 1180px ─┐
│ $ cargo test                                    │
│ test result: FAILED. 370 passed; 1 failed      │
│                     ██████  ← 一致              │
│ ---- tests::the_brand stdout ----               │
│ panicked at src/tests.rs:3796                   │
├────────────────────────────────────────┤
│ 🔍 [ FAILED              ]  2 / 3   ‹  ›   ✕   │
└────────────────────────────────────────┘
```

The states that are not the happy one:

- **Bar closed.** No row, and no cost. This is the ordinary state.
- **Empty query.** The counter is blank and nothing is washed.
- **No match.** 「一致しません」 where the counter would be, in the muted ink.
  The arrows are disabled.
- **Composing in an IME.** The field shows the preedit and the count does not
  move until the composition is committed.
- **The buffer grows while the bar is open.** The matches are recomputed and the
  current one keeps its position by index, because the alternative — keeping it
  by line number — jumps when output scrolls the buffer.
- **The session has no output yet.** The bar opens and finds nothing.

## Design

`src/ui/terminal.rs`:

```rust
pub(crate) struct TerminalMatch { pub(crate) line: usize, pub(crate) column: usize, pub(crate) columns: usize }
pub(crate) fn terminal_display_columns(text: &str) -> usize
pub(crate) fn terminal_search_matches(lines: &[LayoutJob], query: &str) -> Vec<TerminalMatch>
```

`terminal_display_columns` is where the 1:2 rule stops being a comment. The pane
is drawn in Sarasa, whose CJK advance is exactly twice its ASCII advance, so a
column offset is a count of display cells and not of characters.

`terminal_rows` gains the matches and the index of the current one, and paints
a wash behind each match that falls inside the range `show_rows` gave it —
which is why the cost is the viewport and not the buffer.

`src/app.rs` gains `terminal_search: HashMap<Uuid, TerminalSearch>`:

```rust
pub(crate) struct TerminalSearch {
    pub(crate) open: bool,
    pub(crate) query: String,
    pub(crate) matches: Vec<TerminalMatch>,
    pub(crate) current: usize,
    pub(crate) needs_focus: bool,
    pub(crate) scroll_to: Option<usize>,
}
```

`⌘F` is consumed in the frame loop with `consume_key`, beside `⌘K`, so it is
gone before the pane reads its events. `Esc` closes when the bar is open, and
the palette's own `Esc` handling already runs first, so the two do not fight.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The two washes are `palette.search_match` and `palette.search_match_focus` if they exist; otherwise the existing selection and accent washes are reused at their documented opacities. No new role is introduced by this change. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | The bar uses `ICON_SEARCH`, which already exists, and `ICON_CLOSE`. The arrows are `ICON_PREVIOUS` / `ICON_NEXT` if present; otherwise they are added with their vocabulary entries. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | Nothing new is spelled twice. The wide-character ranges live in `terminal_display_columns` alone. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | Nothing is persisted. A search is a question being asked right now. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No child processes. The change is about **not** sending anything to one: while the field has focus, the pane does not read input. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Text already in memory. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Matching is bounded by the scrollback the pane already holds and runs only when the query or the buffer changes; `TERMINAL_SEARCH_MAX_MATCHES` bounds the list so a one-character query on a large buffer cannot build a vector per keystroke without limit. |

## Flagged concerns

- **A keystroke reaching the agent by accident.** Requirement 7 is the whole
  answer, and it falls out of focus rather than being enforced by a flag: the
  pane reads input only when it has focus, and a focused text field means it
  does not.
- **Column arithmetic in a proportional world.** It is exact only because the
  pane is monospace with a 1:2 CJK ratio, which the module already relies on for
  the cursor and the preedit. If that ever stops being true, three things break
  together and this is one of them.

## Acceptance

- `cargo test --locked` passes, including:
  - `counts_a_cjk_character_as_two_terminal_columns`
  - `finds_every_occurrence_including_several_on_one_line`
  - `matches_without_regard_to_case`
  - `reports_the_column_a_match_starts_at_in_display_cells`
  - `finds_nothing_for_an_empty_query`
  - `stops_collecting_matches_at_its_ceiling`
- In the running app: `⌘F`, type, step with Enter, and nothing appears in the
  agent's prompt; `Esc` returns focus to the pane.

## Rejected alternatives

- **Search from the ⌘K palette.** The other option on the agreed screen. The
  palette covers the pane, so you cannot see what you found, and stepping to the
  next occurrence means reopening it.
- **Regular expressions.** A bar that is one row, and a syntax error state that
  needs explaining. Substring matching is what the question "where did it say
  that" actually is.
- **Highlight the whole matching line.** Cheaper, and it answers "which line"
  rather than "where". The column arithmetic already exists for the cursor.
- **Search the raw capture including escape codes.** The codes are styling; a
  query would match invisible text and the match would be washed at a column
  nothing is drawn in.
