# Intent: a session cannot be closed from its row in the left column, nor from its tab's right-click menu

- **Status**: approved
- **Opened**: 2026-11-11

## Problem

The person's request, as written:

> セッションの左サイドバーのセッションに閉じる機能も追加したい。同じく上のターミナルセッション名にも×ボタンがあるが、右クリックで出てくる閉じるを入れたい。

Today a session's tab can be closed only with the tab's × (or ⌘W on the front
tab). The left column lists the same sessions, but a row offers no way to close
one: its right-click menu holds 「名前を変更」 and 「ピン留め」 only. The tab's
right-click menu holds the same two entries, so a person who right-clicks a
tab's name to close it finds nothing.

## Who feels it, and when

A person on the session page with several sessions open, working from the left
column: to close one they must find its tab in the strip above and hit its
small ×. And a person who right-clicks a tab's name, as tabs elsewhere allow,
expecting a 閉じる entry.

## Desired outcome

- Right-clicking a session's row in the left column offers 「閉じる」; choosing
  it closes that session's tab, so the row and the tab both go, as the tab's ×
  does. The session itself keeps running.
- Right-clicking a session tab's name offers 「閉じる」 with the same effect.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- A pinned session does not close (change 285): its tab has no ×.

## Systems likely affected

`src/app/screens.rs` (the two menus), `src/app.rs` and
`src/app/session_order.rs` (closing a tab that belongs to a project not shown).

## Open questions

None. Adopted without asking, each undone by a revert: the entry is omitted on
a pinned session, as its × is; the row gets a menu entry, not a × button.

## Not in scope

- A × button drawn on the column's rows.
- Deleting or stopping a session from these menus.
- A right-click menu on document tabs.
