# Screen: compact vertical spacing and type scale in the session sidebar

Left sidebar (`SIDEBAR_DEFAULT_W = 268px`), before and after:

```
Before (outer 10px spacing, 28px horizontal row min, 12.5px text, ~42px/row)
┌─ 268px ───────────────────────┐
│ プロジェクト (12.5)     [履歴]│  10px gap
│ [ ! 要対応 1 ] [ ⟳ 実行中 3 ] │   4px gap
│ [ ‖ アイドル 2              ] │  10px gap
│ [Cx][Cl][Ag][…]              │  10px gap
│ ───────────────────────────── │  10px gap
│   ▾ proj-a (12.5)     !1 ⟳3 +│   8px margin + 10px
│   ─────────────────────────── │  10px gap
│   [Ag] セッション10 (12.5)   │  32px row + 10px gap
│   [Cl] セッション9           │  32px row + 10px gap
│   [Tm] セッション6           │  32px row + 10px gap
│                               │  8+8+10+8 = 34px between projects
│   ▾ proj-b (12.5)           +│

After (4px column / 2px row gap, ~19-20px rows, 12.0/11.5px text, ~21-22px/row)
┌─ 268px ───────────────────────┐
│ プロジェクト (11.5)     [履歴]│   4px gap
│ [ ! 要対応 1 ] [ ⟳ 実行中 3 ] │   4px gap
│ [ ‖ アイドル 2              ] │   4px gap
│ [Cx][Cl][Ag][…]              │   4px gap
│ ───────────────────────────── │   4px gap
│ ▾ proj-a (12.0)       !1 ⟳3 + │   2px margin + 2px gap
│ ───────────────────────────── │   2px gap
│ [Ag] 📌 セッション10 (12.0)   │  ~19px row + 2px gap
│ [Cl] セッション9              │  ~19px row + 2px gap
│ [Tm] セッション6              │  ~19px row + 2px gap
│ [Tm] セッション4              │  ~19px row + 2px gap
│                               │   8px between projects
│ ▾ proj-b (12.0)             +│   2px margin + 2px gap
│ [Cl] 調査タスク               │  ~19px row + 2px gap
└───────────────────────────────┘
```

Other states inside the compact sidebar (all 268px wide):
- Empty project: `▾ proj-b` / `開いているセッションはありません` (small, weak).
- Filtered empty: `一致するセッションはありません。` above project list; `N 件` (`11.5px`) + `クリア` beside the CLI segment.
- Long session title: single-line truncated cleanly before the right edge with full title and status on hover; pinned rows show `ICON_PIN` (`11.5px`, hover `ピン留め中`) before the title and match the center-top tab strip.
- Error saving order/pin: `"並び順を保存できませんでした: {error}"` floating notice; existing order remains active for the session.
