# Screen: each workspace panel docks on any of the four edges

Approved by the person on 2026-10-13 (option 「同じ縁では一覧と詳細を分割」).

## Before (window 1200 × 800)

```
┌──────────────┬──────────────────────────┬────────────────────┐
│プロジェクト別 ┃ ヘッダ                    ┃[⠿ファイル][⠿会話]… │
│セッション     ┃                          ┃                    │
│ ● session A  ┃       ターミナル          ┃  📁 src/           │
│ ● session B  ┃                          ┃                    │
└──────────────┴──────────────────────────┴────────────────────┘
 dragging any of ⠿ファイル/⠿会話/⠿変更 moves all three, left or right only;
 the session list cannot move.
```

## After — default (unchanged look)

```
┌──────────────┬──────────────────────────┬────────────────────┐
│⠿プロジェクト別┃ ヘッダ                    ┃[⠿ファイル][⠿会話]… │
│ セッション    ┃       ターミナル          ┃  📁 src/           │
└──────────────┴──────────────────────────┴────────────────────┘
```

## After — list left, files bottom, conversation + changes right

```
┌──────────────┬──────────────────────────┬────────────────────┐
│⠿プロジェクト別┃ ヘッダ                    ┃[⠿会話 3][⠿変更 2]  │
│ セッション    ┃                          ┃                    │
│ ● session A  ┃       ターミナル          ┃  会話の一覧         │
│ ● session B  ┃                          ┃                    │
├━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━┤
│[⠿ファイル]                                 [⚙][>]            │
│ 📁 src/   📁 docs/   README.md …                              │
└───────────────────────────────────────────────────────────────┘
```

## After — list and files both on the top edge (side by side)

```
┌──────────────────────┃────────────────────────────────────────┐
│⠿プロジェクト別セッション┃[⠿ファイル]                   [⚙][>]   │
│ ● session A          ┃ 📁 src/  📁 docs/                       │
├━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━━━━━━┤
│ ヘッダ                                          ┃[⠿会話][⠿変更]│
│                 ターミナル                       ┃              │
└─────────────────────────────────────────────────┴──────────────┘
```

## States that are not the happy one

| State | Shows |
|---|---|
| Dragging a grip | Closed-hand cursor; four faint edge frames; the one under the pointer framed in accent; tooltip 「ファイルを下部に移動」 |
| Pointer in the middle | No frame highlighted; tooltip 「中央で離すとキャンセル」 |
| Released in the middle / Escape | Nothing moves, no notice |
| Dropped on an edge | That panel moves; brief notice 「ファイルを下部に移動しました」 |
| Right-click a grip | Menu 「左側に移動」「右側に移動」「上部に移動」「下部に移動」, current edge disabled |
| [⚙] 「パネルの配置を変更」 | Same four entries, moving the whole group |
| Folded (「サイドパネルを畳む」) | All view groups gone, list stays; header shows 「サイドパネルを表示」 |
| Narrow window (< ~760px) | Edges shrink to minimum, then right/bottom groups drop; terminal keeps ≥ 300 × 240 |
| No project for the session | Only 会話 offered; a group of only ファイル/変更 is not drawn |
