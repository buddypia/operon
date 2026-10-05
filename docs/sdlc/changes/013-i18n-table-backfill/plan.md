# Plan: batch-1 translation rows for the priority screens

- **Spec**: `./spec.md`
- **Approved**: 2026-09-04
- **Status**: in progress

This is the plan produced before any source file was edited. Departures go in
the section at the bottom, not by rewriting the sections above.

## Files that change

| File | Change |
|---|---|
| `src/i18n_tables.rs` | +100 EN rows, +100 KO rows in byte order (`設定` already has both) |
| `docs/sdlc/changes/013-i18n-table-backfill/state.yaml` | readiness Go, contract |

## Order of work

1. Commit this plan (docs only, no gates).
2. Add the EN rows in one edit, the KO rows in one edit, each in byte order.
   Verify order with a byte-sort check before running the suite.
3. Run the three gates. The sort/placeholder/coverage-table tests are the
   proof; eyeball EN and KO in the running app if it is open.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A row out of byte order is silently unreachable | one string stays Japanese under EN/KO | `every_translation_table_is_sorted_by_message_id` |
| A renamed/dropped placeholder renders literally | `{name}` on screen | `every_translation_keeps_the_placeholder_names_of_its_message_id` |
| A mistranslation reads as product quality | a person reads it | nothing automated; kept small (101 ids) and flagged for native read-through |
| An id missed from the batch | still Japanese under EN/KO | eyeballing; full coverage guard lands in the last batch |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — green, same count as before batch 1.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `git diff --stat` — only `src/i18n_tables.rs` plus the change directory.
- Running app: EN and KO read correctly on the priority screens.

## Departures from the plan

None yet.
