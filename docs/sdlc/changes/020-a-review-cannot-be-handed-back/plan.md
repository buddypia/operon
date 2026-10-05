# Plan: notes on a diff, handed to the agent in one message

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | The sidecar file name, the note row height, the body and message ceilings. |
| `src/glyphs.rs` | `ICON_COMMENT` and its `ICON_VOCABULARY` entry. |
| `src/git/comments.rs` (new) | The note, the sidecar, the anchor digest, and the message form. |
| `src/git.rs` | `pub(crate) mod comments;` and its re-export. |
| `src/ui/diff.rs` | `DiffRow::Comment`; `diff_row_offsets`; `show_viewport` with a binary search; drawing and editing a note. |
| `src/app.rs` | `diff_comments`, the footer, `send_diff_comments`, and choosing the session. |
| `src/i18n_tables.rs` | EN and KO rows for every new message id. |
| `src/tests.rs` | The eight tests named in the spec's Acceptance. |
| `CLAUDE.md` | The module map gains the child file. |
| `README.md`, `README.ja.md`, `README.ko.md` | One bullet each. |

## Order of work

1. `src/config.rs` and `src/glyphs.rs`. Compiles; green.
2. `src/git/comments.rs`: the type, the anchor, the message form, the sidecar,
   with their tests. Compiles; green.
3. `src/ui/diff.rs`: `diff_row_offsets` and the row list, tested pure, before
   anything is drawn with them.
4. `src/ui/diff.rs`: the viewport search and the note row's drawing and editing.
5. `src/app.rs`: the footer, the session choice, the send. `src/i18n_tables.rs`.
6. `CLAUDE.md` and the three READMEs.
7. Gates, bands, a real review in the app, commit.

The tree compiles between every step.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The viewport search is off by one | A note vanishes while scrolling through it, or the pane jumps | `offsets_every_diff_row_by_the_height_of_the_rows_above_it` and `finds_the_visible_rows_when_the_viewport_begins_inside_a_comment`, both pure. This is the bug lesson 005 is about: only the caller proves the screen, so the search is separated from the drawing and pinned. |
| A comment body ends its own quoting | The agent reads one note as two, or as broken JSON-ish text | The escaping test, with a body containing a quote, a backslash, and a newline. |
| A note is sent into a working agent and truncates its turn | A cut-off prompt, and a confused agent | The send refuses unless the target is idle. |
| A note is silently lost when the sidecar cannot be written | The person believes it is saved | The write failure is surfaced, and the note stays in memory for the session. |
| Per-frame cost grows with the diff | A large diff drops frames | The offsets are built in the same pass that already builds the row list, so the order does not change. Stated, and the row list was already per-frame. |
| `src/app.rs` grows again | `scripts/check-bands.sh` warns harder | The note model and the message form are in the child module and the drawing is in `src/ui/diff.rs`. `src/app.rs` gains the footer and the send. |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `ok`, eight higher than before, `6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no new breach beyond what is recorded.
- Each new guard watched failing by mutation: the escaping removed; the stale
  mark suppressed; the offsets made uniform; the binary search replaced by a
  division; the ceiling removed.
- In the running app: a note written, surviving a restart, delivered, and
  refused while the agent is working.

## Departures from the plan

- **`diff_pane_rows` takes a predicate rather than the notes.** It only ever
  asks one question of them — does this line have a note under it — and taking
  the whole set would have made a pure row-layout function depend on the note
  model.
- **The change is reported by a flag on a disjoint field, not a return value the
  caller acts on.** The pane runs while `self.diff_file_cache` is borrowed, so
  it cannot reach the whole application to write the file. `diff_comments_dirty`
  is set from inside that borrow and drained at the top of the next frame, which
  is also where every other per-frame drain already happens.
- **Writing an empty note deletes it.** One gesture for "never mind" rather than
  two. 削除 stays, because a note being deleted deliberately should not require
  selecting its text first.
- **The notes are all in one list, with the project part of every lookup.** The
  alternative was a per-project view kept beside the file's contents, which is
  two copies of the same thing that have to be kept in step.
- **`largest_module_lines` rose to 9269 from 9078.** Third change in a row to
  record it. `src/app.rs` is 469 lines past the warn band; splitting it should
  now come before the next feature.
- **A pane shipped without its footer, and now a guard pairs them.** The review
  pane is drawn twice — the Git tab and the editor's diff view — and only the
  first got the footer. A note written in the editor was sendable only by going
  to the Git tab and finding a count there, which is a note somebody writes and
  then cannot find. `every_pane_that_takes_a_note_offers_a_way_to_send_it`
  counts `diff_pane(` call sites and `ui_diff_comment_footer(` call sites across
  `src/` and asserts they are equal, so a third pane cannot ship half-wired
  either. Watched failing against the shipped state.
