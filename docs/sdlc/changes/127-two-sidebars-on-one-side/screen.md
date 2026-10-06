# Screen: one left sidebar — sessions above, files / conversation / changes below

## Before

```
┌ プロジェクト別セッション ┬ [ファイル][会話][変更] ┬ terminal ───────────┐
│ すべて 要対応 …          │ filter                 │                     │
│ ▸ operon                 │ src/                   │                     │
│   ● session A            │ docs/                  │                     │
│   ○ session B            │ …                      │                     │
│                          │                        │                     │
└──────── 268px ───────────┴─────── ~280px ─────────┴─────────────────────┘
```

## After

```
┌ プロジェクト別セッション ┬ A session title ● 実行中 · operon / branch ──┐
│ すべて 要対応 …          │ ┌terminal─────────────────────────────────┐ │
│ ▸ operon                 │ │                                         │ │
│   ● session A            │ │   (~280px wider than before)            │ │
│   ○ session B            │ │                                         │ │
├──────── ⇕ drag ──────────┤ │                                         │ │
│ [ファイル][会話 3][変更 2]│ │                                         │ │
│ filter                   │ │                                         │ │
│ src/  docs/  …           │ └─────────────────────────────────────────┘ │
└──────── 268px ───────────┴─────────────────────────────────────────────┘
```

## States that are not the happy one

| State | Shows |
|---|---|
| Side panel folded | session list fills the column; 「サイドパネルを表示」 in the header |
| Side panel moved right | session list fills the left column; panel is a right column as in 125 |
| History view open / no session selected | session list fills the column |
| Window short (column < 240px) | each half gets half the column |
