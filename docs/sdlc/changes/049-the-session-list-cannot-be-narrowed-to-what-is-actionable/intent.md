# Intent: finding the one agent that is free means reading every agent

- **Status**: approved
- **Opened**: 2026-09-15

## Problem

The list of sessions beside the terminal shows all of them, always, and offers
no way to see fewer. It is grouped by project, newest first, and every project
appears whether or not it has a session worth looking at. Each entry stands
about five lines tall — a name, a state, a purpose, a branch, and a control for
restoring the conversation somewhere else — so a person with a handful of
projects and a few agents each is already scrolling a column taller than the
window.

That scroll is the cost, and it is paid for the wrong reason. The list is sorted
by when a session was created, which has nothing to do with whether the session
can be given work. An agent that is mid-task, an agent that finished an hour
ago, and an agent sitting idle waiting for the next instruction are
interleaved, told apart only by a small coloured word on the entry's second
line. The one thing a person came to the list to find — an agent that will take
an instruction right now — has to be found by reading all the others.

The state is already computed and already drawn on every entry. Nothing uses it
to decide what to show.

## Who feels it, and when

Whoever comes back to the app after setting several agents going, at the moment
they have the next piece of work in hand and want to give it to whichever agent
is free. The more agents are running — which is the situation the app exists to
support — the worse it gets, so the cost grows exactly where the app is meant
to be strongest.

A second, sharper version of the same moment: an agent has stopped to ask a
question and is waiting on an answer. The count of those is on screen, but it
cannot be acted on, so finding *which* one is asking is the same full read of
the list.

## Desired outcome

From the session list, in one action, a person can see only the sessions in a
state they care about, and can tell how many are in each state before taking
the action. In particular the sessions that will accept an instruction
immediately, and the sessions stopped waiting for a human answer, can each be
brought to the top of the window without scrolling past the ones that are busy
or finished.

Observably: with twenty sessions across four projects, of which two are idle,
reaching those two takes one click and no scrolling.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI. Whatever counts the
  states runs inside a draw path sixty times a second.
- User-facing text is Japanese; the state words the entries already show are
  short English tokens and stay that way.
- A narrowed list is a list that hides things. Whatever hides a session must
  say so on screen, and must not survive a restart silently — a person who
  launches the app and sees an incomplete list with no explanation has lost
  work they can still see in `tmux`.

## Systems likely affected

`src/app.rs` and `src/app/screens.rs` for the list and its header,
`src/ui/widgets.rs` for the state each entry reports, `src/glyphs.rs` if a
state group needs a mark of its own, `src/i18n_tables.rs` for the new strings,
`src/tests.rs` for the guard. The persisted store is deliberately not on this
list.

## Open questions

- ~~Which groups of states are worth a control of their own? The entries
  currently report thirteen distinct states, and thirteen controls will not fit
  beside a 268px list.~~ **Answered, 2026-09-15:** four — the two that a person
  can act on now kept apart from each other, and the two they cannot folded
  together. 要対応 is an agent stopped waiting for a human; 空き is an agent
  live with nothing to do; 実行中 is anything in flight; 終了 is a terminal
  that is gone. `spec.md` carries the mapping.

## Not in scope

- **Moving the list to the top of the window.** It was the original request.
  Turning a five-line entry into something that fits a horizontal strip means
  dropping the purpose, the branch, and the restore control from the entry, and
  it converts a long vertical scroll into a long horizontal one rather than
  shortening it. The scroll is caused by how many entries there are and how
  tall each one is; this change addresses both of those directly instead.
- **Collapsing the list away to give the terminal the full width.** A real
  want, and a different one: it is about the terminal's size, not about finding
  a session. It also argues against itself here, since a list you cannot see is
  a list you cannot search. Worth revisiting once the list is short.
- **Sorting.** Grouping and hiding are enough to test the idea; re-ordering by
  state as well would make it impossible to say which of the two helped.
- **Any change to what a restore does**, or to the stores it reads.
