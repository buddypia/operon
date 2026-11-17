# Plan: `Operon agents`

- **Spec**: `./spec.md`
- **Approved**: 2026-11-17
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tmux/control.rs` | new: snapshot types, `agents_cli`, `agents_wait`, `AgentTerminal` + tmux impl, `run_agents_cli` |
| `src/tmux.rs` | `mod control; pub(crate) use control::*;` |
| `src/app/agent_snapshot.rs` | new: `agent_snapshot`, `track_agent_snapshot`, removal on exit |
| `src/app.rs` | `mod agent_snapshot;`, two fields, one call in `logic`, one in `on_exit` |
| `src/config.rs` | constants |
| `src/main.rs` | dispatch `agents` beside `knowledge` |
| `src/i18n_tables.rs` | strings in every table |
| `src/tests.rs` | `agents_309_*`, one per requirement as named in the spec |
| `docs/sdlc/changes/309-…/mutations.py`, `verification.yaml` | the sweep and the evidence |
| `CLAUDE.md` | map lines for the two new files |
| `README*.md`, `docs/MANUAL*.md` | section in three languages |

## Order of work

1. Constants and snapshot types; contract test of the round trip (R1 first half).
2. Failing tests against a fake `AgentTerminal` and an in-memory snapshot, then `agents_cli` / `agents_wait`:
   R3 `…caller_is_found_by_session_then_folder`, R4 `…stale_or_missing…`, R5 `…list_shows_only…`, R6 `…target_prefix…`, R7 `…send_types_only…` + `…send_rechecks_the_screen…`, R8 `…send_refuses_empty_long_or_control_text`, R9 `…read_returns_the_tail…`, R10 `…tmux_failure_is_reported…`, R11 `…wait_returns…` + `…wait_options_are_checked`, R12 `…usage_names_every_verb`.
3. App side: `agent_snapshot` and the due rule (R1), the single writer (R2 `…writer_applies_in_order_and_removal_is_last`), wired in `logic` / `on_exit`.
4. `main.rs` dispatch, strings, docs, CLAUDE.md lines.
5. `mutations.py` (send-ignores-state, send-skips-screen-recheck, other-projects-listed, control-text-allowed, stale-accepted, self-send-allowed, removal-not-last) and `verification.yaml`.
6. `make q.fast`, `cargo test --locked agents_309_`, a live check from a tmux session.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Snapshot built every frame | stutter | the due check runs at most once a second; `agents_309_snapshot_is_due_on_change_or_heartbeat_only` |
| Text typed into a working agent | prompt cut in half | R7 test; mutation `send-ignores-state` |
| Another project's session reached | leak across projects | R5/R6 tests; mutation `other-projects-listed` |
| Stale file after a crash believed | send into a closed app's guess | R4 staleness test |
| A write in flight resurrects the file after quit | CLI trusts a dead app for 60 s | single writer, R2 test |
| UI thread stalls; snapshot says idle while a turn started | text cut into a running turn | screen recheck before typing, R7 test |
| Control characters typed literally | an escape or Ctrl-C reaches another agent | R8 refusal |
| A `;`-ending text truncated by tmux | wrong text | reuse `tmux_send_input_arguments` (change 280) |

## Proof of completion

- `make q.fast` — fmt, clippy clean.
- `cargo test --locked agents_309_` — all pass.
- Live: two tmux sessions under a running build; `agents list/read/send/wait` behave as the spec says, including an unrecognised mid-turn screen (the snapshot's state still holds the send), a stopped session (not listed / wait ends) and a stale snapshot after quitting (exit 3).

## Departures from the plan

Review (rust-reviewer a5a85fd7, subprocess-safety-reviewer a032e008) found
what the plan had not:

1. **`send` then `wait` returned at once.** The snapshot still says `idle` for
   a moment after a send, so `wait --until idle` answered before the turn it
   should wait for had begun. `wait` now accepts a state it started in only
   once that state has been left, or has held for `AGENTS_WAIT_SETTLE_SECONDS`
   (5 s), or the timeout is reached with it still held. Tests:
   `agents_309_wait_returns_when_the_state_arrives_or_times_out` (after-send,
   settle, short-timeout cases); mutations `wait-returns-before-the-turn-starts`,
   `wait-never-settles`.
2. **The heartbeat lived in `logic`.** A window that is not drawing would let
   the file go stale and the command say Operon is not running while it was.
   The heartbeat moved into the writer thread (`recv_timeout`): the last
   snapshot is rewritten under a new time every 15 s, and a removal forgets it
   so nothing brings the file back. `logic` now hands over a snapshot only when
   its content changed, built once. Spec R1 holds as written. Tests:
   `agents_309_writer_applies_in_order_and_removal_is_last` (heartbeat and
   after-removal cases), `agents_309_snapshot_is_due_on_change_or_heartbeat_only`.
3. **Output and other programs' text.** `print!` panics on a closed pipe
   (`agents read | head`); the entry writes with `write_all` and ignores the
   error. `list` drops control characters from names and branches, and `read`
   drops every control character but newline and tab, so a forged name or a
   carriage return cannot redraw the caller's terminal or forge a row.
4. **"Same project" is a guardrail, not a boundary.** The caller names itself
   through `OPERON_SESSION` and its directory, both its own to set. The module
   comment, the manuals, and the READMEs now say so: letting an agent run
   `agents send` lets it prompt the project's other agents with their
   permissions.
5. **A control character at either end slipped through.** empirical-verifier
   af4a0fab reproduced `send <id> "hello\n"` typing `hello` with exit 0: the
   text was trimmed before it was judged. It is now judged before trimming.
   `agents_309_f1_control_characters_at_the_edges_are_refused` failed at
   0a49f72 and passes; mutation `control-judged-after-trim`. Its parsing
   probes (`--lines` twice, `--json` twice, an uppercase hyphenated prefix, no
   session) held and are kept as `agents_309_parsing_edges_hold`. `list` also
   drops control characters from the agent and state columns.
6. **Main brought a new state.** Change 308 landed `未開始` (`NotStarted`):
   a session up at its prompt that has not had its first request. It is
   published as `not-started`, and `send` takes it as well as `idle` — the
   window already groups both as free, and handing a fresh session its first
   task is what a coordinating agent most needs. `wait --until idle` does not
   count it. Tests: `agents_309_snapshot_names_each_live_session_with_its_state`
   (every state has a word), `agents_309_send_types_only_into_an_idle_agent_of_the_same_project`
   (not-started taken, every busy state refused); mutations `not-started-refused`,
   `not-started-published-as-idle`.

Left as they are, and why:

- The screen recheck knows the CLIs' running and prompt shapes; a dialog it
  does not recognise reads as idle. It can only withhold a send; the state that
  grants one is the window's, which takes the CLIs' own hook reports first.
- `finish` joins the writer at quit without a bound; the queue holds at most a
  second of work and the file is small.
- `agents.json` is written with the default mode, inside the per-user
  application-support directory that already holds the store.

