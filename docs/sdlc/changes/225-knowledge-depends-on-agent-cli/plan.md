# Plan: multi-CLI fallback for knowledge distillation, fixed `agy` flag order, and explicit EmbeddingGemma 2 status

- **Spec**: `./spec.md`
- **Approved**: 2026-10-26
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/agents.rs` | Put `--sandbox` before `--print` for `CliProvider::Gemini` in `knowledge_judge_command` and `"gemini"` in `commit_message_command`; add `knowledge_judge_providers(preferred: CliProvider) -> [CliProvider; 3]`. |
| `src/knowledge.rs` | Add `CliJudge::new` and `CliJudge::judge_with` with `failed: std::sync::Mutex<Vec<CliProvider>>`; use `tool_available` and `knowledge_judge_providers` in `Judge::judge` for `CliJudge`; replace the single `judging: bool` in `run_project` / `process_conversation` with per-provider `paused_providers: Vec<CliProvider>`. |
| `src/app/knowledge.rs` | Construct `CliJudge::new(knowledge_judge_directory(&root))`; include `EmbeddingGemma 2 ({device})` in `knowledge_status_line`; show `使用可能 ({device} / 310MB)` when ready and update CLI bullet in `ui_knowledge_disabled`; clarify subtitle and `ローカル履歴の検索` radio label in `ui_model_block`. |
| `src/i18n_tables.rs` | Update JA/EN/KO translation rows in byte-sorted order for the updated and added strings. |
| `README.md`, `README.ja.md`, `README.ko.md` | Update the project knowledge bullet to state that candidates are summarised by an available CLI (`Claude Code`, `Codex`, or `Antigravity`). |
| `src/tests.rs` | Update `judge_command_is_read_only_ephemeral_and_runs_outside_the_project` and `builds_a_read_only_draft_command_for_every_agent_operon_launches`; add `cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones` and `knowledge_ui_shows_embedding_gemma_device_and_available_cli_summary`. |

## Order of work

1. Update and add the unit tests in `src/tests.rs` first so they fail against the unedited code.
2. Update `src/agents.rs` and `src/knowledge.rs` to fix the `agy` flag order, add `knowledge_judge_providers`, add `CliJudge::new` / `CliJudge::judge_with`, and isolate provider pauses in `run_project`.
3. Update `src/app/knowledge.rs`, `src/i18n_tables.rs`, and `README*.md` for the UI status line, disabled card, and Settings › Data copy.
4. Run the named unit tests, translation/harness record tests, and `mutations.py` to verify all acceptance criteria and mutation kills.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Unsorted or unused row in `src/i18n_tables.rs` | Test failure in `every_translation_table_is_sorted_by_message_id` or `every_message_row_is_one_the_code_shows` | `cargo test --locked tests::every_translation_table_is_sorted_by_message_id -- --exact` and `tests::every_message_row_is_one_the_code_shows -- --exact` |
| Failing CLI retried on every conversation in a run | Multiple slow calls to a broken provider in one run | `tests::cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones` asserts the failed provider is invoked only once across two conversations |
| Golden embedding fixture broken if `KEEP_EXEMPLARS` is touched | `embedding_golden_fixture_is_well_formed` fails | Leave `KEEP_EXEMPLARS` untouched and run `tests::embedding_golden_fixture_is_well_formed` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked tests::judge_command_is_read_only_ephemeral_and_runs_outside_the_project -- --exact` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked tests::builds_a_read_only_draft_command_for_every_agent_operon_launches -- --exact` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked tests::cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones -- --exact` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked tests::knowledge_ui_shows_embedding_gemma_device_and_available_cli_summary -- --exact` — `test result: ok. 1 passed; 0 failed`.
- `python3 docs/sdlc/changes/225-knowledge-depends-on-agent-cli/mutations.py` — all mutations killed.
- In the packaged `/Applications/Operon.app`, `プロジェクト › ナレッジ` shows `EmbeddingGemma 2 (GPU (Metal))` in the status line and `今すぐ更新` completes without the Antigravity CLI error.

## Departures from the plan

None.
