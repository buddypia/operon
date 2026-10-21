# Intent: A handoff drops what the tools found, and leaks tool output as conversation

- **Status**: approved
- **Opened**: 2026-10-21

## Problem

> 「引き継ぎでツール出力を圧縮」「続けて進めて」

The person chose this from a list of improvements offered for the handoff:
keep the output the source session's tools produced — test runs, build logs,
search results — in a shortened form, instead of throwing it away.

Today a handoff throws away every tool result from the source conversation.
The session that receives the handoff reads the person's task and the last few
things the two sides said, but not the failing test, the compiler error, or
the log line that the conversation was about. It has to run the tools again
to learn what the source already knew, or it continues on a guess.

The same rule leaks: a tool result that contains a blank line is thrown away
only up to that blank line. Everything after it — the second half of a test
log, a block of a file that was read — reaches the receiving session as if
the assistant had said it. The same happens to a reasoning passage with a
blank line in it.

## Who feels it, and when

- A person who hands a Claude Code conversation to Codex CLI (or the other way
  round) in the middle of fixing a failing test: the receiving session does not
  know which test failed or why, and starts by re-running everything.
- A person whose source conversation read a long file or ran a verbose command
  just before the handoff: part of that output shows up in the receiving
  session as the assistant's own words, which it then trusts as a statement.

## Desired outcome

- The receiving session reads the most recent tool results from the source,
  each shortened so that the lines that say what went wrong, and the start and
  end of the output, survive, and repeated lines appear once with a count.
- Where Operon shortened a tool result, the bundle says so and points at the
  transcript snapshot for the whole output.
- No part of a tool result or a reasoning passage appears in the bundle as
  something the user or the assistant said.
- The bundle stays within its existing size budget, and the person's
  instructions, the task, and the last turn still outrank the tool results.

## Constraints this change inherits

- Local-first: no telemetry, accounts, or cloud calls — the shortening is done
  by rule on this machine, never by a model.
- The handoff never mutates the destination CLI's internal store.
- The existing byte budget of a handoff bundle holds.

## Systems likely affected

`src/history.rs` (the handoff bundle), `src/config.rs` (its limits),
`src/tests.rs`, and the README handoff section in three languages.

## Open questions

None: the person chose the direction; the limits are decided in the spec.

## Not in scope

- Shortening what the CLIs send to their models during a session.
- Tool calls: neither their names nor their arguments are carried, only what
  the tools returned. Pairing a result with its call is not possible from the
  text the transcript reader produces once calls run in parallel.
- Any change to the full transcript snapshot, which stays byte-for-byte.
