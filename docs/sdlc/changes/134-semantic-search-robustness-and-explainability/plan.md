# Plan: Semantic Search Robustness, Explainability, and AI Governance

## Execution Steps

1. **Readiness Check**:
   - Run `bash scripts/check-readiness.sh docs/sdlc/changes/134-semantic-search-robustness-and-explainability`.

2. **Data Model & Attribution**:
   - Add `pub(crate) score: Option<f64>` to `TranscriptMatch` in `src/cli.rs`.
   - Update `search_local_transcripts_in` to propagate scores for semantic fallback and reranking.
   - Update `src/app/screens.rs` to render the semantic match indicator.

3. **Circuit Breaker & Adaptive Threshold**:
   - In `src/cli.rs`, define circuit breaker using `AtomicU64`.
   - Update `rank_transcripts_with_eg2` and `rank_fallback_candidates_with_eg2` to query circuit status and trip on timeout/failure.
   - Implement adaptive margin scoring logic in `rank_fallback_candidates_with_eg2`.

4. **AI Governance**:
   - Append Lesson 068 to `docs/sdlc/lessons.md`.

5. **Verification & Tests**:
   - Add unit tests in `src/tests.rs` for circuit breaker, adaptive threshold, and score attribution.
   - Run `make q.fast` (cargo fmt, clippy).
   - Run `cargo test --locked`.

6. **Ship & Release**:
   - Generate `review.yaml` with `check-review.sh`.
   - Commit, push, monitor CI with `gh run watch`.
   - Merge into `main` and clean up worktree.
   - Package macOS release bundle.

## Proof of completion

- `bash scripts/check-readiness.sh docs/sdlc/changes/134-semantic-search-robustness-and-explainability` exits 0.
- `make q.fast` passes with zero formatting diffs and zero clippy warnings.
- `cargo test --locked eg2` and `cargo test --locked transcript` pass completely.
- `bash scripts/check-review.sh docs/sdlc/changes/134-semantic-search-robustness-and-explainability` approves.
- CI passes on the pushed branch.
