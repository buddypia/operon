# Screen: landing refusals and conflict notices

- **Spec**: `./spec.md`
- **Status**: approved

No new screen, modal, or layout. Every change is wording inside the existing
notice bar, which already wraps long notices; change 110's landing modal and
worktree row are untouched. The commit button already greys out while
`git_worktree_busy` is true, so a landing in flight now greys it the same way;
the push button, which greyed out only on a commit or push, now reads the same
predicate — same widget, same disabled state, a different condition.

## Notice bar, before and after (900px window)

Before — a conflict:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ マージに失敗しました: マージに失敗したため操作を中断しました（コンフリクト   │
│ または制約）: Auto-merging src/a.rs CONFLICT (content): Merge conflict in… │
└──────────────────────────────────────────────────────────────────────────┘
```

After — a conflict, restored:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ マージに失敗しました: 競合が発生したため取り込みを取り消し、main を元の状態 │
│ に戻しました。競合したファイル: src/a.rs、README.md                         │
└──────────────────────────────────────────────────────────────────────────┘
```

After — too many conflicts (cap 10):

```
│ …競合したファイル: a1、a2、a3、a4、a5、a6、a7、a8、a9、a10 ほか 32 件       │
```

After — the merge stopped for another reason (a hook), restored:

```
│ マージに失敗しました: マージを完了できなかったため取り込みを取り消し、main │
│ を元の状態に戻しました: pre-merge-commit hook refused                      │
```

After — git refused to start the merge, `HEAD` verified unchanged:

```
│ マージに失敗しました: マージを開始できませんでした。main は変更されていま │
│ せん: error: Your local changes to the following files would be overwritt… │
```

After — the abort did not restore the checkout (the one a person must act on):

```
│ マージに失敗しました: マージを取り消せませんでした。メイン作業ツリーにマー │
│ ジ途中の状態が残っています: fatal: Unable to create '…/.git/index.lock'… │
```

After — a merge already in progress:

```
│ マージに失敗しました: メイン作業ツリーで別のマージが進行中のため取り込めま │
│ せん。そのマージを完了するか中止してから、もう一度お試しください。         │
```

After — an agent busy in the main checkout:

```
│ この worktree またはメイン作業ツリーで動作中・待機中のセッションがあります。│
│ 停止・完了・キャンセルしてからマージしてください。                         │
```

After — a commit or push in flight: `Git の操作が既に実行中です。` (existing).
