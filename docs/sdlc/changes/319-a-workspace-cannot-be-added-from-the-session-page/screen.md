# Screen: the session page's column — its heading, its add menu, its menus

The left column of the session page (⌘3). Nothing moves; a + and an empty
column's button are added, three right-click menus appear, 「履歴」 takes the
quiet style, and two text sizes change.

## 1. The heading and its + menu

```
┌─ column ───────────────────────────────┐
│ プロジェクト                 履歴   [+] │  ← heading 13.5px strong, muted
│                                ┌──────────────────────┐
│                                │ 新しいセッション…     │
│                                │ プロジェクトを追加…   │
│                                └──────────────────────┘
│ [要対応 2] [実行中 1] [アイドル 3]      │
│ [Codex][Claude][Antigravity][その他]    │
│ ▸ operon          ⚠2  ▶1          [+]  │  ← project name 13.5px strong, text ink
│ ▾ astra-event               ●3    [+]  │
│ ──────────────────────────────────────  │
│   ◉ ログイン画面を直す                  │  ← session title 12px strong (unchanged)
│   ◉ テストを足す                        │
│                                          │
│              (empty space)               │  ← right-click: the same two items
└──────────────────────────────────────────┘
```

- 「履歴」 becomes the quiet button that sits beside a heading elsewhere
  (wording in the link ink, no frame until hovered).
- The new + is the 24px icon button the project headings already use; hover
  says 「追加」. It opens the menu under itself; a choice, a click elsewhere or
  Esc closes it.
- 「新しいセッション…」 opens the 新しいセッション sheet as ⌘N does, on the
  selected session's project.
- 「プロジェクトを追加…」 opens the folder picker. The folder opened there is
  added, appears in the column unfolded, and becomes the workspace's project;
  the window stays on the session page. The notice says
  「プロジェクトを追加しました。」, or 「そのプロジェクトは追加済みのため、開きました。」
  when it was already there (then it is unfolded and chosen too).

## 2. A project heading's right-click menu

```
│ ▸ operon ⚠2 ▶1 [+]│
│  ┌────────────────────┐
│  │ セッションを開始     │  ← as the heading's own +
│  │ プロジェクトを追加…  │  ← as in section 1
│  └────────────────────┘
```

## 3. No project yet

```
│ プロジェクト                 履歴   [+] │
│ プロジェクトがありません                │  (weak, as today)
│ [ プロジェクトを追加… ]                 │  ← secondary button, opens the picker
```

## Unchanged

Session rows, their menu, the filters, drag and drop, the fold and its saving,
and the per-project + all stay as they are.
