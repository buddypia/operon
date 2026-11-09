# Intent: a working agent does not say how long it has been working

- **Status**: approved
- **Opened**: 2026-11-10

## Problem

The person's request, verbatim: 「AIを主に利用するIDEです。利便性やパフォーマンスなど多様な観点で改善を行ってください。」
Asked which candidates to carry out, they chose 「作業中の経過時間を表示」 — offered
as: 「待機時間は出ますが作業時間は出ません。「作業中 12分」をタブ/行に表示し、エージェントが止まっている
のか遅いだけなのか判断できるようにします。保存データの変更なし。」

A session that waits for the person says how long it has waited. A session
whose agent is working says only that it is working. Two minutes into a turn
and forty minutes into a turn look the same, so a stuck agent and a busy one
cannot be told apart without opening each terminal and reading it.

## Who feels it, and when

Someone running several agents at once, looking down the session list to decide
which one to check on: the one that has been "working" since before lunch is
the one that needs a look, and nothing on the list says which that is.

## Desired outcome

- A session whose agent has been working for a minute or more says for how
  long, wherever its state is put into words: beside the word where one is
  printed, and in the hover where only a mark is.
- The time is the current turn's: when the agent stops and later starts again,
  the count starts again.
- Nothing is added for a session that is not working, or has only just begun.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted shape changes: the time is a reading of a live state, and a
  restart measures from when Operon next sees the agent working.
- Session tabs stay as terse as change 250 left them.

## Systems likely affected

`src/app.rs` (session state tracking), `src/app/screens.rs` (the session row).

## Open questions

None.

## Not in scope

- Warning about, or acting on, a long turn (a threshold, a notification).
- The time printed on a session tab or on a project column row, which changes
  250 and 253 left without a state word: their hover says it.
- How long a finished turn took.
