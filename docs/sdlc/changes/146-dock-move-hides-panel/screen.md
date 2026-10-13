# Screen: a refused dock move

No layout changes. Two states are added inside existing palette roles.

## Dragging over an edge that has no room (window 1200 × 640, files on top)

```
┌───────────────────────────────────────────────────────────────┐
│[⠿ファイル]                                                     │
├━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━━━━━━━━┤
│⠿プロジェクト別┃       ターミナル          ┃[⠿会話][⠿変更]       │
│┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐ │
│  (warning outline)      ウィンドウが小さいため下部には置けません    │
│└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘ │
└───────────────────────────────────────────────────────────────┘
 release → nothing moves; notice 「ウィンドウが小さいため下部には置けません」
```

## A grip's right-click menu in the same window

```
 ┌──────────────┐
 │ 左側に移動    │
 │ 右側に移動    │   (greyed: the panel is already here)
 │ 上部に移動    │
 │ 下部に移動    │   (greyed; hover: ウィンドウが小さいため下部には置けません)
 └──────────────┘
```

The placement menu (the group's 「パネルの配置を変更」 button) greys its entries
the same way, with the same text on hover.

## Over the panel's own edge, or the middle

The hint reads 「ここで離すとキャンセル」.
