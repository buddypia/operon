# Screen: the session column is a list of projects, each folded until opened

Left column (`SIDEBAR_DEFAULT_W = 268px`) and the top of the centre.

```
Before (every project unfolded; 37px from one folded name to the next)
┌─ 268px ───────────────────────┐┌─ centre ─────────────────────────┐
│ プロジェクト            [履歴]││ [Cl 📌 調査] [Cx 実装] [Tm 3] +  │
│ [ ! 要対応 1 ] [ ⟳ 実行中 3 ] ││ operon › 調査                    │
│ [Cx][Cl][Ag][…]              ││ ┌ terminal ───────────────────┐  │
│ ───────────────────────────── ││                                  │
│ ▾ operon              !1 ⟳2 + │
│ ───────────────────────────── │
│ [Cl] 📌 調査                  │
│ [Cx] 実装                     │
│ [Tm] セッション3              │
│ ▾ astra-event            ⟳1 + │
│ ───────────────────────────── │
│ [Ag] 図版                     │
│ ▾ notes                     + │
│ ───────────────────────────── │
│ 開いているセッションはありません│
└───────────────────────────────┘

After (every project folded; 28px from one name to the next, no rule)
┌─ 268px ───────────────────────┐┌─ centre (unchanged) ─────────────┐
│ プロジェクト            [履歴]││ [Cl 📌 調査] [Cx 実装] [Tm 3] +  │
│ [ ! 要対応 1 ] [ ⟳ 実行中 3 ] ││ operon › 調査                    │
│ [Cx][Cl][Ag][…]              ││ ┌ terminal ───────────────────┐  │
│ ───────────────────────────── ││                                  │
│ ▸ operon              !1 ⟳2 + │  24px heading + 4px
│ ▸ astra-event            ⟳1 + │  24px heading + 4px
│ ▸ notes                     + │
└───────────────────────────────┘

After, operon unfolded by its ▸ (kept across a relaunch)
│ ▾ operon              !1 ⟳2 + │
│ ───────────────────────────── │
│ [Cl] 📌 調査                  │  rows as today
│ [Cx] 実装                     │
│ [Tm] セッション3              │
│ ▸ astra-event            ⟳1 + │
│ ▸ notes                     + │
```

Other states (268px):
- Tooltips on the disclosure: `▸` 「展開する」, `▾` 「折りたたむ」.
- Empty project unfolded: `▾ notes` / rule / 「開いているセッションはありません」. Folded: `▸ notes` only.
- No projects: 「プロジェクトがありません」.
- Filter on with matches (`N 件` + 「クリア」 beside the CLI segment): each
  project with a match draws `▾`, the rule and its matching rows even if the
  person left it folded; hovering `▾` says 「フィルタ中はすべて表示されます」
  and clicking it does nothing. 「クリア」 folds it back.
- Filter on, no match: 「一致するセッションはありません。」.
- Long project name: truncated before the counts, full name and path on hover.
- Save failed: notice 「並び順を保存できませんでした: {error}」; the fold holds for this run.
- Sixty folded projects: the column scrolls; 28px each.
