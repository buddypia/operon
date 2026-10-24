# Screen: change 221

Layouts follow `docs/images/03-session-claude.png` (sidebar chips),
`docs/images/04-diff.png` (project tabs) and `docs/images/06-new-session.png`
(launch sheet) at their drawn proportions; widths are not restated here because
they were not measured. Japanese strings throughout. Only the marked lines
change.

## 1. Sub-tabs under a project's changes tab 

Before

```
┌─ acme-storefront ─────────────────────────────────────────────────────────┐
│  概要   変更 3   worktree 2   PR   ファイル   エージェント設定   ナレッジ      │
│  ───────                                                                  │
│  Git   [Changes] [ Diff ] [History]  ↻                                     │
└───────────────────────────────────────────────────────────────────────────┘
```

After

```
┌─ acme-storefront ─────────────────────────────────────────────────────────┐
│  概要   変更 3   worktree 2   PR   ファイル   エージェント設定   ナレッジ      │
│  ───────                                                                  │
│  Git   [変更ファイル] [ 差分 ] [コミット履歴]  ↻                  ◄ changed │
└───────────────────────────────────────────────────────────────────────────┘
```

Longest label: 「コミット履歴」 (six full-width characters). The three tabs and ↻
sit in one row beside the 「Git」 heading and take a small fraction of it at this
width; they are measured in the running window at 1440px and at the narrowest
window the app allows when the change is verified. Empty, loading and error
states are drawn below this row and do not change.

## 2. Session filter chips (sidebar)

Before                                           After

```
[すべて 3] [要対応 1] [実行中 1] [待機中 1] [終了]    [すべて 3] [要対応 1] [実行中 1] [指示待ち 1] [終了]
```

「指示待ち」 is one character longer than 「待機中」. The chips wrap onto a second
row when they do not fit — the behaviour they already have — so a wider word
cannot clip. A session row's status column is a fixed 104px (DESIGN.md) holding
the glyph and the word: a four-character word fits where a three-character one
did, and is measured on screen at verification. Filter-empty text
「選んだ状態のセッションはありません」 is unchanged.

## 3. Launch sheet footer

Before (agent: Codex)                             After (any agent)

```
┌──────────────────────────────────────┐          ┌──────────────────────────────────────┐
│  ターミナルだけ開く   [Codex CLI を起動 ⌘↩]│          │  ターミナルだけ開く  [▶  Codex CLI を起動  ⌘↩]│
│                        ▲ teal fill      │          │                       ▲ accent (orange) fill │
└──────────────────────────────────────┘          └──────────────────────────────────────┘
```

Disabled (`launch_ready` false: tmux missing, agent missing, or a worktree
operation running): the same button drawn disabled, with the reason line above
it as today. Claude Code and Antigravity draw the identical orange button; the
agent's own colour stays on its mark in the agent chooser.
