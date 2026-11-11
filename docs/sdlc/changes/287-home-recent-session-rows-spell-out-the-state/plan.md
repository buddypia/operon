# Plan: a Home session row shows its state as a mark, named on hover

- **Spec**: `./spec.md`
- **Approved**: 2026-11-11
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | `home_287_row_draws_no_state_word`, `home_287_state_word_appears_on_hover`, `home_287_clicking_the_mark_opens_the_session`; the comment in `tapping_a_session_row_opens_its_terminal` that calls the leading column a label |
| `src/app/screens.rs` | `home_session_row`: `status_chip` becomes `session_status_icon` with a hover of the state's word and hint |
| `docs/sdlc/changes/287-…/` | `mutations.py`, `verification.yaml`, `state.yaml` |

## Order of work

1. Write the three tests. Against the row as it is, the first two fail: the
   word is drawn with the pointer away, and hovering adds no second one. The
   click test finds the mark by where the hover answers, so it fails with
   them until the mark exists, and from then on guards the click.
2. Replace the call in `home_session_row`; the three tests and the existing
   click test pass.
3. `make q.fast`, `bash scripts/check-bands.sh`, the mutation sweep.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The hover target swallows the row's click | clicking the mark does nothing | `home_287_clicking_the_mark_opens_the_session`, and the mutation that makes the mark sense clicks |
| The hover names nothing, so the state is told by shape and colour alone | no tooltip | `home_287_state_word_appears_on_hover`, and the mutation that removes the hover |
| The word comes back through another widget in the row | a word beside the mark | `home_287_row_draws_no_state_word`, and the mutation that restores `status_chip` |
| `README` screenshot `docs/images/01-home.png` shows the old row | a stale picture | not caught; reported to the person, retaken with the next screenshot pass |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked home_287_` — 3 passed.
- `cargo test --locked tapping_a_session_row_opens_its_terminal` — 1 passed.
- `python3 docs/sdlc/changes/287-home-recent-session-rows-spell-out-the-state/mutations.py` — every mutation caught.
- In the packaged app: Home rows begin with a mark and no word; the pointer
  on the mark shows the word.

## Departures from the plan

- `home_287_clicking_the_mark_opens_the_session` passed against the old row
  too: it finds the indicator by the hint on hover, which the old chip also
  had, and a click on the chip opened the session. It was watched failing by
  mutation instead (`the-mark-takes-the-rows-click`).
- The tests draw nine sessions, one or more under each of the four words,
  rather than the one idle session the plan implied; the independent
  verifier could not write in its own tree (the worktree guard judged it as
  trunk), so the cases it proposed were written here.
