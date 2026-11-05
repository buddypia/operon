# Screen: the session workspace, before and after (change 246)

Realistic width: the workspace takes the window edge to edge; the session
column is drawn at its 268px default. Four things wear the colour today and
all four are shown below: the selected session row and the selected project
card in that column, the pinned waiting row on the home page (drawn at page
width under this box), and the current toolbar page. Two more places borrow
the tint without being selections — the home attention banner and the
knowledge map's hover — and are named under the box. The mock the person
judged is `.tmp/selection-color-ui-proposals/proposals.html` (B案); this file
is the same screen in the change record. `■` marks the status dot, ringed
after the change; words are the real Japanese labels.

## Before — dark theme, warm tint, muted inks

```
┌ toolbar (52px, the title bar) ────────────────────────────────────────────┐
│ ● ● ●  Operon   ホーム  プロジェクト  [セッション]      [ 新しいセッション ] │
│                            └ tint #36271D, words in orange #FF9438        │
├ session column 268px ──────┬ terminal ─────────────┬ inspector ──────────┤
│ operon                5件  │ $ cargo test --locked │ ファイル 会話 変更 3  │
│ ┌────────────────────────┐ │ running 12 tests      │  theme.rs           │
│ │■ 選択色の扱いを見直す 今│ │                       │                     │
│ │  Claude · 実行中 · …   │ │                       │                     │
│ └────────────────────────┘ │                       │                     │
│   └ tint #36271D; vendor word, status, time, branch in muted inks         │
│ ■ tmux停止時の復元テスト   │                       │                     │
│ ┌ selected project card ─┐ │                       │                     │
│ │ operon        ●2 ●1 ＋ │ │  (project card above, when selected,     │
│ └────────────────────────┘ │   wears the same tint; counts in tones)   │
└────────────────────────────┴───────────────────────┴─────────────────────┘
  Home attention banner: tint #36271D fill, warning words and outline.
  Knowledge map hover: the hovered row fills with the same tint.
  Home pinned waiting row (page width): tint fill; title in the strongest
  ink, meta and the waiting time in muted and tone inks.
```

## After — dark theme, selection colour, bright inks

```
┌ toolbar (52px, the title bar) ────────────────────────────────────────────┐
│ ● ● ●  Operon   ホーム  プロジェクト  [セッション]      [ 新しいセッション ] │
│                            └ fill #005C80, words in #F2F4F7               │
├ session column 268px ──────┬ terminal ─────────────┬ inspector ──────────┤
│ operon                5件  │ $ cargo test --locked │ ファイル 会話 変更 3  │
│ ┌────────────────────────┐ │ running 12 tests      │  theme.rs           │
│ │◎ 選択色の扱いを見直す 今│ │                       │                     │
│ │  Claude · 実行中 · …   │ │                       │                     │
│ └────────────────────────┘ │                       │                     │
│   └ fill #005C80; title in #F2F4F7, vendor/status/time/branch in         │
│     #C9CDD3; ◎ = status dot in its tone, ringed in #F2F4F7                │
│ ■ tmux停止時の復元テスト   │                       │                     │
│ ┌ selected project card ─┐ │                       │                     │
│ │ operon        ●2 ●1 ＋ │ │  (same fill; name and counts in the two     │
│ └────────────────────────┘ │   selected inks, no tone text on the blue)  │
└────────────────────────────┴───────────────────────┴─────────────────────┘
  Light: fill #B4D6FA, words in #0F1113 / #24282C. High contrast: fill
  #0A6E96, words in #FFFFFF / #F5F7FA. Home attention banner: neutral card
  fill, warning words and outline. Knowledge map hover: neutral card fill.
  Home pinned waiting row: same fill; title, meta, quoted line, and waiting
  time in the two selected inks; its dot ringed the same way.

  States: no session selected — no row wears the colour, screen unchanged.
  A title too long truncates exactly as today; the time keeps its place.
  The rename field still sits on the inset surface inside the row.
```
