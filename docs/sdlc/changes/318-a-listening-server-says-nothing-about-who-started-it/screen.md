# Screen: Who started each listening server, when, and with what

- **Status**: approved

Session header and the ports menu and stop confirmation behind it (change 262).
The agent chip, title, status chip and branch are unchanged.

## Header badge

Unchanged, except that when the listed servers (this folder's and the rest of
the project's) include any marked 「終了忘れ?」, a second muted-warning segment
follows the count:

```
│ [Claude] Operon を改善   ● 稼働中 · ⎇ main · ↗ 3 ポート稼働中 · ⚠ 終了忘れ? 1 │
```

`⚠` is `ICON_ATTENTION`, the segment is `palette.warning`, 11.5px like the badge.
It is part of the same button and opens the same menu.

## Menu rows

Each row keeps its first line (`[:port process]`, folder label in the
「プロジェクト内のほかの場所」 section, copy, stop) and gains two lines, 11.5px,
`text_muted`, indented under the port button:

```
┌───────────────────────────────────────────────┐
│ この作業フォルダ                                │
│ [:5173 node]  ⚠ 終了忘れ?              ⧉ ■    │
│   「ログイン修正」(停止中) · 11/24 19:30 · 14時間前 │
│   npm run dev -- --port 5173                    │
│ [:3000 node]                           ⧉ ■    │
│   「API 追加」 · 10:02 · 4分前                   │
│   node server.js                                │
│ [:8080 python3]                        ⧉ ■    │
│   起動元不明 · 親は終了済み · 11/23 09:12 · 1日前 │
│   python3 -m http.server 8080                   │
└───────────────────────────────────────────────┘
```

- 「終了忘れ?」 is `palette.warning`, after the port button, with a hover text
  saying why (「起動したセッションは停止しています」 or
  「起動したセッションは削除されています」).
- The command line is truncated to the row width; hovering shows it in full
  (redacted) with the working folder.
- A field that could not be read is left out of the line, not shown as blank.

## Stop confirmation

```
┌ ポートを停止 ────────────────────────────────────┐
│ :5173 node を停止しますか？                       │
│ 起動したセッション  「ログイン修正」(停止中)         │
│ 起動時刻            11/24 19:30 (14時間前)          │
│ コマンド            npm run dev -- --port 5173      │
│ フォルダ            ~/dev/app/.worktrees/login      │
│ PID                 20014                           │
│ 親プロセス          終了済み                         │
│ 終了シグナル (SIGTERM) を送ります。…                │
│                     [キャンセル] [停止する]          │
└───────────────────────────────────────────────────┘
```

The command wraps in the modal rather than truncating.
