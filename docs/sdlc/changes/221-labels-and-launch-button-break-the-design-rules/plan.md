# Plan: the sub-tabs, the idle group and the launch button say what the product's documents promise

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | four new tests; two existing assertions follow the new idle word (`every_status_label_is_one_of_the_four_group_words`, the message in the group-pinning test) |
| `src/app.rs` | `GitView::label` returns `tr(…)` ids |
| `src/ui/widgets.rs` | `SessionStatusGroup::message_id` for `Free` |
| `src/app/screens.rs` | launch button through `primary_button` |
| `src/i18n_tables.rs` | rows for 「変更ファイル」, 「コミット履歴」, 「指示待ち」; drop 「待機中」; sorted |
| `README.md:108`, `README.ja.md:22,99`, `README.ko.md:22,97`, `docs/MANUAL.ja.md:202,205,435`, `docs/MANUAL.ko.md:201,204,436` | the idle filter's word; `MANUAL.ja.md:152-153` and `MANUAL.ko.md:150-151` mean queued sessions and stay |
| `docs/sdlc/changes/221-…/` | this change's record |

## Order of work

1. Write the four tests; run them and watch each fail for its stated reason
   (ASCII labels; the idle word is still 「待機中」; the fill behind the button is
   the vendor colour; the documents still say the old word).
2. Table rows and the label/word edits, then re-run: tests 1 and 2 pass. The
   tree compiles between every step.
3. Launch button through `primary_button`: test 3 passes.
4. Documents: test 4 passes. It reads the named lines by the phrase around the
   word, so the queued sense in `MANUAL.ja.md:152-153` and `MANUAL.ko.md:150-151`
   neither trips nor is lost.
5. `cargo test --locked` on the filters the spec names, `make q.fast`.
6. Look at the running app at the three places in `screen.md`; fix what clips.
7. Mutations, review, close.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A table row out of order or left behind | a label silently untranslated | `every_translation_table_is_sorted_by_message_id`, `every_message_id_has_a_row_in_every_table` |
| 「指示待ち」 wider than the 104px status column | clipped word on a row | the screen check in step 6 |
| The disabled launch button reads as an enabled one | a person presses a dead button | the existing reason line above it; checked in step 6 |
| Another place restating 「待機中」 as the idle word | two words for one state | grep in step 5, `the_idle_group_word_is_not_the_queued_session_word` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked <the four new names>` — `test result: ok. 4 passed`, each failed before.
- `cargo test --locked every_translation every_status every_message_id every_source` — ok.
- `cargo clippy --locked -- -D warnings` — clean (runs in the commit gate).
- The running app shows the words and the orange button as in `screen.md`.

## Departures from the plan

- `the_launch_button_sits_at_the_sheets_bottom_right` in `src/tests.rs` was also updated to include `{ICON_START}  ` in the expected button label, matching `primary_button`'s formatting.
