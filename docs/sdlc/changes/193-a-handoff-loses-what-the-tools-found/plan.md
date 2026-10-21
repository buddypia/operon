# Plan: A handoff carries the latest tool results, shortened, and never as conversation

- **Spec**: `./spec.md`
- **Approved**: 2026-10-21
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `HANDOFF_TOOL_RESULTS_MAX`, `HANDOFF_TOOL_RESULT_MAX_CHARS`, `HANDOFF_TOOL_RESULT_HEAD_LINES`, `HANDOFF_TOOL_RESULT_TAIL_LINES`, `HANDOFF_TOOL_RESULT_SIGNAL_LINES`, `HANDOFF_TOOL_RESULT_SIGNAL_WORDS`, `HANDOFF_TOOL_RESULT_LINE_MAX_CHARS`, and the omission, line-cut, and repeat wordings, beside `HANDOFF_BUNDLE_*` |
| `src/history.rs` | block splitter; `compact_handoff_turn_text` keeps said blocks; `handoff_tool_results`; `shorten_tool_result`; `build_compact_handoff_bundle` renders the tool section and gives way in the spec's order |
| `src/tests.rs` | the four tests the spec names |
| `README.md`, `README.ja.md`, `README.ko.md` | the handoff bullet names the shortened tool results |

## Order of work

1. Commit this plan with the intent and spec.
2. Write the four tests against the constants (the tree does not compile until
   step 3 adds them — the constants and an empty-section stub land in the same
   step so the tests compile and fail for the stated reasons): watch
   `a_tool_result_with_blank_lines_never_reads_as_conversation` fail on the
   leaked paragraph, `a_handoff_carries_the_latest_tool_results_shortened` on
   the missing section, `a_shortened_tool_result_keeps_its_error_lines_and_counts_repeats`
   on the stub, and `a_handoff_gives_up_tool_results_before_the_last_turn_task_or_note`
   on the missing section.
3. Block splitter and `compact_handoff_turn_text` — test for Requirement 3 passes.
4. `shorten_tool_result` — test for Requirement 2 passes.
5. `handoff_tool_results` and the render/give-way change — tests for
   Requirements 1 and 4 pass; `cargo test --locked handoff` all green
   (Requirement 5).
6. READMEs in three languages.
7. `make q.fast`, `cargo test --locked handoff`, `cargo test --locked -- --ignored`.
8. Mutation sweep from `docs/sdlc/templates/mutations.py` in the change directory.
9. Review (route `modify`: `rust-reviewer`, widened by `scripts/check-review.sh`
   for `src/history.rs`), close, push, CI, land, package.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A bundle over 6 000 bytes once tool results are added | the destination receives a larger first prompt than budgeted | every new test asserts `bundle.len() <= HANDOFF_BUNDLE_MAX_BYTES`; Requirement 4's test is built to overflow |
| A bundle without tool results changes | existing handoff tests fail, or an old snapshot's bundle differs | Requirement 5: the existing handoff tests are unchanged and must pass |
| Said text after a tool block is moved to the tool section | a person's interruption note appears under `### Tool result` | accepted in the spec's Design; still shown |
| A tool result's line starting `### ` reads as a heading | the destination sees an extra section heading | accepted: the heading names no speaker, so nothing is attributed to the user or assistant |
| Quadratic work on a huge result | a slow handoff on a multi-megabyte log | signal candidates capped at six before combinations; one pass to collapse repeats |
| The splitter treats said text that begins with `[Thinking]` as reasoning | a user turn quoting the label is dropped from the excerpt | pre-existing: today's filter already drops such a chunk |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked handoff` — `test result: ok`, the four new tests among them.
- `cargo test --locked -- --ignored` — 6 passed, or say that it could not run.
- The four new tests each fail on main's builder for the stated reason (step 2).
- `mutations.py` in the change directory: each mutation caught.
- The CI run on the branch head is green.

## Departures from the plan

None yet.
