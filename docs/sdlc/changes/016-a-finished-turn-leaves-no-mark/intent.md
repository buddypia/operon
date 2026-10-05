# Intent: a turn that finishes while you are looking elsewhere leaves no mark

- **Status**: approved
- **Opened**: 2026-09-05

## Problem

Operon tells you a turn finished exactly once, as a macOS notification, and only
if notifications are on. Notifications are ephemeral: they are missed while the
Mac is asleep, dismissed by accident, and swallowed by Do Not Disturb. Nothing
in the app remembers that something happened while you were not looking.

The session list is where a person decides what to look at next, and it shows
only what is true *now*: a session that finished an hour ago and one that was
idle all along look identical. Supervising several agents means the list has to
answer "which of these has said something since I last read it", and it cannot.

The same gap swallows a question. A session that stopped to ask something shows
WAITING for as long as it waits, which is right — but once it has been answered
there is no way to say "come back to this one", and no way to tell a session you
have already read from one you have not.

## Who feels it, and when

Anyone running more than two sessions at once, which is what the app is for.
It bites hardest exactly when the app is working well: five agents running, four
of them finishing while you read the fifth.

## Desired outcome

- A session whose turn ended, or which stopped to ask something, while it was not
  the session on screen is marked in the list until it is read.
- Opening a session clears its mark.
- A session can be marked unread again by hand, to come back to it later.
- The count already in the sidebar header answers "how many need me", counting
  both the sessions waiting for an answer and the ones with something unread.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls, no new dependency.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a `paused` surface in `docs/sdlc/risk.yaml`, so this
  change does not add a field to it. Unread is therefore in memory, and a
  restart clears it — which is consistent with everything else the app derives
  from a live terminal, and is the one thing here worth revisiting with a person.

## Systems likely affected

`src/app.rs` (the unread set, where it is set and cleared, the sidebar count and
the row), `src/tests.rs`.

## Open questions

- Whether unread should survive a restart. Answered by the risk table for now:
  persisting it means a store field, the store is paused, and the value of the
  feature does not depend on it. Recorded here so the next person can disagree
  cheaply.

## Not in scope

A separate inbox or activity feed, a Dock badge, marking a whole project unread,
and any change to when a notification fires.
