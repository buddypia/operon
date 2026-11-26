# Spec: ファイルとエディタ

## Behaviour
1. The 「ファイル」 tab shows a project tree (268 px wide) on the left and the editor on the right. The tree is built from one background scan (`FILE_TREE_LIMIT` = 12,000 paths); directories come first, then files, each by name. Symlinks are skipped, and any path with a `.git`, `node_modules`, `target` or `.next` component is skipped.
2. The tree has a filter box (hint 「ファイルを絞り込む」) and a refresh button that rereads the file list and Git state. Files with a Git status carry a badge: A (new or added), D, R, U, otherwise M.
3. Opening a file creates an open document keyed by (project, root, path). The root is the project for tree-opened files, or the session worktree for files followed from a terminal link. Two roots with the same relative path are two tabs; a non-project tab shows the branch name after the file name. Pinned by `a_document_opens_and_saves_under_the_root_it_was_opened_from`.
4. A path that resolves outside the project and the session worktree is refused. Pinned by `the_editor_cannot_reach_outside_the_project`.
5. View switch: 「編集」, 「プレビュー」 (Markdown and images only), 「差分」, 「注釈」. Images show only Preview. Documents not rooted at the project show neither 「差分」 nor 「注釈」, and a remembered Diff/Annotate view falls back to Edit. Pinned by `a_document_rooted_outside_the_project_is_not_offered_a_diff`.
6. Text files up to `EDITOR_FILE_MAX_BYTES` (128 KiB) are edited with a line-number gutter, no wrapping, and grammar-based highlighting by file extension. Unknown extensions are plain text. ⌘S saves, and the 「保存」 button is enabled only while the buffer differs from disk. Pinned by `the_editor_classifies_common_project_text_formats`.
7. Text files above 128 KiB and up to `VIEWER_FILE_MAX_BYTES` (8 MiB) open in a read-only viewer. The first view tab reads 「表示」 with a note giving the size and the 128 KiB limit, no 「保存」 button is drawn, and a Markdown file is shown as highlighted source. Lines longer than `VIEWER_ROW_MAX_CHARS` (400) continue on following rows that have no line number. Pinned by `a_text_file_past_the_editable_ceiling_opens_read_only`, `a_large_markdown_file_is_shown_as_source_not_preview`, `viewer_rows_wrap_long_lines_and_number_only_their_first_row`.
8. Files over 8 MiB are not read. The pane says 「N MiB のファイルです。表示できるのは 8 MiB までです。右上の「既定のアプリで開く」から開けます。」. Pinned by `a_text_file_past_the_viewer_ceiling_says_how_to_open_it`.
9. Images up to `EDITOR_IMAGE_MAX_BYTES` (10 MiB) that decode are previewed. A larger image reports 「N MiB の画像ファイルです。プレビューできるのは 10 MiB までです。」, and an undecodable one 「画像ファイルをデコードできませんでした。」. Pinned by `test_read_file_for_editing_image_outcome`.
10. Binary files (a NUL byte) report 「バイナリファイルはエディタで開けません。」, and non-UTF-8 files 「UTF-8 ではないため、エディタで開けません。」. A file being read shows 「読み込んでいます…」. Pinned by `a_file_the_editor_cannot_edit_says_so_instead_of_failing`.
11. Markdown renders headings, lists, tables, task items (☐ / ☑, any case of `x`), `~~strike~~`, and fenced code coloured with the same grammar as the source view when the language is known. The parse and layout are built once per distinct (text, theme) pair. Pinned by `a_checklist_renders_as_checkboxes`.
12. Save writes atomically (temporary file, `sync_all`, rename) and carries the original file mode onto the replacement. Pinned by `saving_keeps_the_file_mode_and_cleans_up_after_itself`.
13. Save refuses when the bytes on disk differ from the bytes last read: 「このファイルは開いたあとに変更されています。上書きすると、その変更が失われます。再読み込みしてください。」. Pinned by `a_save_refuses_to_overwrite_work_that_landed_after_the_file_was_opened`.
14. The front document of the active project is compared to disk every 2 seconds on a background task. With no unsaved edits it is replaced silently and the notice 「…はエディタの外で変更されたため、読み直しました。」 appears. With unsaved edits nothing changes and a bar reads 「このファイルはエディタの外で変更されました。保存すると、その変更を上書きします。」 with 「読み直す」 and 「差分を見る」 (the latter only for project-rooted documents). The bar clears on reload or a successful save, and appears together with a save error, the error first. Pinned by `a_changed_file_is_reported_and_an_unchanged_one_is_not`, `an_agents_write_reaches_an_unmodified_buffer_by_itself`, `an_agents_write_never_overwrites_what_a_person_typed`, `a_watch_answer_about_an_older_document_is_dropped`.
15. The tab row offers 「既定のアプリで開く」, a menu 「外部エディタで開く」 (VS Code, Cursor, Zed, Windsurf, Antigravity, Void, Finder, jumping to the current line), 「エージェントに送る」 (not for images, loading or unopenable documents), a refresh button 「ディスクから読み直す（編集内容は破棄されます）」, and 「保存」.
16. 「エージェントに送る」 sends an excerpt to the latest running agent of the project, or puts it in a new session's request when none runs.
17. The palette lists up to 16 project files and 8 symbols of the open files (`fn struct enum trait interface class type`, at most 64 per file); the `@` mention menu in the new-session request lists up to 8 candidates. Pinned by `ide_261_palette_ranks_project_files_and_document_symbols`.

