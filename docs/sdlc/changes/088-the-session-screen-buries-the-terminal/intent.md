# Intent: the session screen buries the terminal under repeated labels, empty panels, and a notice that never leaves

- **Status**: approved
- **Opened**: 2026-09-27

## Problem

The owner looked at the session screen with two Antigravity sessions open and
asked whether it was usable ("使いづらいよね？"). It is not, in ways visible in one
screenshot:

- A notice saying a file was opened ("ファイルを開く: 完了") spans the window and
  stays until it is closed by hand, pushing everything else down.
- The two sessions in the list read the same ("design-lookbook · Antigr…"), so
  they cannot be told apart, and each takes about 130px of mostly empty space.
- The four status filters take two rows, and each project repeats its full path
  on a line of its own.
- The session header says the project and agent in its title and again on the
  line under it.
- A red "停止" is the loudest control on the screen while the session is idle.
- Two side panels sit beside the terminal: a conversation history that is empty
  most of the time, and a file tree whose first four lines are a yellow warning
  and whose next ten rows are hidden folders.

## Who feels it, and when

Every time the owner switches to the session screen to watch or talk to an
agent — the screen they spend most of their time on.

## Desired outcome

Observable on the session screen, as in the approved "After" in
`.tmp/ui-before-after/index.html` (section ①):

- A successful result appears briefly in a corner and leaves on its own; a
  failure still stays until it is read.
- Each session in the list is one compact row pair that says what the session
  is about, so two sessions on the same project read differently.
- Status filters fit on one line, and a project is one line.
- The header is one line, and "停止" only draws attention while something is
  actually running.
- One side panel, switchable between files, conversation, and changes, can be
  folded away; jumping to the newest output is on the terminal itself.
- The file panel leads with the files the terminal has mentioned and a filter,
  folds hidden folders into one row, and states the scan limit in one line at
  the bottom.
- Nothing that was reachable before becomes unreachable: rename, mark unread,
  restore to another CLI, worktree, open in Terminal.app, copy log, the port list.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No change to the persisted store.

## Systems likely affected

`src/app.rs` (notice state), `src/app/screens.rs` (sidebar, header, terminal
workspace), `src/ui/session_tree.rs` (side columns), `src/i18n_tables.rs`,
`src/config.rs` for the toast duration.

## Open questions

None. The owner answered "全項目 OK、進めて" to the Before/After and its ten-item
checklist on 2026-09-27, and scoped this change to the session screen; the
toolbar "+", search, status vocabulary, launch sheet, home, and settings are
later changes.

## Not in scope

- Moving "+ 新しいセッション" and search into the toolbar, and renaming statuses
  (IDLE → 待機中) — change two of the approved sequence.
- The launch sheet, Home, Projects, Settings.
- Persisting the conversation turns; the row name falls back when they are gone.
