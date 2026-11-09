# Intent: a relaunch does not bring back the sessions the person had, and a terminal running Claude Code reads as a terminal

- **Status**: draft
- **Opened**: 2026-11-09

## Problem

The person asked, in one message: 「Claude Codeだけど、なんでターミナルとして認識されている？そしてアプリを終了してから起動すると全ての覚えておいたセッションで起動おくメカニズムにして。」

Two things are wrong as they see it.

1. A session they opened as a plain terminal and then started Claude Code in
   (「セッション3」 in their store: kind `terminal`, a Claude conversation
   already recorded against it by Claude's own status hook) keeps the terminal
   mark on its tab, the terminal label in the left column, and is not read as
   an agent: its screen is never watched for "working / needs you / done", it is
   never offered review notes, and nothing could reopen its conversation. The
   app decides what a session is once, when it is created, and never looks again.
2. Quitting and reopening Operon does not put them back where they were. The CLIs
   keep running in tmux across a quit, but the workspace comes back as "the live
   sessions of one project, each in its own tab": the other projects' tabs, the
   split panes, which tab was in front, and which tabs they had closed are all
   forgotten. When the terminals themselves are gone — the Mac restarted, the tmux
   server died — every session that was running comes back stopped, and each has
   to be reopened by hand, one 「再開」 at a time.

## Who feels it, and when

- Every time someone opens a terminal from the tab strip's 「+」 and types
  `claude`, `codex`, or `agy` into it — the fastest way to start an agent — the
  session looks and behaves like a shell for the rest of its life.
- Every quit and relaunch (an update swaps the installed app, a crash, Cmd-Q at
  the end of the day), and every Mac restart with several agents open across
  projects.

## Desired outcome

The person's answers to the two questions put to them (2026-11-09) decide the
shape:

- **Kind**: 「その CLI のセッションに切替」 — once a CLI reports a conversation
  from a terminal, that session *is* a session of that CLI: its tab mark and
  label say so, its state is read like any agent's, and reopening it reopens the
  conversation.
- **Relaunch**: 「配置の復元＋自動再開」 — after quitting and reopening, every
  project's tabs, splits, and front tab are as they were, closed tabs stay
  closed, and any session that was running at the quit but whose terminal is now
  gone is started again by itself in the same tab: a CLI session reopens its
  saved conversation, a plain terminal opens a fresh shell in its folder. A
  session that ended on its own (the person typed `/exit`) is not started again.

Observable: open a terminal, run `claude`, send one prompt — the tab shows the
Claude mark. Arrange two projects with splits, quit, reopen — the same tabs are
on screen. Kill the tmux server, reopen — the same tabs come back with their CLIs
running their previous conversations, without any click.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI; no I/O in the draw path.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface; the arrangement is a per-machine
  working preference and lives beside it, as the session column's order does
  (change 181).
- Launch commands still pass `is_safe_agent_command` and the existing resume
  validation; nothing new reaches a shell.

## Systems likely affected

The hook-event intake and session lifecycle in `src/app.rs`, the workspace model
in `src/ui/session_tree.rs`, a file-name constant in `src/config.rs`, strings in
`src/i18n_tables.rs`, and tests in `src/tests.rs`.

## Open questions

None left: both were put to the person and answered (see Desired outcome).

## Not in scope

- Document (file) tabs: they are not reopened at launch; only session tabs are.
- Starting again a session that was already stopped before the quit, one that
  exited by itself or failed, or a queued one.
- A CLI session with no saved conversation: it keeps its manual 「再試行」.
- Changing a session that was launched as one CLI into another when a different
  CLI is typed inside it.
- The manual 「再開」 button keeps creating a new record, as today.
