# Spec: Semantic Search Robustness, Explainability, and AI Governance

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. `score_attribution_is_recorded_on_transcript_matches`: `TranscriptMatch` carries `pub(crate) score: Option<f64>`. Exact substring matches have `score: None` by default; semantic matches carry their similarity score.
2. `adaptive_relevance_admits_high_margin_matches`: For items scoring between `0.65` and `0.72`, they are included when they exhibit at least `+0.035` separation above the background/lowest candidate score, preventing false negatives on queries like `"login credentials"`.
3. `subprocess_circuit_breaker_prevents_repeated_hangs`: When `eg2 rank` times out or fails, a 30-second cooldown is armed (`AtomicU64`), bypassing `eg2` immediately on subsequent queries to avoid repeated background delays.
4. `ui_indicates_semantic_matches_transparently`: In `src/app/screens.rs`, results with `score: Some(...)` display a localized weak badge indicating the match is semantic rather than exact keyword.
5. `sdlc_stage_invariant_is_permanently_recorded`: Lesson 068 is documented in `docs/sdlc/lessons.md` to prevent AI sessions from setting invalid stage names in `state.yaml`.

## Behaviour

- When searching transcripts, exact keyword matches continue to display normally.
- When results originate from semantic fallback, the UI displays an explicit match badge (e.g., `시맨틱 76%`) next to the session identifier, removing ambiguity.
- If the `eg2` daemon hangs, the first query times out safely in 3.0s, and subsequent queries for the next 30 seconds immediately return exact search results with zero latency.
- Conceptual queries previously dropped by the `0.73` cliff now surface cleanly if they are distinct from background noise.

## Design

- **Placement Verdict**: `EXTEND` (`src/cli.rs`, `src/app/screens.rs`, `src/tests.rs`, `docs/sdlc/lessons.md`).
- **Data Model**:
  - `TranscriptMatch { pub(crate) provider: String, pub(crate) session_id: String, pub(crate) title: Option<String>, pub(crate) path: PathBuf, pub(crate) snippet: String, pub(crate) score: Option<f64> }`
- **Circuit Breaker**:
  - `static EG2_FAILURE_COOLDOWN_UNTIL: AtomicU64 = AtomicU64::new(0);`
  - Checked prior to spawning `Command::new("eg2")`.
- **Adaptive Threshold**:
  - `EG2_SEMANTIC_NOISE_FLOOR = 0.65;`
  - `EG2_SEMANTIC_HIGH_CONFIDENCE = 0.72;`
  - `EG2_SEMANTIC_MIN_MARGIN = 0.035;`

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | Uses existing weak/subtle text styling. |
| Icons | No | No new icons added. |
| Identifier SSOT | No | No cross-module shared identifier strings added. |
| Durability | No | No changes to SQLite store or persistent JSON schemas. |
| Subprocess safety | Yes | Subprocess timeouts tightened to 3.0s and circuit breaker prevents hammering. |
| Documentation | Yes | Documentation and comments in English; UI messages in Japanese. |
| Local-first | Yes | 100% on-device embedding execution via local `eg2` daemon; zero telemetry. |
| Budgets | Yes | Invocations limit candidate items to 24 documents and 64 KiB output ceiling. |

## Flagged concerns

- **Concern**: Does lowering the noise floor to 0.65 introduce noise if all candidates are completely unrelated?
  - **Answer**: No, because candidates below 0.72 require a +0.035 margin above the set minimum. Unrelated documents score tightly together (e.g. 0.52-0.55) without separation, so they are filtered out.

## Acceptance

- `cargo test --locked` passes, including new tests:
  - `eg2_circuit_breaker_trips_and_cooldown_recovers`
  - `eg2_semantic_search_adaptive_threshold_admits_close_matches`
  - `transcript_match_preserves_score_attribution`
- In the running app, conceptual queries display the semantic match percentage badge.

## Rejected alternatives

- Hardcoding a lower static threshold (e.g. 0.68): Rejected because single-document queries or different model checkpoints still suffer cliff effects. Dynamic margin is more robust.
- Spawning a long-lived health check background thread: Rejected to avoid unnecessary polling and CPU cycles when Operon is idle.
