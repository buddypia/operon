# Plan: find something in a session's scrollback

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `TERMINAL_SEARCH_MAX_MATCHES`. |
| `src/glyphs.rs` | `ICON_PREVIOUS`, `ICON_NEXT`, and their vocabulary entries. |
| `src/ui/terminal.rs` | `TerminalMatch`; `terminal_display_columns`; `terminal_search_matches`; `TerminalSearchOverlay`; `terminal_rows` paints the washes and honours a scroll request. |
| `src/app.rs` | `TerminalSearch`; the per-session map; the key handling; recompute on new output. |
| `src/app/screens.rs` | The bar under the pane, and the overlay passed into it. |
| `src/i18n_tables.rs` | EN and KO rows. |
| `src/tests.rs` | The six tests named in the spec's Acceptance. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. `src/config.rs`, `src/glyphs.rs`, and the pure half of `src/ui/terminal.rs`
   with its tests: columns first, then matching, because the second is measured
   in the first.
2. The overlay and the painting inside `show_rows`.
3. `src/app.rs` state, keys, and the recompute; `src/app/screens.rs` bar.
4. `src/i18n_tables.rs` and the three READMEs.
5. Gates, bands, a real search in the app, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A keystroke reaches the agent | Characters appear in a running prompt | It falls out of focus rather than a flag: the pane reads input only when focused, and a focused field means it is not. `⌘F` and `Esc` are consumed before the pane sees the event. |
| The wash is drawn beside the text rather than on it | Looks broken on any Japanese line | `terminal_display_columns`, with the CJK test, and a match test that measures a column after a Japanese prefix. |
| Matching every frame | A stutter proportional to fifty thousand lines | Recomputed only when the query changes or `set_session_output` replaces the buffer. Painting touches only the rows `show_rows` handed over. |
| Enter commits an IME composition and steps at once | The search jumps while a word is being typed | The preedit is checked before Enter is consumed — change 014's rule. |
| A scroll request fights the person's own scrolling | The pane snaps back every frame | The request is cleared by the frame that used it. |

## Proof of completion

- `cargo fmt --check`, `cargo test --locked` six higher with `6 ignored`,
  `cargo clippy --locked -- -D warnings`, `bash scripts/check-bands.sh`.
- Each guard watched failing by mutation.
- In the running app: `⌘F`, type, step, and nothing appears in the prompt.

## Departures from the plan

- **`stick_to_bottom` is switched off while a jump is pending.** The pane
  follows new output by default, which would drag the view back down the moment
  it arrived at a match.
- **The current match is kept by index, not by line.** Output arriving under an
  open search moves every line number; the third match is still the third match.
- **Folding is ASCII-only, deliberately.** A Unicode fold can turn one character
  into several, and then the wash is drawn a cell or two from the thing it is
  highlighting. Terminal output is searched for ASCII words; a Japanese query
  needs no folding at all.
