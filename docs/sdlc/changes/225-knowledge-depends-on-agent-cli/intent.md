# Intent: knowledge distillation must not fail on Antigravity CLI, depend on a single vendor CLI, or leave EmbeddingGemma 2's status unclear

- **Status**: approved
- **Opened**: 2026-10-26

## Problem

Three issues combine to make `プロジェクト › ナレッジ` fail with a confusing error and appear to require additional setup:

1. Clicking `今すぐ更新` in a project that contains Antigravity conversations fails with `Antigravity CLI で判定できませんでした。次回の更新で再試行します。` because Antigravity (`agy`) is invoked with `--print --sandbox <prompt>`, causing `agy` to treat `"--sandbox"` as the prompt text and exit with status 2.
2. Each conversation is only ever summarised by the CLI that originally held it, and the first CLI pause stops judging for the entire project on that run. When a project contains conversations from a CLI that is not installed, not signed in, or failing on this Mac, those conversations can never be distilled and block subsequent conversations in the same run — even when another supported CLI (Claude Code, Codex, or Antigravity) is installed and ready.
3. Once knowledge collection is enabled on a project, the `プロジェクト › ナレッジ` tab never shows that `EmbeddingGemma 2` is active; before enabling, the card does not show `使用可能 (GPU (Metal) / 310MB)` when the model is already downloaded; and `設定 › データ` labels the local-history radio toggle simply `検索` without explaining that project knowledge uses the model automatically once downloaded.

## Who feels it, and when

- Anyone clicking `今すぐ更新` (or waiting for the 10-minute background distillation) in a project with Antigravity (`agy`) conversations, or conversations from a CLI that is not currently installed or signed in on this Mac.
- Anyone looking at `プロジェクト › ナレッジ` or `設定 › データ` to check whether `EmbeddingGemma 2` is already working or requires manual setup.

## Desired outcome

- Clicking `今すぐ更新` in a project with Antigravity conversations distils candidates without failing on `agy`'s `--print` flag order.
- A conversation from any supported CLI is summarised by its own CLI when available and healthy, or automatically by another installed CLI (`Claude Code`, `Codex`, or `Antigravity`) when the conversation's own CLI is unavailable or fails; a failing CLI is skipped for the rest of the run rather than retried on every conversation, and does not block other providers.
- The `プロジェクト › ナレッジ` tab shows `EmbeddingGemma 2 ({device})` in its normal status line, shows `使用可能 ({device} / 310MB)` beside `このプロジェクトでナレッジを集める` when the model is ready, states that any available CLI (`Claude Code・Codex・Antigravity`) summarises candidates, and `設定 › データ` states that project knowledge uses the model automatically while the radio toggle is for `ローカル履歴の検索`.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese (with English and Korean translations in `src/i18n_tables.rs`); code, comments, and docs are English.

## Systems likely affected

- `src/agents.rs`
- `src/knowledge.rs`
- `src/app/knowledge.rs`
- `src/i18n_tables.rs`
- `src/tests.rs`

## Open questions

None.

## Not in scope

- Changing the local ranking algorithm, exemplar vectors (`KEEP_EXEMPLARS` / `DROP_EXEMPLARS`), or SQLite schema.
- Adding a separate picker in Settings to manually pin a single knowledge judge CLI.
