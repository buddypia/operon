# Intent: an agent's state is guessed from the bottom of its screen

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

Operon decides whether an agent is working, waiting on a question, or idle by
reading the last twelve non-blank lines of its terminal every two seconds and
looking for phrases: an interrupt hint, an elapsed-seconds counter, "do you want
to proceed", "(y/n)". The CLIs themselves know exactly what they are doing, and
all three of them can say so: Claude Code, Codex, and Antigravity each run a
configurable command at the start of a turn, before and after every tool, when
they need permission, and when the turn ends. Reading state from those commands, with the screen as a
fallback, is the reliable approach. Operon reads only the screen.

The guess is wrong in ways a person notices:

- A pasted log or a quoted transcript containing "esc to interrupt" reads as a
  running turn. A tool whose approval wording is not in the list reads as idle.
- A turn that ends is noticed two to four seconds late, and only after two polls
  agree; a permission prompt the same.
- The app never learns which tool the agent is running, what its last answer
  was, or the id of the conversation it is in. The conversation id is instead
  recovered afterwards by matching file timestamps, which refuses ties.
- After `/clear` or a resume, an idle prompt looks like a finished turn, and a
  notification says something completed when nothing did.

## Who feels it, and when

Anyone supervising more than one session from the list. The status chip is what
they read to decide where to look next, and the notification is what pulls them
back to the app; both are inferred, and the inference is the part they cannot
trust. It is worst with Claude Code, whose working line has no interrupt hint
and whose approval prompts change wording between releases.

## Desired outcome

- A session launched from Operon shows WORKING, WAITING, and IDLE within about a
  second of the CLI changing state, because the CLI said so.
- The chip's hint names the tool the agent is running, when the CLI reports one.
- A finished turn notifies once; a session that merely started, resumed, or was
  cleared does not.
- The conversation id and transcript path the CLI reports are what Operon uses
  for native resume, so a session's resume identity is known from its first
  turn rather than reconstructed later.
- The screen heuristic still works, for a CLI whose hooks are not installed and
  for the minutes between a restart and the next event.
- The person can see, in Settings, which CLIs are wired up and switch the whole
  mechanism off; switching it off removes what was added to their CLI settings.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls, no new dependency.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface. This change does not touch its
  shape; what it needs to remember lives beside the hook scripts instead.
- The CLI settings files are not Operon's: an edit must keep everything the
  person and other tools wrote, be idempotent, and be reversible.
- Each hook command must never make the CLI fail, hang, or change its
  decisions.

## Systems likely affected

`src/tmux.rs` (the launch passes identity into the session's environment), a
new module for the hook scripts, their installation, the receiving socket, and
the state machine, `src/app.rs` (the poll consults hook state before the screen;
Settings gains the toggle), `src/config.rs` (the identifiers), `src/tests.rs`.

## Open questions

- Whether Codex needs a trust entry in `config.toml` for a hook in the
  user-level `hooks.json`. Answered by the machine this runs on: the entries it
  holds cover project-level files only, while a user-level hook set with no
  entry is in daily use. The change writes the entries anyway — the hash
  algorithm was verified against three existing entries — because a version
  that starts asking costs the person a prompt and Operon nothing.
- Whether the toggle belongs in the store. Answered for now by the paused
  surface: it lives in the hook module's own file, and a later change may move
  it with a schema bump.

## Not in scope

Subagent and teammate rosters, interrupt inference from Ctrl+C, the Claude
`statusLine` usage feed, hibernation, unread state, a dock badge. Each is its own
change of its own.
