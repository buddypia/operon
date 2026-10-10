# Intent: Replace External eg2 Dependency with Built-in Opt-in Semantic Search

- **Status**: approved
- **Opened**: 2026-10-10

## Problem

Operon's semantic transcript search currently relies on an external CLI daemon (`eg2`), which causes three major issues:

1. **Non-portable dependency**: Only machines with `eg2-cli` installed and configured (primarily the original developer's local environment) can use semantic search. General users who clone the repository fall back to keyword search only.
2. **Subprocess and circuit-breaker complexity**: To guard against `eg2` daemon hangs and crashes, `src/cli.rs` maintains subprocess spawning, timeout handling (3.0s), an `AtomicU64` circuit breaker with a 30s cooldown, and JSON output parsing. This adds architectural complexity to an in-memory search operation.
3. **No built-in opt-in mechanism**: Users who wish to benefit from semantic ranking have no seamless, zero-config way to download and enable a local embedding model directly within the Operon application.

## Who feels it, and when

- Users who clone Operon and expect rich conceptual transcript search (e.g. finding authentication fixes via "login credentials"), but receive no semantic fallback results because `eg2` is not installed.
- Maintainers who must support complex subprocess failure modes and synchronization rather than a clean, in-process engine.

## Desired outcome

1. **Eliminate external tool dependency**: Remove `eg2` subprocess execution, CLI JSON parsing, and the circuit breaker from `src/cli.rs`.
2. **Built-in in-process engine**: Integrate a pure-Rust / in-process embedding engine for EmbeddingGemma-2-270M (or comparable lightweight embedding model).
3. **Explicit opt-in UX**:
   - Keyword search remains the fast, zero-resource default.
   - Settings (or search view) provides an opt-in toggle to download the model (~250MB) and enable built-in semantic search.
   - Users can delete the downloaded model to reclaim disk space at any time.

## Constraints this change inherits

- macOS Apple Silicon / Intel compatibility; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no external API calls for search queries; models are downloaded strictly upon explicit user opt-in and stored in standard app cache directories (`~/Library/Caches/local.operon/models`).
- User-facing text is Japanese; code, comments, and docs are English.
- Bounded memory and execution: model inference must not freeze the GUI thread.

## Systems likely affected

- `Cargo.toml`: in-process ML / embedding dependencies.
- `src/cli.rs`: search execution, removing `eg2` subprocess calls, replacing with in-process search ranker.
- `src/app/screens.rs`: Settings UI for model download/deletion, status display, and search view integration.
- `src/store.rs`: persisting user preference for semantic search (opt-in flag).
- `src/tests.rs`: tests for semantic ranking, model management, and keyword fallback.

## Open questions

None.

## Not in scope

- Cloud-based embedding APIs (OpenAI, Gemini API, etc.).
- Multi-modal embeddings (images, audio). Text search only.
