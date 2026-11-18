# Intent: the session column opens every project unfolded, so the projects themselves are pushed out of sight

- **Status**: draft
- **Opened**: 2026-11-18

## Problem

The person's request, verbatim:
「またデグレしているよ。既存GithubのセッションページのUIを参考にして。1. 左のサイドバーはプロジェクトを主に表示。プロジェクトとの間隔は少し縮める。2. プロジェクト内では折り畳みUIでセッション一覧が表示される。デフォルトは折り畳んだ状態 3. 中央トップにセッション名とタブが表示される。4. タブ名はピン留めやタブの移動ができること」

On the session page the left column lists each project with all of its open
sessions under it. Every project starts unfolded, so with several projects the
column is mostly session rows: the list of projects — what the person reads the
column for first — does not fit without scrolling, and a project has to be
folded by hand, one at a time, to get it back. Even folded, each project keeps
a rule under its name and a margin around it, so the projects sit further
apart than a plain list needs.

Points 3 and 4 of the request — the session name and its tabs at the top of
the centre, a tab that can be pinned and moved — are in the current build
(changes 285 and 302). They looked lost because the app the person had open
was again `/Applications/Operon Stable.app`, built 2026-10-24, started
2026-11-18 13:53, and not the current `/Applications/Operon.app`. The same
thing produced the same report in change 302.

## Who feels it, and when

Anyone with several projects, each with a few open sessions, opening the
session page: the column shows the first project's sessions and little else.

## Desired outcome

- Opening the session page, the column is a list of projects: every project
  is folded until the person unfolds it, and each still shows its counts of
  要対応 / 実行中 / アイドル sessions and its 「セッションを開始」.
- Unfolding a project lists its sessions under it; the choice is kept across a
  relaunch. A project added later starts folded.
- Folded projects sit closer together than today, so more of them fit.
- Narrowing the column with its filters still shows the sessions that match,
  whether or not their project is folded.
- The session name and tabs at the top of the centre, pinning, and moving a
  tab keep working as they do in the current build, and the person is told
  which installed app shows them.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI; nothing new per frame.
- User-facing text is Japanese; code, comments, and docs are English.
- Sizes, colours and icons stay on `DESIGN.md`'s existing scale and roles.
- The column's order file is a per-machine preference on real machines: a file
  written by the current build must still load.

## Systems likely affected

`src/app/screens.rs` (the session column), `src/ui/session_tree.rs` (the
arrangement file), `src/app/session_order.rs`, `src/tests.rs`.

## Open questions

None for the change. Whether `/Applications/Operon Stable.app` is removed is
the person's to answer; it is asked in the final report.

## Not in scope

- Redesigning the centre-top tab strip, pinning, or tab dragging.
- Deleting or replacing `/Applications/Operon Stable.app`.
- Unfolding a project automatically when one of its sessions is opened from
  elsewhere (a count, ⌘J): the tab opens and the project stays as the person
  left it.
