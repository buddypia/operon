# Plan: Session restore resilience and pre-resume validation

- **Spec**: `./spec.md`
- **Approved**: 2026-09-14
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/cli.rs` | Add `TranscriptResumeReadiness` enum, `check_transcript_resume_readiness`, `sanitize_transcript_tail_if_needed`, and `validate_session_transcript_for_resume`. |
| `src/app.rs` | Integrate transcript validation into `resume_managed_native_session`, adding safety fallback to clean restart on empty/missing transcripts and user notification on repair. |
| `src/i18n_tables.rs` | Add Japanese notice strings for empty transcript fallback and transcript repair. |
| `src/tests.rs` | Add unit tests covering transcript readiness validation, empty transcript fallback, and truncated trailing line sanitization. |

## Order of work

1. Implement `TranscriptResumeReadiness` and validation/sanitization functions in `src/cli.rs`.
2. Add i18n messages to `src/i18n_tables.rs` if needed or use `tr()` inline.
3. Wire transcript validation into `resume_managed_native_session` in `src/app.rs`.
4. Add comprehensive unit tests in `src/tests.rs`.
5. Run `cargo fmt --check`, `cargo test --locked`, and `cargo clippy --locked -- -D warnings`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Truncating a valid trailing line in a healthy transcript | Loss of final conversation turn | Unit tests with various valid JSONL formats and multiline messages |
| False positive on empty transcripts for newly started valid sessions | Premature fallback to clean start | Test ensuring valid >=1 turn transcripts return `Ready` |
| File permission or backup failure during sanitization | Failed resume with I/O error | Checked error propagation and non-destructive fallback |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 486 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- Unit tests verifying:
  - `resume_refuses_empty_transcript_and_falls_back_to_clean_start`
  - `resume_sanitizes_truncated_trailing_transcript_line`
  - `resume_readiness_validates_healthy_transcripts`

## Departures from the plan

None so far.
