# Screen: quiet session-list label, 15px project names, and leading pin marks

Session column (`SIDEBAR_DEFAULT_W = 268px`) and central tab strip, with one pinned session (`調査`) and two unpinned sessions (`実装`, `図版`).

```text
Before
┌─ 268px ───────────────────────┐   ┌ タブ ───────────────────────────────────┐
│ プロジェクト (15 bold strong)+ 履歴│   │ [Cl] 調査 📌 │ [Cx] 実装 × │ +       │
│ [ ! 要対応 1 ] [ ⟳ 実行中 2 ] │   └─────────────────────────────────────────┘
│ [ ○ アイドル 0 ]               │
│ [Cx][Cl][Ag][…]              │
│ ───────────────────────────── │
│ ▾ operon (13.5 bold)  !1 ⟳1 + │
│ ───────────────────────────── │
│     [Cl] 📌 調査              │   ← pin sits after AI mark;
│     [Cx] 実装                 │     top label reads like a page title
│     [Ag] 図版                 │
│ ▸ notes (13.5 bold)         + │
└───────────────────────────────┘

After
┌─ 268px ───────────────────────┐   ┌ タブ ───────────────────────────────────┐
│ セッション一覧 (12.5 muted)  + 履歴│   │ 📌 [Cl] 調査 │ [Cx] 実装 × │ +       │
│ [ ! 要対応 1 ] [ ⟳ 実行中 2 ] │   └─────────────────────────────────────────┘
│ [ ○ アイドル 0 ]               │     ↑ pin sits left of AI mark, no ×
│ [Cx][Cl][Ag][…]              │
│ ───────────────────────────── │
│ ▾ operon (15.0 bold)  !1 ⟳1 + │   ← project names outrank top label & rows
│ ───────────────────────────── │
│   📌 [Cl] 調査                │   ← pin in 24..38px gutter;
│      [Cx] 実装                │     all AI marks stay aligned at 42px
│      [Ag] 図版                │
│ ▸ notes (15.0 bold)         + │
└───────────────────────────────┘
```

Other states:
- Selected pinned row / active pinned tab: 2px `accent` bar on selected row (`row.rect.left() + 1px`, clear of `24px` gutter); `📌` uses `selected_secondary_ink()` for AA contrast.
- Inline rename / drag reorder: name editor stays right of `[AI]` after the gutter `📌`; drag target line and scroll behaviour are unchanged.
- Narrow sidebar (`180px`): gutter `📌` uses only existing indent space (`24..38px`), so title truncation starts at the exact same x as unpinned rows.
- Legacy custom-command session (no vendor mark): gutter `📌` precedes the `16px` circled status mark at `42px`; in central tabs, `📌` precedes `session_badge`'s status mark fallback.
- Empty / no projects / filter with no match / save error: 「プロジェクトがありません」「一致するセッションはありません。」「開いているセッションはありません」「並び順を保存できませんでした: {error}」 unchanged.
