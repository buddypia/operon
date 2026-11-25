# Intent: A listening server says nothing about who started it, when, or with what

- **Status**: approved
- **Opened**: 2026-11-25

## Problem

The person asked what 「ポート稼働中」 points at, was told it counts listeners
whose working directory is inside the session's folder, and then said:

「このプロジェクト内でAIが稼働したサーバーなのか？プロセスなのか？それを記録して、サーバーの終了忘れなどに対応したいけど、このままで対応できない気がする。詳細がほしんだよね。いつ起動して、プロセス内容とかパスとか。詳細がわからないとプロセスを終了することを躊躇うから正しくレビューができない。」

Today each row of the ports list shows a port and a process name (`:5173 node`),
and the stop confirmation adds a PID and a folder. Nothing says which agent
session started the server, when it started, what command it is, or whether the
thing that started it is still there. A `node` that has listened for a day and a
`node` an agent started four minutes ago look the same, so the person cannot
tell a forgotten server from one in use, and does not stop either.

Measured on the person's machine on 2026-11-25: several `node` listeners had run
for more than a day with their parent gone (parent PID 1); Operon's managed
sessions already carry their session name in their environment, and processes
started under them inherit it, though not every child does (MCP servers started
by Codex and Antigravity did not); and one listener's command line carried an
`Authorization: Bearer` header.

## Who feels it, and when

The person running several agent sessions that each start dev servers, who
opens the ports list to clean up — after a session is closed, or when the
machine has accumulated listeners — and has to decide, row by row, whether
stopping a process is safe.

## Desired outcome

Answers the person chose on 2026-11-25 (four questions, then the screen):

- Every row in the ports list, and the stop confirmation, shows which Operon
  session started the server (or that it is not known), when it started and how
  long it has run, its command line and working folder, and whether its parent
  is gone.
- A server whose starting session has been closed or removed, and still
  listens, is marked 「終了忘れ?」 in the list, and the session header's badge
  says how many such servers the list holds. No notification is raised.
- Nothing is recorded across scans: what is shown is what is running now.
- Secrets in a command line never reach the screen.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37; nothing slow runs in a draw path.
- Local-first: no telemetry, no accounts, no cloud calls, no new `unsafe`, no new
  dependency.
- User-facing text is Japanese; code, comments, and docs are English.
- Every child process goes through `src/exec.rs` with a timeout and an output
  ceiling.
- The store's shape does not change (the person chose not to record).

## Systems likely affected

`src/git/ports.rs` (the scan and its parsers), `src/app.rs` (scan inputs),
`src/app/screens.rs` (badge, list, stop confirmation), `src/config.rs`,
`src/i18n_tables.rs`, the three READMEs, `docs/features/worktrees/`, tests in
`src/tests.rs`.

## Open questions

None left: who started a server, what is shown, how a forgotten server is
marked, and whether anything is recorded were answered by the person; the
screen was chosen by the person from a drawn alternative.

## Not in scope

- A history of servers that have already stopped.
- Showing listeners whose working folder is outside every project, even when an
  Operon session started them.
- Notifications, automatic stopping, or stopping more than one server at once.
