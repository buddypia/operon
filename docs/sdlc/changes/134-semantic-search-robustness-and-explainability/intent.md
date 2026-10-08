# Intent: Semantic Search Robustness, Explainability, and AI Governance

- **Status**: approved
- **Opened**: 2026-10-07

## Problem

Adversarial verification of the initial `eg2-cli` integration (change 133) identified four critical friction points that limit long-term autonomous reliability and user experience:

1. **Threshold Cliff (False Negative Trap)**: A single hardcoded constant `EG2_SEMANTIC_MATCH_MIN_SCORE = 0.73` acts as a rigid cutoff. Legitimate conceptual and cross-lingual queries (e.g. `"login credentials"` scoring `0.6872` against JWT token fixes, or Japanese query `"ターミナルのエスケープシーケンス不具合"` scoring `0.7041`) are completely dropped even when `eg2` ranks them as the clear #1 candidate with a large margin over unrelated noise.
2. **Lack of Explainability in Data Model & UI**: `TranscriptMatch` does not carry match score or attribution metadata (`score`). In the UI, a semantic fallback result displays identically to an exact substring hit, causing users to distrust results when their query words are not literally in the snippet.
3. **Subprocess Liveness & Hang Risk**: `tool_available("eg2")` only checks file presence. If the daemon crashes, hangs, or is slow to initialize, every search execution pays a full timeout penalty without a failure cooldown / circuit breaker.
4. **AI Cognitive Drift on SDLC Stage Names**: AI agents conflate Stage 5 review with machine-checked stage names in `state.yaml`, causing CI test failures on `every_state_file_names_a_route_and_a_stage_that_exist`.

## Who feels it, and when

A developer using Operon who types conceptual search queries and gets false-negative zero results due to the 0.73 cliff, or faces background search delays when the local embedding daemon stalls; and future AI agents maintaining Operon who need clear cognitive guards.

## Desired outcome

Operon's semantic transcript search becomes robust and transparent:
1. Dynamic relevance filtering admits high-margin matches (score >= 0.65 with margin) without false negatives.
2. `TranscriptMatch` carries `score: Option<f64>`, enabling clear semantic attribution badges in the UI.
3. A circuit breaker prevents repeated timeout stalls when the `eg2` server is unresponsive.
4. SDLC Lesson 068 is permanently recorded so future AI sessions never hallucinate stage names.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- Bounded execution: all external command invocations guarded by strict timeouts.

## Systems likely affected

- `src/cli.rs`: `TranscriptMatch` data model, circuit breaker, dynamic thresholding.
- `src/app/screens.rs`: semantic match attribution badge rendering.
- `src/tests.rs`: unit tests for circuit breaker, adaptive threshold, and score attribution.
- `docs/sdlc/lessons.md`: Lesson 068 on state.yaml stage names.

## Open questions

None.

## Not in scope

- Adding new persistent database tables (SQLite vector storage is deferred to Phase 2).
- Changing exact keyword matching behavior.