## States
- Loading: 「読み込んでいます…」, and the document is not watched.
- Empty: 「左のツリーからファイルを開いてください。」; a document of another project says 「開いているファイルは別のプロジェクトのものです。」.
- Tree empty: 「読み取り可能なファイルはありません。」 or, with a filter, 「一致するファイルはありません。」. A truncated or partly unreadable scan adds a warning naming the omission and the count of unreadable directories.
- Unopenable (binary, non-UTF-8, too large, undecodable image): the document is read-only, offers no save, and is not watched; the refresh button rereads it.
- A watch read that fails shows nothing; a save to a vanished file fails with an error and keeps the buffer.
- A watch answer about an older state of the document (a save or reload landed first) is dropped.
- A file that shrinks under 128 KiB while open becomes editable again; one that becomes binary or exceeds 8 MiB is shown as unopenable.

## Invariants
- A save never overwrites bytes that differ from what the editor last read, and the editor never replaces a buffer that holds unsaved edits.
- No filesystem call happens in a draw path; reading, saving and watching run through background tasks.
- Reads never exceed `VIEWER_FILE_MAX_BYTES` + 1 bytes, even if the file grows during the read.
- The watch and the open share one reader, so the ceilings and the binary/UTF-8 checks are decided once.
- The viewer highlights only the chunks (`VIEWER_CHUNK_ROWS` = 64 rows) it draws, and rows never split a UTF-8 character.
- A document is opened, saved and watched under its own root, never a path joined by hand to the project's.
- Closing a document tab with unsaved edits is refused.
- Colours and icons come from the palette and `ICON_*` constants only.

## Code and tests
- Symbols:
  - `src/files.rs::visit_with_limits`: bounded directory scan used for the tree and for other file lookups.
  - `src/files.rs::editor_language`: extension to language classification; `fence_language` maps fence tags.
  - `src/files.rs::extract_file_symbols`: symbols for the palette.
  - `src/git.rs::read_file_for_editing`: size, image, binary and UTF-8 decisions.
  - `src/git.rs::read_file_for_editing_if_changed`: the watch's reader; `save_project_file` is the guarded save.
  - `src/git.rs::write_project_file_atomically`: atomic write that keeps permissions.
  - `src/app.rs::watch_active_document`: the 2-second watch; `apply_watched_file` applies or drops its answer.
  - `src/app.rs::open_document_at`: opens a document under its root.
  - `src/app/screens.rs::ui_files`, `ui_file_tree`, `ui_editor`, `ui_editor_viewer`, `ui_editor_text`, `ui_editor_image_preview`: drawing.
  - `src/ui/syntax.rs::viewer_rows`, `EditorTextLayoutCache`, `ViewerRowLayout`: splits text into viewer rows, caches `OnceLock` parsed syntect selectors, and retains shaped `Arc<egui::Galley>` instances per buffer/row.
  - `src/ui/markdown_view.rs::MarkdownPreviewCache`, `markdown_view`: builds the rendered Markdown once per text and theme, caches block galleys by quantized width/DPI, and culls off-screen blocks above `MARKDOWN_VIEWPORT_CULL_THRESHOLD`.
  - `src/markdown.rs::parse_markdown`: Markdown blocks and spans.
  - `src/ui/files.rs::file_tree_rows`, `file_tree_rows_cached`, `FileTreeRowCache`, `git_status_badge`: tree rows, cached flattened rows, and badges.
  - `src/config.rs::EDITOR_FILE_MAX_BYTES`, `VIEWER_FILE_MAX_BYTES`, `EDITOR_IMAGE_MAX_BYTES`, `FILE_TREE_LIMIT`, `MARKDOWN_VIEWPORT_CULL_THRESHOLD`: the ceilings and culling threshold.
