# Spec: multi-CLI fallback for knowledge distillation, fixed `agy` flag order, and explicit EmbeddingGemma 2 status

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `knowledge_judge_command(CliProvider::Gemini, prompt)` and `commit_message_command("gemini", prompt)` in `src/agents.rs` MUST return arguments `["--sandbox", "--print", prompt]` in that order so `agy` receives `prompt` as the argument to `--print`.
2. `knowledge_judge_providers(preferred: CliProvider) -> [CliProvider; 3]` in `src/agents.rs` MUST return `preferred` first, followed by the remaining two providers in `[CliProvider::Claude, CliProvider::Codex, CliProvider::Gemini]` order.
3. `CliJudge` in `src/knowledge.rs` MUST try providers in `knowledge_judge_providers(preferred)` order, skipping any provider whose binary is not installed (`!tool_available(program)`) or that has already returned `JudgeOutcome::Pause` earlier in the same `CliJudge` instance (`failed: std::sync::Mutex<Vec<CliProvider>>`), returning the first `JudgeOutcome::Answer` and returning `JudgeOutcome::Pause` only when no available CLI can answer.
4. `run_project` and `process_conversation` in `src/knowledge.rs` MUST track paused providers per `CliProvider` (`paused_providers: Vec<CliProvider>`) so a `JudgeOutcome::Pause` for one provider does not skip judging conversations of other providers in the same run, while `report.capped` still stops all judging once `KNOWLEDGE_DAILY_JUDGE_CALLS` is reached.
5. `knowledge_status_line` in `src/app/knowledge.rs` MUST include `EmbeddingGemma 2 ({device})` when `ModelState::Ready { device }` and no warning or running state applies: `{n} 件 ・ 最終更新 {updated} ・ EmbeddingGemma 2 ({device}) ・ 本日の判定 {used}/{cap} 回`.
6. `ui_knowledge_disabled` in `src/app/knowledge.rs` MUST state `・残す候補だけを、利用可能な CLI (Claude Code・Codex・Antigravity) で要約します (1日 30 回まで)。` and, when `ModelState::Ready { device }`, display `使用可能 ({device} / 310MB)` beside `このプロジェクトでナレッジを集める`.
7. `ui_model_block` in `src/app/knowledge.rs` MUST explain `この Mac 上で動くモデルです。会話の内容は外部に送りません。プロジェクトのナレッジではモデルがあればそのまま使われます。` and label the radio row `ローカル履歴の検索`.

## Behaviour

- **Project › ナレッジ (disabled, model ready)**: Shows the explanation card with `・残す候補だけを、利用可能な CLI (Claude Code・Codex・Antigravity) で要約します (1日 30 回まで)。`, the primary button `このプロジェクトでナレッジを集める`, and `使用可能 ({device} / 310MB)` in `palette.success` beside it.
- **Project › ナレッジ (enabled, normal state)**: Shows `{n} 件 ・ 最終更新 {updated} ・ EmbeddingGemma 2 ({device}) ・ 本日の判定 {used}/{cap} 回` under the `ナレッジ` heading.
- **Project › ナレッジ (enabled, warning/running states)**: Unchanged (`会話を整理しています… ({done}/{total})`, `モデルがないため停止中`, `{cli} で判定できませんでした。次回の更新で再試行します。`, `本日の判定回数の上限に達しました。明日続きを処理します。`, `固定したナレッジが上限 (2000 件) に達したため、新しいナレッジを追加していません。`).
- **Settings › データ (`ui_model_block`)**: Under `意味検索（EmbeddingGemma 2）`, shows `この Mac 上で動くモデルです。会話の内容は外部に送りません。プロジェクトのナレッジではモデルがあればそのまま使われます。` and the radio row `ローカル履歴の検索  ( ) キーワードのみ  (●) 意味検索`.

## Design

- `src/agents.rs`:
  - Swap `"--print"` and `"--sandbox"` in `knowledge_judge_command` (`CliProvider::Gemini`) and `commit_message_command` (`"gemini"`).
  - Add `pub(crate) fn knowledge_judge_providers(preferred: CliProvider) -> [CliProvider; 3]`.
