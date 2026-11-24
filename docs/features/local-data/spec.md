# Spec: データの保存と復旧

## Behaviour
1. The app data directory is `ProjectDirs("com","local","operon").data_local_dir()` (`~/Library/Application Support/com.local.operon/`), falling back to the relative `.operon` when no home is known. The index is `store-v2.json` in it. Pinned by `current_and_legacy_builds_use_distinct_store_files`.
2. `Store` is the persisted index: `schema_version`, projects, sessions, `pending_cancellations`, `notifications_enabled` (default false), `auto_approve_workspace_prompts` (default true), `theme`, `language`, `terminal_font`, native-session references, CLI handoffs, agent launch presets, `search_engine_mode`, `diff_layout`. Every field added after the first version has a serde default so an older file loads. Pinned by `a_store_without_presets_still_loads`, `saves_the_local_store_atomically_and_loads_it_back`.
3. `STORE_SCHEMA_VERSION` is 5 (5 added `SessionStatus::Lost`). `persist` stamps it on every save. A file with a larger version is refused by `parse_store_contents` (「保存データのスキーマ {schema_version} は、対応スキーマ {STORE_SCHEMA_VERSION} より新しいです」); a file with no version reads as the oldest. Failed records written before schema 5 are re-judged once on load. Pinned by `loads_legacy_store_without_a_schema_field_and_rejects_future_schema`.
4. Every save is `write_file_atomically`: a temporary file `.<name>.tmp-<uuid>` is created beside the target (`create_new`), written, `sync_all`ed, renamed over the target, then the directory is `sync_all`ed. The result is `WriteOutcome::Durable`, or `CommittedButNotSynced(error)` when only the directory sync failed; a failure before the rename removes the temporary file and returns an error with the old file intact. Pinned by `atomic_write_distinguishes_precommit_failure_from_committed_unsynced_data`, `model_check_store_survives_every_interruption_of_a_write`.
5. `CommittedButNotSynced` keeps the change in memory, sets a pending-retry flag and a durability warning 「変更は書き込まれましたが、ディスクに保存されたことを確認できませんでした。Operon が自動的に再試行します。確認が済むまで強制終了しないでください。…」. Every `SESSION_POLL_INTERVAL` (2 s) tick retries `save_store_then_cancellation_intents`; success clears the warning and shows 「保留していたセッションのメタデータを保存しました。」. A plain error shows 「データを保存できませんでした。直前の変更は適用されていません: …」 and callers roll the change back. Pinned by `committed_unsynced_persistence_keeps_memory_and_schedules_a_retry`, `ordinary_store_save_does_not_clear_a_pending_recovery_retry`.
6. Pending stop requests are mirrored to `<store>.pending-cancellations.json`. `save_store_then_cancellation_intents` writes the store first and only then the sidecar, so a failed store write never clears the old recovery copy. Pinned by `failed_store_write_preserves_existing_cancellation_recovery`, `cancellation_intent_survives_restart_before_external_stop_result_is_saved`.
7. At startup `main` runs the instance lock before anything else: `acquire_instance_locks` takes a non-blocking `flock` on `.operon-instance.lock` in the data directory, and also on `.xirp-copy-instance.lock` in the pre-rename data directory when that directory exists. Locks are held for the life of the process. Pinned by `instance_lock_rejects_a_second_process_and_releases_after_crash`, `instance_lock_distinguishes_storage_errors_from_contention`.
8. A held lock shows a modal 「Operon は既に開いています」 with 「もう一方の Operon を閉じてから、もう一度お試しください。」; any other lock or directory failure shows 「Operon はローカルデータを開けませんでした」 with 「データフォルダの権限と空き容量を確認してから、もう一度お試しください。」. In both cases the process exits without opening a window.
9. After locking, `import_legacy_store_if_available` considers, in order, `xirp-copy/store-v2.json`, `com.local.operon/store.json`, `xirp-copy/store.json`. The first candidate that exists decides the import and the rest are not consulted. It only runs when `store-v2.json` does not exist. Pinned by `the_newest_legacy_snapshot_wins_and_none_of_them_are_modified`, `imports_the_legacy_store_once_without_modifying_its_snapshot`.
10. An import copies the legacy bytes and rebuilds the sidecar from the legacy sidecar plus the legacy store's own pending list (de-duplicated). The legacy files are never modified. The person gets 「既存のセッションメタデータを保護された現在の記録に取り込みました。…」. Pinned by `malformed_legacy_sidecar_is_preserved_and_rebuilt_for_the_current_store`, `imports_the_legacy_store_once_without_modifying_its_snapshot`.
11. At each load, `load_store_safely` returns the store, or on any read or parse error (including a newer schema) renames the file to `store.corrupt-<time>-<uuid>.json`, starts with an empty `Store` and a notice 「保存データを読み取れなかったため、{p0} に保全しました。Operon は空の記録で起動しています。…」. Pinned by `preserves_an_unreadable_store_before_starting_empty`.
12. Settings → 「データと復旧」 shows the data file path and 「データフォルダを表示」 (`open -R` in Finder); 「tmux をスキャン」 lists tmux sessions not in the store; 「セッションを引き継ぐ」 attaches one to the registered project containing its working directory. Pinned by `recovery_scan_failure_is_reported_instead_of_as_zero_results`, `recovery_scan_distinguishes_no_server_from_a_real_tmux_failure`, `adopts_a_recovered_terminal_result_without_blocking_the_ui`.
13. Other files in the directory (owned by their features): `store-v2.json.pending-cancellations.json`, `recent-agent-settings.json`, `session-order.json`, `agent-accounts.json`, `keybindings.json`, `diff-comments.json`, `setup-trust.json`, `approved-scripts/`, `agent-hooks/`, `shared-sessions/`, `knowledge/`, `models/`, `.operon-instance.lock`.
14. Every child process goes through `src/exec.rs`: `run_command_with_timeout` caps stdout at `COMMAND_OUTPUT_MAX_BYTES` (8 MiB) and stderr at `COMMAND_ERROR_MAX_BYTES` (256 KiB); overflowing stdout is an error 「コマンド出力が安全上限の {p0} KiB を超えました」, overflowing stderr is truncated with a marker. `run_command_cancellable` reports `CommandCancelled` when stopped.
15. `reveal_path`, `open_path` and `open_url` in `src/sys.rs` run `open` through `run_system_command`; notifications use the native macOS API with an AppleScript fallback when the app is not bundled.

