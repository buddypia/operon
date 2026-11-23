# Spec: 履歴と CLI をまたぐ Handoff

## Behaviour
1. Discovery: 「ローカル CLI セッションを検出」 runs `scan_native_cli_sessions(project, 80)` in the background over Codex (`~/.codex/sessions` and its index), Claude Code (`~/.claude/projects`) and Antigravity (history index, process logs, trajectory store). Sessions belong to the project or its worktrees only. Each card shows the conversation id, branch, and last user request. Pinned by `discovers_local_claude_cli_sessions_for_a_project`, `discovers_native_codex_sessions_only_for_the_project`, `discovers_antigravity_sessions_with_conversation_ids`.
2. A card offers 「元の CLI で再開」 (native resume of that conversation) or a 「復元先」 choice with optional model and effort (remembered per CLI), a 「Handoff メモ（任意）」, and 「Handoff バンドルで引き継ぐ」. Choosing the same CLI is labelled 「〈エージェント名〉（同じ CLI に複製）」 and starts a second conversation without touching the source. No destination is preselected. Pinned by `full_history_restore_offers_only_destinations_that_can_hold_the_history`, `a_handoff_starts_its_destination_on_the_chosen_model_and_effort`.
3. A session row's `···` menu lists 「〈エージェント名〉 へ Handoff」 for every supported CLI other than the session's own, labelled with the destination's configured model and effort (`handoff_menu_label`); disabled while an import runs. Pinned by `the_session_menu_names_the_handoff_model_and_effort`.
4. A handoff never writes into the destination CLI's own store. `import_full_cli_history_with_note` validates the destination launch options first (a refused option costs no archive), snapshots the source transcript byte for byte, builds the bundle, and the destination starts a new session whose first message is the bundle. Pinned by `import_full_cli_history_writes_handoff_bundle_and_manifest_without_touching_destination_store`, `shared_history_snapshot_preserves_the_source_jsonl_byte_for_byte`.
5. Bundle (`build_compact_handoff_bundle`) holds: the task (first user turn, else title or last request), source and destination CLI and the snapshot path, up to `HANDOFF_BUNDLE_MAX_TURNS` (8) recent User/Assistant turns of at most `HANDOFF_BUNDLE_TURN_MAX_CHARS` (1,200) characters each, up to `HANDOFF_TOOL_RESULTS_MAX` (3) latest tool results of at most `HANDOFF_TOOL_RESULT_MAX_CHARS` (1,000) characters (error and failure lines kept, repeats counted), and the person's note last. Total under `HANDOFF_BUNDLE_MAX_BYTES` (6,000). Pinned by `compact_handoff_bundle_bounds_turns_and_bytes_and_strips_tool_spam`, `a_long_handoff_keeps_the_persons_instruction_whole`, `a_handoff_carries_the_latest_tool_results_shortened`.
6. Over budget, what gives way, in order: older excerpt turns; tool results oldest first; the last turn's text; the task; the note only when it alone exceeds the budget. Every cut is marked with `HANDOFF_SHORTENED_MARKER` and never splits a marker. Tool results are never presented as User or Assistant turns; reasoning is not carried as native reasoning. Pinned by `a_handoff_gives_up_tool_results_before_the_last_turn_task_or_note`, `a_handoff_cut_never_splits_a_marker_inside_the_text`, `a_shortened_tool_result_keeps_its_error_lines_and_counts_repeats`.
7. Archive: each handoff stores `source.jsonl` (`SHARED_SESSION_SNAPSHOT_FILE_NAME`), `handoff-bundle.md` and `manifest.json` under `shared-sessions/<id>/` beside the store, directories `0700` and files `0600`. A conversation whose fingerprint is unchanged reuses its snapshot and note (a new non-empty note replaces the old one); one that moved on is snapshotted again; a reuse whose snapshot is gone or empty takes a fresh one. Pinned by `the_transcript_archive_is_readable_only_by_its_owner`, `an_unchanged_conversation_handed_off_again_reuses_its_snapshot_and_note`, `a_conversation_that_moved_on_is_snapshotted_again`.
8. Expiry: an archive unused by any handoff for `HANDOFF_ARCHIVE_TTL_SECONDS` (7 days) is deleted at the next launch or next handoff; only compact-handoff archives expire, archives made by earlier full-history restores stay. Sweep, reuse lookup and rewrite share one lock so a sweep cannot delete an archive a handoff is using. Pinned by `handoff_archives_unused_for_the_ttl_are_deleted_and_nothing_else_is`.
9. Restore accounting: every source record is classified by `TRANSCRIPT_VOCABULARY` (`transcript_class`) as conversation, reasoning, or operational; a kind with no row is restored as a counted fragment and named in the notice (`restore_classification_notice`), never dropped silently. The notice promises no marker the text does not carry. Pinned by `the_transcript_vocabulary_is_well_formed`, `an_unclassified_record_is_restored_and_reported`, `the_notices_printed_counts_account_for_every_record_read`.
10. Resume safety: before a native resume the transcript is checked (`check_transcript_resume_readiness`): empty or missing falls back to a fresh start with a notice; a truncated last line is repaired first. Pinned by `resume_refuses_empty_transcript_and_falls_back_to_clean_start`.
11. A managed Claude terminal's own conversation is read from Claude Code's session registry (`~/.claude/sessions/<pid>.json`), which outranks the stored id; Codex and Antigravity are recovered by time and workspace, and an ambiguous Codex match is refused. Pinned by `a_live_managed_claude_terminal_recovers_its_own_conversation_not_an_older_one`, `refuses_to_recover_an_ambiguous_managed_codex_conversation`.
12. 「ローカル履歴」 search (`search_local_transcripts`, background, one at a time) scans recent Codex and Claude transcripts for the words of the query, up to 50 hits. 「ローカル履歴を {count} 件見つけました」 or a limit notice (「一部の履歴は省略されています」) reports the result. 「一致する履歴はありませんでした。」 appears only after a finished search for the text now in the field; while one runs the panel says 「履歴を検索しています…」. Pinned by `zero_match_transcript_search_obeys_shared_byte_and_line_budgets`.
13. 「セッションを検索」 keeps a session when the trimmed, case-insensitive text is contained in its list title, name, request, agent name, stored agent word, project name, or branch; list and grid use the same function.

