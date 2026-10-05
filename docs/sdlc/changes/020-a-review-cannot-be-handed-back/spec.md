# Spec: notes on a diff, handed to the agent in one message

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Clicking a changed line in the diff opens a note against that line. The note
   records the project, the file, the line number on the new side, and a digest
   of what that line said when the note was written.
2. Notes are drawn under the line they belong to, inside the same scrolling
   column as the diff, so a note and its line are never separated by scrolling.
3. The pane keeps virtualising. Rows are no longer all one height, so the
   visible range is found by a binary search over cumulative offsets rather than
   by dividing by a row height. Nothing becomes proportional to the whole diff
   per frame that was not already.
4. Notes survive quitting. They are written to a sidecar file beside the store,
   never into the store, which `docs/sdlc/risk.yaml` holds at `paused`.
5. A note whose line no longer says what it said is marked, and says so in the
   text that is sent. It is not moved, and it is not deleted.
6. One action turns every note in the project into a single message and sends it
   to the agent session working in that project. The form is one block per note:
   the file, the line, and the comment, blank-line separated, with no preamble.
7. Nothing is sent to a session that is working. The agent is mid-turn, and
   typing into it is how a prompt gets cut in half. The person is told to wait.
8. Nothing is sent when there is no agent session in the project. The person is
   told that, rather than the notes being silently dropped.
9. A successful send clears the notes it sent. What was asked is in the terminal
   from that moment, and a second copy that has to be kept in step with it is a
   second copy that will not be.

## Behaviour

The screen was agreed before this was written:

```
┌─ 変更 ──────────────────────────────────────── 1180px ─┐
│ ▾ src/app.rs                                 +12 −3    │
│  120  118    fn create_worktree(&mut self) {           │
│  121  119 -    let destination = worktree_dest(…);     │
│  122  120 +    let base = detect_worktree_base(…);  💬 │
│ ┌── 💬 src/app.rs:120 ───────────────────────────────┐ │
│ │ base が None のときの分岐が抜けている               │ │
│ │                          [ 削除 ]  [ 保存 ]        │ │
│ └────────────────────────────────────────────────────┘ │
│  123  121    }                                         │
│ ▾ src/git.rs                                  +48 −0   │
├────────────────────────────────────────────────────────┤
│ 💬 3 件      [ すべて消す ]  [ セッションにまとめて送る ]│
└────────────────────────────────────────────────────────┘
```

The states that are not the happy one:

- **No notes.** The footer is not drawn. This is the ordinary state of the
  screen and it must cost nothing.
- **Nothing selected to send to.** 「このプロジェクトで動いているエージェントの
  セッションがありません。」
- **The session is working.** 「エージェントの応答が終わるまで待ってください。」
- **The line has changed since the note was written.** The note's header reads
  「行が変わっています」 in the muted ink, and the sent text carries
  `Note: the line has changed since this comment was written.`
- **The send fails.** The tmux error is surfaced and the notes are kept. A note
  is only cleared by a send that succeeded.
- **The sidecar cannot be written.** 「コメントを保存できませんでした: {error}」
  The notes stay in memory for the rest of the session.

## Design

`src/git/comments.rs`, a child of `src/git.rs` beside `src/git/setup.rs`, for
the same reason: a new top-level module is a line in the paused `src/main.rs`.

```rust
pub(crate) struct DiffComment {
    pub(crate) project: Uuid,
    pub(crate) file: String,
    pub(crate) line: usize,
    pub(crate) anchor: String,
    pub(crate) body: String,
}
pub(crate) struct DiffCommentStore { pub(crate) comments: Vec<DiffComment> }
pub(crate) fn diff_comments_path(data_file: &Path) -> PathBuf
pub(crate) fn load_diff_comments(data_file: &Path) -> DiffCommentStore
pub(crate) fn save_diff_comments(data_file: &Path, store: &DiffCommentStore) -> Result<()>
pub(crate) fn comment_anchor(line: &str) -> String
pub(crate) fn diff_comment_message(comments: &[DiffComment], stale: &[bool]) -> String
```

`diff_comment_message` has a fixed form, which is worth keeping exactly because it
was arrived at by use: the comments joined by a blank line, no preamble, and
each one

```
File: src/app.rs
Line: 120
User comment: "base が None のときの分岐が抜けている"
```

