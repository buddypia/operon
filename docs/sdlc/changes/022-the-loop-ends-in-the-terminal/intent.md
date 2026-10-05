# Intent: everything up to the commit happens in Operon, and the commit does not

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

Operon now carries a piece of work most of the way. A worktree is made from the
mainline, an agent is launched in it, its state is read from its own hooks, its
diff is read here, and the notes on that diff are handed back to it in one
message. Then the agent fixes what was wrong, and the person has to leave to
finish: switch to a terminal, remember which files they wanted and which they
did not, type a commit message from memory of a diff they are no longer looking
at, and push.

The commit message is the part that suffers. It is written after the reading,
away from the reading, and it ends up describing the change less well than the
person understood it ten seconds earlier — or it becomes "fix" and the
understanding is lost.

## Who feels it, and when

Every completed piece of work. It is worst when several worktrees are in flight,
because the terminal a person switches to is often not the one they meant.

## Desired outcome

From the screen where the change was read: choose which files go in, write or
have written a message, commit, and push. An agent that touched a file nobody
asked it to touch can be left out without leaving the screen. The message can be
drafted from the staged change itself by an agent CLI already on this machine,
and the person edits it before it becomes a commit.

Pushing never rewrites anybody else's history.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls. An agent CLI running
  locally is not a cloud call by this rule; it is the same binary Operon already
  launches into tmux.
- Every child process goes through the wrappers in `src/exec.rs` with a timeout
  and an output ceiling.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface.
- Nothing may run `git push --force`, in any spelling.

## Systems likely affected

`src/git.rs` for staging, committing, and pushing, and for the prompt the
message is drafted from. `src/agents.rs` for the non-interactive invocation of
each CLI. `src/app/screens.rs` for the Changes view. `src/app.rs` for the
background work.

## Open questions

- **Which CLI writes the message?** Answered: whichever of Claude Code, Codex,
  or Antigravity is present, in that order, with the one used named beside the
  button. No new setting.
- **Staging granularity?** Answered: per file. A hunk-level interface is a
  second diff pane with its own selection model and is not what this change is.

## Not in scope

- Hunk and line staging, amend, force-with-lease, pull, sync, merge or rebase
  conflict handling. Each is its own change; several are how a person loses work.
- Opening a pull request, and generating its title and body. Operon already
  lists pull requests; creating one is the next change after this.
- A per-repository prompt template for the message.
