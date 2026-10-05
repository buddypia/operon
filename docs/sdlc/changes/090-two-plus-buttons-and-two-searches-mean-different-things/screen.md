# Screen: toolbar, sidebar header, and status words, before and after

- **Status**: approved

The owner approved the rendered mock in `.tmp/ui-before-after/index.html`,
"共通ルール" 1–3 and row G of section ① ("全項目 OK、進めて", 2026-09-27). This
file is the part of it that this change builds.

## Toolbar (right side)

```
Before:  … [rate limits]  🔍  ＋  ⚙
After:   … [rate limits]  [⌕ 検索・操作  ⌘K]  [＋ 新しいセッション  ⌘N]  ⚙
```

- 検索・操作 is a framed, field-like button (secondary look, `ICON_SEARCH`);
  新しいセッション is the primary button (`ICON_ADD`). The chord text is read
  from the keymap, so a rebound chord shows the new keys and an unbound one
  (`null`) shows no chord text.
- The project-add icon leaves the toolbar. ⌘O, drag-and-drop, the Projects
  screen buttons, and the ⌘K row still add a project.

## Sidebar header

```
Before:  プロジェクト別セッション 2 件            ＋ 🔍
After:   プロジェクト別セッション 2 件             履歴
```

- 「履歴」 is a small text button; it opens the view that was 「セッションツール」,
  now headed 「履歴」, with 「ターミナルに戻る」 as today.
- The per-project "+" on each project row stays (it starts a session there).

## ⌘K palette, empty query

```
新しいセッション                ⌘N
履歴を開く
プロジェクトを開く              ⌘2
…(the other rows of today, same order)
```

## Status words

| Where | Before | After |
|---|---|---|
| Sidebar chips | 要対応 / 空き / 実行中 / 終了 | 要対応 / 実行中 / 待機中 / 終了 (the mock's order) |
| Session row, header chip, session cards, grid, palette detail | IDLE, WORKING, WAITING, BLOCKED, RUNNING, READY, QUEUED, STARTING, STOPPING, CHECKING, DONE, FAILED, STOPPED, UNKNOWN | the group word of that state: 要対応 / 待機中 / 実行中 / 終了; the per-state sentence stays on hover; the colour stays per state (a failure is still red) |
| Projects list header | 「フォルダを選択…」 primary | the same button, secondary |
| Home header | 「AI セッションを始める」 (primary, goes to Projects) | removed; the toolbar's 「新しいセッション」 is the one primary action, as in the mock's Home |
| Home tiles | プロジェクト / RUNNING / QUEUED / DONE | 要対応 / 実行中 / 待機中 / 終了, as in the mock (no プロジェクト tile), counted as the sidebar counts; a group tile opens the session list filtered to that group |
| Session row status word | no hover | the per-state sentence on hover, as the chip has |
| Launch form dependency list | `title (RUNNING)` | `title (実行中)` — the same group word |

The ⌘K palette window title becomes 「検索・操作」, the toolbar's name for it. The
Projects list's 「フォルダを選択…」 becomes a secondary button, so the toolbar's
「新しいセッション」 is the only primary button on that page.

## Non-happy states

| State | Shown |
|---|---|
| No project yet, 新しいセッション pressed | today's: notice 「先にプロジェクトを追加してください。」 and the Projects screen |
| `session.new` unbound in keymap | button without chord text; ⌘N does nothing |
| A group tile with 0 | the tile shows 0 and still opens the filtered (empty) list |
