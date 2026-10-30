# Spec: show text files past the editable ceiling, read-only

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A UTF-8 text file larger than `EDITOR_FILE_MAX_BYTES` (128 KiB) and no larger
   than `VIEWER_FILE_MAX_BYTES` (8 MiB) is read and its text reaches the
   document. Test: `a_text_file_past_the_editable_ceiling_opens_read_only`.
2. Such a document is view-only: it is never `modified()`, the save action does
   nothing for it, and the pane says why it cannot be edited. Same test.
3. A file larger than `VIEWER_FILE_MAX_BYTES` is not read; the pane says its
   size, the ceiling, and that 「既定のアプリで開く」 opens it. Test:
   `a_text_file_past_the_viewer_ceiling_says_how_to_open_it`.
4. The viewer splits the text into rows once per read: one row per line, and a
   line longer than `VIEWER_ROW_MAX_CHARS` continues on following rows that
   carry no line number. Ranges cover the text without its line endings and
   never split a UTF-8 character. Test:
   `viewer_rows_wrap_long_lines_and_number_only_their_first_row`.
5. Highlighting is done per chunk of `VIEWER_CHUNK_ROWS` rows, only for chunks
   that are drawn, and a drawn chunk is not highlighted again while the
   document, language, and palette are unchanged. Test:
   `the_viewer_highlights_only_the_chunks_it_draws`.
6. Each row's layout holds exactly that row's text, coloured by the file's
   language (an HTML tag is not drawn in the plain text colour). Test:
   `viewer_rows_keep_the_colours_of_their_language`.
7. Binary and non-UTF-8 files past 128 KiB still say they cannot be opened, and
   a file under 128 KiB still opens editable, unchanged. Test:
   `a_file_the_editor_cannot_edit_says_so_instead_of_failing` (updated).
8. A file that grows past `VIEWER_FILE_MAX_BYTES` while it is open is reported
   by the watch as a change. Test: the existing watch test, updated to cross the
   new ceiling instead of the old one.
9. A Markdown viewer document is drawn as highlighted source even when its
   remembered view is Preview, and the rendered preview is never built for it.
   Test: `a_large_markdown_file_is_shown_as_source_not_preview`.
10. Every read resets the document's viewer state: a file that shrinks under
    128 KiB, becomes binary, or grows past 8 MiB while open is no longer drawn
    by the viewer, and a small file becomes editable again. Test:
    `a_document_that_shrinks_under_the_ceiling_becomes_editable_again`.

## Behaviour

- **Open, 128 KiB < size ≤ 8 MiB, text.** The pane shows, under the tab row, the
  muted line 「748 KiB のファイルのため、読み取り専用で表示しています。編集できるのは
  128 KiB までです。」 and below it the file: line numbers in a gutter, text in the
  monospace face, coloured as the editor colours that language. The first view
  tab reads 「表示」 instead of 「編集」; Diff stays available as before. The save
  button is not drawn. The pane scrolls both ways; rows not on screen cost
  nothing.
- **Markdown in that range** is drawn in 「表示」 (highlighted source), not in
  Preview, even though a Markdown tab opens in Preview before its size is
  known; the Preview tab is not offered for it.
- **A long line** (minified HTML or JSON on one line) is shown as consecutive
  rows of at most 400 characters; only the first carries the line number.
- **Too large, > 8 MiB.** The view tab keeps reading 「編集」 as every
  unopenable document does today. The pane shows 「12 MiB のファイルです。表示できるのは
  8 MiB までです。右上の「既定のアプリで開く」から開けます。」 (size rounded up), in
  the place the old message sat. The open-externally button in the tab row is
  already there.
- **Binary / not UTF-8.** Unchanged: 「バイナリファイルはエディタで開けません。」 /
  「UTF-8 ではないため、エディタで開けません。」.
- **Loading.** Unchanged: 「読み込んでいます…」 while the read runs.
- **Changed on disk.** The two-second watch rereads the file as before; a
  view-only document has no edits to lose, so it adopts the new text and the
  existing notice 「… はエディタの外で変更されたため、読み直しました。」 appears.
