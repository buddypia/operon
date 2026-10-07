# Spec: Semantic Transcript Search and Ranking with eg2

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. `eg2_availability_is_detected_without_error`: When `eg2` is executable on `PATH`, Operon detects its presence via `tool_available("eg2")` without throwing an error or spawning long-lived processes.
2. `semantic_reranking_orders_matches_by_relevance`: When `eg2` is available and multiple transcript search hits exist, matches are reranked by semantic similarity score returned from `eg2 rank` rather than filesystem modification time.
3. `semantic_fallback_captures_meaning_when_verbatim_keywords_miss`: When keyword substring matching returns no results, Operon scans candidate session titles and recent user turns, evaluating them with `eg2 rank` if available, and surfaces sessions whose semantic similarity exceeds 0.60.
4. `absence_of_eg2_preserves_verbatim_search_identically`: When `eg2` is not installed on `PATH`, transcript search executes verbatim keyword substring matching identically to previous behavior with no failure notices or delays.
5. `eg2_execution_is_bounded_by_timeout_and_output_limits`: Invocations of `eg2 rank` enforce a 5-second timeout and a 64 KiB stdout cap via `run_command_with_output_limit`, preventing thread blockage or memory bloat.

## Behaviour

- In the **履歴 (History)** tab or when searching transcripts, the user types a query in the search input box.
- When `eg2` is available and matches are reranked or augmented semantically:
  - The results list displays the most relevant sessions at the top.
  - If a result was found via semantic fallback rather than verbatim keyword substring match, its snippet clearly displays the matched topic or turn context.
- When no matches are found, the existing empty state message ("一致するセッションはありません") is displayed.
- If `eg2` times out or fails during ranking, the search silently and safely retains the unranked keyword matches without raising an alarming banner to the user.

## Design

- **Placement Verdict**: `EXTEND` (`src/cli.rs` and `src/history.rs`).
- **New Symbols & Helper Functions**:
  - `pub(crate) fn rank_transcripts_with_eg2(query: &str, matches: &mut [TranscriptMatch]) -> bool`: Calls `eg2 rank <query> <doc1> <doc2> ... --json` via `run_command_with_output_limit` and sorts `matches` in descending order of score.
  - `pub(crate) fn parse_eg2_rank_scores(json: &str) -> Vec<f64>`: Parses the JSON output from `eg2 rank` safely.
  - `pub(crate) fn semantic_transcript_fallback_in(...)`: Evaluates recent candidate sessions with `eg2 rank` when exact keyword matching yields no hits.
- **External Command**:
  - `eg2 rank "<query>" "<doc1>" "<doc2>" ... --json`
  - Timeout: 5 seconds.
  - Output limit: 64 KiB.
  - Drops `INHERITED_REPOSITORY_POINTERS` via `src/exec.rs`.
- **Thread Context**: Executed exclusively inside the background task spawned by `BackgroundKey::TranscriptSearch`, keeping the immediate-mode UI frame rate fluid.
- **Persisted Shape Change**: None. No change to `src/store.rs` or `Store` schema.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | No new colour roles or palette tokens added. |
| Icons | No | No new glyphs or icons added. |
| Identifier SSOT | No | No cross-module shared identifier strings added. |
| Durability | No | No changes to store schema or persistence format (`STORE_SCHEMA_VERSION` unchanged). |
| Subprocess safety | Yes | All `eg2` subprocess invocations strictly go through `run_command_with_output_limit` in `src/exec.rs` with 5s timeout and 64 KiB limit. |
| Documentation | Yes | Documentation and comments in English; UI messages in Japanese. |
| Local-first | Yes | 100% on-device embedding execution via local `eg2` daemon (Metal/MPS); zero telemetry, zero cloud calls. |
| Budgets | Yes | Invocations limit candidate items to 20 documents and enforce 64 KiB output ceiling. |

## Flagged concerns

- **Concern**: What if `eg2` server is idle and takes 1-2 seconds to wake up on first invocation?
  - **Answer**: The call runs entirely inside the background thread (`BackgroundResult::TranscriptSearch`), so the UI does not freeze. The 5-second timeout easily accommodates on-demand startup on Apple Silicon.

## Acceptance

- `cargo test --locked` passes, including new tests:
  - `test_parse_eg2_rank_scores`
  - `test_rank_transcripts_with_eg2_ordering`
  - `test_search_local_transcripts_fallback_when_eg2_unavailable`
- In the running app, typing a semantic or cross-lingual query in the History search box surfaces relevant sessions when `eg2` is available, and falls back cleanly when disabled.

## Rejected alternatives

- Linking a heavy Rust ML embedding runtime (e.g. `candle` or `ort`) directly into the Operon binary: Rejected because it drastically bloats the binary size, complicates build tooling, and wastes resources when the user is not actively searching transcripts.
- Cloud OpenAI Embedding API integration: Rejected because it violates Operon's core local-first and zero-cloud privacy guarantees.
