# Intent: starting a session in a folder that is not a project yet means leaving for the projects page

- **Status**: approved
- **Opened**: 2026-11-13

## Problem

The person's request, verbatim (2026-11-13):

> セッションページでも新しいセッションの中に既存のプロジェクトだけでなく新しいプロジェクトも
> 作れるようにしたい。なぜなら新規プロジェクトを作るためにプロジェクトページに移動するのは
> UX上よくないから。他にも良い提案があれば言って。

Asked which scope they meant and which of the proposed extras to include, they
answered (2026-11-13):

- Scope: 「既存フォルダの登録のみ」 — register an existing folder (the macOS
  folder picker's own 「新規フォルダ」 covers making one); no mkdir / `git init`
  form.
- Extras, all three chosen: 「プロジェクト0件時もシートを開く」,
  「シートへのフォルダのドロップ」, 「最近使った順と絞り込み」.

Today the 新しいセッション sheet's project list offers only registered
projects. Adding one means closing the sheet, going to プロジェクト, adding the
folder there (which also lands on that project's page), and opening the sheet
again. With no project at all, 新しいセッション does not open the sheet: it
says 「先にプロジェクトを追加してください。」 and moves to the projects page. A
folder dropped while the sheet is open is added and the window moves to the
projects page behind it. The list is in the order projects were added, with no
way to narrow it.

## Who feels it, and when

Whenever the person wants an agent in a folder they have not opened in Operon
before — a fresh clone, a new repository — and on first launch, when there is
no project yet.

## Desired outcome

- The sheet's project list ends with 「＋ 新しいプロジェクト…」, which opens the
  folder picker; the chosen folder becomes a project and the sheet switches to
  it, without leaving the page. A folder that is already a project is switched
  to, not refused.
- With no project, 新しいセッション opens the sheet anyway, and the sheet offers
  choosing a folder.
- A folder dropped on the window while the sheet is open becomes (or selects)
  the sheet's project; the page does not change.
- The list puts recently used projects first and can be narrowed by name.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI; `rfd` 0.15 for the picker.
- User-facing text is Japanese; code, comments, and docs are English.
- No file-system work in the draw path; no persisted-shape change wanted.

## Systems likely affected

The launch sheet, project registration, dropped-folder handling, the message
tables.

## Open questions

None — answered above.

## Not in scope

- Creating a folder or running `git init` from Operon.
- Adding projects from the projects page (unchanged).
- Workspace scanning from the sheet.
