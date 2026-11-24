# Intent: a project folder cannot be typed or browsed in one place, and a folder that does not exist cannot become a project

- **Status**: draft
- **Opened**: 2026-11-24

## Problem

The person's request, verbatim (2026-11-24):

> プロジェクトを生成や開く時にパスで入力して開けるように機能を追加したい。フォルダー構造の選択式で開くことも可能に

Told what exists today and asked how far to go, they answered (2026-11-24):

- Scope: 「アプリ内の統合ピッカー」 — the option described to them as: one
  in-app dialog holding a path field (candidate folders offered while typing)
  and a folder-tree choice, opened from both the projects page and the
  新しいセッション sheet.
- Whether 「生成」 means making a folder that does not exist yet:
  「フォルダ作成も含む」 — the option described to them as: when the typed path
  does not exist, create the folder after a confirmation and register it as a
  project. They did not choose the option that also runs `git init`.

Today, opening a folder as a project is spread over three places that do not
agree. The projects page has a macOS folder dialog behind 「フォルダを選択…」
and, folded away under 「パスを手入力」, a bare text field that offers nothing
while a path is typed and refuses a path that is one character wrong. The
新しいセッション sheet has only the macOS dialog: a person who knows the path
cannot type it there. Nowhere can a folder that does not exist yet become a
project: the person leaves for a terminal or Finder, makes it, and comes back.

## Who feels it, and when

- A person who knows where the project is (`~/dev/…`) and would type it faster
  than they can click through a system dialog — every time they add a project,
  and with no way to do it at all from the 新しいセッション sheet.
- A person starting something new: the folder is not there yet, so the first
  step of a new project happens outside the application.

## Desired outcome

- From the projects page and from the 新しいセッション sheet, the same in-app
  place opens. In it a person types a path and the folders that match what they
  typed so far are listed as they type; or they walk the folders by clicking
  down and up; or they mix the two. Opening the folder they arrive at makes it
  the project (projects page) or the sheet's project (新しいセッション), exactly
  as choosing it in the macOS dialog does today.
- Typing a path that does not exist and confirming once leaves a new, empty
  folder at that path, registered as a project. Without the confirmation
  nothing is created.
- The macOS folder dialog is still reachable for a person who prefers it.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI: no file-system call in a
  draw path.
- Local-first; user-facing text is Japanese with rows in every language table.
- macOS asks the person, in Operon's name, before an application reads
  Desktop, Documents, Downloads or a mounted volume. Listing one is acceptable
  only because the person walked or typed their way into it.
- A listing has a stated ceiling.
- No persisted shape changes: a project is still `id, path, name, added_at`.

## Systems likely affected

`src/app.rs` and `src/app/screens.rs` (where the pickers are called),
`src/app/launch_projects.rs` (the sheet's folder entry), a new child of
`src/app.rs` for the picker, `src/app/keymap.rs` (⌘O), `src/config.rs` (the
ceiling), `src/i18n_tables.rs`, and the feature documents for `project-list`
and `new-session`.

## Open questions

None. Scope and folder creation were answered by the person above.

## Not in scope

- `git init`, templates, or any file written inside a created folder.
- The workspace scan (「ワークスペースをスキャン…」) and the agent-account folder
  choice keep the macOS dialog.
- Removing the projects page's 「パスを手入力」 group: it stays as it is.
- Remembering visited folders across launches, favourites, or a multi-column
  browser.
