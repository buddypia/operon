# Intent: a session row on Home spells out the state its mark already shows

- **Status**: approved
- **Opened**: 2026-11-11

## Problem

The person's request, verbatim:
「ホームの最近セッションのアイドルなどステータスの文字の表現は不要とか不要。カーソルをステータスアイコンに当てるとステータスの文字が出るだけでOK。」

Each row under 最近のセッション on Home begins with a state glyph followed by
the state's word — アイドル, 実行中, 要対応, 終了 — in the state's colour. The
tabs and the left column already tell the state by a mark alone and name it on
hover (changes 283 and 284); the Home rows still spend a column on the word,
on every row, for something the mark says.

## Who feels it, and when

Anyone opening Home to pick a recent session: the word repeats down the list
and pushes the title, which is what the list is read for, to the right.

## Desired outcome

- A session row on Home shows its state as a mark only; no state word is
  drawn in the row.
- Resting the pointer on that mark names the state in words.
- Clicking anywhere on the row still opens the session.

## Constraints this change inherits

- `eframe`/`egui` 0.31 immediate-mode GUI on macOS; nothing new per frame
  beyond painting.
- Colours come from `Palette` roles; no new role, glyph, or string.

## Systems likely affected

The session row drawn on Home.

## Open questions

None.

## Not in scope

- The state filter tiles and counts, which are labels for a group, not for a
  session.
- The status chip in the session detail header and the sessions list, which
  are not Home.
