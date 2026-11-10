# Intent: idle agents cannot be told something at once

- **Status**: approved
- **Opened**: 2026-11-10

## Problem

The person's request, verbatim: 「AIを主に利用するIDEです。利便性やパフォーマンスなど多様な観点で改善を行ってください。」
Asked which candidates to carry out, they chose 「アイドル中の全エージェントへ
一括送信」. Shown two shapes of it, they chose 「コマンドパレットから開くシート」:
the palette row opens a small sheet, one line is typed and sent, and it
reaches the current project's sessions that are running and idle.

With several agents finished and idle, giving each the same instruction
(「作業をコミットして」, 「main を取り込んで」) means opening each terminal in
turn and typing it again.

## Who feels it, and when

Someone running several agents on one project, at the points where all of
them should do the same thing next.

## Desired outcome

- One instruction, typed once, reaches every idle agent of the project.
- Before sending, the person sees which sessions it will reach.
- An agent that is working, or waiting on a prompt, is not typed into.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted shape changes.
- Input reaches a terminal only through `queue_terminal_input`.

## Systems likely affected

`src/app.rs` (state, the frame), `src/app/screens.rs` (the palette).

## Open questions

None.

## Not in scope

- Sending across projects (the shape the person did not choose).
- Choosing recipients one by one, a multi-line instruction, or a history.
- A keyboard shortcut of its own.
