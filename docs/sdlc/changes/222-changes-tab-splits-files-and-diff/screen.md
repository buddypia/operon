# Screen: change 222

- **Spec**: `./spec.md`
- **Status**: approved

## Project 「変更」 tab (`GitView::Changes`)

Before (`GitView::Changes` shows only the file list and commit box; clicking a file switches to `GitView::Diff` and hides the file list and commit box):

```
┌─ acme-storefront ───────────────────────────────────────────────────────────────┐
│  概要   変更 2   worktree 1   PR   ファイル   エージェント設定   ナレッジ            │
│  ─────────────────────────────────────────────────────────────────────────────  │
│  Git   [変更ファイル]  差分   コミット履歴   ↻                                      │
│  ─────────────────────────────────────────────────────────────────────────────  │
│  変更されたファイル                                                               │
│  [x] M  src/app.rs  ✎                                                           │
│  [ ] M  src/ui/widgets.rs  ✎                                                    │
│                                                                                 │
│  ┌─ コミットメッセージ ───────────────────────────────────────────────────────┐   │
│  │                                                                           │   │
│  └───────────────────────────────────────────────────────────────────────────┘   │
│  [AI に書かせる] Claude                                   [1 ファイルをコミット]  │
│  origin/main と同じ状態です。                               [PR を作成…] [push]  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

After (`GitView::Changes` splits into a 280px left column with the changed files and commit box and a right column with the live diff pane):

```
┌─ acme-storefront ───────────────────────────────────────────────────────────────┐
│  概要   変更 2   worktree 1   PR   ファイル   エージェント設定   ナレッジ            │
│  ─────────────────────────────────────────────────────────────────────────────  │
│  Git   [変更ファイル]  差分   コミット履歴   ↻                                      │
│  ─────────────────────────────────────────────────────────────────────────────  │
│  変更されたファイル                │  作業ツリーの差分  1 ファイル  +4 −1          │
│  [x] M [src/app.rs] ✎            │  [すべて表示]                  [AIレビュー]   │
│  [ ] M  src/ui/widgets.rs ✎      │  ┌─ src/app.rs  +4 −1 ────────────────────┐ │
│                                  │  │  @@ -10,4 +10,7 @@                     │ │
│  ┌─ コミットメッセージ ──────────┐ │  │  - old line                            │ │
│  │                              │ │  │  + new line                            │ │
│  └──────────────────────────────┘ │  └────────────────────────────────────────┘ │
│  [AI に書かせる] [1 ファイルをコミット]│                                             │
│  origin/main と同じ状態です。      │                                             │
│  [PR を作成…] [push]              │                                             │
└──────────────────────────────────┴──────────────────────────────────────────────┘
```

### States

- **Empty (`snapshot.files.is_empty()`)**: Left column shows 「作業ツリーに変更はありません。」 and `ui_push_row`; right column shows the empty diff pane (`0 ファイル`).
- **Loading**: Left column shows 「Git の状態を更新しています…」 while `BackgroundKey::GitChanges` is running; right column shows 「差分を読み込んでいます…」 while `BackgroundKey::GitDiff` is running.
- **Error**: Left column shows `Git の状態を取得できません: {error}` on `git_changes_cache` error; right column shows `Git の差分を取得できません: {error}` on `git_diff_cache` error.
- **File selected (`self.git_selected_file == Some(path)`)**: The clicked file row is highlighted in the left list, `self.git_view` stays `GitView::Changes`, and the right pane shows only that file's diff plus the 「すべて表示」 button to return to the full working-tree diff.
