# Intent: Transcript search requires exact verbatim keywords and lacks semantic ranking

- **Status**: approved
- **Opened**: 2026-10-07

## Problem

In Operon's History view, transcript search currently relies solely on exact whitespace-delimited keyword substring matching. When a developer searches using conceptual terms, paraphrasing, or a different language (for example, typing queries in Korean or Japanese when transcripts are predominantly English), zero matching sessions are returned. Additionally, when keyword matches are returned, they are ordered solely by file modification timestamps rather than semantic relevance to the search query.

## Who feels it, and when

A developer supervising multiple AI coding agents (Claude Code, Codex, Antigravity) across different projects who needs to find a past conversation or solution in the local transcript archive, but does not remember the exact verbatim strings typed in the terminal.

## Desired outcome

When searching local transcripts, if the local `eg2` CLI (EmbeddingGemma 2) is available on the user's machine, Operon utilizes it to semantically rank keyword matches and find relevant session transcripts even when exact verbatim keywords differ. When `eg2` is absent, Operon transparently and seamlessly preserves its existing fast keyword search behavior without error or performance penalty.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls. Embedding and ranking must run 100% locally on-device.
- User-facing text is Japanese; code, comments, and docs are English.
- Bounded execution: all external command invocations must be guarded by strict timeouts (at most 5 seconds) and output byte limits to prevent hanging the application.

## Systems likely affected

- `src/cli.rs`: transcript search, candidate collection, and semantic ranking integration.
- `src/tests.rs`: unit tests verifying keyword matching, semantic reranking fallback, and absent tool handling.

## Open questions

None.

## Not in scope

- Cloud embedding providers or remote LLM APIs.
- Replacing the primary keyword scanner entirely; exact substring search remains the primary fast path and fallback.
