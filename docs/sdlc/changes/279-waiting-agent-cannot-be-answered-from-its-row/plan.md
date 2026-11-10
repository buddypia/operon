# Plan: answer a waiting agent from its 要対応 row

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/waiting_reply.rs` | new: `ui_pinned_sessions` moved here with the reply field; `send_waiting_reply` |
| `src/app.rs` | `mod waiting_reply`; field `waiting_reply_drafts`; `track_attention` drops drafts that outlived their wait |
| `src/app/screens.rs` | `ui_pinned_sessions` leaves |
| `src/i18n_tables.rs` | two strings, English and Korean |
| `src/tests.rs` | eight `reply_279_` tests; `only_selections_wear_the_selection_colour` counts the new file |
| `CLAUDE.md` | module map line |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | the 要対応 paragraph |
| `docs/sdlc/changes/279-waiting-agent-cannot-be-answered-from-its-row/` | `mutations.py`, `verification.yaml` |

## Order of work

1. Move `ui_pinned_sessions` to the new file with no other edit; the pinned
   section's existing tests (`cargo test --locked pinned`, `attention`) pass
   unchanged.
2. The tests (seven at first, eight after the departures below), driven with injected egui events (pointer, `Text`,
   `Key`, `Ime`) through `Context::run`, the section drawn in a side panel
   and, for requirement 7, a terminal pane beside it; each watched failing by
   the mutation sweep.
3. The field, the drafts map, `send_waiting_reply`, the two strings.
4. The six documents and `CLAUDE.md`.
5. `make q.fast`; `cargo test --locked reply_279_`, `pinned`, `attention`,
   `claude_md`, `harness`, `readme`, and the message-table guards.
6. `mutations.py`, `verification.yaml`, the verifiers.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The row takes the click meant for the field | Clicking the field opens the terminal | `reply_279_typing_then_enter_sends_and_clears` clicks the field and checks the selected session |
| The terminal pane takes the keys typed in the field | Words typed for one agent reach the agent on screen | `reply_279_keys_typed_in_the_field_stay_out_of_the_pane` gives a pane the keyboard, then types in the field |
| The move changes what the section draws | Pinned-section layout tests fail | Step 1 runs them before anything is added |
| A reply reaches a session that is not running | A send to a dead tmux name | Requirement 1's test; `queue_terminal_input` also refuses |
| The Enter that confirms a Japanese conversion sends half a sentence | 「はい」 sent before it was finished | `reply_279_enter_that_confirms_a_conversion_does_not_send`, on the function and through the field |
| A string missing from one table | Japanese shown in English or Korean | The message-table guards in the suite |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked reply_279_` — `8 passed; 0 failed`.
- `python3 docs/sdlc/changes/279-waiting-agent-cannot-be-answered-from-its-row/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — no tier newly reached; `src/app/screens.rs`
  shorter than before.

## Departures from the plan

- The evaluator's first reading asked for the IME rule, key isolation as a
  requirement, and the announcement tied to an actual send; the spec gained
  requirements 6 and 7 and the wording of 3 before the documents were
  committed.
- The verifier found three things. A draft outlived its wait when the last
  waiting session left, because the section that pruned it is then not
  drawn: the pruning moved to `track_attention`, with the verifier's test
  kept as `reply_279_f1_…`. The move left
  `only_selections_wear_the_selection_colour` counting one file short: it now
  counts the new file. And `tmux_send_input` mangles text ending in 「;」 or
  beginning with 「-」 — older than this change and in a `subprocess` file, so
  it is change 280's, recorded under the spec's flagged concerns.
