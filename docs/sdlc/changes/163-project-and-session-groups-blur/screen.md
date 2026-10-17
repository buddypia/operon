# Screen: the session column groups by project and stops from the row

No new screen. The session column of the terminal workspace (about 280px wide
at the default split in a 1280px window) changes how it groups and what its row
corner offers.

## Before (measured: dev build, copied store, 1280 × 820)

```
│ ⠿ プロジェクト別セッション 7 件   履歴 │
│ [すべて 7] 要対応 実行中 待機中 4 終… │
│ ───────────────────────────────────── │
│ ▭ design-lookbook  3 件          [＋] │  ← 12px name, smaller than titles
│                                       │
│ ● 계속                        ··· 12時│  ← time cut at the edge
│   Codex · 終了 · main                 │
│                                       │  ← same gap as between projects
│ ● 계속                        ··· 12時│
│   Claude · 終了 · main                │
│                                       │
│ ● design-lookbook · Antigravity ··· 1…│
│   Antigravity · 待機中 · main         │
│                                       │
│ ▭ EmbeddingGemma2-ultrafast  1 件 [＋]│
│ ● EmbeddingGemma2-ultrafast …   ··· 6…│
│ hover: ··· and ✕ only; 停止 only in the open terminal's header
```

## After

```
│ ⠿ プロジェクト別セッション        履歴 │  ← whole; the count is 「すべて 7」's
│ [すべて 7] 要対応 実行中 待機中 4     │
│ 終了 3                                │  ← chips wrap instead of widening
│                                         the column
│ ───────────────────────────────────── │
│ ╭───────────────────────────────────╮ │
│ │ ▭ design-lookbook  3 件      [＋] │ │  ← 15px strong, text_strong
│ │ ● 계속                12時間前 ···│ │
│ │   Codex · 終了 · main             │ │
│ │ ───────────────────────────────── │ │  ← border_subtle rule
│ │ ● 계속                12時間前 ···│ │
│ │   Claude · 終了 · main            │ │
│ │ ───────────────────────────────── │ │
│ │ ● design-lookbook ·…  [■][✕] ···│ │  ← selected (row_selected fill) or hovered, running
│ │   Antigravity · 待機中 · main     │ │
│ ╰───────────────────────────────────╯ │
│ ╭───────────────────────────────────╮ │
│ │ ▭ EmbeddingGemma2-ultra… 1 件 [＋]│ │  ← long name truncates; hover: full name, path
│ │ ● EmbeddingGemma2-ul…   6日前 ···│ │
│ │   Antigravity · 待機中 · main     │ │
│ ╰───────────────────────────────────╯ │
│ ╭───────────────────────────────────╮ │
│ │ ▭ new-app  0 件              [＋] │ │  ← empty project: header only
│ ╰───────────────────────────────────╯ │
```

`···` is pinned to the far right of every row: when the pointer leaves the
row for its open menu, ■ ✕ turn back into the time and `···` must not move
with them. Measured screenshots: `.tmp/shots/after-dark-s.png`,
`.tmp/shots/after-light-s.png` (dev build, copied store, 1280 × 820).

Frames: fill `raised`, 1px `border_subtle`, radius 10, padding 8, 8 between.
`[■]` hover: 「実行中の CLI を停止します（記録は残ります）」. `[✕]` hover:
「セッションを削除」. `···` hover: 「その他の操作」.

## Row menu (`···` and right-click, same list)

Running session:

```
┌──────────────────────┐
│ 名前を変更            │
│ worktree              │
│ 未読にする            │
│ 停止                  │
│ セッションを削除      │
│ ──────────────────── │
│ Codex CLI へ履歴を復元│
│ Antigravity へ履歴を復元│
└──────────────────────┘
```

Finished, queued, or starting session: the same without 「停止」.
Stop pending: 「停止中…」 in its place (pressing retries).

## States

| State | Shows |
|---|---|
| 要対応 queue present | its own frame above the projects, header 「要対応 n」 in the needs-you ink |
| Selected row | `row_selected` fill inside the frame; rules unchanged |
| 停止 pressed | the row's status becomes 停止中…; then 終了 when tmux confirms |
| 削除 on a running row | the existing inline 「実行中のセッションを削除しますか？…」 under the row |
| Filter hides every session of a project | no frame for it |
| Narrow column | the heading's grip truncates, the chips wrap, titles truncate; the time / corner controls stay whole |
| Project name too long | `▭ EmbeddingGemma2-ultra… 1 件 [＋]` — the name truncates, 「n 件」 and ＋ stay inside the block |
| Session title too long | `● EmbeddingGemma2-ultrafa… ··· 6日前` — the title truncates first |
| Stop fails | the existing notice from `stop_session` (e.g. 「tmux を実行できませんでした: …」); the row keeps 停止中… so pressing ■ again retries |
| Theme | judged on the dark (the person's) and light screenshots: in dark the block reads by its hairline, like the home screen's cards |
