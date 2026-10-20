# Screen: arranging the session column by dragging

No new screen, nothing re-laid out. The session column of the terminal
workspace (about 280px at the default split of a 1280px window) gains a drag
on two things it already draws — a project's name and a session's row — and a
line that shows where a release lands. At rest it is drawn exactly as on main.

## Before (main, at rest — and after, at rest, with no arrangement)

```
│ ⠿ プロジェクト別セッション        履歴 │
│ [すべて 5] 要対応 実行中 待機中 3     │
│ 終了 2                                │
│ ───────────────────────────────────── │
│ ╭───────────────────────────────────╮ │
│ │ ▭ design-lookbook  3 件      [＋] │ │  ← projects by name
│ │ ● 新しい依頼          3分前   ···│ │  ← sessions newest first
│ │   Codex · 待機中 · main           │ │
│ │ ───────────────────────────────── │ │
│ │ ● レビュー対応        2時間前 ···│ │
│ │   Claude · 終了 · main            │ │
│ │ ───────────────────────────────── │ │
│ │ ● 常駐セッション      3日前   ···│ │  ← the one used all day, last
│ │   Claude · 待機中 · main          │ │
│ ╰───────────────────────────────────╯ │
│ ╭───────────────────────────────────╮ │
│ │ ▭ operon  2 件               [＋] │ │
│ │ ● 計測                1日前   ···│ │
│ │   Codex · 終了 · main             │ │
│ │ ───────────────────────────────── │ │
│ │ ● 設計                2日前   ···│ │
│ │   Antigravity · 待機中 · main     │ │
│ ╰───────────────────────────────────╯ │
```

## During: a session row dragged up within its project

Pressed on 「常駐セッション」 and dragged above the first row. The cursor is the
grabbing hand; the 2px `accent` line spans the card's inner width in the gap
the release lands in.

```
│ ╭───────────────────────────────────╮ │
│ │ ▭ design-lookbook  3 件      [＋] │ │
│ │ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │ │  ← accent, 2px: lands here
│ │ ● 新しい依頼          3分前   ···│ │
│ │   Codex · 待機中 · main      ✊    │ │  ← pointer (grabbing)
│ │ ───────────────────────────────── │ │
│ │ ● レビュー対応        2時間前 ···│ │
│ │   Claude · 終了 · main            │ │
│ │ ───────────────────────────────── │ │
│ │ ● 常駐セッション      3日前   ···│ │  ← drawn in place while dragged
│ │   Claude · 待機中 · main          │ │
│ ╰───────────────────────────────────╯ │
```

Over 「operon」's card, outside the column, or over the gap 「常駐セッション」
already sits in: no line, and a release moves nothing.

## After the release

```
│ ╭───────────────────────────────────╮ │
│ │ ▭ design-lookbook  3 件      [＋] │ │
│ │ ● 常駐セッション      3日前   ···│ │  ← kept here across a restart
│ │   Claude · 待機中 · main          │ │
│ │ ───────────────────────────────── │ │
│ │ ● 新しい依頼          3分前   ···│ │
│ │   Codex · 待機中 · main           │ │
│ │ ───────────────────────────────── │ │
│ │ ● レビュー対応        2時間前 ···│ │
│ │   Claude · 終了 · main            │ │
│ ╰───────────────────────────────────╯ │
```

A session started later in this project appears above 「常駐セッション」.

## During: a project dragged by its name

Pressed on 「operon」 and dragged above the first card. The line spans the
column in the gap between cards.

```
│ ───────────────────────────────────── │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │  ← accent, 2px: lands here
│ ╭───────────────────────────────────╮ │
│ │ ▭ design-lookbook  3 件      [＋] │ │
│ │ …                                 │ │
│ ╰───────────────────────────────────╯ │
│ ╭───────────────────────────────────╮ │
│ │ ▭ operon ✊ 2 件              [＋] │ │  ← pressed on the name
│ │ …                                 │ │
```

Held within 32px of the column's visible top or bottom edge, the list scrolls
towards it, 12px a frame.

## Hovers

- Session row: 「クリックでターミナル表示 · ダブルクリックで名前を変更 · ドラッグで並べ替え」
  (was the same without 「 · ドラッグで並べ替え」).
- Project name, under the name and path it already shows: 「ドラッグで並べ替え」.
  The cursor over the name is the open hand.

## Other states

- **Filtered** (a status chip on): only drawn cards and rows are targets; the
  line never sits next to a hidden row.
- **One project, or one session in a project**: the drag has nowhere to go and
  no line is drawn.
- **Escape with the button held**: the line disappears, nothing moves.
- **Save failed** (the notice bar, as other failures): 
  「並び順を保存できませんでした: {error}」. The column keeps the new order.
- **Long project name**: it truncates as today; the whole visible name is the
  grip.
- **Light and dark**: the line is `accent` on `raised` (cards) and on the
  column's panel fill; both are pairs `ui_dock_drop` already paints.
