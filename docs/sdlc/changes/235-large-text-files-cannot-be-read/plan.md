# Plan: show text files past the editable ceiling, read-only

- **Spec**: `./spec.md`
- **Approved**: 2026-10-30
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | `VIEWER_FILE_MAX_BYTES`, `VIEWER_ROW_MAX_CHARS`, `VIEWER_CHUNK_ROWS` |
| `src/git.rs` | `read_file_for_editing` refuses past the viewer ceiling, not the editor one, and reads with `take` |
| `src/ui/syntax.rs` | `ViewerRows` (u32 row starts), `viewer_rows`, `row_of_line`, `ViewerLayoutCache` (inside `EditorTextLayoutCache`) |
| `src/app.rs` | `OpenDocument::viewer`, `modified()`; `apply_opened_file` sets or clears `viewer` in every arm |
| `src/app/screens.rs` | Preview→Edit coercion, 「表示」 label, no Preview / save for viewer documents, notice, `ui_editor_viewer` taking the jump line |
| `src/i18n_tables.rs` | three new rows per table, one removed |
| `src/tests.rs` | seven new tests, two updated |
| `README.md`, `README.ja.md`, `README.ko.md` | name the 8 MiB ceiling |

## Order of work

1. Constants in `src/config.rs`; tests for requirements 1–10 written against
   symbols that do not exist yet, watched failing to compile / failing.
2. `src/git.rs` ceiling and message; strings in `src/i18n_tables.rs`.
3. `viewer_rows` and `ViewerLayoutCache` in `src/ui/syntax.rs`.
4. `OpenDocument::viewer`, `modified()`, and `viewer` assigned in every arm of `apply_opened_file`.
5. Pane drawing in `src/app/screens.rs`: coercion, label, notice, viewer, jump line. The Markdown preview keeps reading `buffer` and is never reached for a viewer document.
6. README sentences; `make q.fast`; named tests; run the app on a 748 KiB HTML.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A view-only document is saved with its empty `buffer` | the file on disk is emptied | `modified()` false for viewer documents; requirement 2 test calls save and checks the file |
| Per-frame cost proportional to file size | scrolling stutters on a large file | rows built in `apply_opened_file`; requirement 5 test counts highlight builds; manual scroll of the real file |
| A row range splits a UTF-8 character | panic on slicing | requirement 4 test with multibyte text over the wrap width |
| Highlight sections cut wrongly at row boundaries | panic or wrong colours | requirement 6 test asserts each job's sections cover exactly its text |
| Watch compares against an empty `known` | spurious reload every two seconds | `known` is `disk`, which holds the text; existing watch tests |
| A stale `viewer` after a reread | an editable small file stays read-only, or rows index past the text | `viewer` assigned in every arm; requirement 10 test |
| Markdown viewer document drawn in Preview | blank pane, or 8 MiB through `markdown_view` per frame | draw-time coercion; requirement 9 test |
| Row table size | memory spike on a file of newlines | 4-byte rows; stated budget |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked` filtered to each test spec.md names — all named tests pass.
- The running app opens a 748 KiB HTML file with numbered, coloured source and
  the read-only notice, and scrolls to its end.

## Departures from the plan
