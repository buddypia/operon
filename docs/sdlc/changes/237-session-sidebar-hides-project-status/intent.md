# Intent: the session page's project list hides what each project's sessions need

- **Status**: approved
- **Opened**: 2026-10-30

## Problem

The user, verbatim: 「セッションページの左側にプロジェクト一覧があるが、新しいセッションの
生成が重複しているし、UI/UXがイマイチだから改善したい。プロジェクト別に承認待ちなど
ステータスをまとめて表示して、タップすると移動するようにしたい。折りたたみ形式で
プロジェクト一覧のプロジェクトは上下自由に移動できるようにしておいて
承認待ちなどステータスをでセッションのタブに簡潔に表示したい。」

Today the left column offers "start a session" twice (a column-wide button and
one per project, beside the toolbar's own), shows only a bare count of running
sessions per project, and nothing on a central session tab says whether that
session is waiting for a permission, working, or free.

## Who feels it, and when

Someone running several agents across projects who wants to know which one is
asking for approval and get to it: today they must open each project and each
tab to find out.

## Desired outcome

- One way per project to start a session in the column; no column-wide duplicate.
- Each project shows, even when folded, how many of its sessions need the
  person, are working, and are free; tapping a count or a listed session brings
  that session's terminal to the front.
- Projects fold open and closed, stay reorderable by dragging, and keep their
  fold state across a restart.
- Every central session tab shows that session's state in one glyph.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/app/screens.rs` (left column, tabs), `src/ui/session_tree.rs`
(arrangement file), `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None open: the layout question put to the person went unanswered; the
recommended option was adopted and is recorded in `state.yaml`.

## Not in scope

The toolbar's 「新しいセッション」, the history library, the inspector docks,
and finished sessions (still reached through 履歴).
