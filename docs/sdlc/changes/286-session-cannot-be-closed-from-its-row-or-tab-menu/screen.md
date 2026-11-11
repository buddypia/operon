# Screen: 「閉じる」 in the row's and the tab's right-click menu

No layout changes: each existing menu gains one last entry. A menu sizes to
its longest entry (about 120px here).

## Before

```
left column row, right click        session tab name, right click
┌──────────────┐                    ┌──────────────┐
│ 名前を変更   │                    │ 名前を変更   │
│ ピン留め     │                    │ ピン留め     │
└──────────────┘                    └──────────────┘
```

## After — a session that is not pinned

```
left column row, right click        session tab name, right click
┌──────────────┐                    ┌──────────────┐
│ 名前を変更   │                    │ 名前を変更   │
│ ピン留め     │                    │ ピン留め     │
│ 閉じる       │                    │ 閉じる       │
└──────────────┘                    └──────────────┘
```

Choosing 「閉じる」 closes the menu; the tab and the row are gone on the next
frame. The session keeps running and ⇧⌘T brings the tab back.

## After — a pinned session (unchanged from today)

```
┌────────────────┐
│ 名前を変更     │
│ ピン留めを外す │
└────────────────┘
```

No 「閉じる」, as a pinned tab has no ×.

## Other states

- Last row of a project closed: the project's card shows
  「開いているセッションはありません」 (existing).
- Too-long title: the menu does not show the title; nothing to truncate.
- No loading or error state: closing a tab runs no command and writes no file.
