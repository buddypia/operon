# Spec: Built-in Opt-in Semantic Search Replacing External eg2 Dependency

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. `external_eg2_subprocess_logic_is_completely_removed`: `rank_transcripts_with_eg2`, `rank_fallback_candidates_with_eg2`, `parse_eg2_rank_output`, circuit breaker constants/atomics, and `Command::new("eg2")` are removed from `src/cli.rs`.
2. `semantic_search_engine_is_in_process_and_opt_in`: Semantic search runs strictly in-process using embedded weights/engine (or falls back safely to keyword search when not enabled/cached).
3. `search_preference_and_model_state_are_tracked`: The user's search preference (Keyword vs Built-in Semantic) and download state are tracked in `OperonStore` or local cache state.
4. `settings_ui_provides_opt_in_download_and_removal`: `ui_settings_data` displays radio options for "キーワード検索のみ" and "内蔵 EmbeddingGemma 2", with download progress bar and model deletion controls.
5. `search_gracefully_falls_back_when_model_absent`: If the model is not downloaded or disabled, transcript search operates in fast keyword mode without errors, background hangs, or missing-file warnings.

## Behaviour

- **Default State**:
  - When the app starts, search mode is "キーワード検索のみ".
  - Transcript search performs exact substring matching with zero background delays and zero network usage.
- **Opt-in Flow**:
  - In Settings > Data, selecting "内蔵 EmbeddingGemma 2" shows `[ モデルをダウンロード (約 250MB) ]`.
  - Clicking download initiates a background download thread with progress updates rendered at 60fps in the settings view.
  - Upon completion, state transitions to `● 有効 (Metal GPU / キャッシュ済み: 248MB)` and semantic ranking is immediately enabled for subsequent transcript searches.
  - Clicking `[ キャッシュを削除 ]` removes model weights from `~/Library/Caches/local.operon/models` and switches back to keyword mode.
- **Search Experience**:
  - When semantic search is active, results carry similarity scores and display the percentage badge (e.g. `(76%)`).
  - When keyword-only search yields 0 results, an inline hint informs the user that semantic search is available in Settings.

## Design

- **Placement Verdict**: `EXTEND` (`src/cli.rs`, `src/app/screens.rs`, `src/store.rs`, `src/tests.rs`).
- **Data Model**:
  - `TranscriptMatch { pub(crate) provider: String, pub(crate) session_id: String, pub(crate) title: Option<String>, pub(crate) path: PathBuf, pub(crate) snippet: String, pub(crate) score: Option<f64> }` (retained).
  - `SearchEngineMode`: `Keyword` (default) | `EmbeddedGemma2`.
- **Model Cache Directory**:
  - `~/Library/Caches/local.operon/models/embeddinggemma-2-270m/`

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | No | Uses existing theme colors and badges. |
| Icons | No | Uses existing Phosphor icons. |
| Identifier SSOT | No | No cross-module shared identifier strings. |
| Durability | Yes | Local cache cleanup handles missing or partial files safely. |
| Subprocess safety | Yes | External `eg2` subprocess spawning is eliminated. |
| Documentation | Yes | Documentation and comments in English; UI in Japanese. |
| Local-first | Yes | Zero telemetry; downloads occur strictly on explicit user action. |
| Budgets | Yes | Invocations limit candidate items to 24 documents; memory freed on demand. |

## Flagged concerns

- **Concern**: Binary size bloat if model weights were bundled.
  - **Answer**: Model weights are NOT bundled in the application binary; they are downloaded on-demand to standard cache directories.
- **Concern**: Network or download failure mid-transfer.
  - **Answer**: Downloads use a temporary file (`.downloading`) and atomic rename on completion. Incomplete downloads are cleanly cleaned up on error or cancel.

## Acceptance

- `cargo test --locked` passes.
- All references to `eg2` subprocess and circuit breaker are eliminated.
- Settings view renders the engine radio buttons and model management controls as described in `screen.md`.

## Rejected alternatives

- Retaining external `eg2` CLI as a third option: Rejected per user decision to keep Operon simple, self-contained, and maintainable.
- Bundling model weights in `Operon.app`: Rejected due to binary bloat (+250MB) and violation of lean download principles.
