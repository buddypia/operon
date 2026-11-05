# Screen: in-tab headings, before and after

Real widths: project detail body ~900px, Sessions page ~1100px, settings
sidebar 200px. The person approved this classification in the proposals
HTML (`.tmp/sidebar-project-header-ui-proposals/`, "他のタブはどうするか")
and said 「実装を進めて」; this file is the same layout in text.

## Project tab (変更, the others are the same shape)

```
Before:
[変更] [worktree] [PR] [ファイル] ...        tabs, 変更 underlined
Git   [変更ファイル] [差分] [コミット履歴]  [↻]
...

After:
[変更] [worktree] [PR] [ファイル] ...
[変更ファイル] [差分] [コミット履歴]  [↻]
...
```

worktree: the 「worktree」 heading goes, the refresh stays.
PR: the 「プルリクエスト」 heading goes; 「認証済み GitHub CLI 経由」 and
the refresh stay on the row.

## Sessions page

```
Before:
セッション  [↻] [プロジェクト別|グリッド] [セッションを検索      ]
プロジェクト別セッション   (large)
[すべて n] [要対応] [実行中] ...

After:
[↻] [プロジェクト別|グリッド] [セッションを検索      ]
プロジェクト別セッション   (12.5 muted, panel label)
[すべて n] [要対応] [実行中] ...
```

The same demoted title heads the docked session panel, where it is the
panel's only name.

## エージェント設定 tab

```
Before:  スキル (large) [↻]  ...   ルールと指示 (large) [↻] ...
After:   スキル (15 strong) [↻] ...  ルールと指示 (15 strong) [↻] ...
```

## Settings

```
Before:  設定 (20 strong)          After:  設定 (12.5 muted)
         [外観]                            [外観]
         [エージェント]                    [エージェント]
         ...  page title is the page_header on the right in both.
```

## States

- Empty, loading, and error rows are unchanged; none of the changed rows
  carries an empty-state message.
- Empty-state headings elsewhere (「まずはプロジェクトを開いてください」,
  「セッションを選択」) keep their size: the heading is the message.
- Tabs, their selected marks, and every `page_header` are unchanged.