- **Jump to line** from a terminal link scrolls the viewer to the row that starts
  that line.
- **Empty / interrupted.** An empty file is under 128 KiB, so it is not this
  path. A tab closed while a read is in flight is dropped as today.

## Design

**Placement: `EXTEND`.** `git grep -n EDITOR_FILE_MAX_BYTES` shows the editor's
ceiling decided once in `read_file_for_editing` (`src/git.rs`) and the pane in
`OperonApp::ui_editor` (`src/app/screens.rs`); the change grows those two and
the highlighter beside them (`src/ui/syntax.rs`). Route `modify`.

- `src/config.rs`: `VIEWER_FILE_MAX_BYTES = 8 MiB`, `VIEWER_ROW_MAX_CHARS = 400`,
  `VIEWER_CHUNK_ROWS = 64`, each with its reason. The comment on
  `EDITOR_FILE_MAX_BYTES` already says "past this it is shown read-only".
- `src/git.rs` `read_file_for_editing`: the size refusal moves from
  `EDITOR_FILE_MAX_BYTES` to `VIEWER_FILE_MAX_BYTES` with the new sentence; the
  text read itself takes at most `VIEWER_FILE_MAX_BYTES + 1` bytes, so a file
  that grows between the metadata call and the read is refused with the same
  sentence rather than read whole. The binary and UTF-8 checks are unchanged.
  `FileOpenOutcome` keeps its shape: editability is a property of the length,
  decided by the document that holds it (no change to `src/models.rs`).
- `src/ui/syntax.rs`: `ViewerRows` (fields `starts`, `chunk_lines`, `lines`,
  `widest`; the first two are `u32` vectors) and `viewer_rows(text)`. A row is its start offset only
  (4 bytes); its end is the next row's start with the line ending trimmed,
  whether it begins a line is whether the byte before it is `\n`, and the line
  number of each chunk's first row is stored so a chunk can number its own
  rows. Offsets past `u32::MAX` are never produced (the loop stops; the read
  ceiling keeps real files far below it). `ViewerRows::row_of_line(n)` finds the
  row that starts line `n`; `ViewerLayoutCache` keyed by the
  `Arc<ViewerRows>` it was built for (`Arc::ptr_eq` — the cache holds a clone,
  so the address cannot be reused while the key lives), the language, and
  `SyntaxColours`, holding `HashMap<chunk, Vec<(LayoutJob gutter, LayoutJob
  text)>>`, cleared when it passes 64 chunks. A chunk is highlighted by running
  `syntax_layout` over the chunk's slice of the text and cutting its sections at
  the row ranges. A chunk starts from a fresh parser state, so a construct that
  began in an earlier chunk (a `<script>` body) may be coloured as its
  surroundings until the next line that re-establishes context; this is the
  price of not parsing 8 MiB up front. The cache lives inside
  `EditorTextLayoutCache` so the per-group swap in the workspace already loans
  it.
- `src/app.rs` `OpenDocument`: `viewer: Option<Arc<ViewerRows>>`, assigned in
  every arm of `apply_opened_file` — `Some` when the text is longer than
  `EDITOR_FILE_MAX_BYTES` (one linear pass over at most 8 MiB, once per read —
  not per frame), `None` in every other arm. For a viewer document `disk` holds
  the text and `buffer` stays empty. `modified()` is false when `viewer` is set,
  which is the only gate in front of `save_active_document` (⌘S included). The
  watch's early return on `read_only.is_some()` does not apply: a viewer
  document has `read_only: None`, so it keeps being watched.
- `src/app/screens.rs` `ui_editor`: for a viewer document the view is coerced
  from Preview to Edit at draw time, beside the existing image and Diff
  coercions; the Edit tab is labelled 「表示」 with `ICON_PREVIEW` there (the
  label lives in `src/models.rs`, so the override is local); no Preview tab, no
  save button, the notice line, and `ui_editor_viewer` drawing
  `ScrollArea::both().show_rows` with the cached jobs. `ui_editor_viewer` takes
  `editor_jump_line` as `ui_editor_text` does and turns it into a row with
  `row_of_line` and a scroll offset.
