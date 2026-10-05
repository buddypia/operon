# Intent: Antigravity CLI session restore into Codex CLI fails to restore conversation history

- **Status**: done
- **Opened**: 2026-09-14

## Problem

When attempting to restore an Antigravity CLI (`agy`) session's conversation history into Codex CLI, the restore fails or opens as an empty Codex session showing `› Ask Codex to do anything` without any of the restored conversation history.

Two distinct bugs cause this failure:

1. **Missing Gemini Native Session Path Lookup in `src/cli.rs`**:
   In `native_session_path_for_project`, `CliProvider::Gemini` always returned `None`. When restoring a managed Antigravity session whose `source.native_session_path` is not directly populated, `handoff_managed_session` in `src/app.rs` falls back to `native_session_path_for_project(source_provider, workspace, &native_session_id)`. Returning `None` causes immediate failure with:
   `元のネイティブ会話ログが見つかりません。完全復元ではログの全記録が必要です。`

2. **Codex CLI v0.154.0+ Rollout Replay Protocol Drift in `src/history.rs`**:
   Operon previously generated Codex replay records as legacy `event_msg` items with payloads of type `user_message` and `agent_message`.
   In modern Codex CLI (v0.154.0+), the Codex TUI / app-server projection engine (`codex_tui::thread_transcript` projecting into `thread_history_1.sqlite` `thread_turns` and `thread_items`) ignores these legacy payload types and only indexes turns when structured with:
   - `{"type": "task_started", "turn_id": "...", "started_at": ..., "model_context_window": 258400, "collaboration_mode_kind": "default"}`
   - `{"type": "item_completed", "thread_id": "...", "turn_id": "...", "item": {"type": "UserMessage", "id": "...", "content": [{"type": "text", "text": "..."}]}}`
   - `{"type": "item_completed", "thread_id": "...", "turn_id": "...", "item": {"type": "AgentMessage", "id": "...", "content": [{"type": "Text", "text": "..."}], "phase": "final_answer"}}`
   - `{"type": "task_complete", "turn_id": "...", "last_agent_message": "...", "started_at": ..., "completed_at": ...}`
   Because Operon did not emit these modern replay events, Codex resumed with 0 turns in its projection database, leaving the terminal conversation empty.

## Who feels it, and when

Any user who switches or hands off a conversation from Antigravity CLI to Codex CLI in Operon, or who attempts to restore a saved Antigravity session into Codex CLI.

## Desired outcome

1. Restoring a managed or discovered Antigravity CLI session into Codex CLI successfully locates the native transcript, completes the restore, and when `codex resume <id>` starts in the native terminal, the full restored conversation is immediately rendered on screen.
2. The ignored e2e tests `a_saved_antigravity_conversation_restores_into_a_resumable_codex_terminal` and `codex_app_server_restores_a_short_transcript_into_a_resumable_terminal` both pass cleanly.
3. Unit tests verifying rollout replay generation and ordinal continuity pass.

## Systems likely affected

- `src/cli.rs`: `native_session_path_for_project` for `CliProvider::Gemini`.
- `src/history.rs`: `add_codex_replayable_history`, `codex_replay_turn_events`, `verify_codex_replayable_history`.
- `src/tests.rs`: unit tests for codex replay records and e2e restore tests.
