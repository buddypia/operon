# Screen: headings that read as headings, and session rows with room

Session column (`SIDEBAR_DEFAULT_W = 268px`), one project unfolded.

```
Before
┌─ 268px ───────────────────────┐
│ プロジェクト  (13.5 muted)  + 履歴│
│ [ ! 要対応 1 ] [ ⟳ 実行中 2 ] │
│ [Cx][Cl][Ag][…]              │
│ ───────────────────────────── │
│ ▾ operon (13.5)       !1 ⟳1 + │
│ ───────────────────────────── │
│[Cl] 調査                      │  20px row, 4px gap → 24px
│[Cx] 実装                      │
│[Ag] 図版                      │
│ ▸ notes                     + │
└───────────────────────────────┘

After
┌─ 268px ───────────────────────┐
│ プロジェクト (15 bold strong)+ 履歴│
│ [ ! 要対応 1 ] [ ⟳ 実行中 2 ] │
│ [Cx][Cl][Ag][…]              │
│ ───────────────────────────── │
│ ▾ operon (13.5)       !1 ⟳1 + │
│ ───────────────────────────── │
│                               │
│     [Cl] 調査                 │  28px row (6px above/below),
│                               │  4px gap → 32px; mark under
│     [Cx] 実装                 │  the project name
│                               │
│     [Ag] 図版                 │
│ ▸ notes                     + │
└───────────────────────────────┘
```

Projects page (window 1280px):

```
Before                                   After
──── toolbar ─────────────────────       ──── toolbar ─────────────────────
プロジェクト (20px)      [スキャン][選択]                                  (+6px)
開くか、フォルダを…                       プロジェクト (24px)   [スキャン][選択]
──────────────────────────────────       開くか、フォルダを…
                                         ──────────────────────────────────
```

The same title size on ホーム, a project's overview, 設定 and ナレッジ.

Other states:
- Selected row: fill and the 2px accent bar span the 28px row.
- Renaming: the editor sits in the indented row.
- Narrow column (180px): the indented name truncates earlier.
- Empty project / no projects / filter with no match: strings as today.