with `\`, `"`, carriage return and newline escaped inside the quotes, so a
multi-line comment cannot end the quoting early or be read as two notes.

`src/ui/diff.rs` changes shape. `DiffRow` gains `Comment(usize, usize)`, and
`diff_pane_rows` places one after each line that has a note. Because rows now
have two heights, `diff_row_offsets` builds the cumulative offset of every row
in the same pass, and the pane uses `show_viewport` with a `partition_point`
over those offsets instead of `show_rows`. Both functions are pure and both are
tested; the binary search is the part that would be wrong silently.

`diff_pane` gains one parameter, `&mut DiffAnnotations`, holding the project's
notes, which line is being edited, and the draft text. It reports back whether
the set changed, so `src/app.rs` knows when to write the sidecar.

`src/app.rs` gains the footer, `send_diff_comments`, and the choice of session:
the most recently launched `Active` agent session in the project. Not the
selected session — a person reviewing a diff is not necessarily looking at the
session that produced it — and not a terminal session, which has no agent to
read the message.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The note frame uses `palette.raised` and `palette.border_subtle`, its header `palette.text_muted`, exactly as the file bar above it does. No new role. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | One new mark for a note, added as `ICON_COMMENT` with its `ICON_VOCABULARY` entry, because it appears in two places — beside the line and in the footer — and a glyph literal at a call site is what `.claude/rules/palette-and-glyphs.md` forbids. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | The sidecar file name and the note-row height are `src/config.rs` constants. The message form exists only inside `diff_comment_message`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | No store shape changes. The sidecar is written with `write_file_atomically`, and a file that cannot be parsed yields no notes rather than a partial set that looks whole. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | The only child is the existing `tmux_send_input`, which is already bounded. The message is sent with `send-keys -l`, which is literal: nothing in a person's comment is interpreted as a key name. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet in each README. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | A local file and a tmux pane. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | `DIFF_COMMENT_BODY_MAX_CHARS` bounds one note and `DIFF_COMMENT_MESSAGE_MAX_BYTES` bounds the message; a set that would exceed it is refused with a count rather than truncated mid-note. |

## Flagged concerns

- **Typing into somebody's session.** Resolved by requirements 6 to 8: the send
  is an explicit action on notes the person wrote, it refuses while the agent is
  working, and it refuses when there is no agent rather than choosing a
  surprising target. Nothing is ever sent as a side effect of drawing.
- **Variable row heights in a virtualised list.** This is where a layout bug
  would hide. `diff_row_offsets` and the search over it are pure and tested at
  the boundaries — first row, last row, a viewport that begins inside a note.

## Acceptance

- `cargo test --locked` passes, including:
  - `writes_one_block_per_comment_in_the_agreed_form`
  - `escapes_a_comment_body_so_it_cannot_end_its_own_quoting`
  - `marks_a_comment_whose_line_has_changed`
  - `refuses_a_comment_message_past_its_byte_ceiling`
  - `keeps_diff_comments_across_a_reload`
  - `places_a_comment_row_under_the_line_it_belongs_to`
  - `offsets_every_diff_row_by_the_height_of_the_rows_above_it`
  - `finds_the_visible_rows_when_the_viewport_begins_inside_a_comment`
- In the running app: click a line, type, save; the note is under the line;
  quit and reopen and it is still there; 送る hands it to the agent and clears
  it; 送る with the agent mid-turn refuses.

## Rejected alternatives

- **A comments panel beside the diff.** It was the other option on the agreed
  screen. It costs the diff a third of its width, which is the width a diff
  actually needs, and it puts the note somewhere other than the thing it is
  about.
- **Keep the notes in memory.** A review is interrupted more often than it is
  finished in one sitting, and a note is the part of a review that took thought.
- **Put them in the store.** Paused surface, and this is a per-machine working
  set rather than something whose loss would be a loss of work — the sidecar is
  the same bargain the setup approvals and the cancellation intents make.
- **Send each note as it is written.** It is the interruption the batch exists
  to avoid, and it makes every note a separate turn.
- **Send to the selected session.** A person reading a diff is often looking at
  something else. The rule "the agent working in this project" is the one that
  is right without being looked at.
- **Re-anchor notes when lines move.** Checking an
  identity and marking what no longer matches is the alternative. Moving a note to a line that might
  not be the right one is worse than saying the line changed.
