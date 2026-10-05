# Intent: Read and revise agent-authored project files without losing the meaning in their format

- **Status**: approved
- **Opened**: 2026-08-29

## Problem

The project editor can open and change text files, render Markdown, and show a
working-tree diff, but its editing surface renders every file as one colour.
Source, configuration, and structured data therefore lose the visual structure
that people expect from a text editor. It is needlessly hard to inspect the
files an agent created before deciding whether to save, review, or compare them.

## Who feels it, and when

A person monitoring a coding agent opens a source file, configuration file, or
Markdown document from a project tree to make a small correction or inspect the
agent's output before a final Git-diff review.

## Desired outcome

The editor presents common project text formats with legible, deterministic
syntax highlighting while remaining editable. Markdown retains its rendered
preview, and every open text file remains reviewable against the last commit in
the existing diff view.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The editability and file-size safety boundaries must remain unchanged.
- Drawing uses only semantic `Palette` roles and glyphs from `src/glyphs.rs`.

## Systems likely affected

- `src/files.rs`
- `src/ui/`
- `src/app.rs`
- `src/tests.rs`
- user documentation in all supported languages

## Open questions

- None. “Industry-standard formats” is interpreted as the common project
  source, scripting, markup, data, and configuration extensions Operon already
  recognizes in its project tree, plus plaintext fallback.

## Not in scope

- Language-server features, completion, formatting, linting, or a replacement
  for a dedicated IDE.
- Editing binary files or files past the existing editor byte limit.
- Persisting editor layout or preferences.
