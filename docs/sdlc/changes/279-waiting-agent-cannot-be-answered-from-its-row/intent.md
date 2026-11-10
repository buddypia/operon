# Intent: a waiting agent cannot be answered from its row

- **Status**: approved
- **Opened**: 2026-11-10

## Problem

The person's request, verbatim: 「AIを主に利用するIDEです。利便性やパフォーマンスなど多様な観点で改善を行ってください。」
Asked which candidates to carry out, they chose 「待機中の行からそのまま返信」 —
offered as a one-line reply on the 要対応 row, sent to the agent's terminal
without opening it, with no change to what is stored.

The 要対応 section lists every session that waits for the person, with what
its agent last said. Answering one means opening its terminal, which takes the
screen away from the session being read; with several agents asking short
questions (「続けて」, 「はい」, 「2」), each answer costs a change of screen
and a way back.

## Who feels it, and when

Someone running several agents at once, working in one session while others
stop to ask something a few words would answer.

## Desired outcome

- A waiting session whose terminal is running can be answered from its 要対応
  row: type, press Enter, and the words reach its agent as if typed in its
  terminal.
- The screen stays where it was.
- A row whose session has no running terminal offers no reply, as there is
  nothing to type into.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted shape changes: a reply being typed is not kept across a
  restart.
- Input reaches a terminal only through the existing queue
  (`queue_terminal_input`), which keeps order and reports a failed send.

## Systems likely affected

`src/app.rs` (state), `src/app/screens.rs` (the 要対応 section).

## Open questions

None.

## Not in scope

- Replying to several sessions at once (a separate change).
- A multi-line reply, attachments, or a history of replies.
- Choosing a permission prompt's option with buttons: the reply is the keys a
  person would type.
