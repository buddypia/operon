# Screen: three actionable status groups on Home and in the session sidebar

## 1. Home (`ホーム`) — status summary row (~960px window)

```
Before
┌──────────────────────────────────────────────────────────────────────────────┐
│ ✓ あなたを待っているセッションはありません  要対応になったセッションは…       │
└──────────────────────────────────────────────────────────────────────────────┘
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ ! 要対応        │ │ ⟳ 実行中        │ │ ‖ アイドル      │ │ ✓ 終了          │
│ 0               │ │ 2               │ │ 3               │ │ 14              │
└─────────────────┘ └─────────────────┘ └─────────────────┘ └─────────────────┘

After
┌──────────────────────────────────────────────────────────────────────────────┐
│ ✓ あなたを待っているセッションはありません  要対応になったセッションは…       │
└──────────────────────────────────────────────────────────────────────────────┘
┌───────────────────────┐ ┌───────────────────────┐ ┌───────────────────────┐
│ ! 要対応              │ │ ⟳ 実行中              │ │ ‖ アイドル            │
│ 0                     │ │ 2                     │ │ 3                     │
└───────────────────────┘ └───────────────────────┘ └───────────────────────┘
```

- Home's metric row drops `✓ 終了` and gives the full row width to the three
  live/actionable groups: `要対応`, `実行中`, and `アイドル`.
- Hovering a past completed or stopped row under `最近のセッション` shows its
  specific state word and hint rather than `終了`:
  - Completed (`Done`): `正常終了` / `ターミナルは正常に終了しました`
  - Cancelled (`Stopped`): `停止済み` / `停止操作による終了`

## 2. Session page (`セッション`) — left sidebar filters & workspace tabs

```
Before (default 268px sidebar: 2x2 grid with 終了, followed by the 4 CLI icon toggles)
┌─────────────┐ ┌─────────────┐
│ 要対応    1 │ │ 実行中    2 │
└─────────────┘ └─────────────┘
┌─────────────┐ ┌─────────────┐
│ アイドル  1 │ │ 終了      5 │
└─────────────┘ └─────────────┘
[ C | O | G | ⋯ ]

After — default 268px sidebar (2 columns on row 1, full-width row 2, no gap hole)
┌─────────────┐ ┌─────────────┐
│ 要対応    1 │ │ 実行中    2 │
└─────────────┘ └─────────────┘
┌─────────────────────────────┐
│ アイドル                  1 │
└─────────────────────────────┘
[ C | O | G | ⋯ ]

After — wide sidebar (>= 344px: 3 equal tiles on 1 row)
┌─────────┐ ┌─────────┐ ┌─────────┐
│ 要対応 1│ │ 実行中 2│ │アイドル1│
└─────────┘ └─────────┘ └─────────┘
[ C | O | G | ⋯ ]

After — minimum 180px sidebar (1 column stacked)
┌───────────────────┐
│ 要対応          1 │
├───────────────────┤
│ 実行中          2 │
├───────────────────┤
│ アイドル        1 │
└───────────────────┘
[ C | O | G | ⋯ ]
```

- Below the status tiles, `ui_sidebar_agent_segment` remains the existing 4-icon
  CLI toggle strip (`Codex`, `Claude`, `Antigravity`, `その他(⋯)`) with no
  `"すべて"` button; `"クリア"` and `"{n} 件"` appear on the right only while
  a status or CLI filter is active.

## 3. Session library / History (`履歴`) chip bar

```
Before
プロジェクト別セッション
[すべて 20]  [要対応 1]  [実行中 2]  [アイドル 1]  [終了 16]

After
プロジェクト別セッション
[すべて 20]  [要対応 1]  [実行中 2]  [アイドル 1]
```

## 4. Non-happy states: empty, loading/restoring, error, and too-long

- **Empty**:
  - Home with zero sessions needing attention shows the existing all-clear card:
    `✓ あなたを待っているセッションはありません  要対応になったセッションは、ここの一番上に表示されます`
    and `0` in each of the three metric tiles.
  - Sidebar with an active filter matching zero open tabs shows
    `一致するセッションはありません。`; History list shows
    `選んだ状態のセッションはありません`.
- **Loading / auto-restoring in place**:
  - When a live session crashes or its tmux pane disappears and Operon
    auto-restores it (`LaunchFocus::Keep`), no modal launch sheet opens and
    focus/page do not jump. The existing tab stays in place in `Starting`
    (counted under `実行中`, badge pulsing in `StatusTone::Info`, hover hint
    `tmux と CLI を準備中`), and the status bar briefly shows
    `強制終了されたセッションを 1 件自動で復元しました。` (or
    `前回開いていたセッションを {count} 件再開しました。` on startup recovery).
- **Error (unrestorable abnormal exit / repeat crash within 15s cooldown)**:
  - The session's tab stays open so the person can read the terminal error
    output, its badge shows the failure/lost mark, and it is counted under
    `要対応` (hovering the `要対応` tile reads
    `回答待ち・異常終了・前提が壊れて止まっているセッション`). Once the person
    closes that tab and has read it, it leaves `要対応` and remains in History
    under `すべて`.
  - Clean `/exit` or explicit `停止` closes an unpinned tab automatically and
    moves the session to History (`すべて`); a pinned tab stays open in place
    with no refusal banner.
- **Too-long labels / multi-digit counts**:
  - `status_filter_tile` lays out the right-aligned numeric count first (even
    for 3+ digits such as `128`) and clips/truncates the left group label before
    the count gap, so at the 344px 3-column boundary (~104px per tile) or 180px
    minimum width the count is never overlapped or clipped. Long session titles
    in sidebar rows and Home rows continue to truncate with `…` before the
    right-aligned metadata/action controls.
