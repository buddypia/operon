# Spec: unread sessions

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A session becomes unread when its activity settles on a state that would
   notify — a turn that finished, or a stop to ask something — and it is not the
   session currently on screen.
2. "On screen" means the Sessions page is the open page, the session library is
   not covering it, and the session is the selected one. Any other arrangement
   leaves the session unread.
3. Both paths that decide activity mark unread: the CLIs' own hooks and the
   screen heuristic. The mark does not depend on notifications being enabled.
4. Selecting a session clears its mark, whether it was selected from the sidebar,
   a session card, the queue, or a restore.
5. A session can be marked unread by hand from the row menu it already has.
   Marking by hand while the session is on screen leaves it unread until another
   session is selected, so the action is not undone by the click that made it.
6. The count beside the attention glyph in the sidebar header is the number of
   sessions that are waiting for an answer or unread, counted once each.
7. An unread session's title is drawn bold in the sidebar; nothing else changes.
8. Removing a session, or closing its terminal, forgets its mark.
9. Unread is in memory. A restart clears every mark, and nothing is written to
   the store.

## Behaviour

- Two sessions run. One finishes while the other is open: its row goes bold and
  the header count rises by one. Clicking it clears both.
- The open session finishes: nothing is marked, because it was read as it
  happened.
- A session stops to ask something while another is open: it is both WAITING and
  unread, and the header counts it once.
- Right-clicking, or opening `⋯` on, a session row offers 「未読にする」. Choosing
  it on the open session marks it, and it stays marked until a different session
  is selected.
- After a restart nothing is unread, which the app does not claim otherwise.

## Design

`OperonApp` gains `unread_sessions: HashSet<Uuid>` and
`unread_hold: Option<Uuid>` — the session a person has just marked by hand,
exempt from the clear-on-view rule until the selection moves.

| Piece | Shape |
|---|---|
| Marking | `note_session_activity_change(session_id, notice)` is called from both the hook path and the screen path, at the point each already decides a notification is warranted. It inserts into `unread_sessions` unless `is_session_on_screen(session_id)`. |
| On screen | `is_session_on_screen(id) -> bool`: `page == Page::Sessions && !session_library_open && selected_session == Some(id)`. |
| Clearing | `select_session(id)` — a new single entry point the existing selection sites call — sets `selected_session`, removes `id` from `unread_sessions`, and clears `unread_hold` when it names another session. |
| Holding | `mark_session_unread(id)` inserts and sets `unread_hold = Some(id)`. `select_session` clears the hold when the selection changes; `is_session_on_screen` is not consulted by the hold. |
| Counting | `attention_count()` replaces `awaiting_input_count()`: sessions that are `Active` and awaiting input, unioned with `unread_sessions`, counted once. |
| Drawing | the sidebar row title takes `.strong()` when unread; the row menu gains 「未読にする」. |
| Forgetting | the paths that drop a session from the list also remove it from `unread_sessions`. |

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No colour is introduced; an unread title uses the same ink at `.strong()`. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | The header glyph is the existing `ICON_ATTENTION`; no new mark. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | No | No new identifier; the unread set is keyed by the session's own `Uuid`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | Nothing is persisted, so there is no schema change and no migration. Stated in the intent as a deliberate limit rather than an oversight. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | No child process is spawned. |
| Documentation — user-facing docs change in all three languages together | Yes | One bullet under Supervision in `README.md`, `README.ko.md`, and `README.ja.md`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | A `HashSet` in memory. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | No new scan. The set is bounded by the session list, which is already bounded by the store. |

## Flagged concerns

- **Unread does not survive a restart.** Resolved by the risk table: persisting
  it means a field on a `paused` surface. Recorded in the intent's open
  questions so it can be reopened in one line.
- **Marking the open session unread could be undone by the same frame.**
  Resolved by `unread_hold`, which exempts one session until the selection moves.
- **Two paths can mark the same session.** Resolved by the set: marking is
  idempotent, and the count unions rather than sums.

## Acceptance

- `cargo test --locked` passes, including `a_turn_that_finishes_off_screen_is_unread`,
  `viewing_a_session_clears_its_mark`, `marking_unread_by_hand_survives_the_click_that_did_it`,
  and `the_attention_count_counts_a_waiting_unread_session_once`.
- In the running app: two sessions, finish the one that is not open, and its row
  goes bold with the header count rising; open it and both clear.

## Rejected alternatives

- **A separate unread badge on the row.** A second mark beside the status chip
  competes with it; bolding the row serves the same purpose.
- **A timestamp per session compared against the state's start.** It is the right shape for persistence, and without persistence it is the same
  answer as a set with more moving parts.
- **Counting unread separately from waiting in the header.** Two numbers where
  the question is "how many need me" is one number too many.
