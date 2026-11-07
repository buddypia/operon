# Screen: the knowledge opt-in card says what agents are given

- **Change**: `258-agents-do-not-know-the-knowledge-exists`
- **Status**: approved

## Where it lives

プロジェクト › ナレッジ, disabled state (`ui_knowledge_disabled` in
`src/app/knowledge.rs`). One bullet added to the existing list; once enabled,
one muted line under the status line. No new control, icon, colour, or layout.

## ASCII wireframe

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ このプロジェクトの会話からナレッジを集めます                             │
│                                                                          │
│ Claude Code・Codex・Antigravity との会話から、…                          │
│                                                                          │
│ ・意味の判定は、この Mac 上の EmbeddingGemma 2 で行います。              │
│ ・残す候補だけを、利用可能な CLI (…) で要約します (1日 30 回まで)。      │
│ ・ナレッジはこの Mac の Operon データフォルダに保存し、リポジトリには    │
│   書きません。                                                           │
│ ・Claude Code と Codex のセッション開始時に、ピン留め・検証済み・最近    │  ← new
│   使われたものを優先してナレッジの題名 (最大 10 件) と検索方法をその     │
│   CLI に渡します。他の CLI との会話から作られた題名も含まれます。        │
│                                                                          │
│ [ このプロジェクトでナレッジを集める ]  使用可能 (GPU (Metal) / 310MB)   │
└──────────────────────────────────────────────────────────────────────────┘
```

### Enabled tab, under the status line

```text
ナレッジ                 [表示: 一覧 | マップ] [今すぐ更新] [一時停止] [ナレッジを削除…]
5 件 ・ 最終更新 2分前 ・ EmbeddingGemma 2 (GPU (Metal)) ・ 本日の判定 2/30 回
Claude Code と Codex のセッション開始時に、ナレッジの題名 (最大 10 件) と検索方法をその CLI に渡しています。   ← new
```

## Tokens

The bullet: same label call as the three bullets above it. The enabled line:
the status line's own muted small style, no new role or size.
