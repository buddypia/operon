# Intent: an agent cannot see, direct, or wait for the other agents of its project

- **Status**: approved
- **Opened**: 2026-11-17

## Problem

The person asked: 「HerdrとORCAの良いところをこのプロジェクトに取り込んで実装したい。できる限りこのプロジェクトに合わせて良い機能を実装して。」

Operon already has most of what both products are known for — per-session
state from the CLIs' own hooks, a 要対応 queue, parallel runs in worktrees,
diff notes sent back to an agent, setup scripts, port maps, checkpoints. What
it lacks is the idea at the centre of both: **the agents themselves can drive
the workspace.** Herdr gives agents a socket API to see every pane's state,
prompt another agent, and wait until another agent is idle or blocked; Orca's
CLI lets a coordinating agent hand work to worker agents and collect what they
did. In Operon only the person can look across sessions. An agent running in
one session cannot tell that its sibling in another worktree finished, cannot
read what that sibling last printed, and cannot hand it the next step — so the
person relays every message between two terminals by hand.

## Who feels it, and when

A person who runs two or three agents on one project — one implementing in a
worktree, one reviewing, one running tests — and asks the first to "have the
reviewer look when you are done". Today the first agent cannot, and the person
has to watch both, notice the finish, copy, switch, and paste.

## Desired outcome

From inside any session Operon manages, an agent can, with a command it runs in
its own terminal:

- list the sessions of its own project with each one's state (working, waiting
  on a prompt, idle, …);
- read the last lines on another session's screen;
- type an instruction into another session whose agent has finished its turn;
- wait until another session's agent has finished its turn or is waiting on a
  prompt, with a time limit.

Nothing reaches a session of another project, and nothing is typed into an
agent mid-turn.

## Constraints this change inherits

- macOS only; local-first: no network, no accounts, no telemetry.
- User-facing text is Japanese; code, comments, and docs are English.
- Every child process goes through `src/exec.rs`.
- The draw path does no file I/O.

## Systems likely affected

`src/main.rs` (one more subcommand beside `knowledge`), `src/tmux.rs` (the
concern that owns managed terminals), `src/app.rs` (the live state the app
already computes), `src/config.rs`, `src/i18n_tables.rs`, the manuals.

## Open questions

None left for the person: the choice of which Herdr/Orca idea to bring in was
delegated by the request above (「できる限りこのプロジェクトに合わせて良い機能を」).

## Not in scope

- Starting new sessions from the command (Orca's worker spawn): it needs the
  app to create a record, a worktree, and a launch, and is its own change.
- Remote machines over SSH, plugins, a mobile app, scheduled automations.
- Any change to what the person sees in the window.
