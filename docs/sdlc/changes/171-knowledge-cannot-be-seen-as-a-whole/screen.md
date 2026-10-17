# Screen: 171 — ナレッジ tab, マップ view

- **Status**: approved
- Structure: a new view inside the existing ナレッジ tab (a person approves).
  The list view, the disabled state, and Settings are unchanged.

## Before (157) → After: the controls row

Before:
```
│  [🔍 ナレッジを検索                                    ]                        │
│  並び: (●) 鮮度  ( ) 参照回数  ( ) 新しい順        種類: [すべて ▾]              │
```

After (shown once the project has ≥ 1 entry; 「並び」 is hidden in マップ):
```
│  [🔍 ナレッジを検索                         ]   表示: [ 一覧 ][■マップ]  種類: [すべて ▾] │
```

## マップ — nothing chosen (880px content width)

```
┌─ … エージェント設定  [ナレッジ] ───────────────────────────────────────────────── 880px ─┐
│  ナレッジ                                                [ 今すぐ更新 ]  [ 収集を止める ▾ ] │
│  142 件 ・ 最終更新 3分前 ・ 本日の判定 7/30 回                                             │
│  [🔍 ナレッジを検索                         ]   表示: [ 一覧 ][■マップ]  種類: [すべて ▾]   │
│ ┌──────────────────────────── 560 × 420 ──┐  ┌─ グループ ──────────── 290 ─┐            │
│ │        ╭──────────╮                [⤢]  │  │ 復元で最初のターンが消…  18 件 │            │
│ │        │ ● ●  ◆   │  復元で最初の…      │  │ ███████░░░                   │            │
│ │        │  ●─■  ●  │                     │  │ tmux の再試行は3回で…    11 件 │            │
│ │        ╰──────────╯       ╭─────╮       │  │ █████░░░░░                   │            │
│ │   ▲                       │◆ ◆ ■│       │  │ リリースは package-m…     9 件 │            │
│ │        ○      ╭────╮      │ ◉ ● │       │  │ ████████░░                   │            │
│ │               │■ ● │      ╰─────╯       │  │ …                            │            │
│ │  tmux の再試… │ ▲ ◆│  リリースは pa…    │  │ その他 (14 件)               │            │
│ │               ╰────╯                    │  │                              │            │
│ └─────────────────────────────────────────┘  └──────────────────────────────┘            │
│  ◆ 決定  ● 原因  ■ 手順  ▲ 注意  ○ 事実 ・ 大きさ: 参照回数 ・ 濃さ: 鮮度 ・ ◉ 固定          │
│ ─────────────────────────────────────────────────────────────────────────────────────── │
│  エージェントから参照: /Applications/Operon.app/Contents/MacOS/Operon knowledge search "…" │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

- Points: shape = kind; radius 3–9px by √(参照 + 再出現); opacity = 鮮度
  (floor 25%); 鮮度 < 30% drawn in `warning`; 固定 = a ring (◉).
- Lines: thin `border_subtle` between near meanings; the hovered or chosen
  point's lines in `accent`, its neighbours full, everything else dimmed.
- Groups: soft `accent_soft` outline around members, name in `text_muted`
  beside it (the most central entry's title, 16 characters + 「…」).
- Zoomed in, entry titles appear next to points where they do not overlap,
  most-used first. [⤢] is 「全体を表示」. Drag pans; scroll or pinch zooms.
- Hover tooltip:
  ```
  ┌──────────────────────────────────────┐
  │ 復元で最初のターンが消えるのは header … │
  │ 原因 ・ 鮮度 62% ・ 参照 4              │
  └──────────────────────────────────────┘
  ```
- Choosing a group in the panel frames its members, dims the rest, and the
  panel lists its titles:
  ```
  ┌─ ← グループ一覧 ─────────────── 290 ─┐
  │ 復元で最初のターンが消…   18 件      │
  │  ● 復元で最初のターンが消えるのは…   │
  │  ◆ header 行は session_meta として…  │
  │  ■ 復元の確認は ignored テストで…    │
  │  …                                   │
  └──────────────────────────────────────┘
  ```

## マップ — an entry chosen

The right panel becomes 157's detail block for that entry; the list shows the
same entry open when switched back.

```
┌─ ← グループ一覧 ─────────────────────── 290 ─┐
│ 原因  復元で最初のターンが消えるのは header   │
│       行の誤分類                              │
│ ██████░░░░  参照 4 ・ Codex 10/09              │
│                                               │
│ header 行を session_meta として捨てていた。   │
│ TRANSCRIPT_VOCABULARY に分類を追加して直した。│
│                                               │
│ 出典: Codex ・ 8日前          [ 会話を開く ]  │
│ [ 固定 ]  [ 削除 ]                            │
└───────────────────────────────────────────────┘
```

## States

Computing for the first time (map area):
```
│ ┌─────────────────────────────────────────┐  ┌─ グループ ─────────────────┐ │
│ │                                         │  │                            │ │
│ │          ⟳ 配置を計算しています…         │  │                            │ │
│ │                                         │  │                            │ │
│ └─────────────────────────────────────────┘  └────────────────────────────┘ │
```
When a layout exists and entries changed, the old map stays and new points
appear when the update finishes; no spinner.

Failure (`danger` banner above the map; the list still works):
```
│  ⚠ マップを作れませんでした: database is locked                [ 再試行 ]   │
```

One or two entries: the points, no lines, and the panel reads
「グループはまだありません」.

Search with hits: non-matching points dimmed to 15%; the panel's first line
reads 「12 件が一致」 above the groups. No hit: every point dimmed, panel reads
「一致するナレッジはありません。」.

No entries: the 表示 switch is not shown; 157's empty text stays.

Narrow window (content < 700px): the panel moves below the map
(map full width × 360, panel full width).
