# Screen: the session column lists the open tabs, in their order, and a session can be pinned

- **Status**: approved

No new screen or modal. The column's session rows change membership and order
(they no longer group by state), rows gain a drag and a pin mark, tabs gain a
pin mark in place of ×, and both context menus gain one entry.

## Session page, project p with four sessions (window 1400px)

Before — the column groups by state; the strip in open order; they disagree:
```
┌ 左列 ──────────────────────┐ ┌ タブ ──────────────────────────────────────────┐
│ p            [!]1 [●]1  +  │ │ ● A × │ ● B × │ ● D(完了) × │ +                  │
│  [✳!] B   (要対応)          │ │                                              │
│  [✳●] C   (実行中)          │ │  ← C's tab was closed; D ended               │
│  [✳·] A   (アイドル)        │ │                                              │
└────────────────────────────┘ └──────────────────────────────────────────────┘
```
After — one list, one order; D stays until closed; C (closed) is gone from both:
```
┌ 左列 ──────────────────────┐ ┌ タブ ──────────────────────────────────────────┐
│ p            [!]1 [·]1  +  │ │ ● B 📌 │ ● A × │ ● D ×  │ +                     │
│  📌 [✳!] B                  │ │   ↑ pinned: first, pin in place of ×           │
│     [✳·] A                  │ │                                              │
│     [✳✓] D                  │ │                                              │
└────────────────────────────┘ └──────────────────────────────────────────────┘
```
`📌` stands for `ICON_PIN` (Phosphor push-pin, 11.5px, muted ink); `[✳!]` is the
CLI mark with its state badge (change 284), unchanged.

## Context menu (row and tab)

```
┌──────────────┐        ┌────────────────┐
│ 名前を変更    │        │ 名前を変更      │
│ ピン留め      │        │ ピン留めを外す  │   ← on a pinned session
└──────────────┘        └────────────────┘
```

## Dragging a row

Hover adds 「ドラッグで並べ替え」. While dragging, the accent line (the project
drag's, change 181) shows the gap among the project's rows of the same pin
state; release moves the row and the tabs reorder to match.

## States

- ⌘W on a pinned tab: brief notice
  「ピン留め中のセッションは閉じられません。ピン留めを外してから閉じてください。」
- A project with no tabs: 「開いているセッションはありません」
- Order file not written: 「並び順を保存できませんでした: {error}」 (existing).