## States
- No project selected or no sessions found: the list is empty with no card.
- Source transcript unreadable: 「この会話のローカル履歴ファイルを読めないため復元できません: …」, nothing is written.
- Source with no restorable records: 「選択した元履歴には、復元できる記録がありません。」.
- Destination option refused by the launch gate: refused before any archive is made.
- Reused bundle: the finish notice says so.
- Search limits reached (entries, bytes, lines) or unreadable files: the result says history was omitted.
- A search already running: 「ローカル履歴の検索が既に実行中です。」
- Antigravity's trajectory store is read-only (`SQLITE_OPEN_READ_ONLY`), every value passed through `params!`.

## Invariants
- Bundle under 6,000 bytes, 8 turns, 1,200 characters per turn, 3 tool results of 1,000 characters each (`src/config.rs` constants).
- The person's note is cut last and never from its end unless it alone exceeds the budget.
- Creating a bundle runs no model and spends no tokens; the destination store is untouched.
- The source snapshot is a byte-for-byte copy; archive directory mode `0700`, file mode `0600`.
- Transcript scan budgets: `TRANSCRIPT_CANDIDATE_LIMIT` 600 files, `TRANSCRIPT_SEARCH_BYTE_LIMIT` 64 MiB, `TRANSCRIPT_SEARCH_LINE_LIMIT` 500,000, `TRANSCRIPT_FILE_MAX_BYTES` 20 MiB per file, `FILE_SCAN_VISIT_LIMIT` 100,000 entries; the manifest read is capped at `HANDOFF_MANIFEST_MAX_BYTES` and the archive scan at `HANDOFF_ARCHIVE_SCAN_MAX_ENTRIES`.
- A reader keeps no judgement of its own about another CLI's record kinds; `src/transcript.rs` is the only table.
- A conversation id is never inferred from what a terminal looks like when its own CLI publishes it.

