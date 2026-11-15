# Screen: notices on the Sessions page float in the bottom-right corner without moving the terminal

Sessions page (`Page::Sessions`), 1120px wide, before and after when an ordinary
notice arrives or leaves:

```
Before (an interrupting notice inserts a row above the workspace and pushes everything down ~50px)
┌─ [● ● ●] Operon   ホーム  プロジェクト  [セッション]       検索 ⌘K  [+ 新しいセッション]  ⚙ ─┐
├───────────────────────────────────────────────────────────────────────────────────────────────┤
│ ┌─ ℹ ファイルを開く…                                                                    [✕] ─┐│  ← 割り込みで行が増え、
│ └────────────────────────────────────────────────────────────────────────────────────────────┘│    下の全要素が下がる
│ プロジェクト          [履歴] │ [claude · operon ✕] [+]                  │ ファイル  会話  変更 │
│  [すべて][要対応][実行中]…   ├──────────────────────────────────────────┼─────────────────────┤
│  ▾ operon                    │ operon › ログイン画面の修正              │ [ファイルを絞り込む]│
│    (⟳) ログイン画面の修正    │ [Claude] ログイン画面の修正  ⟳ 実行中 [停止]│ ▾ src               │
│                              ├──────────────────────────────────────────┤   app.rs            │
│                              │ $ claude                                 │                     │
│                              │   ... (高さが縮み tmux もリサイズされる) │                     │
└──────────────────────────────┴──────────────────────────────────────────┴─────────────────────┘
  ↓ 50ms 後に「ファイルを開く: 完了」(brief) へ変わると上のバナーが消え、画面全体が上へ跳ね戻る

After (workspace top y and height stay fixed; every notice/warning on Sessions floats at bottom-right)
┌─ [● ● ●] Operon   ホーム  プロジェクト  [セッション]       検索 ⌘K  [+ 新しいセッション]  ⚙ ─┐
├───────────────────────────────────────────────────────────────────────────────────────────────┤
│ プロジェクト          [履歴] │ [claude · operon ✕] [+]                  │ ファイル  会話  変更 │  ← 上端・高さは常に不変
│  [すべて][要対応][実行中]…   ├──────────────────────────────────────────┼─────────────────────┤
│  ▾ operon                    │ operon › ログイン画面の修正              │ [ファイルを絞り込む]│
│    (⟳) ログイン画面の修正    │ [Claude] ログイン画面の修正  ⟳ 実行中 [停止]│ ▾ src               │
│                              ├──────────────────────────────────────────┤   app.rs            │
│                              │ $ claude                                 │                     │
│                              │   ...                                    │                     │
│                              │                                  ┌─ ℹ ファイルを開く…    [✕] ─┐│
│                              │                         [↓ 最新へ]└────────────────────────────┘│
└──────────────────────────────┴──────────────────────────────────────────┴─────────────────────┘
```

States of the bottom-right overlay on `Page::Sessions` (at most 420px wide, `18px` from bottom-right):

```
1. Empty (notice = None, durability_warning = None):
   (nothing drawn; no Area layer)

2. Brief completion notice (notice_briefly, clears after 4s or on ✕):
   ┌────────────────────────────────────────┐
   │ ✓ ターミナルのログをコピーしました。 ✕ │
   └────────────────────────────────────────┘
   (1px border_subtle stroke, success mark)

3. Persistent / error notice (self.notice = Some(_), stays until ✕ or replaced):
   ┌──────────────────────────────────────────────────┐
   │ ℹ セッションを開始できませんでした:              │
   │   tmux が PATH 上に見つかりません。            ✕ │
   └──────────────────────────────────────────────────┘
   (1px info stroke, info mark, wraps inside <= 420px leaving room for ✕)

4. Durability warning + persistent notice together (warning has no ✕, stays until sync confirms):
   ┌──────────────────────────────────────────────────┐
   │ ⚠ 変更は書き込まれましたが、ディスクに保存され   │
   │   たことを確認できませんでした。Operon が自動的  │
   │   に再試行します。                               │
   └──────────────────────────────────────────────────┘
   ┌──────────────────────────────────────────────────┐
   │ ℹ セッションをキャンセルしました。             ✕ │
   └──────────────────────────────────────────────────┘
```
