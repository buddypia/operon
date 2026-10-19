# Intent: Cross-CLI handoff synthesizes full conversation history into destination stores

- **Status**: approved
- **Opened**: 2026-10-19

## Problem

> 「コンパクトなHandoffバンドルの方が確かに保守・運用も考えるとその方向性が良い。それで進めて。現状「会話履歴の完全インポート機能」は廃止で」

Today, handing a conversation from one CLI to another (or duplicating it in the
same CLI) reads every turn from the source transcript and synthesizes a fake
native conversation inside the destination CLI's own internal storage — injecting
turns through Codex's app-server and rewriting its rollout log, building a
binary protobuf trajectory SQLite database and history index row for Antigravity,
or fabricating a JSONL conversation file under Claude Code's project store — and
then launches the destination CLI in resume mode on that synthesized record.

Because Operon does not own those three internal storage schemas, any upstream
change in Codex, Claude Code, or Antigravity's local session format or RPC breaks
cross-CLI handoff or risks corrupting the destination CLI's store. In addition,
carrying an entire raw transcript across CLIs floods the new session's context
window with old intermediate turns and foreign tool-call formatting when all the
receiving agent needs to continue the work is the task goal, optional handoff
note, workspace context, and a compact summary of recent conversation turns
backed by the local snapshot on disk.

## Who feels it, and when

- A person switching from one coding CLI to another (for example, moving from
  Claude Code to Codex CLI when a rate limit is hit or when seeking a second
  opinion, or handing a discovered conversation in `履歴` or a managed session in
  the session list to another CLI) who needs the receiving CLI to start cleanly
  with the task and recent context rather than replaying a synthesized
  multi-hundred-turn store file.
- Maintainers of Operon whenever Codex CLI, Claude Code, or Antigravity CLI
  changes its private conversation storage format or app-server protocol.

## Desired outcome

- Initiating a cross-CLI (or same-CLI) handoff from `履歴` or from a managed
  session row no longer synthesizes conversation files inside the destination
  CLI's private store (`~/.codex`, `~/.claude/projects`, or
  `~/.gemini/antigravity/conversations`).
- Instead, Operon snapshots the source transcript into its own local
  `shared-sessions/<id>/` archive, writes a human-readable `handoff-bundle.md`
  and `manifest.json` beside it, and starts a fresh native session in the
  chosen destination CLI whose initial interactive prompt is a bounded, compact
  Handoff bundle containing:
  1. the original task to continue (peeled so repeated handoffs never nest
     handoff envelopes),
  2. the user's optional handoff note when provided,
  3. workspace, branch, source CLI/session identity, and the local path to the
     saved snapshot and bundle, and
  4. the most recent user and assistant turns from the source conversation
     within a strict turn and byte budget, without raw tool-call or tool-result
     dumps.
- The UI in `履歴`, the session row menu, the progress modal, and completion
  notices describe the action as a compact Handoff bundle rather than a
  zero-token full-history store import, while existing `CliHandoff` records in
  `store.json` continue to load and display accurately.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese (with matching `EN_TABLE` and `KO_TABLE` rows in
  `src/i18n_tables.rs`); code, comments, and docs are English.
- The persisted store (`src/store.rs`, `src/models.rs`) is on a `paused` surface
  and is on real machines — keep `Store`, `Session`, `SessionOrigin`, and
  `CliHandoff` schema-compatible without touching `src/store.rs` or
  `src/models.rs`.
- Transcript reading and classification (`read_transcript_conversation`,
  `full_history_restore_transcript`, `TRANSCRIPT_VOCABULARY`, and
  `snapshot_native_transcript` with `0600`/`0700` permissions) must remain intact
  because `src/knowledge.rs` and the transcript vocabulary invariants depend on
  them.

## Systems likely affected

- `src/history.rs` — replace destination store synthesis in
  `import_full_cli_history` with `build_compact_handoff_bundle` and retire
  `import_full_history_into_codex`, `import_full_history_into_antigravity`,
  `import_full_history_into_claude`, and `CodexAppServer`.
- `src/cli.rs` — carry the compact handoff bundle and optional pre-allocated
  native session path on `ImportedNativeSession`.
- `src/config.rs` — define bounded constants for the compact Handoff bundle
  (`HANDOFF_BUNDLE_MAX_TURNS`, `HANDOFF_BUNDLE_TURN_MAX_CHARS`,
  `HANDOFF_BUNDLE_MAX_BYTES`).
- `src/agents.rs` — update handoff progress and target helper copy while keeping
  `original_task_from_session_goal` peeling the compact Handoff bundle envelope.
- `src/app.rs` & `src/app/screens.rs` — launch the destination session as a
  fresh interactive CLI session with the compact Handoff bundle prompt, record
  `CliHandoff { full_history: false, .. }`, and update `履歴` and session-menu
  labels.
- `src/i18n_tables.rs` — add English and Korean translations for updated
  Japanese strings.
- `src/tests.rs` — test the compact Handoff bundle builder, budgets, envelope
  peeling, non-synthesis of destination stores, and end-to-end handoff flow.
- `README.md`, `README.ja.md`, `README.ko.md` — update user-facing descriptions
  of cross-CLI handoff in all three languages together.

## Open questions

- None. The user explicitly directed replacing the full conversation history
  import feature with the compact Handoff bundle and retiring the full-history
  import feature.

## Not in scope

- Changing same-CLI native session resume (`resume_cli_session` and
  `resume_managed_native_session`), which reopens an existing conversation in the
  same CLI that created it without synthesizing or translating anything.
- Changing how `src/knowledge.rs` indexes local transcripts through
  `read_transcript_conversation`.
