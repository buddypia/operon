# Intent: Home buries what waits under tall cards

- **Status**: approved
- **Opened**: 2026-09-27

## Problem

The owner's Before/After review (section ② ホーム) found Home still hard to
read after change 090 fixed its tiles:

- Each recent session is a card five or six lines tall. `tmux:`, `cwd:`, and
  `native resume:` are always shown, so six sessions fill the screen and the
  eye has to hunt for the one that matters.
- The 要対応 notice appears only when something waits. With nothing waiting,
  Home does not say so, and a person cannot tell "nothing waits" from "Home
  did not look".
- Starting a session in a particular project means leaving Home: the toolbar
  button picks a project for you, and choosing another means going to
  Projects first.

## Who feels it, and when

Every time the owner opens Home to see what is running and what waits — the
first screen of the app.

## Desired outcome

As in the approved `.tmp/ui-before-after/view-home-after.html` (② rows 2, 4, 5):

- The 要対応 line is always the first thing under the heading: "nothing waits
  ✓" when zero, and the waiting sessions with a way to open each when not.
- Recent sessions are one line each: state, name, agent, project, branch, age,
  and one action ("開く", or the resume/retry verb for a finished one). The
  internal details stay reachable behind "···".
- A "プロジェクトから始める" column lists each project with "＋ セッション",
  starting a session there in one click.
- Nothing that was reachable from a Home card becomes unreachable.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31. User-facing text is Japanese, through
  `tr`/`tf!` with a row in every table.
- No persisted-store change.

## Systems likely affected

`src/app/screens.rs` (Home), `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None. The owner answered "全項目 OK、進めて" to the Before/After and its
checklist on 2026-09-27, and "状況をみて自律で良い。進めて" after change 090.

## Not in scope

- ② row 3 ("＋ 新しいセッション opens the sheet in place"): the sheet is the
  next change; the toolbar button keeps opening today's launch form.
- Section ③ (launch sheet, project tabs, 概要) and ④ (settings).
