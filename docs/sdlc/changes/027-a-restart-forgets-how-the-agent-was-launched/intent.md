# Intent: restarting an agent silently drops the options it was launched with

- **Status**: approved
- **Opened**: 2026-09-06

## Problem

A terminal is opened with choices: which model, whether the agent may act
without asking, whether it can search the web. Those choices are made once, on
the launch screen, and some of them are made through a confirmation that exists
precisely because the choice is dangerous.

When that terminal's agent exits, the button in the corner offers to reopen the
conversation. It reopens it as a different agent than the one that was running:
none of the launch choices survive. The conversation is the same, the model may
not be, and an agent that was allowed to work without stopping now stops at
every step — or, read the other way, a person who never turned off the
permission prompts is not the one this affects, but a person who did turned them
off deliberately and gets no notice that they are back.

Nothing says this happened. The verb reads "会話 ID から再開", it does reopen the
conversation, and the difference only shows up in how the agent then behaves.

## Who feels it, and when

Anybody who launches with anything other than the defaults and whose agent then
exits — which is the normal end of a Claude Code or Codex session, not a
failure. It is worst for the choices that took an extra confirmation to make,
because those are the ones a person believes they have already decided.

## Desired outcome

Reopening a conversation runs the agent the way it was running. The options the
terminal was launched with are carried into the resume, except the ones that
would contradict a resume, and the person can see which options are coming
before they press the button.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface: this must work from what a session
  record already holds, without a new field and without a schema bump.
- Every option that reaches a command line still passes the existing validation
  gates. Carrying an option forward is not a reason to trust it less.

## Systems likely affected

`src/agents.rs` for what a resume command is allowed to carry, `src/cli.rs` for
how each provider spells a resume, and the one call site in `src/app.rs` that
builds the reopened session.

## Open questions

- Do the three CLIs accept their launch options next to their resume flag, and
  where in the argument order? — answered by running `--help` against the
  installed `codex`, `claude`, and `agy`; recorded in `spec.md`.
- Should a dangerous option carry forward without asking again? — answered by
  the person, in `spec.md`.

## Not in scope

- Hibernation: killing an idle session and reopening it on demand. Argued
  against rather than built.
- Changing what the launch screen offers.
- Carrying environment variables. Operon's managed launches set their own
  environment from the hook identity, and there is no per-session private
  environment to capture.