## States
- Store file absent: `Store::default()`.
- Store unreadable or newer schema: moved aside as `store.corrupt-*.json`, empty start, notice names the new path. The file is kept byte for byte.
- Store unreadable and the rename fails: the original is not overwritten; new changes go to `store.recovery-<time>-<uuid>.json` and the notice says so.
- Legacy snapshot malformed or from a newer schema: the legacy file is left untouched; a safe empty current store and sidecar are written and a notice says the import was skipped. A malformed legacy sidecar is rebuilt from the store's pending list, with a notice.
- Import written but not confirmed durable: a retry is scheduled and 「取り込んだメタデータをディスクに保存できたか、まだ確認できていません。…」 shows.
- Import fails (permission, space): the modal 「Operon は既存データを取り込めませんでした」 is shown, the legacy data is unchanged, and the app exits.
- Accounts file over 64 KiB or unparseable: treated as empty. Keymap file over 64 KiB: defaults with a notice.
- tmux scan: no server running is a normal empty result; a real tmux failure is reported, not shown as zero results.
- Cancellation with neither recovery copy confirmed durable waits; one durable copy is enough to allow stop and repair the other.

## Invariants
- No telemetry, accounts or cloud calls; data leaves the Mac only through agents the person runs.
- A write is all-or-nothing as seen by a reader of the target path: the target is replaced by rename, never edited in place.
- The newest generation a save reported `Durable` (or that existed at launch) survives any crash; a newer-schema index stays byte for byte on disk (in place, as a legacy snapshot, or moved aside). The model check enumerates crashes between the five write steps.
- The store is written before the cancellation sidecar; the sidecar is never cleared before the store is durable.
- A stored enum variant (`AppTheme`, `Language`, `TerminalFont`, `SessionStatus`) is never renamed or removed; a schema bump accompanies any variant an older build cannot read.
- One process per data directory, and also one per legacy directory that exists; the lock is non-blocking and fails closed.
- Legacy snapshots are only read, never changed, so the pre-rename build remains a rollback path.
- Scan and output limits (8 MiB stdout, 256 KiB stderr per command; 64 KiB accounts and keymap files; 5 000 directory entries per folder-picker listing) are fixed constants in `src/config.rs`.
- A raw `.output()` is not used; every external tool has a timeout.

