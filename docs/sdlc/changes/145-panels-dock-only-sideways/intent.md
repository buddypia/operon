# Intent: the side panels can only be moved as one block, and only sideways

- **Status**: approved
- **Opened**: 2026-10-13

## Problem

The session workspace has four panels around the terminal: the session list
grouped by project, and the files, conversation, and changes views. Change 144
put a drag grip on the files, conversation, and changes tabs, but dragging one
of them moves the whole side panel — all three views together — and only
between the left and the right. The session list cannot be moved at all. The
grip promises that a view can be put where a person wants it, and it cannot.

In the person's words: 「ファイル、会話、変更などにドラッグできるアイコンを付けても
結局サイドウィンドウしか移動できないから意味ないよね。ちゃんと設計見直して対応して。
プロジェクトと別セッションも同じだよ。左右トップボトムに移動できるようにする必要がある。」

## Who feels it, and when

Anyone arranging the workspace around a running agent: someone who wants the
file tree under the terminal while the conversation stays on the right, or the
session list along the top of a wide screen. Today the only arrangement
available is "list on the left, the three views together on the left or right".

## Desired outcome

- Each of the four panels — the session list, files, conversation, changes — can
  be dragged by its own grip and dropped on the left, right, top, or bottom edge
  of the workspace, independently of the others.
- While dragging, the four drop targets are visible and the one under the
  pointer is highlighted; releasing in the middle, or pressing Escape, leaves
  everything where it was.
- Panels that land on the same edge share it: the views become tabs of one
  group, and the session list sits beside that group with a rule between them,
  so the list and a view are visible together (as the left dock shows today).
- Each edge can be resized by dragging the rule between it and the terminal.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/ui/session_tree.rs` (the side panel and its layout types), `src/app/screens.rs`
(the workspace and the terminal panel), `src/app.rs` (the state), `src/config.rs`
(sizes), `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

- Same edge, list plus views: tabs for everything, or the list beside a tab
  group? Answered by the person on 2026-10-13: the list beside a tab group.

## Not in scope

- Remembering the arrangement across restarts. The side the panel was docked to
  is not remembered today either; persisting it changes the store's shape and is
  a change of its own.
- Floating or detached windows, and reordering tabs within a group.
- Docking on any page other than the session workspace.
