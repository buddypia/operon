# Screen: session page project column and session tabs

- **Status**: approved

## Before (260px column)

```
┌ プロジェクト ───────────────────┐
│ [新しいセッション] [履歴]        │
│ ┌────────────────────────────┐ │
│ │▣ operon          2 件  [+] │ │
│ └────────────────────────────┘ │
│ ┌────────────────────────────┐ │
│ │▣ astra-event     0 件  [+] │ │
│ └────────────────────────────┘ │
```

## After (260px column)

```
┌ プロジェクト ────────── [履歴] ┐
│ ▾ ▣ operon     ⚠2 ◌1     [+]  │  header: name drag reorders, click selects
│     ⚠ README を直して          │  row click opens that session's tab
│     ⚠ テストを追加             │
│     ◌ リファクタリング…        │  long titles truncate, hover shows full
│ ▸ ▣ astra-event   ○1     [+]  │  folded: counts stay visible
│ ▾ ▣ dailyhangul          [+]  │
│     起動中のセッションはありません │  empty project
└────────────────────────────────┘
```

- Counts per group in order 要対応 (⚠, warning), 実行中 (◌, success),
  指示待ち (○, calm); a group with zero is not drawn. Hover: 「要対応 2 件」.
  Clicking a count opens that group's first session in the project.
- Rows list the project's sessions that are not finished (plus unread finished
  ones, which count as 要対応), ordered 要対応 → 実行中 → 指示待ち, then the
  column's own order. Each row: its state glyph in its state colour + title.
  Hover: 「要対応 · ターミナルを開いて回答してください」.
- ▾/▸ toggles folding; fold state is saved with the column order.
- No projects: 「プロジェクトがありません」.

## Central tabs

```
│⚠ README を直して ×│◌ リファクタ… ×│main.rs ● ×│
```

A session tab starts with its state glyph in its state colour; hover adds
「要対応 · …」 under the title. Document tabs are unchanged.
