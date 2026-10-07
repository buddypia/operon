# Plan: Semantic Transcript Search and Ranking with eg2

- **Spec**: ./spec.md
- **Approved**: 2026-10-07
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/cli.rs` | Implement `parse_eg2_rank_scores`, `rank_transcripts_with_eg2`, and fallback logic in `search_local_transcripts_in` |
| `src/tests.rs` | Unit tests for output parsing, semantic score sorting, fallback search, and absent tool handling |

## Order of work

1. Implement `parse_eg2_rank_scores` in `src/cli.rs` to extract scores from JSON arrays returned by `eg2 rank --json`.
2. Add parser contract tests in `src/tests.rs` to verify score extraction across valid, invalid, and empty JSON responses.
3. Implement `rank_transcripts_with_eg2` in `src/cli.rs` using `run_command_with_output_limit` with a 5-second deadline and 64 KiB ceiling.
4. Implement candidate extraction and semantic fallback in `search_local_transcripts_in` when keyword matches are empty and `tool_available("eg2")` is true.
5. Add unit tests for `rank_transcripts_with_eg2` and semantic search integration in `src/tests.rs`.
6. Verify quality gates with `make q.fast` (`cargo fmt --check` and `cargo clippy --locked -- -D warnings`) and run tests with `cargo test --locked`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Malformed JSON from `eg2 rank` | Parser error | `parse_eg2_rank_scores` fails gracefully and falls back to original order |
| Command timeout or hang | Process hangs for minutes | 5-second timeout in `run_command_with_output_limit` |
| `eg2` not installed on system | `which eg2` / executable lookup fails | `tool_available("eg2")` guard executes standard keyword search |
| Empty search query | Pointless subprocess execution | Early return on empty/whitespace query |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked test_parse_eg2_rank_scores` — passes.
- `cargo test --locked test_rank_transcripts_with_eg2` — passes.
- `cargo test --locked search_local_transcripts` — passes.

## Departures from the plan

None yet.
