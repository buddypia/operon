# Screen: six project tabs with counts

- **Status**: approved

The owner approved the rendered mock `/tmp/operon-ui-before-after/view-projects-after.html`
(a copy sits in the worktree's `.tmp/ui-before-after/`)
(section ③, "全項目 OK、進めて", 2026-09-27). This file is the part of it this
change builds: the tab row.

## Tab row

```
概要   変更 3   worktree 2   PR 0   ファイル   エージェント設定
━━━                                                       (accent rule under the selected tab)
```

- Same `tab_item` look as today: muted text, strong when selected, a 2 px
  accent rule under the selected one, the hairline under the row.
- 変更 N: N is `files.len()` of the project's cached git status. When N > 0 it
  is drawn in the accent colour; the label stays muted/strong as usual.
- worktree N: the number of worktrees in the cached list (the main checkout
  included, as the worktree tab lists it).
- PR N: the number of pull requests in the cached list.
- A count is shown only when its cache holds a result. Not yet loaded, or an
  error: the bare label, no number, no 「—」. The tab row requests nothing:
  概要 already requests changes and worktrees; PR is requested when its tab
  opens, as today (it calls `gh`, which may reach the network).
- エージェント設定: today's スキル list, then today's ルールと指示 list, one
  above the other, each with its own heading and ↻ button, as today.
- Where the row departs from the mock, and why: none.

## Other text

- The session panel's note 「worktree の変更はプロジェクトの Git タブで確認できます。」
  names the tab by its new name: 「…プロジェクトの「変更」タブで確認できます。」.

## Non-happy states

| State | Shown |
|---|---|
| Not a git repository | 変更 and worktree without counts (their caches hold errors) |
| `gh` missing | PR without a count, as its cache holds an error |
| Narrow window | one line, as today; six tabs are narrower than today's seven |
