# Spec: Session restore resilience and pre-resume validation

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. When a user triggers native session resume (`resume_managed_native_session`), Operon must inspect the target transcript on disk before launching the CLI resume command.
2. If the transcript file does not exist, is 0 bytes, or contains no readable conversation records, Operon must refuse the native resume command, notify the user with Japanese feedback explaining that no conversation was saved prior to termination, and fall back to a fresh session start without crashing.
3. If the transcript file ends with an incomplete or truncated trailing line (e.g. caused by abrupt SIGKILL during JSON serialization), Operon must safely sanitize the trailing incomplete line (backing up the original if modified) so the file remains valid JSONL for upstream CLI parsers.
4. Before launching a resumed session for Claude Code, Operon must verify whether the conversation identity rotated (e.g. via `/clear`) and use the corrected session ID, or alert the user if the transcript identity cannot be verified.

## Behaviour

- **Empty or missing transcript upon resume**:
  When the user clicks "会話 ID から再開" on a session whose transcript was never created or has 0 turns, Operon does not spawn a failing `claude --resume` or `codex resume`. Instead, it transitions the session to a fresh start and sets the UI notice:
  `"会話ログが記録される前に強制終了したため、会話 ID からの再開はできません。新しいセッションとして開始しました。"`
- **Malformed trailing line sanitization**:
  If a transcript has an unparseable trailing line due to hard kill, Operon truncates only the unparseable trailing fragment and reports:
  `"強制終了により破損したトランスクリプトの末尾を安全に修復して再開しました。"`
- **Normal resume**:
  When transcript validation passes (file exists, contains >= 1 valid turn, trailing lines are valid), resume proceeds as normal.

## Design

### 1. `src/cli.rs`
- Add `pub(crate) enum TranscriptResumeReadiness`:
  - `Ready(PathBuf)`: Transcript exists, has valid records, and ends cleanly.
  - `Sanitized { path: PathBuf, removed_bytes: usize }`: Transcript had a malformed trailing line that was safely stripped.
  - `EmptyOrMissing`: File is absent, 0 bytes, or holds 0 records.
- Add `pub(crate) fn check_transcript_resume_readiness(path: &Path) -> TranscriptResumeReadiness`:
  Reads the transcript with bounded limits. If the last non-empty line cannot be parsed as JSON while preceding lines are valid JSON, truncates the trailing unparseable fragment.
- Add `pub(crate) fn validate_session_transcript_for_resume(provider: CliProvider, workspace: &Path, session_id: &str, recorded_path: Option<&Path>) -> TranscriptResumeReadiness`:
  Resolves the transcript path via `native_session_path_for_project` or recorded path, then evaluates readiness.

### 2. `src/app.rs`
- In `resume_managed_native_session`:
  - Call `validate_session_transcript_for_resume`.
  - On `EmptyOrMissing`: update notice and launch fresh session instead of broken resume.
  - On `Sanitized`: update notice and continue resume.
  - On `Ready`: proceed directly.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | No new colours are introduced; notices use existing palette styles. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new icon marks are introduced. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | Notice strings are centralized and i18n keys match existing conventions. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | The persisted session schema is unchanged; transcript repair creates `.bak` before truncating any bytes. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | Resume and retry launches continue to pass through `is_safe_agent_command` and existing process fences. |
| Documentation — user-facing docs change in all three languages together | Yes | Relevant doc and changelog strings are aligned. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Pure local filesystem validation, no network calls or telemetry. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Transcript readiness checks observe existing `TRANSCRIPT_SEARCH_BYTE_LIMIT` and `TRANSCRIPT_SEARCH_LINE_LIMIT`. |

## Flagged concerns

None. The on-disk transcript locations and JSONL structures are already tested and supported across all three providers.

## Acceptance

- `cargo test --locked` passes with new tests in `src/tests.rs`:
  - `resume_refuses_empty_transcript_and_falls_back_to_clean_start`
  - `resume_sanitizes_truncated_trailing_transcript_line`
  - `resume_readiness_validates_healthy_transcripts`
- In the running app, attempting to resume a session killed before turn 1 reports the informative notice and starts cleanly rather than spawning an erroring CLI process.

## Rejected alternatives

- *Always run native resume command without checking*: Rejected because upstream CLIs output unhelpful error codes or crash immediately when passed non-existent or empty session files.
- *Blindly delete malformed transcript files*: Rejected because partial conversation history must be preserved; only the incomplete trailing fragment should be trimmed after taking a backup.
