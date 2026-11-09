# Plan: the app's wording is rewritten where it reads as machine-made

- **Spec**: `./spec.md`
- **Approved**: 2026-11-10
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | R1's and R5's tests move to the new words; R2's and R3's tests are new; assertions quoting a re-keyed id follow it |
| `src/i18n_tables.rs` | the 188 messages of `wording.md`; both tables re-sorted by message id |
| `src/ui/widgets.rs` | the idle group id, the idle chip hint, the idle tab hint |
| `src/app.rs`, `src/app/screens.rs`, `src/app/knowledge.rs`, `src/history.rs`, `src/cli.rs`, `src/files.rs`, `src/git.rs`, `src/agents.rs`, `src/agents/accounts.rs`, `src/store.rs` | the re-keyed Japanese literals at their call sites, nothing else |
| `README.md`, `README.ja.md`, `README.ko.md`, `docs/MANUAL.md`, `docs/MANUAL.ja.md`, `docs/MANUAL.ko.md` | the reworded labels they quote; the idle state-table rows |
| `docs/sdlc/changes/274-ui-wording-reads-machine-made/` | this change's record, `wording.md`, `verification.yaml`, `mutations.py` |

## Order of work

1. Write R2's and R3's tests and move R1's and R5's to the new words; run them
   and watch all four fail on the unchanged tables.
2. Apply `wording.md` with one script: replace each re-keyed id as an exact
   string literal (`"old"` → `"new"`) across `src/`, set the English and Korean
   rows, re-sort each table by the unescaped message id, and refuse to run if a
   new id collides with an existing one or a placeholder set differs.
3. Change the documents; R5 passes.
4. Run the named tests, the table invariants, and the two layout tests; fix any
   test that quoted part of a reworded message.
5. Mutation sweep, `make q.fast`, the store model and the launch-gate proof
   that the touched surfaces ask for.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| a re-keyed id left behind at a call site | a raw Japanese id shown in English or Korean | `every_message_the_code_shows_has_a_row_in_every_table`, `every_message_row_is_one_the_code_shows` |
| a row out of order after re-keying | its translation is silently skipped | `every_translation_table_is_sorted_by_message_id` |
| a placeholder renamed or dropped | a literal `{name}` on screen | `every_translation_keeps_the_placeholder_names_of_its_message_id`, the script's own check |
| a test quoting part of an old message | a red test unrelated to behaviour | step 4's run of the named tests, then CI's full suite |
| a longer label clipped | a chip or tab cut off | the sidebar fit loop over the four group words; labels capped at +2 (ja/ko) / +4 (en) characters |
| a store or durability notice that promises more or less than before | a person told data is lost when it is on disk | the spec's durability row; rows 8, 9, 218, 467, 536, 942 keep "could not confirm", none says "failed" |
| a translation drifting from the Japanese meaning | two languages say different things | `wording.md` read by the evaluator against the Japanese |
| a store, launch-gate or git source edited beyond its literals | a behaviour change hidden in a wording diff | the diff of those files is string-literal lines only, checked before commit; the store model and launch-gate proof re-run |

## Proof of completion

- The acceptance commands in `spec.md` pass.
- `mutations.py` catches every mutation.
- `cargo test --locked model_check_store_survives_every_interruption_of_a_write`
  passes, and CI's verify job proves
  `proof_safe_agent_command_admits_no_line_break`.

## Departures from the plan
