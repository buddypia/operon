# Intent: the session column and the tab strip list different sessions, and neither can keep a session in place

- **Status**: approved
- **Opened**: 2026-11-11

## Problem

The person's request, verbatim:
「セッションページの左側のサイドバーのピン留めとタブ名にもピン留め機能を入れておきたい。セッション名の移動も必須。そしてタブ上のタブ名は複数セッションがあるけど、なぜか左のサイドバーのセッションと数が一致していない。データやロジックを同じものにしないとズレが発生しそう。」

On the session page the left column and the tab strip above the terminal both
list a project's sessions, but each decides on its own which sessions to show
and in what order:

- a tab the person closed keeps its row in the column;
- a session that ended, failed, or was stopped keeps its tab but leaves the
  column;
- a finished session opened from history gets a tab and no row;
- a queued session gets a row and no tab;
- the column orders rows by state (要対応, then 実行中, then アイドル), the tabs
  by when they were opened.

So the two counts differ, and the person cannot trust either to be "the
sessions I have open". Nothing keeps a session at the front of either list,
and in this column a session row cannot be dragged to a new place.

## Who feels it, and when

Anyone with several agent sessions open in one project who looks from the tab
strip to the column (or back) to find a session: every close, every session
that ends, and every session opened from history makes the two disagree.

## Desired outcome

Asked to choose (2026-11-11), the person answered:

- **One list, tab rules** — 「タブ基準」: a project's open sessions are one list.
  A session enters it when it starts or is opened, and leaves it when its tab
  is closed (×, ⌘W, 「画面から外す」) or the session is deleted. A session that
  ends stays in both places, its state shown by its mark, until it is closed.
  The column shows exactly the project's tabs, in the same order.
- **Pin** — 「先頭固定＋閉じない」: a session can be pinned from its row or its
  tab. A pinned session sits first in both places with a pin mark, cannot be
  closed (no ×, ⌘W does nothing to it) until it is unpinned, survives its
  session ending, and is still pinned after a relaunch.
- **Move** (required): dragging a session row in the column moves it, and the
  tabs follow; dragging a tab moves it, and the column follows.

Observable: open three sessions, close one tab — the column shows two rows. Let
one session end — it stays in both. Pin the last one — it moves first in both,
shows a pin, and ⌘W leaves it open. Drag a row — the tab order matches. Quit and
reopen — the pin and the order are as left.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI; no I/O in the draw path.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is a paused surface: order and pins are a per-machine
  preference and live beside it, as the column's order already does (change 181).

## Systems likely affected

The session column and the tab strip drawing in `src/app/screens.rs`, the tab
lifecycle in `src/app.rs`, the order file and tab model in
`src/ui/session_tree.rs`, strings in `src/i18n_tables.rs`, the three READMEs,
and tests in `src/tests.rs`.

## Open questions

None left: both were put to the person and answered (see Desired outcome).

## Not in scope

- The Sessions list page (the older list with search and 要対応 pins): it keeps
  its own rows and its drag; only the order file it shares gains pins.
- Document (file) tabs: they have no row, and are not pinned.
- A filter or a fold in the column hides rows on purpose; the tabs are not
  filtered with it.
- A finished, unread session whose tab was closed is no longer surfaced in the
  column; ⌘J and the 要対応 queue still reach it.
