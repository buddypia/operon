# Intent: a machine can only be logged in to one account per CLI

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

Codex and Claude Code each keep their credentials, settings, and history in one
directory under the home folder. Whoever is logged in there is who every session
runs as. A person with a work subscription and a personal one has to log out and
back in to move between them, which takes the conversation history and the
per-account usage window with it, and which nothing in this app can see or help
with.

Both CLIs already have the way out: each reads an environment variable that says
which directory to use, and a launch that sets it runs as a different account
without touching the other one. The app launches every terminal itself and
already passes environment through to it, so the mechanism is one variable away.

## Who feels it, and when

Anybody with more than one subscription for the same CLI — a work account and a
personal one is the ordinary case — every time they change which one they are
working under.

## Desired outcome

A person registers each account once, picks one when starting a session, and the
session runs as that account. Sessions already running keep the account they
started with. Reopening a conversation reopens it under the account that had it.

And the thing that must not quietly break: an account that is not the default
still reports its agent state, still marks its finished turns unread, and still
shows its usage. Those three all come from managed hooks installed into the
CLI's own configuration directory, and a session pointed at a different
directory would find none of them there.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface. Accounts, and which session used
  which, live in a file beside it rather than in it.
- Credentials are never read, copied, or moved by this app. It names a
  directory; the CLI does the rest. Nothing about an account is ever sent
  anywhere.
- No new dependency.

## Systems likely affected

The launch path in `src/agents.rs` and `src/tmux.rs`, the hook installer in
`src/tmux/hooks.rs`, and the launch screen.

## Open questions

- Does each CLI actually honour its variable? — answered by running them.
  Recorded in `spec.md`.
- Do the managed hooks follow the account, or does a non-default account lose
  agent state? — answered by the person, in `spec.md`.
- Where is the account chosen? — answered by the person, in `spec.md`.

## Not in scope

- Logging in, logging out, or reading a credential. Registering an account is
  naming a directory the person has already logged into.
- Mirroring `config.toml` into a managed runtime home, which would isolate
  Operon's own hooks from the user's. Operon installs into the directory
  the person named, so there is nothing to mirror.
- Antigravity. Its CLI has no documented equivalent variable, and inventing one
  would be a guess in a launch path.
- Switching the account of a session that is already running.
