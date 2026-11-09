# Spec: read sessions a third as often while another application is in front

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. While the window has the focus, every live session is read when
   `SESSION_POLL_INTERVAL` (2 s) has passed since the last reading, and not
   before — as today. — `poll_275_front_window_reads_sessions_every_two_seconds`
2. While the window does not have the focus, the sessions are read only when
   `SESSION_POLL_INTERVAL_UNFOCUSED` (6 s) has passed since the last reading. —
   `poll_275_background_window_waits_longer_between_reads`
3. On the first frame after the focus returns, the sessions are read at once
   if the last reading is at least `SESSION_POLL_INTERVAL` old — the longer
   wait is not finished first. — `poll_275_returning_to_the_window_reads_at_once`
4. The frame takes the focus from the window itself: a frame drawn without the
   focus three seconds after a reading starts none, and the same frame with the
   focus starts one. — `poll_275_frame_reads_the_windows_focus`
5. A store write waiting to be retried, and the save of a changed tab layout,
   still run every two seconds whether or not the window has the focus. —
   `poll_275_pending_store_retry_keeps_its_cadence_in_the_background`

## Behaviour

Nothing a person reads changes; no string is added. With Operon behind another
application, session rows, tab status dots, the last-words preview and a
visible terminal's text refresh every six seconds instead of every two. A state
an agent reports through the status hooks arrives by its own socket and wakes
the frame as before, so 「入力待ち」 and its notification are not delayed for
those agents; for a session whose state is only read off its screen, the
notification can come up to four seconds later than today. Clicking back into
Operon reads everything within one frame.

Empty (no live session): nothing is read, as today. tmux missing: nothing is
read, as today. A reading still in flight when the next is due: skipped, as
today (`spawn_background` refuses a duplicate key). Typing into a terminal
needs the focus, so the fast echo reading after a keystroke is unaffected.

## Design

Placement: **EXTEND** — `OperonApp::update` in `src/app.rs` already owns the
two-second tick that calls `request_session_poll`. Route `modify`.

- `src/config.rs`: `SESSION_POLL_INTERVAL: Duration = 2 s` and
  `SESSION_POLL_INTERVAL_UNFOCUSED: Duration = 6 s`; the inline two seconds of
  the tick is replaced by the first.
- `src/util.rs`: `session_poll_interval(focused: bool) -> Duration` returns one
  or the other.
- `src/app.rs`: a new field `last_session_poll: Instant` (not persisted) and
  `poll_sessions_when_due(&mut self, focused: bool, now: Instant) -> bool`,
  which calls `request_session_poll` and stamps the field when
  `now - last_session_poll >= session_poll_interval(focused)`. `update` reads
  `ctx.input(|input| input.focused)` once and calls it every frame; the
  existing two-second block keeps `retry_pending_store` and
  `save_workspace_layout_if_changed` on `last_session_refresh`, and no longer
  calls `request_session_poll`. `ctx.request_repaint_after(2 s)` stays, so a
  frame still arrives to do the housekeeping and to notice a due reading.
- Focus returning is a window event, which egui turns into a frame; that frame
  sees `focused == true` and an old `last_session_poll`, which is requirement 3
  without any extra state.
- No new child process, no persisted shape, no new background task; the draw
  path gains one input read and one `Instant` comparison per frame.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Nothing is painted differently. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No glyph added. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | Both intervals are constants in `src/config.rs`, read by the rule and by the tests; no test restates the number. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | The store retry is deliberately left on its two-second timer (requirement 5); no store, cancellation, migration or byte limit is edited. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | No new child; the existing readings run through the same functions, less often. |
| Documentation — user-facing docs change in all three languages together | No | `README.md` and `docs/MANUAL.md` mention two seconds only for the open file's disk comparison, which this does not touch. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | No | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | No | No scan or output path added. |

## Flagged concerns

- **A visible but unfocused window** — egui 0.31 reports neither occlusion nor,
  after startup, minimizing on macOS, so a window the person is watching beside
  their editor also slows to six seconds. Resolved: accepted and stated in the
  intent's Not in scope; six seconds keeps a watched terminal readable, and one
  click restores two.

## Acceptance

- `cargo test --locked poll_275_` passes (five tests).
- `make q.fast` passes.
- `python3 docs/sdlc/changes/275-background-window-keeps-polling/mutations.py`
  catches every mutation.
- In the running app with several sessions: `Activity Monitor` shows tmux
  children appearing every two seconds with Operon in front and every six with
  another application in front; clicking Operon refreshes a terminal at once.

## Rejected alternatives

- Stop reading entirely while unfocused — an agent without hooks would never
  raise its 「入力待ち」 notification, which is when the person is elsewhere.
- Slow the whole frame wake to six seconds — it would also slow the store retry,
  a durability path this change has no reason to touch.
- Back off further the longer the window is unfocused — a second rule for a
  saving the first one already takes most of.
