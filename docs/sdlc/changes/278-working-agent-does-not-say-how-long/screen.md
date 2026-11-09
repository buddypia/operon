# Screen: where a working agent says how long it has worked

No layout change: a time follows the state word where one is printed, and the
state's explanation where only a mark is.

## The per-project session list (履歴's list, and the docked session panel)

Before:

```
┌ プロジェクト別セッション ──── 280px ─┐
│ operon                               │
│   ログイン画面を直す                 │
│   ✳ · 実行中                         │
│   テストを追加する                   │
│   ◇ · 実行中 · feature/tests        │
│   README を更新                      │
│   ✳ · アイドル                       │
└──────────────────────────────────────┘
```

After:

```
┌ プロジェクト別セッション ──── 280px ─┐
│ operon                               │
│   ログイン画面を直す                 │
│   ✳ · 実行中 · 12分                  │   working for 12 minutes
│   テストを追加する                   │
│   ◇ · 実行中 · 2時間 · feature/te…  │   2 hours; the branch truncates
│   README を更新                      │
│   ✳ · アイドル                       │   not working: unchanged
└──────────────────────────────────────┘
```

## The open pane's heading (セッション page)

```
Before   operon / ログイン画面を直す
         ログイン画面を直す   ● 実行中                         [ 停止 ] …
After    ログイン画面を直す   ● 実行中 · 12分                  [ 停止 ] …
```

## Hovers — the project column's row and the tab's state mark print no word

```
Before   ┌──────────────────────┐      After   ┌──────────────────────────┐
         │ ログイン画面を直す   │              │ ログイン画面を直す       │
         │ 実行中               │              │ 実行中                   │
         │ 応答を生成中         │              │ 応答を生成中 · 12分      │
         └──────────────────────┘              └──────────────────────────┘
```

The first line, the session's title, is the project column row's alone: the
tab mark's hover has the two lines below it. The hover of a state chip or row word that already explained the state
(「応答を生成中」, or 「Bash を実行中」 when a hook names the tool) gains the same
ending.

- **Under a minute**: everything as before.
- **起動中 / 順番待ち** (also shown as 「実行中」): no time — the agent is not
  working yet.
- **Empty** (no sessions), **error**: unchanged.
- **Selected row**: the time takes the selected row's second ink with the
  state word, as the word does today.
