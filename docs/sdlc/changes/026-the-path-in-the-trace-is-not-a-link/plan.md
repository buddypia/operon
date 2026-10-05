# Plan: follow a path or an address out of the output

- **Spec**: `./spec.md`
- **Approved**: 2026-09-06
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `TERMINAL_PATH_MAX_EXTENSION`, `TERMINAL_PATH_PROBE_LIMIT`. |
| `src/ui/terminal.rs` | The target types, the detector, the resolver, the overlay, and the per-row drawing; `terminal_rows` returns what is under the pointer. |
| `src/app.rs` | `resolved_paths`, `unresolved_paths`, `editor_jump_line`, `terminal_path_menu`; `resolve_terminal_paths`; `open_terminal_target`; the right-button menu. |
| `src/app/screens.rs` | The overlay at the pane's draw site, the click handling, and the editor's scroll offset. |
| `src/i18n_tables.rs` | EN and KO rows. |
| `src/tests.rs` | The five tests named in the spec's Acceptance. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. `src/config.rs`, then the detector and the resolver with their tests. The
   detector first and the drawing later, because a detector is where the
   mistakes are and it is the part a test can reach.
2. The overlay and the per-row drawing.
3. `src/app.rs` resolution, click handling, menu; the editor's jump.
4. `src/i18n_tables.rs` and the three READMEs.
5. Gates, bands, a real stack trace in the app, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A false positive underlines a word | The underline stops meaning anything | `does_not_call_ordinary_words_paths`, over six lines of real output, and only resolved paths are drawn at all. |
| A click reads a file outside the project | A trace becomes a way of reading somebody's home directory | `refuses_a_path_that_escapes_the_session_folder`, including a relative escape and an absolute path elsewhere. |
| A filesystem call lands in a draw path | Stutter proportional to the rows on screen | The pane records names; the frame loop resolves them. |
| The click is taken from the pane | An agent loses the keyboard mid-sentence | A plain click while focused goes to the pane; `⌥` opens. |
| Reading every line for targets | Work proportional to the scrollback | Only the rows `show_rows` handed over. |

## Proof of completion

- `cargo fmt --check`, `cargo test --locked` five higher with `6 ignored`,
  `cargo clippy --locked -- -D warnings`, `bash scripts/check-bands.sh`.
- Each guard watched failing by mutation.
- In the running app: hover a panic's path, click, land on the line.

## Departures from the plan

- **The detector was wrong three ways and the tests found all three before any
  of it was drawn.** A panic's trailing colon rode along in the token, so the
  underline was one cell too wide; a URL inside brackets was not found at all,
  because the character before it was required to be whitespace rather than
  merely not a path character; and the `1.2.3` in "version 1.2.3 released" was
  read as a file, because an extension of digits passed a check that only asked
  for alphanumerics. Writing the tests from real output lines rather than from
  the detector is what made all three visible in one run.
- **`harness_documents_only_name_paths_that_exist` failed on the doc comments.**
  They quoted a location as an example, and a backticked span that is a path
  plus a line number is not a path that exists. The comments now name the shape
  rather than an instance — lesson 003's rule, met from the other side.
- **`TerminalClick` was written and never used.** The pane returns the target
  under the pointer and the caller decides; an enum of resolved actions in
  between was a layer that carried nothing. Removed rather than left for
  somebody to wire up later.
