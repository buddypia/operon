# Screen: explicit EmbeddingGemma 2 status and CLI-agnostic copy in Knowledge and Settings

- **Change**: `225-knowledge-depends-on-agent-cli`
- **Status**: approved

## Where it lives

Two existing surfaces in `src/app/knowledge.rs` (no new tabs, modals, or layout structural changes):

1. `プロジェクト › ナレッジ` (`ui_knowledge_disabled` when disabled; `ui_knowledge_header` / `knowledge_status_line` when enabled).
2. `設定 › データ` (`ui_model_block`).

## ASCII wireframes

### 1. `プロジェクト › ナレッジ` (disabled, model ready)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ このプロジェクトの会話からナレッジを集めます                             │
│                                                                          │
│ Claude Code・Codex・Antigravity との会話から、決定・原因と対処・手順・   │
│ 注意点を短いナレッジにまとめ、会話ごと検索できるようにします。使われない │
│ ものは徐々に薄れて消え、参照されたり繰り返し出てきたりしたものは残ります。│
│                                                                          │
│ ・意味の判定は、この Mac 上の EmbeddingGemma 2 で行います。              │
│ ・残す候補だけを、利用可能な CLI (Claude Code・Codex・Antigravity) で    │
│   要約します (1日 30 回まで)。                                           │
│ ・ナレッジはこの Mac の Operon データフォルダに保存し、リポジトリには    │
│   書きません。                                                           │
│                                                                          │
│ [ このプロジェクトでナレッジを集める ]  使用可能 (GPU (Metal) / 310MB)   │
└──────────────────────────────────────────────────────────────────────────┘
```

### 2. `プロジェクト › ナレッジ` (enabled, normal status line)

```text
ナレッジ                 [表示: 一覧 | マップ] [今すぐ更新] [一時停止] [ナレッジを削除…]
5 件 ・ 最終更新 2分前 ・ EmbeddingGemma 2 (GPU (Metal)) ・ 本日の判定 2/30 回
```

### 3. `設定 › データ` (`ui_model_block`)

```text
意味検索（EmbeddingGemma 2）
この Mac 上で動くモデルです。会話の内容は外部に送りません。プロジェクトのナレッジではモデルがあればそのまま使われます。

ローカル履歴の検索  ( ) キーワードのみ  (●) 意味検索

使用可能 (GPU (Metal) / 310MB)  [ モデルを削除 ]
```

## States

- **Disabled + `ModelState::Ready { device }`**: Shows primary button `このプロジェクトでナレッジを集める` followed by `使用可能 ({device} / 310MB)` in `palette.success`.
- **Disabled + `ModelState::Missing` / `Downloading` / `Failed`**: Unchanged (`モデルを取得して始める (310MB)` or `ui_model_state_row`).
- **Enabled + `ModelState::Ready { device }` (idle, no warning)**: Status line shows `{n} 件 ・ 最終更新 {updated} ・ EmbeddingGemma 2 ({device}) ・ 本日の判定 {used}/{cap} 回` in `palette.text_muted`.
- **Enabled + running / warning (`paused`, `capped`, `full_of_pinned`, `rebuilding`, model not ready)**: Existing warning/progress status lines remain unchanged.

## Design rules

- Uses existing semantic palette tokens (`palette.text_strong`, `palette.text_muted`, `palette.success`, `palette.warning`, `palette.danger`) and existing spacing constants (`SPACE_SM`, `SPACE_MD`).
- No new icons or font glyphs introduced.
