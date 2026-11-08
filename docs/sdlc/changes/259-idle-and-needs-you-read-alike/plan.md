# Plan: the idle group is worded 「手すき」

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: done

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

The independent empirical verifier ran all seven obligations (all held) and the
ten mutations (all caught), then found two gaps. First, the guard refused only
待 / 대기 / "waiting" in the English and Korean hint rows, so replacing them with
another state's wording ("Needs you now", 「실행 중입니다」, "Sessions that are
running") survived. The R1 test now pins both hints' exact English and Korean
rows, as R2 already states them. Second, `docs/MANUAL.md` still described Idle
as "Running and waiting for your next request". It was not among R3's five
files, though the English word "Idle" is unchanged. Its state-table row now
reads "Finished its turn and free; the next request is up to you", and R3's test
pins it. Four mutations were added for these cases, and the sweep catches 14 of 14.

Bands: `largest_module_lines` is at 9722 in `src/app.rs`, the same on
`origin/main`. This change does not touch that file.

The first landing attempt was refused. The second commit's verification digest
had been read from the index against that commit's parent, so it covered only
that commit's delta. The landing gate reads the whole branch against main. The
record now carries the digest from `bash scripts/check-review.sh --against
origin/main`, and the code is unchanged.
