# Plan: the store has no model of an interrupted write

- **Spec**: n/a (`refactor` route — the gap is in `./state.yaml`)
- **Approved**: 2026-11-04
- **Status**: approved

## What the model is

`model_check_store_survives_every_interruption_of_a_write` in `src/tests.rs`,
through `exhaustive_check`. A finite model of one Mac's application-support
directory and one Operon process:

| Part | Values |
|---|---|
| What the Mac holds at first launch | the current index or a legacy snapshot, at every schema 1 (versionless) through `STORE_SCHEMA_VERSION + 1` (newer than this build) |
| Files | inodes with a visible and a durable content: empty, whole, or torn; the directory's names, visible and durable |
| A save | the five steps of `write_file_atomically_with_sync` — create the temporary file, write, sync it, rename, sync the directory — **in the order read out of `src/store.rs`**, so moving a step there moves it in the model |
| Interruptions | a crash between any two steps that keeps either view of the names and, for every file, its visible, its durable, or a torn copy of its unsynced bytes; a directory sync that fails (`CommittedButNotSynced`) |
| Start-up | `import_legacy_store_if_needed` when there is no current index, then `load_store_safely`; the bytes go through the real `parse_store_contents` |
| Bound | two saves; crashes are unbounded, since a crash keeps only the files a name reaches |

The invariant, stated apart from the code:

1. after any crash, the index loaded holds at least the newest generation a
   save reported `Durable` (or that was on disk before Operon started);
2. an index from a newer build stays on disk byte for byte — in place, as the
   legacy snapshot, or moved aside.

## Files that change

| File | Change |
|---|---|
| `src/tests.rs` | the model and its helpers; nothing else |
| `docs/sdlc/verification-methods.yaml` | no change: `store` already asks for `formal-model` |
| `docs/sdlc/changes/243-…/` | `state.yaml`, this plan, `approvals.log`, `verification.yaml`, `mutations.py` |

## Order of work

1. Write the model; it passes on `src/store.rs` as it is.
2. Mutate the code, not the model: rename before the file sync, rename before
   the write, drop the directory sync's place after the rename, make
   `parse_store_contents` accept a newer schema, make `load_store_safely`
   delete an unreadable index instead of moving it. Each must turn the model red
   with a short counterexample.
3. Record the obligations; `formal-model` for the model itself.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The model passes because it cannot reach the bad state | a mutation in step 2 survives | the sweep; a lower bound on states visited |
| The step order is read from the wrong function | a reordering in `src/store.rs` does not move the model | the marker search is bounded to the function's body; the reorder mutations |
| The state space explodes | the walk hits its ceiling and refuses | `exhaustive_check` refuses past `max_states`; the bound is two saves |
