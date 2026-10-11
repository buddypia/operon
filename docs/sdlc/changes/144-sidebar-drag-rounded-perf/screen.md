# Screen: sidebar tab button rounded consistency, drag-and-drop affordance, and smooth rendering

## Before

```
┌ プロジェクト別セッション ──────────────────────────┐
│ すべて 1 (rounded)  要対応  待機中 1 …              │
│ ▸ operon 1件                                       │
│   ● session A                                      │
├────────────────────────────────────────────────────┤ <- Splitter: plain 1px line, no drag grip
│ [ファイル] [会話] [変更]                 [⇄] [>]   │ <- [ファイル] selected has sharp square corners (0px)
│ (square)                                           │    No drag handle icon indicating D&D
│ [ファイルを絞り込む          ] [⟳]                  │
│                                                    │
│ 📁 src/ (slow unculled tree with 500 rows)         │
│ 📁 tests/                                          │
└────────────────────────────────────────────────────┘
```

## After

```
┌ プロジェクト別セッション ──────────────────────────┐
│ すべて 1 (rounded)  要対応  待機中 1 …              │
│ ▸ operon 1件                                       │
│   ● session A                                      │
├─────────────── ═══ (grip pill) ────────────────────┤ <- Clear horizontal resize drag handle
│ [⠇ファイル] [⠇会話] [⠇変更]              [⇄] [>]   │ <- [⠇ファイル] rounded with RADIUS_CONTROL (6px)
│  (rounded 6px + drag grip ⠇)                       │    Grip icon (⠇) and hover tooltip "ドラッグして移動"
│ [ファイルを絞り込む          ] [⟳]                  │
│                                                    │
│ 📁 src/ (view-culled fast tree rendering at 60fps) │
│ 📁 tests/                                          │
└────────────────────────────────────────────────────┘
```

## States that are not the happy one

| State | Shows |
|---|---|
| Tab is dragged across screen | Pointer cursor becomes Grabbing; tooltip shows target side dock preview; drop zone highlights |
| Drag is cancelled via Escape | Tab returns to rest position; tooltip and cursor revert cleanly |
| Splitter is hovered | Divider line changes to border stroke; grip pill highlights; cursor is ResizeVertical / ResizeHorizontal |
| File scan in flight | Displays cached previous tree or loading label quietly without blocking UI thread |
| Search filter active | Caches matching subset; only displays visible matching rows |
