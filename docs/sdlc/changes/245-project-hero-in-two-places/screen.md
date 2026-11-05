# Screen: session workspace column and terminal pane, before and after

Real width: the workspace at 1280px; the column is about 260px. The person
approved this layout as HTML (`.tmp/session-project-dedup/` rev 2) and said
「進めて」; this file is the same layout in text.

## Before (column, one project)

```
┌─ プロジェクト ───────────── [履歴] ┐
│ ┌─ card (fill, hero) ────────────┐ │
│ │ ▸ ▣ operon        ●1  +       │ │   name 15 strong text_strong
│ │   ● レビュー対応               │ │   sessions 12.5 regular
│ │   ● ターミナル修正             │ │
│ └────────────────────────────────┘ │
│ ┌─ card ─────────────────────────┐ │
│ │ ▸ ▣ buddypia-web      +       │ │
│ └────────────────────────────────┘ │
└────────────────────────────────────┘
Terminal pane header: [title 15 strong] [status] · project-name · branch
```

## After (column, one project)

```
┌─ プロジェクト (12.5 muted) ─ [履歴] ┐
│ ▸ operon            ●1  +          │   heading 12.5 muted, no card
│ ─────────────────────────────────  │   hairline
│   ● レビュー対応                   │   sessions 12.5 strong text
│   ● ターミナル修正                 │
│                                    │
│ ▸ buddypia-web             +       │
│ ─────────────────────────────────  │
│   ● LP更新                         │
└────────────────────────────────────┘
Terminal pane:
  operon / レビュー対応                crumb 11.5 text_muted
  [title 15 strong] [status] / branch  (project name no longer repeated)
```

## States

- Folded: heading row only, disclosure closed; hairline still under it.
- No live session: heading, hairline, 「起動中のセッションはありません」.
- No projects: 「プロジェクトがありません」 (unchanged).
- Long project name: truncates in the heading; hover shows name, path,
  「ドラッグで並べ替え」 (unchanged).
- Selected project: heading name strong, still in `text_muted`; no fill anywhere.
- Split terminal: same crumb above each pane header.
