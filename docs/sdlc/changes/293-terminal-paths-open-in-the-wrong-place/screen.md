# Screen: terminal path menu at the pointer

Terminal pane, ~1100px wide. Pointer on `Operon_LT_10min.pptx`, right-click.

```
Before (box at a default window position, away from the pointer)
 • リポジトリ内 (dist/): Operon_LT_10min.pptx      ← not underlined: bare name
                                               ┌──────────────────────────┐
                                               │ /…/dist/foo.rs           │
                                               │ [パスをコピー]           │
                                               │ [Finder で表示]          │
                                               │ [閉じる]                 │
                                               └──────────────────────────┘
After
 • リポジトリ内 (dist/): Operon_LT_10min.pptx
                         ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔ ↖ pointer
                         ┌───────────────────────────────────────────┐
                         │ /Users/…/operon/dist/Operon_LT_10min.pptx │  (12px, muted)
                         │ Operon で開く                              │
                         │ 既定のアプリで開く                          │
                         │ Finder で表示                              │
                         │ パスをコピー                                │
                         └───────────────────────────────────────────┘
```

States:
- **Plain click** on the name: focuses the terminal (unchanged).
- **⌘/⌥ + click**: opens a tab 「Operon_LT_10min.pptx」 in the workspace; a pptx
  shows the existing "cannot display" body with 「既定のアプリで開く」.
- **Not found** (file deleted after hover): notice 「ファイルが見つかりません: Operon_LT_10min.pptx」, no menu.
- **Too long path**: the first line is a single label and wraps at the window's
  max width (egui default ~ 360px); items stay one line each.
- **Two files with the name**: no underline, right-click does nothing special.
- Closes on a choice, Escape, or a click outside.
