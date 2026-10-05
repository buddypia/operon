# Intent: a conversation restored into Codex cannot be reopened at all

- **Status**: approved
- **Opened**: 2026-09-18

The `bugfix` route skips intent and spec; this file is here because the bug
report is the intent and the report is worth having in the change directory
rather than only in a session transcript.

## Problem

Moving a conversation into Codex appears to work — Operon reports the turns it
restored and offers the resumed session — and then the terminal it opens dies
immediately with a message about a missing ordinal. Nothing about the failure
points at the restore: the pane prints "Resuming session…", then an error from
Codex's own thread store, then the pane is dead. The conversation is not
recoverable from the Codex side at all, and the archive Operon wrote is fine,
so the loss looks like Codex's fault.

## Who feels it, and when

Every restore whose destination is Codex, on Codex 0.155.0 — that is, every
`codex` destination since Codex started numbering its rollout records, which
the local session files date to somewhere before 2026-09-10. Both directions
into Codex are affected: a Claude Code source and an Antigravity source, since
they share the destination writer.

## Desired outcome

A conversation restored into Codex opens in `codex resume` with the restored
turns visible, on the Codex that is installed. And when a future Codex changes
the rollout format again, the restore fails while Operon still has the person's
attention, naming what it could not write — rather than handing over a session
id that dies on first use.

## Constraints this change inherits

- Local-first: the rollout format is Codex's, read and written on this machine.
- User-facing text is Japanese; code, comments, and docs are English.
- `.claude/rules/transcripts.md`: what a restore does with a foreign record is
  a judgement made in one place, not spread through `match` arms.

## Systems likely affected

`src/history.rs` — the Codex destination writer and its verification. No
persisted Operon shape changes: the file being written belongs to Codex.

## Open questions

None. Both halves were measured before this file was written: what Codex
refuses, and what it accepts.

## Not in scope

The two Antigravity-source end-to-end tests that fail on a different cause —
they count 5 records where the fixture expects 4, from a fixture record kind
the transcript vocabulary never learned. That is a change of its own, named by
its subject rather than by a number: this file guessed 060, and by the time it
was written 058 was taken in another worktree and the number here was already
spoken for by a sibling document. A number cited before the directory exists
is a reference that can only be wrong.
