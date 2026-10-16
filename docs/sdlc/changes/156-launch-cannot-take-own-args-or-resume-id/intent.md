# Intent: a launch cannot take the person's own arguments or reopen a chosen conversation

- **Status**: draft
- **Opened**: 2026-10-16

## Problem

The person asked (2026-10-16):

> 各CLIのパラメータを手動で入力するフラグとマージしてセッションを起動するようにして。例えばチェックボックスで危険だけど自動で実行にチェックして特定セッションIDを入力欄で入れたらそのセッションで自動実行ができるようにフラグをマージして実行するってこと。

Today the launch sheet offers only what its own tables know: a model, one mode,
an effort, and a fixed set of checkboxes per CLI. Anything else a CLI accepts can
be passed only by abandoning the built-in agent for 「別のコマンドを使う」 and
typing the whole command line, which loses every checkbox. And a launch always
starts a new conversation: there is no way to say "open conversation X, this
time without permission prompts". Reopening an existing conversation is possible
only from a session row's resume, which replays the options that session was
first launched with — not new ones.

## Who feels it, and when

Someone who has a conversation (its ID copied from a CLI or another tool) that
stopped on a permission prompt, and wants to continue it unattended: tick
「権限確認をすべてスキップする」, give the ID, launch. Today they leave the app
and type `claude --dangerously-skip-permissions --resume <id>` in a terminal.

Someone who needs one more switch the checkboxes do not list (for example
Claude Code's `--add-dir=../shared`) on every launch.

## Desired outcome

- The person types extra arguments for the chosen CLI on the launch sheet; the
  session runs with the ticked checkboxes and the typed arguments together, and
  the next sheet for that CLI remembers the typed arguments.
- The person types a conversation ID; the session opens that conversation, with
  the ticked checkboxes and typed arguments applied to it.
- A typed argument that runs without permission prompts asks for the same
  「理解したうえで起動する」 acknowledgement a ticked dangerous checkbox does.

## Constraints this change inherits

- User-facing text is Japanese; code, comments, and docs are English.
- Every launch command still passes the gates in `src/agents.rs` (no shell
  operators, no unacknowledged unguarded run).

## Systems likely affected

`src/agents.rs` (command construction and gates), `src/agents/settings.rs`
(remembered options), `src/app.rs` and `src/app/screens.rs` (launch sheet).

## Open questions

None for the person; the defaults adopted are in `spec.md` Flagged concerns.

## Not in scope

- The custom agent (「別のコマンドを使う」): its command line is already free text.
- Choosing a conversation from a list on the launch sheet; the CLI session
  browser already restores past conversations.
- Sending the request text into a reopened conversation.