- Tests:
  - `the_editor_draws_every_view_without_a_window`: each view draws.
  - `a_save_refuses_to_overwrite_work_that_landed_after_the_file_was_opened`: guarded save.
  - `saving_keeps_the_file_mode_and_cleans_up_after_itself`: mode kept, temp removed.
  - `the_editor_cannot_reach_outside_the_project`: path containment.
  - `a_changed_file_is_reported_and_an_unchanged_one_is_not`: byte comparison in the watch.
  - `an_agents_write_reaches_an_unmodified_buffer_by_itself`: silent reload.
  - `an_agents_write_never_overwrites_what_a_person_typed`: the bar, no replacement.
  - `a_watch_answer_about_an_older_document_is_dropped`: stale answers.
  - `a_file_the_editor_cannot_edit_says_so_instead_of_failing`: unopenable messages.
  - `a_text_file_past_the_editable_ceiling_opens_read_only`: viewer range.
  - `a_text_file_past_the_viewer_ceiling_says_how_to_open_it`: 8 MiB limit.
  - `viewer_rows_wrap_long_lines_and_number_only_their_first_row`: row splitting.
  - `a_large_markdown_file_is_shown_as_source_not_preview`: Markdown in the viewer.
  - `the_editor_classifies_common_project_text_formats`: language classes.
  - `a_checklist_renders_as_checkboxes`: task items.
  - `a_document_opens_and_saves_under_the_root_it_was_opened_from`: document root.
  - `a_document_rooted_outside_the_project_is_not_offered_a_diff`: no diff for worktree documents.
  - `test_read_file_for_editing_image_outcome`: image outcomes.
  - `ide_261_palette_ranks_project_files_and_document_symbols`: file and symbol candidates.

## Decisions inherited
- Change 005: highlighting uses bundled grammars with a plain-text fallback, and layouts are cached per (text, language, palette) so a frame does not retokenize.
- Change 010: a changed file is detected by comparing bytes, not modification times, so a rewrite with identical content raises nothing; unsaved edits are never replaced.
- Change 034: a document carries its root, and non-project documents get no diff because the diff cache is keyed by project and relative path.
- Change 076: terminal links and images open inside Operon, and images are preview-only.
- Change 232: opening a session file stays in the session workspace.
- Change 235: the editable ceiling stays 128 KiB; larger text up to 8 MiB is read-only, so the file is still visible without loading unbounded data.
- Change 261: the editor and palette feed the agent loop (send to agent, symbol jump).
- Change 293: terminal paths, including bare file names, resolve against the worktree and project and open in the editor.
- Change 307: 「差分」 and 「注釈」 read the file's history afresh each time they are selected.
- Change 322: syntect `ScopeSelectors` compile once via `OnceLock`, `EditorTextLayoutCache` / `ViewerRowLayout` / `RenderedMarkdown` retain shaped `Arc<egui::Galley>`s keyed by quantised width, DPI, and theme, Markdown views above `MARKDOWN_VIEWPORT_CULL_THRESHOLD` cull off-screen blocks, `file_tree_rows` drops per-node `PathBuf` clones in favour of `FileTreeRowCache`, and `friendly_path` / `png_texture` memoize home prefixes and decoded RGBA pixel buffers (pinned by `perf_322_r4_editor_syntax_galley_and_viewer_cache`, `perf_322_r5_markdown_galley_cache_and_viewport_culling`, and `perf_322_r6_png_tree_and_palette_caches`).
