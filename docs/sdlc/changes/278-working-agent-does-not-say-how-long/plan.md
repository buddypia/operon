# Plan: say how long a working agent has been working

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app/working_time.rs` | new: `track_working`, `working_time`, `working_time_text`, `working_time_label` |
| `src/app.rs` | `mod working_time`; field `working_since`; `update` calls `track_working`; `status_view` appends the time to the hint |
| `src/app/screens.rs` | the per-project list's row and the open pane's heading print the time after the state word; the project column row's hover and the tab mark's hover end with it |
| `src/tests.rs` | eight `working_278_` tests |
| `CLAUDE.md` | module map line |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | one sentence each |
| `docs/sdlc/changes/278-working-agent-does-not-say-how-long/` | `mutations.py`, `verification.yaml` |

## Order of work

1. The tests (five at first, eight after the departures below); they fail to compile until step 2, and requirement 4's is
   then watched failing on the row before step 4.
2. `src/app/working_time.rs`, the field, the `mod` line, `CLAUDE.md`.
3. `update` and `status_view`.
4. The row in `src/app/screens.rs`.
5. The six documents.
6. `make q.fast`; `cargo test --locked working_278_`, `session_25`,
   `claude_md`, `harness`, `readme`.
7. `mutations.py`, `verification.yaml`, the verifiers.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The row is tested through its helper and not on screen (lesson 005) | A green test and no time in the app | `working_278_row_says_how_long_the_agent_has_worked` draws the page and reads the painted text |
| 起動中 or 順番待ち, which share the word 実行中, get a time | A session that has not started "working for 5 minutes" | Requirement 1 keys on `SessionStatusKind::Working`; its test includes a starting session |
| The count survives a turn boundary | "実行中 · 40分" on a turn that began a minute ago | `working_278_a_new_turn_starts_from_zero` |
| A document changes in one language only | The three-language guard in the suite | `cargo test --locked readme` and the documentation guards |
| `src/app/screens.rs` reaches 10 000 lines | `bash scripts/check-bands.sh` propose tier | The row change is a few lines from 9 978; the logic is in the new file |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked working_278_` — `8 passed; 0 failed`.
- `python3 docs/sdlc/changes/278-working-agent-does-not-say-how-long/mutations.py`
  — every mutation caught.
- `bash scripts/check-bands.sh` — no tier newly reached.
- In the running app: 「実行中 · 1分」 a minute into a turn, gone when it ends.

## Departures from the plan

- The first spec put the time on "the session column's row". Its test drew
  the セッション page and found no state word there to follow: the project
  column's rows and the tabs carry a mark and no word (changes 250, 253), and
  the row with `agent mark · 実行中` is the per-project session list's. The
  spec, screen and this plan were corrected before the code was committed:
  the time is printed in that list's row and in the open pane's heading, and
  said in the hovers of the column row and the tab mark. Two tests were added
  for the pane heading and for those two hovers.
- The mutation sweep found nothing watching the frame's call to
  `track_working`; `working_278_frame_keeps_the_clocks` was added for it.
