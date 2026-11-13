# Intent: a path in a terminal opens outside Operon, and a bare file name does not open at all

- **Status**: approved
- **Opened**: 2026-11-13

## Problem

The person's request, verbatim (2026-11-13, with a screenshot of a session whose
output lists `Operon_LT_10min.pptx` and `verification.yml`):

> 画面の真ん中の例えばOperon_LT_10min.pptxとか verification.ymlなどパスが書いてる文字列は
> コマンド + タップでOperon上でファイルとして開く。Command + カーソルを当てて右クリックなら
> Finderやそのまま開くなど選択肢ができるようにしたい。

Today ⌘-click hands the file to the macOS default application, so the person
leaves Operon for the one gesture they reach for first. Right-click offers only
"copy the path" and "show in Finder", in a box that does not appear where the
pointer is. And a name printed without its folder — `Operon_LT_10min.pptx`,
which lives under `dist/` — is not recognised as a file at all, so neither
gesture does anything on it.

## Who feels it, and when

Every time an agent finishes and reports the files it wrote, usually as bare
names or short relative paths, and the person wants to look at one.

## Desired outcome

- ⌘-click on a file named in the terminal opens it as a tab inside Operon.
- Right-click (with or without ⌘) on it shows, at the pointer, a choice of:
  open in Operon, open with the default application, show in Finder, copy path.
- A bare file name that matches exactly one file in the session's folder opens
  that file through both gestures.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- User-facing text is Japanese; code, comments, and docs are English.
- No file-system work in the draw path.

## Systems likely affected

The terminal pane's click handling and its path menu, terminal path resolution,
and the message tables.

## Open questions

None. ⌥-click keeps opening inside Operon (unchanged); opening with the
default application moves from ⌘-click into the menu — decided from the request.

## Not in scope

- Paths broken across two terminal rows by wrapping.
- A name that matches several files: it stays plain text rather than guessing.
- URLs: ⌘-click still opens the browser; right-click on a URL stays as today.
