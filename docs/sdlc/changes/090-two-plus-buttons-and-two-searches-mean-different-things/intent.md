# Intent: two "+" buttons and two searches mean different things, and a status has three names

- **Status**: approved
- **Opened**: 2026-09-27

## Problem

The owner's review of the session screen (change 088) found three things that
cross every screen, and approved fixing them as the second of four changes:

- The toolbar "+" adds a project; the sidebar "+" starts a session. They look
  identical. A person who wants a session presses the toolbar one first.
- The toolbar magnifier opens ⌘K; the sidebar magnifier opens a different
  "セッションツール" view. Same icon, different place, different thing.
- One state has several names: the Home tiles say RUNNING / QUEUED / DONE, the
  sidebar chips say 要対応 / 空き / 実行中 / 終了, and each session says IDLE,
  WORKING, WAITING, BLOCKED… in English.

## Who feels it, and when

Whenever the owner starts a session, searches, or reads what a session is doing
— on Home, in the session list, and in a session's header.

## Desired outcome

As in the approved `.tmp/ui-before-after/index.html` ("共通ルール" 1–3):

- The toolbar has one labelled "+ 新しいセッション ⌘N" that starts a session
  from anywhere; adding a project is on the Projects screen, ⌘O, and
  drag-and-drop, as today.
- The toolbar has one search control, "検索・操作 ⌘K". The sidebar has no
  magnifier; what it opened is reached as 「履歴」 from the session list and ⌘K.
- A session's state is named with one of four words everywhere — 要対応,
  実行中, 待機中, 終了 — in the same colours as today.
- Nothing that was reachable becomes unreachable.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31. User-facing text is Japanese, through
  `tr`/`tf!` with a row in every table.
- No persisted-store change. A person's `keybindings.json` keeps working.

## Systems likely affected

`src/app/screens.rs` (toolbar, sidebar header, Home tiles, palette),
`src/app.rs` (new-session entry, shortcut), `src/app/keymap.rs`,
`src/ui/widgets.rs` (status words), `src/i18n_tables.rs`, `README*.md`.

## Open questions

None. The owner answered "全項目 OK、進めて" to the Before/After and its
checklist on 2026-09-27, and "進めて" again after change 088 landed.

## Not in scope

- The launch sheet (the new-session button opens today's launch form), the
  Home redesign beyond its status words, Projects tabs, Settings — changes 3
  and 4.
