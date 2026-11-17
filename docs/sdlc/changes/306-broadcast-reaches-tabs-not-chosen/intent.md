# Intent: an instruction to idle agents reaches tabs nobody chose

- **Status**: approved
- **Opened**: 2026-11-17

## Problem

The person's request, verbatim: 「エージェントに送る機能があるけど、これ勝手に一番最初になったりすると困るからタブを洗濯できるようにするとか工夫が必要」
(洗濯 is a typo for 選択: make the tabs selectable).

The sheet 「アイドル中のエージェントへ送信」 (change 282) types its line into
every agent of the project that has finished its turn, and nobody chooses
which. An agent that has only just started also counts as finished — it sits
at its prompt — so a tab opened a moment ago receives the line as its very
first instruction. The list is also read again while the sheet is open, so a
tab that falls idle between opening the sheet and pressing Enter is typed into
without having been shown when the line was written.

Asked which tabs the sheet should have chosen when it opens — none, every idle
one, or only the one in view — the person chose 「何も選ばない」: every tab
starts unchosen, and a 「すべて選択」 control chooses them all at once.

## Who feels it, and when

Someone running several agents on one project who wants to tell some of them
the same thing — and has, among the idle ones, a tab just opened for other
work, or one they mean to leave alone.

## Desired outcome

- The line reaches exactly the tabs the person ticked, and no other.
- A tab nobody ticked — one just opened, or one that fell idle after the sheet
  opened — receives nothing.
- Telling every idle agent stays one click away.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No persisted shape changes: the choice lives only while the sheet is open.

## Systems likely affected

`src/app/broadcast.rs`, `src/i18n_tables.rs`, the README and the manual.

## Open questions

None. The default choice was the person's.

## Not in scope

- Choosing recipients by selecting several tabs in the tab strip.
- Sending to agents that are working or waiting on a prompt.
- Sending across projects, a multi-line instruction, or a history.
- Remembering the choice after the sheet closes.
