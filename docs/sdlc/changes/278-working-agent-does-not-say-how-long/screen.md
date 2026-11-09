# Screen: the session column's row for a working agent

セッション page, the left column at its usual 280px. No layout change: the
row's second line gains a time after its state word.

## Before

```
┌ プロジェクト別セッション ──── 280px ─┐
│ operon                               │
│ ▍ ログイン画面を直す                 │
│   ✳ · 実行中                         │
│   テストを追加する                   │
│   ◇ · 実行中 · feature/tests        │
│   README を更新                      │
│   ✳ · アイドル                       │
└──────────────────────────────────────┘
```

## After

```
┌ プロジェクト別セッション ──── 280px ─┐
│ operon                               │
│ ▍ ログイン画面を直す                 │
│   ✳ · 実行中 · 12分                  │   working for 12 minutes
│   テストを追加する                   │
│   ◇ · 実行中 · 2時間 · feature/te…  │   working for 2 hours; the branch truncates
│   README を更新                      │
│   ✳ · アイドル                       │   not working: unchanged
└──────────────────────────────────────┘
```

- **Under a minute**: 「実行中」 alone, as before.
- **起動中 / 順番待ち** (also shown as 「実行中」): no time — the agent is not
  working yet.
- **Empty** (no sessions), **error**: unchanged.
- **Hover** on the state word: 「応答を生成中 · 12分」 or 「Bash を実行中 · 12分」.
- **Selected row**: the time takes the selected row's second ink with the
  state word, as the word does today.
