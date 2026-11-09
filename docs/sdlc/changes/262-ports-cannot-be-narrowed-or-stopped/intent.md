# Intent: A session's ports cannot be narrowed to its worktree, told apart from the rest of the project, or stopped

- **Status**: approved
- **Opened**: 2026-11-09

## Problem

The person asked what 「{n} ポート稼働中」 counts, then:
「本当にセッション内のwortreeのみ表示している？それともプロジェクトで内で起動したプロセス？不要なら停止することも可能？」
and, told it is neither exactly and that nothing can be stopped:
「進めて。できればworktreeだけにするか、プロジェクト全体の形で分けて表示ができると良い」

Today the count in a session's header is every listening TCP port whose process
works under the deepest folder any session is working in. A session working at
the project's root — every restored or imported session is one — therefore
counts a dev server running in one of the project's own worktrees whenever no
session happens to be open on that worktree, and the person cannot tell which
servers are this worktree's and which belong elsewhere in the project. A server
that is no longer wanted can be opened or have its URL copied, but stopping it
means finding its PID in another terminal.

## Who feels it, and when

The person running the same task in several worktrees of one project, each with
its own dev server, who looks at a session's header to see what that worktree
is serving — and finds servers from other worktrees in the count, or wants to
shut down a server left running in a worktree they finished with.

## Desired outcome

- A session's header counts only the servers working in that session's own
  folder — never a server in another worktree of the project, whether or not a
  session is open on that worktree.
- The list behind it shows, separately, this folder's servers and the other
  servers running elsewhere in the same project, each of the latter named by
  the folder it runs in. A session whose own folder serves nothing but whose
  project does still offers that list.
- From either part of the list the person can stop a server: they are shown
  which process, PID and folder will be stopped and confirm; afterwards the
  server is gone from the list, or they are told it is still running, that it
  had already changed, or why it could not be stopped.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI; nothing slow runs in a
  draw path.
- Local-first: no telemetry, no accounts, no cloud calls, no new `unsafe`.
- User-facing text is Japanese; code, comments, and docs are English.
- Every child process goes through `src/exec.rs` with a timeout and an output
  ceiling.
- Stopping a process is destructive: it is never done without the person's
  confirmation, and never to a process other than the one they confirmed.

## Systems likely affected

`src/git/ports.rs` (attribution, stopping), `src/app.rs` (scan request, the
header badge and its menu, a confirmation window, background result),
`src/config.rs` (constants), `src/i18n_tables.rs`, the three READMEs, tests in
`src/tests.rs`.

## Open questions

None left for the person. Recommended defaults adopted and recorded in
`state.yaml`: the split is two labelled sections of the one list rather than a
persisted toggle; stopping sends SIGTERM only.

## Not in scope

- Forcing a process that ignores SIGTERM (SIGKILL), or stopping a whole process
  group.
- Listing UDP sockets, or listeners whose working directory is outside every
  known project.
- A project-level ports view outside the session header, or the split-pane
  header, which shows no ports today.
