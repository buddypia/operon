# Intent: a text file past 128 KiB cannot be read at all

- **Status**: approved
- **Opened**: 2026-10-30

## Problem

On the Sessions page a person tapped an HTML file in the Files tab on the right
and got only 「748 KiB のファイルです。エディタで開けるのは 128 KiB までです。」.
Nothing of the file is shown. The ceiling is about what can be *edited*, yet it
also decides what can be *read*, so every generated report, bundled HTML page,
lock file, or long log an agent writes is a blank pane.

The person's words: 「中身を表示することが重要です。適切なフォーマットに合わせて
ファイルビューする機能を実装してください。」

## Who feels it, and when

Every time an agent produces a text file larger than 128 KiB — a single-file
HTML report, a JSON dump, a CSV, a log, a long Markdown document — and the
person opens it from the Files tab to check what the agent made.

## Desired outcome

- Opening such a file shows its contents in the pane, in the same coloured,
  line-numbered form a small file of the same kind gets.
- The person can see that the file is shown read-only and why, and nothing they
  do there can overwrite it.
- Scrolling a file of several MiB stays as smooth as scrolling a small one, and
  a file whose content is one very long line (minified HTML, JSON) is readable
  rather than one row nobody can scroll to the end of.
- A file too large even for that says so, and the way to open it elsewhere is
  right there.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI: nothing proportional to
  the file's size may run every frame.
- User-facing text is Japanese; code, comments, and docs are English.
- No new dependency.

## Systems likely affected

`src/git.rs` (the file read), `src/config.rs` (the ceilings), `src/app.rs` and
`src/app/screens.rs` (the open document and its pane), `src/ui/syntax.rs`
(highlighting), `src/i18n_tables.rs`.

## Open questions

None. The ceiling for read-only display is decided in `spec.md`.

## Not in scope

- Editing a file past 128 KiB. The editable ceiling is a layout cost and stays.
- Rendering HTML as a web page inside the app: egui has no HTML engine, and the
  existing 「既定のアプリで開く」 button already opens it in the browser.
- The rendered Markdown preview for a file past 128 KiB. That view lays out
  every block on every frame; making it virtual is its own change. Such a file
  is shown as highlighted Markdown source instead.
- Pretty-printing or reformatting JSON / minified files.
- Binary files and images: unchanged.
