# Intent: the launch form is a long page behind a tab named 概要

- **Status**: approved
- **Opened**: 2026-09-27

## Problem

The owner's Before/After review (section ③ プロジェクト詳細 / 新しいセッション)
found:

- The project tab named 「概要」 is not an overview: it is the session launch
  form. What the project looks like — its sessions, uncommitted changes,
  worktrees — is nowhere on it.
- The launch form is tall: three numbered steps and three large agent cards,
  and the launch button (360 px wide) sits under all of it, sometimes below
  the fold. Starting a session from Home or the session screen means being
  taken to Projects first.
- Under the form, the same page lists the project's sessions, local CLI
  session discovery, and conversation restore — a second job on a page
  whose first job is starting one session.

## Who feels it, and when

Every time the owner starts a session (toolbar, ⌘N, Home, sidebar "+"), and
every time they open a project to see where it stands.

## Desired outcome

As in the approved `.tmp/ui-before-after/view-projects-after.html` (③ rows 1,
3, 4, 5):

- 「新しいセッション」 opens a sheet over whatever screen is showing. The sheet
  asks three things — agent (one row, a green dot when usable), request,
  where to work — with the rest folded under 「詳しい設定」. The launch button
  sits at the sheet's bottom right, and ⌘↩ presses it.
- 「概要」 summarises the project: how many sessions, uncommitted changes, and
  worktrees, and its sessions.
- Finding and restoring past CLI conversations moves to 「履歴」.
- Nothing that was reachable becomes unreachable, and no launch safeguard
  (danger acknowledgement, blocking reasons, account choice) is weakened.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31. User-facing text is Japanese, through
  `tr`/`tf!` with a row in every table.
- No persisted-store change. Launch-command construction and its validation
  gates are unchanged.

## Systems likely affected

`src/app/screens.rs` (launch form, 概要, 履歴), `src/app.rs` (sheet state,
entry points, key handling), `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None. The owner answered "全項目 OK、進めて" to the Before/After and its
checklist (including 「起動シートの項目」) on 2026-09-27, and "続けて" after
change 091.

## Not in scope

- ③ row 2, the project tabs (7 → 6 with count badges): the next change.
- A 「新しい worktree」 choice inside the sheet: the launch form never created
  worktrees; that stays on the worktree tab.
- Settings (④).
