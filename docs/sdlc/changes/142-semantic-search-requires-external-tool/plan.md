# Plan: Built-in Opt-in Semantic Search Replacing External eg2 Dependency

## Execution Steps

1. **Readiness & Screen Approval**:
   - Run `bash scripts/check-readiness.sh docs/sdlc/changes/142-semantic-search-requires-external-tool`.
   - Screen design in `screen.md` approved by user.

2. **Remove External eg2 Dependency**:
   - In `src/cli.rs`, remove `Command::new("eg2")`, `parse_eg2_rank_output`, `rank_transcripts_with_eg2`, `rank_fallback_candidates_with_eg2`, and circuit breaker logic (`EG2_FAILURE_COOLDOWN_UNTIL`, `is_eg2_circuit_open`, `trip_eg2_circuit`).
   - Clean up external `eg2` references and tests.

3. **Data Model & In-Process Search Abstraction**:
   - Define `SearchEngineMode`: `Keyword` | `EmbeddedGemma2`.
   - Implement in-process semantic ranker interface and download manager.
   - Persist search engine preference in `OperonStore` (`src/store.rs`).

4. **UI Implementation**:
   - In `src/app/screens.rs` (`ui_settings_data`), implement radio choices for search mode, download progress bar, and model cache deletion button.
   - In transcript search UI, render inline discovery hint when keyword search yields 0 results.

5. **Verification & Tests**:
   - Add unit tests in `src/tests.rs` verifying keyword fallback, model state transitions, and search ranking.
   - Run `make q.fast` (cargo fmt, clippy).
   - Run `cargo test --locked`.

6. **Review & Ship**:
   - Run `bash scripts/check-review.sh docs/sdlc/changes/142-semantic-search-requires-external-tool`.
   - Commit, push, monitor CI, merge to `main`, and package macOS bundle.

## Proof of completion

- `bash scripts/check-readiness.sh docs/sdlc/changes/142-semantic-search-requires-external-tool` exits 0.
- All references to `Command::new("eg2")` removed from `src/`.
- `make q.fast` passes.
- Targeted tests pass in `src/tests.rs`.
- `cargo test --locked` passes.
