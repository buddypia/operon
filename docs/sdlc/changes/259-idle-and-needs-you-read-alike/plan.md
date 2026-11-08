# Plan: the idle group is worded 「手すき」

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | rename the change-221 word test to R1's, update the document test and the tests naming the old words |
| `src/ui/widgets.rs` | `Free` message id and hint id, and the `Idle` hint |
| `src/i18n_tables.rs` | replace three rows in each table at sorted positions |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | the idle filter's word in captions, filter lists, state tables and troubleshooting lines; the state-table description |

## Order of work

1. Change the tests to the new words; watch R1 and R3 fail.
2. Change `src/ui/widgets.rs` and `src/i18n_tables.rs`; R1 and R2 pass.
3. Change the documents; R3 passes.
4. Mutation sweep, `make q.fast`, the named tests.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| a table row left behind or out of order | a raw Japanese id in English or Korean | `every_translation_table_is_sorted_by_message_id`, `every_message_id_has_a_row_in_every_table` |
| an old word left in a document | the manual names a chip that does not exist | R3's test refuses 「指示待ち」/「지시 대기」 anywhere in the five files |
| a "waiting" hint left behind | the chip or tab still reads as waiting on you | R1/R2's test checks both hints in all three languages |
| the new word overflows the sidebar | clipped chip | the sidebar fit loop over the four words |

## Proof of completion

- The acceptance commands in `spec.md` pass.
- `mutations.py` catches every mutation.

## Departures from the plan
