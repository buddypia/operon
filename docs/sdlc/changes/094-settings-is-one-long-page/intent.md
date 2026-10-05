# Intent: settings is one long page

- **Status**: approved
- **Opened**: 2026-09-27

## Problem

The owner's Before/After review (section ④ 設定) found:

- Appearance, readiness, local data, notifications, accounts, and recovery sit
  on one page, one under another; a person scrolls to find the one they want.
- Its title is drawn with `ui.heading`, unlike every other page's header.
- Whether a tool is there, whether its hook is registered, and which accounts
  it has are three places on the page, and the verdicts are English
  (AVAILABLE / NOT FOUND / INSTALLED).
- Notifications, auto-approving the folder-trust prompt, and hooks are three
  checkboxes in a row with nothing to tell their weight apart.

## Who feels it, and when

Whenever the owner opens settings: to fix a missing tool (often sent there by
「設定を開く」 from a refused launch), to change the theme, or to recover a
terminal.

## Desired outcome

As in the approved `/tmp/operon-ui-before-after/view-settings-after.html`
(a copy sits in the worktree's `.tmp/ui-before-after/`), ④ rows 1–4:

- A left list of five sections — 外観 / エージェント / 通知 / データと復旧 /
  キー割り当て — and the chosen one on the right. エージェント carries a
  readiness badge.
- Each section starts with the page header every other page uses: its name and
  one line on what it holds.
- エージェント lists each tool on one row — available or not, its hook, its
  accounts — in Japanese, with a way to act on what is missing in that row.
- The startup behaviours (hooks, folder-trust auto-approval, the latter saying
  it approves only folder trust) sit together; notifications move to 通知.
- Every setting reachable today stays reachable.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31. User-facing text is Japanese, through
  `tr`/`tf!` with a row in every table.
- No persisted-store change; the chosen section is in-memory.
- The draw path does no I/O.

## Systems likely affected

`src/app/screens.rs` (`ui_settings` and its sections), `src/app.rs` (the
section field, the entry points that open settings), `src/ui/widgets.rs` (the
tool and hook rows), `src/i18n_tables.rs`, `src/tests.rs`.

## Open questions

None. The owner answered "全項目 OK、進めて" to the Before/After, including the
checklist row 「設定のカテゴリ分け」, on 2026-09-27, and "続けて" since.

## Not in scope

- Editing key bindings in the app: キー割り当て shows the bindings and where
  the file that changes them lives, as the file is the way to change them today.
- A toggle-switch widget: the settings stay checkboxes.
