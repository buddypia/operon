# Intent: two things are written down as true that the running application does not do

- **Status**: draft
- **Opened**: 2026-09-06

## Problem

An audit ran the real screens and read what they actually
painted, rather than reading the code that was supposed to paint them. Two
claims did not survive it.

The first is a choice a person made that the screen hides. A second login can be
registered, and the launch screen offers it — inside a section that is folded
shut by default and labelled as optional detail. A person who set up a work
account and a personal account has to know to unfold "詳しい設定" before they can
say which one this run belongs to, and until they do, the screen shows them
nothing about which login they are about to use.

The second is two places on one screen that disagree about the same command.
Restarting an exited conversation carries the switches it was launched with —
including one a person had to acknowledge in writing, because it lets the agent
work without asking permission. The button's hint says so. The line above it,
which spells out the command that will run, does not: it prints a shorter
command that omits exactly that switch. One of the two is wrong, and it is the
one that looks most like evidence.

The two are the same mistake: something was written down, the
thing it described changed or never arrived, and nothing was watching the gap.

## Who feels it, and when

The first, every time somebody with two logins starts a session — which is the
whole reason the second login was registered.

The second, at the moment somebody restarts an agent they had granted permission
to work unsupervised. The screen offers them a command to check, and the command
it offers is missing the part worth checking.

## Desired outcome

- Which login a run will use is visible on the launch screen without unfolding
  anything, and any other choice made inside the folded section is visible from
  outside it.
- The command shown for a restart is the command the restart runs. Not a
  similar one, not a shorter one — the same string.
## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- The launch screen and the session card are drawing surfaces, so the screen is
  agreed with a person before the code exists.
- No new message ids where an existing format string can carry a different
  value.

## Systems likely affected

`src/app/screens.rs` for the launch screen and the session card, `src/app.rs`
for the resume verb the card must agree with, `src/agents.rs` for the one
function both sides call, and `src/tests.rs`.

## Open questions

- Where the account row goes, and what a folded section is allowed to hide —
  answered by the person: the row moves up beside the agent, and the folded
  header carries a summary of every choice made inside it.
- What the card should show for a restart — answered by the person: the command
  that will actually run.

## Not in scope

Nothing beyond the two claims above.