## Code and tests
- Symbols:
  - `src/history.rs::import_full_cli_history_with_note`: the handoff pipeline (validate, snapshot or reuse, bundle, manifest).
  - `src/history.rs::build_compact_handoff_bundle`, `shorten_tool_result`, `handoff_tool_results`, `shorten_to_bytes`: bundle assembly and cuts.
  - `src/history.rs::sweep_expired_handoff_archives`, `reusable_handoff_archive`, `transcript_fingerprint`: archive reuse and expiry.
  - `src/history.rs::full_history_restore_destination`, `full_history_restore_transcript`: allowed destinations and source.
  - `src/history.rs::read_transcript_conversation`, `restore_classification_notice`, `RestoreClassification`: classification and its notice.
  - `src/transcript.rs::TRANSCRIPT_VOCABULARY`, `TranscriptClass`, `transcript_class`: the keep/exclude table.
  - `src/cli.rs::scan_native_cli_sessions`, `scan_codex_cli_sessions_in`, `scan_claude_cli_sessions_in`, `scan_gemini_cli_sessions_in`: discovery.
  - `src/cli.rs::search_local_transcripts`, `search_transcript_hit_limited`, `TranscriptSearchLimits`: full-text search with budgets.
  - `src/cli.rs::snapshot_native_transcript`, `shared_session_archive_directory`, `create_private_directory`: the archive on disk.
  - `src/cli.rs::check_transcript_resume_readiness`, `recover_managed_claude_session_in`, `corrected_managed_claude_conversation_in`, `read_claude_session_registry_in`: resume safety and identity.
  - `src/app.rs::handoff_cli_session`: the app entry for both menus.
  - `src/app/screens.rs::handoff_menu_label`: the menu label.
- Tests:
  - `compact_handoff_bundle_bounds_turns_and_bytes_and_strips_tool_spam`: turn, byte and plumbing limits.
  - `a_long_handoff_keeps_the_persons_instruction_whole`: the note survives.
  - `a_handoff_cut_never_splits_a_marker_inside_the_text`: marker integrity.
  - `a_handoff_carries_the_latest_tool_results_shortened`: tool results section.
  - `a_shortened_tool_result_keeps_its_error_lines_and_counts_repeats`: error lines kept.
  - `a_handoff_gives_up_tool_results_before_the_last_turn_task_or_note`: cut order.
  - `a_tool_result_with_blank_lines_never_reads_as_conversation`: tool output is not a turn.
  - `import_full_cli_history_writes_handoff_bundle_and_manifest_without_touching_destination_store`: no destination write.
  - `a_handoff_starts_its_destination_on_the_chosen_model_and_effort`: chosen model and effort.
  - `the_session_menu_names_the_handoff_model_and_effort`: menu label.
  - `an_unchanged_conversation_handed_off_again_reuses_its_snapshot_and_note`: reuse.
  - `a_conversation_that_moved_on_is_snapshotted_again`: fresh snapshot.
  - `handoff_archives_unused_for_the_ttl_are_deleted_and_nothing_else_is`: expiry.
  - `shared_history_snapshot_preserves_the_source_jsonl_byte_for_byte`: byte-for-byte copy.
  - `the_transcript_archive_is_readable_only_by_its_owner`: modes.
  - `full_history_restore_offers_only_destinations_that_can_hold_the_history`: destinations.
  - `restore_never_preselects_the_cli_the_conversation_already_lives_in`: no preselection.
  - `the_transcript_vocabulary_is_well_formed`: table shape.
  - `an_unclassified_record_is_restored_and_reported`: unknown kinds.
  - `the_notices_printed_counts_account_for_every_record_read`: accounting.
  - `discovers_local_claude_cli_sessions_for_a_project`: Claude discovery.
  - `discovers_native_codex_sessions_only_for_the_project`: Codex discovery.
  - `discovers_antigravity_sessions_with_conversation_ids`: Antigravity discovery.
  - `zero_match_transcript_search_obeys_shared_byte_and_line_budgets`: search budgets.
  - `resume_refuses_empty_transcript_and_falls_back_to_clean_start`: empty transcript.
  - `a_live_managed_claude_terminal_recovers_its_own_conversation_not_an_older_one`: registry identity.
  - `refuses_to_recover_an_ambiguous_managed_codex_conversation`: ambiguity.

## Decisions inherited
- Change 003: what to carry from another CLI's records is one table (`TRANSCRIPT_VOCABULARY`), because a restore had silently carried under half of a conversation.
- Change 061: a record with no row is restored and counted, not dropped, and the notice no longer claims fidelity nothing checks.
- Change 004: a terminal's conversation comes from Claude Code's own registry, not inference.
- Change 045: validate and repair the transcript before any native resume.
- Change 179: handoff builds a compact bundle plus a byte-for-byte source snapshot and does not synthesize history into the destination's store.
- Change 180: the destination's model and effort are chosen and remembered; unchanged conversations reuse their archive; archives expire after 7 days.
- Change 185: a long handoff keeps the person's instruction whole and says where Operon cut.
- Change 187: a cut never leaves a marker fragment.
- Change 193: the last tool results travel, shortened with errors kept, as a separate section and never as conversation.
- Change 310: session search reads what the row shows; "no match" is said only after a finished history search.
