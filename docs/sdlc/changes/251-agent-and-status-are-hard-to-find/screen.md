# Screen: Left sidebar filters

- **Status**: approved

The user requested these controls in the left sidebar. Existing width approximately 280px; existing project headers and their start/fold controls remain below them.

Before: プロジェクト [履歴], then live sessions without filters.

After:
```text
プロジェクト                 [履歴]
CLI  [すべて] [Codex mark] [Claude mark] [AG mark] [その他]
ステータス
[すべて] [承認・回答待ち] [要対応]
[実行中] [指示待ち] [終了]
──────────────────────────
▾ operon                       ＋
  ● [AG mark] セッションの題名
  ● [Codex mark] 別の題名
```

Choices wrap; marks have full names on hover and in accessibility. The current selection has the existing selected fill and border/focus treatment. The finished choice includes ended sessions. With no matches, show 一致するセッションはありません。 Empty store retains プロジェクトがありません. Loading/error/interrupted status resolution uses existing state; filters perform no IO. Long titles truncate with the full title on hover.