## Code and tests
- `src/store.rs::Store`, `store_schema_version`, `default_auto_approve_workspace_prompts`: the persisted record and its defaults.
- `src/store.rs::app_data_directory`, `app_data_file`, `legacy_brand_data_directory`, `legacy_app_data_files`: locations.
- `src/store.rs::load_store`, `parse_store_contents`, `load_store_safely`: read, version guard, preserve-and-start-empty.
- `src/store.rs::save_store`, `write_file_atomically`, `write_file_atomically_with_sync`, `WriteOutcome`: the atomic write and its two success kinds.
- `src/store.rs::save_store_then_cancellation_intents`, `cancellation_intents_file`, `load_cancellation_intents`, `save_cancellation_intents`: the sidecar.
- `src/store.rs::import_legacy_store_if_available`, `import_first_legacy_store`, `import_legacy_store_if_needed_with_writer`, `MigrationOutcome`: the one-time import.
- `src/store.rs::acquire_instance_locks`, `acquire_instance_lock`, `lock_file_exclusively`, `InstanceLockError`: single-instance flock.
- `src/store.rs::ToolStatus`: PATH-based tool detection without running anything.
- `src/models.rs`: persisted `Project`, `Session`, `SessionStatus`, `OrphanedTmuxSession` and related types.
- `src/config.rs::STORE_SCHEMA_VERSION`, `EVIDENCE_BASED_FAILURE_SCHEMA_VERSION`, `COMMAND_OUTPUT_MAX_BYTES`, `COMMAND_ERROR_MAX_BYTES`, `AGENT_ACCOUNTS_FILE_MAX_BYTES`, `KEYMAP_FILE_MAX_BYTES`, `FOLDER_PICKER_ENTRY_LIMIT`: versions and ceilings.
- `src/exec.rs::run_command_with_timeout`, `run_command_with_output_limit`, `run_command_cancellable`, `CommandCancelled`: bounded child processes.
- `src/sys.rs::reveal_path`, `open_path`, `open_url`, `send_macos_notification`: platform calls.
- `src/app.rs::retry_pending_store`, `apply_persist_result`, `scan_orphaned_tmux_sessions`, `adopt_orphaned_tmux_session`: retry and recovery actions.
- `src/tmux.rs::list_operon_tmux_sessions`: the tmux listing behind the scan.
- Tests: `saves_the_local_store_atomically_and_loads_it_back` pins save and reload.
- Tests: `atomic_write_distinguishes_precommit_failure_from_committed_unsynced_data` pins the two failure kinds.
- Tests: `committed_unsynced_persistence_keeps_memory_and_schedules_a_retry` pins the retry flag.
- Tests: `preserves_an_unreadable_store_before_starting_empty` pins the corrupt-file preservation.
- Tests: `loads_legacy_store_without_a_schema_field_and_rejects_future_schema` pins the version guard.
- Tests: `a_store_without_presets_still_loads` pins field defaults.
- Tests: `instance_lock_rejects_a_second_process_and_releases_after_crash`, `instance_lock_distinguishes_storage_errors_from_contention` pin the lock.
- Tests: `imports_the_legacy_store_once_without_modifying_its_snapshot`, `current_and_legacy_builds_use_distinct_store_files`, `the_newest_legacy_snapshot_wins_and_none_of_them_are_modified` pin the import order and read-only sources.
- Tests: `malformed_legacy_import_is_preserved_without_changing_its_source`, `malformed_legacy_sidecar_is_preserved_and_rebuilt_for_the_current_store`, `future_legacy_schema_is_preserved_and_does_not_block_current_startup`, `partial_legacy_import_retries_without_modifying_the_legacy_snapshot`, `committed_unsynced_legacy_import_requests_an_automatic_retry` pin import failure paths.
- Tests: `cancellation_waits_when_neither_recovery_copy_is_confirmed_durable`, `one_durable_cancellation_copy_is_enough_to_allow_stop_and_repair_the_other`, `durable_cancellation_reconfirmation_clears_a_stale_warning`, `failed_store_write_preserves_existing_cancellation_recovery`, `ordinary_store_save_does_not_clear_a_pending_recovery_retry`, `cancellation_intent_survives_restart_before_external_stop_result_is_saved`, `cancellation_sidecar_survives_a_previous_store_read_modify_write` pin the sidecar.
- Tests: `model_check_store_survives_every_interruption_of_a_write` pins the durability invariant over every interruption point.
- Tests: `recovery_scan_failure_is_reported_instead_of_as_zero_results`, `recovery_scan_distinguishes_no_server_from_a_real_tmux_failure`, `adopts_a_recovered_terminal_result_without_blocking_the_ui` pin the tmux recovery.

## Decisions inherited
- Change 243: the store's write is modelled step by step and checked against every crash point, with the model reading the write order from `src/store.rs`; the rule is that no interruption may lose a generation reported durable.
- Change 045: resuming after an abrupt end must validate the saved conversation first (missing or truncated transcript, `/clear` rotating ids) rather than run the native resume blindly.
- Change 123: a tmux "error connecting to" message means the server is gone, so a session can still be stopped or deleted and background resizes stop failing.
- Change 291: killed or abnormally ended sessions are restored automatically from the saved conversation id in the same tab, with a cool-down against crash loops (15 s); what cannot be restored is shown as needing attention.
- Change 303: a missing tmux pane is recovered or replaced by a terminal rather than looping read errors.
- Change 270: the workspace's tabs are written again on exit and on layout change, so a relaunch finds them.
- Change 142: the semantic-search model is built in and opt-in, stored under `models/`, with no external daemon.
- Change 307: the store files keep layout and projects through `load_store_safely`, pinned by a model check.
