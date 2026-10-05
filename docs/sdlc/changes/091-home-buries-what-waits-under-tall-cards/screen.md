# Screen: Home, before and after

- **Status**: approved

The owner approved the rendered mock `.tmp/ui-before-after/view-home-after.html`
(section ②, "全項目 OK、進めて", 2026-09-27). This file is the part of it this
change builds; the tiles were built by change 090.

## Layout

```
ホーム
いま何が動いていて、どれがあなたを待っているか。
──────────────────────────────────────────────────────────────
[✓ あなたを待っているセッションはありません — 要対応になるとここに先頭表示されます]
[⚠ 要対応 0] [◌ 実行中 0] [○ 待機中 2] [✓ 終了 5]

最近のセッション                    すべて表示 │ プロジェクトから始める
○ 待機中  .tmpにテスト生成  A  design-lookbook  ⎇ develop  3分前 [開く] ··· │ ▢ design-lookbook  [＋ セッション]
○ 待機中  新しいセッション   A  design-lookbook  ⎇ develop 12分前 [開く] ··· │ ▢ operon           [＋ セッション]
✓ 終了    Command+click…    C  operon          ⎇ feature…  2時間前 [再開] ··· │
```

## 要対応 line

| Waiting | Shown |
|---|---|
| 0 | one quiet card: `ICON_STATUS_DONE` in the success colour, 「あなたを待っているセッションはありません」, muted 「— 要対応になるとここに先頭表示されます」 |
| n > 0 | today's warning card (「n 件のセッションがあなたを待っています」 and 「要対応のセッションを見る」), and under it one row per waiting session, at most five, in the recent-row form |

## Recent row

One line; the whole row opens the session's terminal, as the card did.

| Part | Content |
|---|---|
| state | the status chip (group word, hover sentence) |
| name | session title, strong, truncated to fit |
| agent | agent chip |
| project | project name, muted |
| branch | `ICON_BRANCH` branch, branch colour, when there is one |
| age | relative time, muted |
| action | the session's verb when it restarts something (再開 / 会話 ID から再開 / 再試行 / 今すぐ開始); otherwise secondary 「開く」 |
| ··· | menu: 停止 (when running), ターミナルを閉じる / 残ったターミナルを閉じる (when finished or failed), セッションを削除; then the details the card showed — goal, tmux name, cwd, native resume command or 「ネイティブ再開 ID を待機中」, dependency count, blocked-by line — as small weak text |

The removal confirmation for a running session appears under its row, as it
did under the card.

## プロジェクトから始める

A right column, 300 px wide, beside the recent list. Heading
「プロジェクトから始める」; one row per project: `ICON_PROJECT` name, and a
secondary 「＋ セッション」 that opens that project's launch form (what the
sidebar's per-project "+" does). With no project, the column is not drawn.

## Non-happy states

| State | Shown |
|---|---|
| No sessions, no projects | today's empty state; no right column |
| No sessions, projects | today's empty state in the left column; the right column still lists projects |
| A narrow window (< 720 px content) | the right column moves under the recent list |