- `src/knowledge.rs`:
  - Add `failed: std::sync::Mutex<Vec<CliProvider>>` to `CliJudge`, constructor `CliJudge::new(directory: PathBuf) -> Self`, and `CliJudge::judge_with` taking `available: impl Fn(CliProvider) -> bool` and `run_one: impl FnMut(CliProvider, &str) -> JudgeOutcome` so unit tests can deterministically exercise every fallback and memoisation branch without spawning real CLIs.
  - Implement `Judge for CliJudge` by creating the private judge directory and delegating to `self.judge_with(provider, prompt, |candidate| tool_available(knowledge_judge_command(candidate, "").0), |candidate, prompt| ...)`.
  - In `run_project` and `process_conversation`, replace `judging: &mut bool` with `paused_providers: &mut Vec<CliProvider>` and check `if report.capped || paused_providers.contains(&session.provider)`.
- `src/app/knowledge.rs`, `src/i18n_tables.rs`, `README.md`, `README.ja.md`, `README.ko.md`:
  - Use `CliJudge::new(knowledge_judge_directory(&root))` in `start_knowledge_run`.
  - Update `knowledge_status_line`, `ui_knowledge_disabled`, `ui_model_block`, sorted translation rows in `src/i18n_tables.rs`, and the project knowledge section in `README*.md`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Uses existing semantic roles (`palette.text_muted`, `palette.success`) with no new colour roles |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | Uses existing `ICON_KNOWLEDGE` and adds no new icons |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | Reuses `ANTIGRAVITY_COMMAND` and `CliProvider` methods from `src/config.rs` / `src/agents.rs` |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | No | No store or SQLite schema change |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | All judge calls continue to build commands via `knowledge_judge_command` in `src/agents.rs` and run via `run_command_with_output_limit` in `src/exec.rs` inside `knowledge_judge_directory` |
| Documentation — user-facing docs change in all three languages together | Yes | Updates `src/i18n_tables.rs` (JA/EN/KO) and `README.md`, `README.ja.md`, `README.ko.md` together |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Uses only locally installed agent CLIs and the local EmbeddingGemma 2 model; no new `unsafe` |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Keeps existing `KNOWLEDGE_JUDGE_SECONDS`, `KNOWLEDGE_JUDGE_ANSWER_MAX_BYTES`, `COMMAND_ERROR_MAX_BYTES`, and `KNOWLEDGE_DAILY_JUDGE_CALLS` ceilings unchanged |

## Flagged concerns

- **Repeated timeouts when a fallback CLI is installed but unauthenticated** — If an installed CLI fails or times out once during a knowledge run, trying it again for each of the remaining conversations in the batch would multiply the wait time. Resolved in `CliJudge::judge_with` by recording any `CliProvider` that returns `JudgeOutcome::Pause` in `self.failed` (`Mutex<Vec<CliProvider>>`) so each failing CLI is tried at most once per `CliJudge` run.

## Acceptance

- `cargo test --locked` passes for `tests::judge_command_is_read_only_ephemeral_and_runs_outside_the_project`, `tests::builds_a_read_only_draft_command_for_every_agent_operon_launches`, `tests::cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones`, and `tests::knowledge_ui_shows_embedding_gemma_device_and_available_cli_summary`.
- In the running app, `プロジェクト › ナレッジ` shows `EmbeddingGemma 2 (GPU (Metal))` in the status line when enabled, `使用可能 (GPU (Metal) / 310MB)` beside the enable button when disabled with the model downloaded, and `今すぐ更新` distils conversations without failing on Antigravity CLI.

## Rejected alternatives

- **Add a Settings dropdown to pick the knowledge judge CLI** — Unnecessary configuration burden when Operon can automatically prefer the conversation's own CLI and fall back to any installed and working CLI.
- **Modify `KEEP_EXEMPLARS` in `src/knowledge.rs`** — `KEEP_EXEMPLARS` is part of the pinned golden embedding fixture (`tests/fixtures/embedding-golden.json`); changing its text would invalidate the fixture vectors.
