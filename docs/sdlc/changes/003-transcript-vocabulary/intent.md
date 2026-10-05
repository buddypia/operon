# Intent: a restore silently carries less than half of the conversation

- **Status**: approved
- **Opened**: 2026-08-28

## Problem

Restoring a conversation's full history from one agent CLI to another carries
only part of it, and says nothing about the rest. A real Claude Code conversation
of 1 165 records restored as 471 turns. The person who ran the restore has no way
to learn this: the notice reports a count with no denominator and calls the
result "そのまま復元しました".

The loss is not a miscount. Whole categories of content never leave the source —
the model's reasoning, tool references, images, and, on the other two providers,
the reasoning items and tool results that make up most of what those CLIs record.

Underneath it is a design problem rather than three bugs. Operon reads formats it
does not own and cannot version. The readers treat "this is deliberately not part
of the conversation" and "this is a shape nobody has ever seen" as the same
answer, so every time one of the three CLIs ships a new kind of record, a little
more of the conversation stops crossing and nothing anywhere says so.

## Who feels it, and when

Every time somebody restores a long conversation into another CLI in order to
keep working in it — which is the one thing this feature exists for. The damage
is worst exactly when the restore matters most: a conversation long enough to be
worth moving is one where the reasoning behind earlier decisions is the part that
cannot be reconstructed.

It also arrives on a schedule nobody controls. Each of the three CLIs updates on
its own, and an update that adds a record type makes an already-shipped Operon
quietly worse without any change to Operon.

## Desired outcome

- A restore carries everything either participant said, did, was shown, or
  reasoned — including the model's reasoning, in whatever form can actually
  survive the crossing between two different vendors.
- Nothing is excluded except by a decision somebody wrote down and can be shown.
- A shape Operon has never met is carried across and reported, not dropped. The
  report names the kind, so the next person knows exactly what to classify.
- Every source record is accounted for, and the accounting is visible to the
  person who ran the restore and durable beside the archive.
- The claim the notice makes matches what the restore did.

## Constraints this change inherits

- Local-first: no telemetry, no accounts, no cloud calls. Whatever notices a gap
  has to notice it on this machine.
- User-facing text is Japanese; code, comments, and docs are English.
- The formats being read belong to three other products. They are undocumented,
  they change without notice, and Operon cannot negotiate a version with them.
- Another vendor's reasoning cannot be replayed as reasoning: Anthropic signs its
  thinking blocks and rejects modified ones, and OpenAI's reasoning is sealed
  under an organisation-scoped key.
- CI has no conversation files, so no check that depends on real transcripts can
  be a gate there.
- `tests_ignored` is banded at 6; the fix cannot buy coverage with a seventh
  ignored test.
