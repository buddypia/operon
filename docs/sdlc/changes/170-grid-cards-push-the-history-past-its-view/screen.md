# Screen: the 「グリッド」 history view stays inside its pane

- **Status**: draft

No new screen. In the history's 「グリッド」 view, session cards get a fixed
width and wrap into rows, as `horizontal_wrapped` was always meant to lay them
out; and a project header truncates its path so the glyph after it stays in view.
Measured at the test's 900px window, where the history pane is about 620px.

## Before

```
┌ p  /Users/me/dev/p  [＋]  [▭] ───────────────────────────┐
│ ┌──────────────────────────────────────────────────────┐ │┌──────────┐
│ │ ● first  Claude                                  [×] │ ││ ● second │  ← past the edge,
│ │ 単独で開始します                                     │ ││ …        │    not painted
│ │ [▭]                                         [停止]   │ │└──────────┘
│ └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
┌ q  [＋]  /Users/me/dev/a-worktree-with-a-long-name-a-worktree-with-a-long… [▭]  ← off screen
└──────────────────────────────────────────────────────────┘
```

The first card fills its row; the second starts where the row ends. The scroll
area widens to fit it (1120px in a 900px window), so every later header is laid
out wider than the pane.

## After

```
┌ p  [＋]  /Users/me/dev/p                              [▭] ┐
│ ┌──────────────────────┐ ┌──────────────────────┐         │
│ │ ● first  Claude  [×] │ │ ● second Claude  [×] │         │
│ │ 単独で開始します     │ │ 単独で開始します     │         │
│ │ [▭]          [停止]  │ │ [▭]          [停止]  │         │
│ └──────────────────────┘ └──────────────────────┘         │
└───────────────────────────────────────────────────────────┘
┌ q  [＋]  /Users/me/dev/a-worktree-with-a-long-name-a-wo…  [▭] ┐
└───────────────────────────────────────────────────────────────┘
```

- A card is 220px wide (its content keeps the 205px it had as a minimum); as
  many as fit share a row, the rest wrap below.
- The header keeps its order up to the path — name, ＋ (change 166), path — and
  the project-page glyph `[▭]` 「プロジェクトを開く」 moves to the right end, so
  the path truncates with … instead of pushing it out.
- Nothing changes colour, wording, or which control does what.

## Too long

A session title longer than the card truncates inside the card's header row,
as titles already do in the session column; the card does not grow.
