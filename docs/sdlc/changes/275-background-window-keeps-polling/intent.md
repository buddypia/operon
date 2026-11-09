# Intent: Operon keeps reading every session at full rate while another app is in front

- **Status**: approved
- **Opened**: 2026-11-10

## Problem

The person's request, verbatim: 「AIを主に利用するIDEです。利便性やパフォーマンスなど多様な観点で改善を行ってください。」
Asked which of the candidates found in the code to carry out, they chose, among
others, 「非アクティブ時の監視を間引く」 — offered to them as: 「現在はウィンドウが背面でも2秒ごとに
全セッションへ tmux/ps を起動（20セッションで約40〜60プロセス）。フォーカスが外れている間は間隔を延ばし、
CPU・バッテリー消費を下げます。」

Every two seconds Operon starts child processes for every live session to read
its state and its screen — two to three per session. It does this at the same
rate whether the person is working in Operon or has left it behind another
application for an hour. With twenty sessions that is some forty to sixty
processes every two seconds, spent on a window nobody is looking at.

## Who feels it, and when

Someone who starts several agents in Operon and then works in an editor or a
browser while they run: the machine's fans and battery pay for a reading rate
only a person at the window needs.

## Desired outcome

- While another application is in front, Operon reads its sessions a third as
  often.
- The moment the person comes back to Operon, what they see is read again at
  once — not after the longer wait.
- An agent that reports its own state (the status hooks) is still noticed as
  promptly as before, so a notification that an agent needs the person is not
  held back by this.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- egui reports whether the window has the keyboard focus; it does not report a
  macOS window being minimized or covered (change 255 found the same), so
  "in front" can only mean "focused".
- A store write that is waiting to be retried keeps its present cadence.

## Systems likely affected

`src/app.rs` (the frame's two-second tick), `src/config.rs` (the intervals),
`src/util.rs` (the rule).

## Open questions

None. The background interval is a tuning value a later change can move.

## Not in scope

- Fewer processes per reading (one tmux call for all sessions).
- The port scan, the knowledge run, and the editor's Git refresh, which keep
  their own timers.
- A window that is visible but not focused: it cannot be told apart from a
  hidden one, so its terminal also updates at the slower rate.
