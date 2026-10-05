# Plan: unread sessions

- **Spec**: `./spec.md`
- **Approved**: 2026-09-05
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `unread_sessions` and `unread_hold` fields; `select_session`, `mark_session_unread`, `is_session_on_screen`, `note_session_activity_change`; `awaiting_input_count` becomes `attention_count`; both activity paths call the marker; every selection site goes through `select_session`; the sidebar row draws bold and gains the menu item; removal forgets the mark. |
| `src/i18n_tables.rs` | EN and KO rows for 「未読にする」. |
| `src/tests.rs` | The four tests in the spec's Acceptance. |
| `README.md`, `README.ko.md`, `README.ja.md` | One bullet under Supervision. |

## Order of work

1. Fields and the four small methods. Compiles; green.
2. Route every `selected_session = Some(..)` through `select_session`, and both
   activity paths through `note_session_activity_change`. Compiles; green.
3. The sidebar row and the count. Tests. Compiles; green; clippy clean.
4. Translations and the three READMEs.
5. Gates, bands, commit.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A selection site left assigning the field directly | A session stays bold after being read | `viewing_a_session_clears_its_mark` goes through the same entry point the UI does; a grep for the raw assignment is part of the work |
| The open session marks itself | Every finish is unread | `a_turn_that_finishes_off_screen_is_unread` asserts both directions |
| The hand mark is cleared by the frame that made it | The menu item appears to do nothing | `marking_unread_by_hand_survives_the_click_that_did_it` |
| The count double-counts a waiting-and-unread session | The header reads two when one session needs attention | `the_attention_count_counts_a_waiting_unread_session_once` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`, N four higher.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `bash scripts/check-bands.sh` — no new breach.
- Each new test watched failing by mutation.
- In the running app: finish a session that is not open and watch its row go bold.

## Departures from the plan

- **An unread title is not bold, because every title on that list already is.**
  The spec said bold; the sidebar draws every session title `.strong()`, so
  bolding one would have distinguished nothing. An unread row is written in
  `palette.text_strong` and a read one in `palette.text` — an existing semantic
  role applied to an existing label, no new element and no layout change. Found
  by reading the row before writing to it, which is why the spec's mechanism is
  corrected here rather than quietly replaced with something that paints more.
- **`remove_session_record` also forgets the hook state.** It already had to
  forget the unread mark; the hook status and launch token for a session that no
  longer exists were being kept for no reader, and dropping them beside the mark
  is one line rather than a second cleanup path.
