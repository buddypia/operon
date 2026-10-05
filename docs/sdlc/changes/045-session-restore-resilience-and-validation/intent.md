# Intent: Session resume crashes or revives wrong conversations after abrupt termination

- **Status**: approved
- **Opened**: 2026-09-14

## Problem

When an agent CLI terminal terminates abruptly (SIGKILL, machine reboot, kernel panic, or crash before writing turns), resuming from the saved conversation ID can fail fatally in several distinct ways:
1. Resuming a session before the CLI flushed its first turn tries to run a native resume command (`claude --resume <id>`, `codex resume <id>`, `agy --conversation <id>`) against a non-existent or empty transcript file, causing the upstream CLI to crash or fail with session-not-found errors.
2. In-flight I/O buffering cut short by a hard kill leaves the transcript file ending with a malformed, truncated JSON record, which can cause subsequent resumes to crash during parsing.
3. In Claude Code, executing `/clear` internally rotates the session UUID, but if the session terminates abruptly before registry synchronization settles, resuming revives the stale pre-clear conversation instead of the latest state.

Currently, Operon unconditionally generates and executes the resume command without pre-validating whether the on-disk transcript exists, is readable, and contains valid turns.

## Who feels it, and when

Anyone whose terminal session terminated unexpectedly due to process termination, crash, or reboot, and who clicks "会話 ID から再開" (Resume from conversation ID) to recover their work.

## Desired outcome

1. Operon validates the on-disk transcript's presence and minimum viable turn count before attempting a native resume.
2. If a session terminated before its transcript was written or if the transcript is empty (0 turns), Operon safely catches this condition, informs the user clearly, and falls back to a clean restart rather than executing a failing native resume command.
3. If an on-disk transcript has a truncated malformed trailing JSON line caused by hard termination, Operon automatically repairs/sanitizes the trailing line so the upstream CLI can resume without parse errors.
4. Pre-resume checks verify the latest registered conversation identity to prevent stale conversation revivals.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Do not mutate or truncate transcript contents unless a trailing line is verifiably unparseable and unclosed.

## Systems likely affected

- `src/cli.rs`: transcript validation and trailing line sanitization.
- `src/app.rs`: pre-resume validation gate and fallback handling.
- `src/app/screens.rs`: informative notice display.
- `src/tests.rs`: unit tests for empty transcript fallback, malformed tail recovery, and pre-resume validation.

## Open questions

None. The upstream CLI file layouts and resume mechanisms are already mapped in `src/cli.rs` and `src/history.rs`.

## Not in scope

- Rewriting upstream CLI recovery binaries or protocols.
- Decrypting OpenAI organisation-scoped encrypted reasoning blocks.
