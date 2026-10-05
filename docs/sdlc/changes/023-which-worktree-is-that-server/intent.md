# Intent: three dev servers are running and nobody knows which is which

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

Running the same task in several worktrees at once is what Operon is for, and it
is the thing that makes a machine full of dev servers. Three worktrees, three
`pnpm dev`, three ports — 5173, 5174, 5175, handed out in whatever order the
processes happened to start. To see the one you want you open a browser, guess,
and read the page to find out whose it was. Guess wrong twice and you have
reviewed the wrong branch.

The information exists on the machine: every listening socket has a process, and
every process has a working directory, and that directory is inside exactly one
worktree. Nothing joins the three together.

## Who feels it, and when

Anybody running more than one worktree of a project that serves anything. It
starts the moment the second worktree comes up, which is the moment Operon
starts being worth using.

## Desired outcome

Looking at a session tells you what that session's worktree is listening on, and
a click opens it. A worktree with nothing listening says nothing, and a machine
where the ports cannot be read says nothing rather than something wrong.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- Every child process goes through the wrappers in `src/exec.rs` with a timeout
  and an output ceiling.
- Nothing may become work proportional to the machine's process table on a
  per-frame draw path.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

A new place for reading listening sockets and attributing them. The session's
terminal header, in `src/app/screens.rs`. The background scanner, in
`src/app.rs`.

## Open questions

- **Where does it go on screen?** Answered before the code: one line in the
  session's terminal header, showing only that session's worktree.

## Not in scope

- Killing a process from Operon. It is a different kind of
  action from "show me what is running" and deserves its own confirmation.
- The advertised-URL watcher that reads a framework's own printed address out of
  the scrollback. It is a good idea and it is a second change.
- Linux and Windows readers. Operon is macOS only.
- Container and external-process grouping. One session's worktree is the whole
  question here.
