# Screen: draggable sidebar contents and left sidebar file list default

## Before

The session workspace had a fixed left column strictly for session tabs, the central terminal, and a right inspector column containing files, conversation, and changes.

```
┌──────────────────────┬──────────────────────────────────────────┬────────────────────────┐
│ 履歴 2件             │ A .tmpにテスト生成 ○IDLE · operon        │ ファイル│会話 1│変更 1 ⟩ │
│                      │ ┌terminal──────────────────────────────┐ │ [ファイルを絞り込む  ] │
│ ● session-1          │ │                                      │ │ ターミナルに出たファイル│
│ ● session-2          │ │                                      │ │ › src                │
│                      │ └──────────────────────────────────────┘ │ › tests                │
└──────────────────────┴──────────────────────────────────────────┴────────────────────────┘
```

## After

The file list is on the left sidebar by default, with sessions available as a tab alongside it. Conversation and Changes are in the right sidebar. Any tab can be dragged between left and right sidebars or reordered. Splitters are horizontally resizable by dragging.

```
┌──────────────────────┬──────────────────────────────────────────┬────────────────────────┐
│[ファイル] [セッション]│ A .tmpにテスト生成 ○IDLE · operon        │ [会話 1] [変更 1]   ⟩ │
│[ファイルを絞り込む  ]│ ┌terminal──────────────────────────────┐ │ 会話履歴                │
│ターミナルに出たファイル│ │                                      │ │ 1. 2026-10-06 01:00   │
│› src                 │ │                                      │ │    "ファイル一覧の…"  │
│› tests               │ │                                      │ │                      │
│                      │ └──────────────────────────────────────┘ │                      │
└──────────────────────┴──────────────────────────────────────────┴────────────────────────┘
   ▲                                                                 ▲
   │ (Drag tab across to swap/relocate)                              │
   └─────────────────────────────────────────────────────────────────┘
```

- When the user drags 「ファイル」 from the left header to the right header, it transfers to the right sidebar.
- When the user drags the splitter line between sidebars and terminal, the cursor shows `ResizeHorizontal` and the sidebar width updates smoothly.
- When all tabs from a sidebar are dragged to the other, the empty sidebar collapses to 0 width.

## States that are not the happy one

| State | Shows |
|---|---|
| Sidebar has no tabs | The sidebar collapses to 0px and the terminal expands |
| No files in project | 「読み取り可能なファイルはありません」 inside the Files tab |
| No active sessions | 「セッションはまだありません。プロジェクトから起動してください。」 inside the Sessions tab |
| Both sidebars collapsed | The terminal takes the full window width |
