# Intent: a file cannot be compared with any past revision, nor read side by side

- **Status**: approved
- **Opened**: 2026-11-17

## Problem

The person asked, in their words: 「差分って二つあって、ローカルとGit状態の差分と
git blameに差分を選んで比較する機能が欲しい。通常よくあるIntelliJの感覚でUI/UXと
体験ができると良い」.

Today a file's diff answers one question only — the working copy against the
last commit — and answers it as one interleaved column. Two things the person
reaches for every day in an IDE are missing:

1. Reading the local-vs-committed difference as two columns, old on the left
   and new on the right, stepping from one change to the next.
2. Asking "who wrote this line, in which commit" and, from that line, comparing
   the file at that commit — what the commit changed, or how far the working
   copy has moved since.

## Who feels it, and when

Reviewing what an agent just did to a file, the person wants the before and the
after side by side; then, meeting a line that looks wrong, wants to know when it
came in and what the commit around it did — and today has to leave Operon for an
IDE or `git blame` in a terminal to find out.

## Desired outcome

- A file's diff can be read side by side (left: the older text, right: the
  newer) or interleaved, and the choice stays across files and relaunches.
- The person can jump to the next and previous change in the diff.
- The diff can be taken against a revision the person picks from the file's
  history, not only the last commit.
- The file can be shown with each line's commit, author, and date beside it;
  from a line the person can open what that commit changed in this file, or
  compare that revision with the working copy.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.37 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.

## Systems likely affected

`src/git.rs` (blame, file log, revision diff), `src/ui/diff.rs` (side-by-side
rows), `src/app/screens.rs` (editor view switch, compare bar),
`src/app.rs` (background tasks, caches), `src/models.rs`, `src/config.rs`
(remembered layout), `src/i18n_tables.rs`, `src/glyphs.rs`.

## Open questions

- Screen layout — a new view and a re-arranged diff screen: the person approves
  `screen.md`.

## Not in scope

- Comparing two arbitrary revisions with each other (only "a revision ↔ local"
  and "a commit ↔ its parent").
- Editing on the side-by-side panes, or moving hunks between them.
- Blame of a file in a worktree-rooted document (the diff view already refuses
  those, change 034).
- Directory or whole-project comparison against a past revision.
