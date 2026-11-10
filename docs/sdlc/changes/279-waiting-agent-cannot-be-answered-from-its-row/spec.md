# Spec: answer a waiting agent from its 要対応 row

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Each 要対応 row whose session's terminal is running draws a one-line reply
   field, with the hint 「返信を入力して Enter」; a row whose session is not
   running draws none. — `reply_279_running_rows_offer_a_reply_field`
2. Sending a reply queues its text, trimmed, followed by Enter, for that
   session's terminal through `queue_terminal_input`, says 「返信を送りました」,
   and leaves the selected session and the page as they were. —
   `reply_279_reply_goes_to_the_sessions_terminal`
3. A reply that is empty or only spaces, or one for a session whose terminal
   is not running, sends nothing and says nothing: 「返信を送りました」 is said
   only for a reply that was queued. — `reply_279_empty_reply_sends_nothing`
4. Typing in the field and pressing Enter sends what was typed and empties the
   field; clicking the field does not open the session. —
   `reply_279_typing_then_enter_sends_and_clears`
5. What was typed for a session is forgotten once the session no longer waits.
   — `reply_279_draft_goes_when_the_session_stops_waiting`, and when it was
   the last one waiting — `reply_279_f1_draft_goes_when_the_whole_queue_empties`
6. The Enter that confirms an IME conversion does not send: one Enter belongs
   to each conversion committed in the frame, as in the terminal pane, the
   field keeps the keyboard, and the next Enter sends. —
   `reply_279_enter_that_confirms_a_conversion_does_not_send`
7. Keys typed in the field reach only the row's session, not the terminal
   pane on screen, even when that pane had the keyboard just before. —
   `reply_279_keys_typed_in_the_field_stay_out_of_the_pane`

## Behaviour

The field sits at the foot of the row, under the line that says what the
agent last said. Its text is sent as typed characters and then the Enter key,
exactly as from the terminal pane, so 「はい」 answers a question and 「2」
picks the second option of a prompt that asks for a number. After a send the
agent begins to work and the row leaves 要対応 on its own, as it does when the
answer is typed in the terminal.

- **Empty** (nobody waits): no section, as today.
- **Not running** (the terminal ended; the row waits for a restart): no field.
- **Error**: a send tmux refuses is reported by the existing notice
  「ターミナルに入力を送れませんでした: …」.
- **Too large**: one line; a pasted line break ends the line as egui's
  single-line field does. No ceiling of its own beyond tmux's argument.
- **Interrupted**: the session stops waiting while a reply is half typed — the
  field goes with the row and the words are dropped (requirement 5); a restart
  of Operon drops them too.
- **Keyboard**: the field takes the keys while it has the focus; the terminal
  pane does not see them. Escape or a click elsewhere leaves the field with
  its text kept for as long as the session waits. After a send the field does
  not take the keyboard back: the row is about to leave the section. A row a
  filter hides keeps its text, since its session still waits.
- **Where it shows**: wherever the 要対応 section is drawn — the per-project
  session list in 履歴 and in the session panel docked beside the terminal,
  which is where a reply is sent without the terminal on screen changing.
- **Trimmed**: spaces around the reply are dropped, so Enter alone cannot be
  sent from the field; a prompt that wants a bare Enter is answered in its
  terminal.

New strings, in every table of `src/i18n_tables.rs`: 「返信を入力して Enter」,
「返信を送りました」.

## Design

Placement: **EXTEND** of the 要対応 section (`git grep -n ui_pinned_sessions`:
`src/app/screens.rs`), written as **NEW_IN_MODULE** code under `src/app/`:
`src/app/screens.rs` stands at 9 992 lines against the 10 000-line tier, so
`ui_pinned_sessions` moves, unchanged in what it draws, to a new file
`src/app/waiting_reply.rs`, and the field is added there.

- `src/app/waiting_reply.rs`: `impl OperonApp { ui_pinned_sessions(ui,
  sessions) -> usize` (moved), `send_waiting_reply(session_id, text: &str) ->
  bool }`. The row loop takes `waiting_reply_drafts` out of `self` for the
  pass (the shape of `track_attention`), draws
  `egui::TextEdit::singleline` with `.font(FontId::proportional(12.0))` for a
  session whose status is `SessionStatus::Active`, and notes the session
  whose field lost the focus with an Enter that `reply_enter_sends(events)`
  says is the person's — Enter presses in the frame outnumber its non-empty
  `ImeEvent::Commit`s; otherwise the field asks for the focus back. After the
  loop it takes the sent draft out, calls `send_waiting_reply`, and puts the
  map back. The moved code keeps its existing per-row work (`title`, `meta`)
  as it was.
- `src/app.rs`: field `waiting_reply_drafts: HashMap<Uuid, String>` (not
  persisted); `mod waiting_reply`; `track_attention`, when the queue changes,
  drops the drafts of sessions no longer in `attention_since` — there and not
  in the section, which is not drawn once nobody waits.
- `src/app/screens.rs`: loses `ui_pinned_sessions`.
- `src/i18n_tables.rs`: the two strings in the English and Korean tables.
- `CLAUDE.md`: the module map names the new file.
- `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`,
  `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md`: one or two sentences each where
  the 要対応 section is described.
- No persisted shape. No new external command: the send is
  `queue_terminal_input`'s existing `tmux send-keys` on its background task
  (5 s timeout in `tmux_send_input`). The draw path gains, per running
  waiting row, one map entry lookup and one text field.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | The field is egui's text field in the theme's existing visuals; no colour named. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No identifier added. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | Nothing is stored; `src/models.rs` and `src/store.rs` are untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No new child: the existing input queue sends, and the text is passed to `send-keys -l` as one argument, never through a shell. |
| Documentation — user-facing docs change in all three languages together | Yes | The README and the manual, in English, Japanese and Korean, each gain the sentences. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path. |

## Flagged concerns

- **A reply sent to a prompt it does not fit** — the person sees the agent's
  last words on the row, not its whole screen, and a permission prompt may
  want a key rather than a sentence. Resolved: accepted; the reply is plain
  typing, the row already says 「許可または回答を待っています」 for that case,
  and one click on the row opens the terminal when the question needs
  reading.
- **A reply tmux reads as its own syntax** — the verifier showed that
  `tmux_send_input` loses a trailing 「;」 and fails on text that begins with
  「-」. That is how every typed or pasted text has reached a terminal, not
  something this change adds, and `src/tmux.rs` is a `subprocess` surface.
  Resolved: fixed where it lives, in change 280, which lands next; until then
  such a reply shows the existing 「ターミナルに入力を送れませんでした」 notice or
  arrives without its last 「;」.
- **A reply typed while the agent starts working** — the queue sends it
  anyway, as typing in the pane would. Resolved: accepted; it is what the
  terminal does.

## Acceptance

- `cargo test --locked reply_279_` passes (eight tests).
- `make q.fast` passes; the guards for the module map, the message tables and
  the three-language documents pass.
- `python3 docs/sdlc/changes/279-waiting-agent-cannot-be-answered-from-its-row/mutations.py`
  catches every mutation.
- In the running app: an agent that asks a question shows the field on its
  要対応 row; 「続けて」 and Enter there set it working without the screen
  changing.

## Rejected alternatives

- A reply button that opens a sheet — a second step for a few words.
- A field shown only on hover — the pointer would have to stay on the row
  while typing.
- Buttons for a prompt's options — they would have to be read off another
  program's screen.