- `src/i18n_tables.rs`: three new rows in each table; the old 「エディタで開けるのは」
  row is removed with its last caller.
- `README.md`, `README.ja.md`, `README.ko.md`: the bounded-scans sentence names
  the 8 MiB read-only ceiling.
- Nothing persisted changes. No external command. No background task is added:
  the read already runs on one; the row split runs when its result is applied.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: gutter uses `text_muted` as the editor's gutter does, notice uses `text_muted`, syntax uses the existing semantic theme. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | The 「表示」 tab reuses `ICON_PREVIEW`; no new glyph. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | The three ceilings live in `src/config.rs`; tests reference the constants, not numbers. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | yes | Read; the change raises a read ceiling (a scan budget) but touches no store, migration, cancellation, or write. Saving is refused for view-only documents by `modified()`, so the atomic-save path is not reached with an empty buffer. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | No child process is started. |
| Documentation — user-facing docs change in all three languages together | yes | Strings added to the ja source and the en and ko tables together. `README.md`, `README.ja.md`, `README.ko.md` already say files past 128 KiB open read-only (which this change makes true); each gains "up to 8 MiB" in the same sentence. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | yes | Read ceiling 8 MiB (`VIEWER_FILE_MAX_BYTES`), enforced on the read itself with `take`; rows ≤ 400 characters; the row table is 4 bytes per row (worst case 32 MiB for 8 MiB of newlines); highlight work ≤ 64 rows per chunk; cache ≤ 64 chunks. |

## Flagged concerns

- **The `FileOpenOutcome::Unopenable` doc comment in `src/models.rs` names
  `EDITOR_FILE_MAX_BYTES`.** After this change the size reason is past
  `VIEWER_FILE_MAX_BYTES`. `src/models.rs` is a paused store surface whose
  edits are a person's; the comment is the only stale piece. Answered: the
  comment is corrected in this change only if the reviewer requires it, and
  otherwise recorded for the next change that edits the file — a comment is not
  worth demoting this change to a person-approved store edit.
- **Memory.** A viewer document holds its text once (`disk`); the watch clones
  `disk` twice every two seconds on the UI thread (`known`, `asked`), up to
  16 MiB transient, as it already does for every open text document. Answered:
  accepted — not per frame, and the image preview already allows 10 MiB; making
  the watch share an `Arc<str>` is its own change.

## Acceptance

- `cargo test --locked` passes, including
  `a_text_file_past_the_editable_ceiling_opens_read_only`,
  `a_text_file_past_the_viewer_ceiling_says_how_to_open_it`,
  `viewer_rows_wrap_long_lines_and_number_only_their_first_row`,
  `the_viewer_highlights_only_the_chunks_it_draws`,
  `viewer_rows_keep_the_colours_of_their_language`,
  `a_large_markdown_file_is_shown_as_source_not_preview`,
  `a_document_that_shrinks_under_the_ceiling_becomes_editable_again`, and the
  two updated tests.
- In the running app, opening a 748 KiB HTML file from the Files tab shows the
  coloured, numbered source with the read-only notice, scrolls smoothly to the
  end, and offers no save.

## Rejected alternatives

- Raise `EDITOR_FILE_MAX_BYTES`: the editable `TextEdit` re-lays out the whole
  buffer per keystroke; that is why the ceiling exists.
- Render HTML in-app: no HTML engine in egui; a new web-view dependency is out of
  proportion, and the browser is one click away.
- Highlight the whole file once in the background: up to 8 MiB through syntect
  takes seconds and holds a layout job as large as the file; per-chunk work is
  bounded by the screen.
- A new `FileOpenOutcome::ViewOnly` variant: editability follows from the
  length, and the variant would make a paused store-surface edit for no
  behaviour the length does not already give.
- Plain, uncoloured text: the person asked for a view fitting the format.
