# Plan: Cross-CLI handoff synthesizes a compact handoff bundle instead of importing full history

- **Spec**: `./spec.md`
- **Approved**: 2026-10-19
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `HANDOFF_BUNDLE_MAX_TURNS`, `HANDOFF_BUNDLE_TURN_MAX_CHARS`, `HANDOFF_BUNDLE_MAX_BYTES`, and `HANDOFF_BUNDLE_FILE_NAME` constants for the compact handoff bundle. |
| `src/cli.rs` | Add `handoff_bundle: String` to `ImportedNativeSession`. |
| `src/history.rs` | Replace full-history destination store synthesis in `import_full_cli_history` (and add `import_full_cli_history_with_note`) with source transcript archiving + `build_compact_handoff_bundle` + `handoff-bundle.md` and `manifest.json` writing. Gate legacy store-synthesis helpers behind `#[cfg(test)]` so existing unit tests compile cleanly with zero dead-code warnings. |
| `src/agents.rs` | Route `handoff_managed_session` through `import_full_cli_history_with_note` (including the user's optional `note`), launching the destination session with the compact handoff bundle and `full_history: false`. |
| `src/app.rs` | Update `finish_full_history_import`, `restore_progress_title`, and `restore_progress_detail` to launch the destination CLI with `imported.handoff_bundle` as its initial prompt, `full_history: false`, `native_session_path: None`, and compact Handoff bundle progress text. |
| `src/app/screens.rs` | Update `render_restore_options` and `ui_cli_sessions` badges, button labels, tooltips, and confirmation texts from full-history import to compact Handoff bundle. |
| `src/ui/widgets.rs` | Update `restore_progress_footer` copy to describe snapshot + compact Handoff bundle creation instead of full-history store conversion. |
| `src/i18n_tables.rs` | Add English and Korean translations for the new compact Handoff bundle UI strings. |
| `README.md` | Update English documentation for Cross-CLI Session Handoff to describe the compact Handoff bundle and removal of full-history import. |
| `README.ja.md` | Update Japanese documentation for Cross-CLI Session Handoff to describe the compact Handoff bundle and removal of full-history import. |
| `README.ko.md` | Update Korean documentation for Cross-CLI Session Handoff to describe the compact Handoff bundle and removal of full-history import. |
| `src/tests.rs` | Add unit tests for `build_compact_handoff_bundle`, `import_full_cli_history_with_note`, `finish_full_history_import`, and `handoff_managed_session`, and update restore modal and live E2E handoff tests for the compact Handoff bundle. |

## Order of work

Numbered, so that the tree compiles between steps wherever it can. Say where it
cannot.

1. Add `HANDOFF_BUNDLE_MAX_TURNS`, `HANDOFF_BUNDLE_TURN_MAX_CHARS`, `HANDOFF_BUNDLE_MAX_BYTES`, and `HANDOFF_BUNDLE_FILE_NAME` in `src/config.rs`, and add `pub(crate) handoff_bundle: String` to `ImportedNativeSession` in `src/cli.rs` (tree does not compile until steps 2-4 populate `handoff_bundle`).
2. Update `src/history.rs` to implement `build_compact_handoff_bundle` and `import_full_cli_history_with_note`, write `handoff-bundle.md` and `manifest.json` (`"restore": "compact handoff bundle"`) atomically into `shared-sessions/<import_id>/`, return `ImportedNativeSession` with `handoff_bundle` populated, and gate unused destination-store synthesis helpers with `#[cfg(test)]`.
3. Update `src/agents.rs` (`handoff_managed_session`) and `src/app.rs` (`finish_full_history_import`, `restore_progress_title`, `restore_progress_detail`) so both `履歴` and managed-session handoffs launch a fresh destination CLI session with `imported.handoff_bundle` as its initial prompt, `CliHandoff.full_history = false`, and `Session.native_session_path = None` (tree compiles again after updating test struct literals in step 5).
4. Update `src/app/screens.rs`, `src/ui/widgets.rs`, `src/i18n_tables.rs`, `README.md`, `README.ja.md`, and `README.ko.md` so all UI surfaces and docs describe the compact Handoff bundle.
5. Update `src/tests.rs` with new unit tests and updated restore modal / E2E handoff assertions, run named tests, `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `bash scripts/check-bands.sh`, and run `mutations.py` to verify test sensitivity.

## Risks

What could break that the compiler will not catch. For each one, what makes it
visible.

| Risk | How it shows up | What catches it |
|---|---|---|
| `Session.goal` carrying the full compact handoff bundle could pollute sidebar session titles or re-nest when handed off a second time | Session cards show multi-line bundle headers or repeated handoffs nest `Task to continue:` blocks | `original_task_from_session_goal` (`src/agents.rs:544`) strips `("Task to continue:\n", "\n\nLatest terminal context")`; unit test asserts `session_display_goal(&session.goal)` equals the original task and second handoff does not nest |
| Storing `handoff-bundle.md` in `Session.native_session_path` would cause a subsequent resume or handoff to read the markdown bundle as if it were a CLI transcript | Subsequent handoff fails or reads the wrong file | `finish_full_history_import` and `handoff_managed_session` set `Session.native_session_path = None` when launching from a compact handoff bundle; unit test asserts `session.native_session_path.is_none()` |
| Multibyte UTF-8 characters (Japanese/Korean) in transcript turns could panic if sliced by byte index or exceed `HANDOFF_BUNDLE_MAX_BYTES` | Panic on char boundary or bundle larger than 6,000 bytes | `truncate_chars` uses `.chars()` iteration and byte-budget loop checks `bundle.len() <= HANDOFF_BUNDLE_MAX_BYTES`; unit test with Japanese turns verifies char safety and byte cap |

## Proof of completion

The commands and observations that say this is done. Include the output that
counts as healthy, not just the command.

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked compact_handoff_bundle` — `test result: ok. N passed; 0 failed`.
- `cargo test --locked full_history_restore_persists_a_resumable_session_before_launching_it` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked the_restore_modal_` — `test result: ok. 2 passed; 0 failed`.
- `bash scripts/check-bands.sh` — exits 0.

## Departures from the plan

Filled in during implementation. What changed, and why the plan was wrong.
