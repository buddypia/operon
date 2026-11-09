# Plan: read sessions a third as often while another application is in front

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `SESSION_POLL_INTERVAL`, `SESSION_POLL_INTERVAL_UNFOCUSED` |
| `src/util.rs` | `session_poll_interval(focused)` |
| `src/app.rs` | field `last_session_poll`; `poll_sessions_when_due`; `update` reads the focus and no longer polls inside the housekeeping block |
| `src/tests.rs` | five `poll_275_` tests |
| `docs/sdlc/changes/275-background-window-keeps-polling/` | `mutations.py`, `verification.yaml` |

## Order of work

1. Write the five tests against the names in the spec; they do not compile
   until step 2, which is the failure watched for 1–3; 4 and 5 are watched
   failing by the mutation sweep.
2. Constants in `src/config.rs`, the rule in `src/util.rs`.
3. Field, method and the `update` wiring in `src/app.rs`.
4. `make q.fast`, `cargo test --locked poll_275_`, and the existing tests that
   set `last_session_refresh` (`repaint_loop_`).
5. `mutations.py`, then `verification.yaml`, then the verifier.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The focus is never reported true, so the app always reads at six seconds | Terminals feel slow in front | `poll_275_frame_reads_the_windows_focus` drives a real frame with `RawInput::focused`; the acceptance observation in the running app |
| The store retry slows with the reading | A pending write is retried late behind another app | `poll_275_pending_store_retry_keeps_its_cadence_in_the_background` |
| Returning to the window waits out the long interval | A stale terminal for up to six seconds after a click | `poll_275_returning_to_the_window_reads_at_once` |
| `src/app.rs` passes 10 000 lines (`largest_module_lines` propose) | `bash scripts/check-bands.sh` | The rule lives in `src/util.rs`; `src/app.rs` gains under twenty lines from 9 941 |
| An existing test relied on the poll sharing `last_session_refresh` | A red test in CI | `cargo test --locked repaint_loop_` locally; the full suite in CI |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked poll_275_` — `5 passed; 0 failed`.
- `python3 docs/sdlc/changes/275-background-window-keeps-polling/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — no new tier reached.
- In the running app: tmux children every two seconds in front, every six
  behind, and an immediate refresh on return.

## Departures from the plan

None yet.
