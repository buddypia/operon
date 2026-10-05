# Intent: a restore reproduces the conversation a terminal has left, not the one it is in

- **Status**: approved
- **Opened**: 2026-08-28

## Problem

A full-history restore out of a managed Claude terminal carried two turns of a
conversation that holds ten. The restore was not lossy: it faithfully reproduced
a different conversation. The terminal had been asked to restore `7ee22502`, the
conversation on its screen, and Operon read `f5cb1562`, a stub whose only
exchange is the word `ping`.

Operon launches a managed Claude terminal with `--session-id <uuid>` and stores
that UUID as the terminal's conversation for ever. The assumption underneath is
that a terminal has one conversation for its lifetime. It does not: `/clear`
starts a new conversation in the same terminal, under a new ID, in a new
transcript file, and every later `/clear` does it again. The stored ID goes on
naming the first one.

The check that exists to catch a wrong identity — `claude_transcript_can_belong_
to_managed_session` — asks only whether the recorded transcript began *before*
the terminal existed. A terminal that has moved on is invisible to it: the
abandoned stub began one second *after* the terminal, so it passes. And the
recovery behind that check matches candidates against the launch goal, which a
terminal started without one does not have, so it declines before it reads
anything.

Across this machine's 25 running Claude processes, 5 have already rotated away
from the ID they were launched with. It is not an edge case; it is what happens
to any terminal used long enough to be worth restoring.

## Who feels it, and when

Anyone who restores or reopens a managed Claude terminal that has been cleared
at least once. The failure is silent and it reads like the feature is broken
rather than misaimed: the destination CLI opens holding a real conversation, just
not the one that was asked for. Nothing in the notice distinguishes "restored two
turns because the conversation is short" from "restored two turns because Operon
read the wrong file".

It is also the failure mode most likely to be repeated. `/clear` is a routine
operation, and the longer a terminal is used — which is exactly what makes its
conversation worth moving — the likelier it has rotated.

## Desired outcome

- A restore or reopen of a managed Claude terminal uses the conversation that
  terminal is in *now*, however many times it has been cleared.
- The identity is read from evidence rather than inferred from ranking. Two
  transcripts that both began after a terminal cannot be told apart by when they
  began, and the abandoned one begins closer.
- When the evidence is absent or ambiguous, Operon says so and restores nothing,
  rather than falling back to a guess that is wrong in the common case.
- The launch-time ID stops being treated as the answer.

## Constraints this change inherits

- Local-first: whatever identifies the terminal has to be on this machine.
- User-facing text is Japanese; code, comments, and docs are English.
- Claude Code owns the format and the lifecycle. Operon cannot ask it anything,
  only read what it writes.
- `tests_ignored` is banded at 6, and CI has no local Claude store, so every
  guard has to run against fixtures the tests build.
- New subprocess work goes through the wrapped runners. Reading a PID's process
  tree to find the terminal's `claude` would be new subprocess work in the
  restore path.
