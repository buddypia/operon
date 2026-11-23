# Spec: diff へのメモと比較

## Behaviour
1. In a diff pane that is editable (the project 「変更」 diff, and the editor's 「差分」 view while comparing HEAD ↔ local), clicking a line that exists on the new side opens a note editor under that line with the hint 「この行について書く」 and buttons 「削除」 and 「保存」. The editor takes the keyboard once, on its first frame. Pinned by `the_note_editor_does_not_take_the_keyboard_back`.
2. A note (`DiffComment`) stores project, file, new-side line number, an anchor (SHA-256 of the line text, trailing whitespace trimmed), the body, and booleans `sent` and `resolved`. Saving an empty body deletes the note; saving an unchanged body changes nothing. The body is cut to `DIFF_COMMENT_BODY_MAX_CHARS` (2000). Pinned by `diff_annotations_lifecycle_tracks_sent_and_resolved_states`.
3. Notes are drawn inside the diff's scrolling column under their line, with header `{ICON_COMMENT} {file}:{line}`, marks 「[送信済み]」, 「[解決済み]」, 「行が変わっています」, and buttons 「解決」/「再開」 and 「削除」. A resolved note's body is struck through. Row heights are measured and cached so rows below start under the note. Pinned by `places_a_comment_row_under_the_line_it_belongs_to`, `a_long_diff_note_wraps_and_pushes_the_next_line_down`, `finds_the_visible_rows_when_the_viewport_begins_inside_a_comment`.
4. A note whose line text no longer hashes to its anchor is stale: it is marked, kept where it is, and never moved or deleted. Pinned by `a_note_on_a_changed_line_says_so_from_the_per_file_review`.
5. Notes persist in the sidecar `diff-comments.json` beside the store, never in the store. Legacy notes without `sent`/`resolved` load as false. Pinned by `keeps_diff_comments_across_a_reload`, `deserializes_legacy_diff_comments_with_defaults`.
6. When at least one note exists for the project, a footer is drawn: `{ICON_COMMENT} {n} 件`, or `{n} 件（未解決 {u}、送信済 {s}、解決済 {r}）` once any is sent or resolved; buttons 「セッションにまとめて送る」, 「Markdownをコピー」, 「解決済みを消去」 (only when some are resolved), 「すべて消す」. With no notes the footer is not drawn.
7. 「セッションにまとめて送る」 (`send_diff_comments`): the target is the project's Active non-terminal session with the latest launch time. It refuses with 「このプロジェクトで動いているエージェントのセッションがありません。」 when none, and 「エージェントの応答が終わるまで待ってください。」 when the target is Working. It sends every unresolved note (sent ones included) as one message followed by Enter, then sets `sent` on them, saves, and says 「コメント {n} 件をエージェントに送りました。」. Notes are not deleted.
8. The message (`diff_comment_message`) has one block per note, blank-line separated, no preamble: `File: <path>`, `Line: <n>`, an optional `Note: the line has changed since this comment was written.`, and `User comment: "<body>"` with backslash, double quote, CR and LF escaped. Pinned by `writes_one_block_per_comment_in_the_agreed_form`, `escapes_a_comment_body_so_it_cannot_end_its_own_quoting`, `marks_a_comment_whose_line_has_changed`.
9. 「Markdownをコピー」 copies all notes (resolved included) as `# Review Notes`, a `## <file>` heading per file, and `- Line n [stale, sent, resolved]:` quoted bodies. Pinned by `formats_diff_comments_as_markdown`.
10. In the editor, the view switch for a project-rooted text document is 「編集」, 「差分」, 「注釈」. In 「差分」 a compare bar shows 「比較:」, a picker, 「↔ ローカル」, the layout switch 「左右」/「統合」, previous/next-change buttons 「前の変更 (⇧F7)」/「次の変更 (F7)」 and a 「n / total」 position (「変更なし」 when none). Pinned by `diff_307_change_blocks_step_forward_and_back`.
11. The picker (「比較するリビジョン」) lists 「HEAD（最新コミット）」 and the file's history from `git log --follow`, at most `FILE_HISTORY_MAX_ENTRIES` (200) rows with short hash, date, author, summary. Choosing one compares that revision ↔ local. 「このコミットの変更を表示」 (from 注釈) shows one commit's change (`parent ↔ commit`, label 「{short} の変更（親 ↔ {short}）」) with 「HEAD と比較に戻る」. Pinned by `diff_307_file_log_parses_fields_and_caps`.
12. Column labels: HEAD ↔ local is 「HEAD」/「ローカル」; a revision is `<short>`/「ローカル」; a commit is 「{short} の親」/`<short>`.
13. Layout 「左右」 pairs a removal run with the following addition run line by line, fills the shorter side with an empty filler, and spans hunk headers and notes across both columns. It is used only at widths ≥ `DIFF_SIDE_BY_SIDE_MIN_WIDTH` (560 px); below it 統合 is drawn and the stored choice is untouched. The choice is stored and old stores load as 左右. Pinned by `diff_307_side_by_side_pairs_rewrites_and_fills_gaps`, `diff_307_side_by_side_falls_back_below_min_width`, `diff_307_diff_layout_round_trips_and_old_stores_default_to_side_by_side`, `diff_307_side_by_side_pane_draws_both_columns`.
14. Hunk buttons and notes exist only while the comparison is HEAD ↔ local. Pinned by `diff_307_only_head_comparison_offers_hunk_actions_and_notes`.
15. 「注釈」 (blame) shows per line a gutter `<short> <date> <author(12 chars)>` shown only on the first line of a run from one commit, uncommitted lines marked, hover text with commit, author, date and summary, and a click menu: 「HEAD とローカルを比較」, 「このコミットの変更を表示」, 「このリビジョンとローカルを比較」, 「コミットハッシュをコピー」. Pinned by `diff_307_blame_porcelain_parses_runs_and_uncommitted`.

## States
- Compare empty: 「このファイルは、最後のコミットから変更されていません。」 (HEAD), 「このリビジョンとローカルに違いはありません。」 (revision), 「このコミットはこのファイルを変更していません。」 (commit).
- Loading/error: 「差分を読み込んでいます…」, 「差分を取得できません: …」, 「履歴を読み込んでいます…」, 「履歴を取得できません: …」, 「注釈を読み込んでいます…」, 「注釈を取得できません: …」.
- Untracked file in 注釈: 「このファイルはまだコミットされていません。」 (a state, not an error).
- Blame output over `GIT_BLAME_MAX_BYTES` (4 MiB): 「大きすぎるため注釈を表示できません（{limit} KiB まで）。」; never shown partially.
- Send message over `DIFF_COMMENT_MESSAGE_MAX_BYTES` (32 KiB): refused with the count and ceiling, nothing sent. No unresolved notes: 「送信する未解決のコメントがありません。」.
- Notes file unreadable or unparseable: yields no notes and is moved aside before any save, so it is not overwritten.
- A revision that is not a full commit id is refused before any git child starts.

## Invariants
- A note is never moved to another line, and stale is reported in the sent text.
- Nothing is typed into an agent that is mid-turn.
- Sending does not delete; resolved notes are excluded from sending, included in Markdown.
- Notes live in the sidecar, not in the store (the store is a paused surface).
- The diff pane stays virtualised: visible rows are found by binary search over cumulative offsets, not per-row height division.
- Git arguments for each comparison come from one function (`revision_diff_arguments`): `diff --no-ext-diff --unified=3 <sha> -- <file>`, or `show --no-ext-diff --unified=3 --format= --diff-merges=first-parent <sha> -- <file>`.
- Every diff line appears in exactly one side-by-side row, in order.
- Comparison and blame requests run in the background.

## Code and tests
- Symbols:
- `src/git/comments.rs::DiffComment`, `DiffCommentStore`, `DiffAnnotations`: note model, store and in-memory editing state (open, save, remove, mark sent, resolve, clear).
- `src/git/comments.rs::diff_comment_message`, `format_diff_comments_markdown`, `comment_anchor`: the message, the Markdown and the anchor.
- `src/git/comments.rs::load_diff_comments_reporting`, `save_diff_comments`: sidecar read (moving aside bad files) and write.
- `src/app.rs::send_diff_comments`, `annotated_diff_comments`, `copy_diff_comments_markdown`, `ui_diff_comment_footer`: send, stale detection, copy, footer.
- `src/ui/diff.rs::diff_pane_with_view`: draws rows, note rows and hunk buttons; `side_by_side_rows`, `effective_diff_layout`, `change_block_starts`: layout helpers.
- `src/app/compare.rs::ui_editor_diff`, `ui_editor_compare_bar`, `ui_editor_annotate`: compare view, bar and blame view.
- `src/git.rs::revision_diff_arguments`, `git_revision_diff`, `git_file_history`, `git_blame`, `parse_blame_porcelain`, `parse_file_log`: git side of comparison and blame.
- `src/config.rs::DIFF_COMMENT_BODY_MAX_CHARS`, `DIFF_COMMENT_MESSAGE_MAX_BYTES`, `GIT_BLAME_MAX_BYTES`: limits.
- Tests:
- `writes_one_block_per_comment_in_the_agreed_form`: the message form.
- `escapes_a_comment_body_so_it_cannot_end_its_own_quoting`: escaping.
- `marks_a_comment_whose_line_has_changed`: stale marking.
- `keeps_diff_comments_across_a_reload`: persistence.
- `places_a_comment_row_under_the_line_it_belongs_to`: note rows in the column.
- `finds_the_visible_rows_when_the_viewport_begins_inside_a_comment`: virtualisation with notes.
- `refuses_a_comment_message_past_its_byte_ceiling`: 32 KiB ceiling.
- `formats_diff_comments_as_markdown`: Markdown copy.
- `deserializes_legacy_diff_comments_with_defaults`: old files.
- `diff_annotations_lifecycle_tracks_sent_and_resolved_states`: sent and resolved states.
- `every_pane_that_takes_a_note_offers_a_way_to_send_it`: each notes pane has the footer.
- `an_unreadable_notes_file_is_kept_rather_than_overwritten`: bad sidecar moved aside.
- `a_long_diff_note_wraps_and_pushes_the_next_line_down`: long notes wrap.
- `the_note_editor_does_not_take_the_keyboard_back`: focus taken once.
- `a_note_on_a_changed_line_says_so_from_the_per_file_review`: stale in the editor's review.
- `diff_307_blame_porcelain_parses_runs_and_uncommitted`: blame parsing.
- `diff_307_file_log_parses_fields_and_caps`: history parsing and 200 cap.
- `diff_307_side_by_side_pairs_rewrites_and_fills_gaps`: row pairing.
- `diff_307_change_blocks_step_forward_and_back`: next/previous change.
- `diff_307_revision_diff_arguments_per_base`: git arguments, refusing non-commit ids.
- `diff_307_diff_layout_round_trips_and_old_stores_default_to_side_by_side`: stored layout.
- `diff_307_only_head_comparison_offers_hunk_actions_and_notes`: notes only against HEAD.
- `diff_307_side_by_side_falls_back_below_min_width`: width fallback.
- `diff_307_side_by_side_pane_draws_both_columns`: both columns draw.
- `diff_307_blame_reports_untracked_and_too_large`: blame states.

## Decisions inherited
- Change 020: a note anchors to what the line said, is marked rather than moved when it changes, and one message carries all notes; nothing is sent to a working agent.
- Change 047: sending marks notes sent instead of deleting them, so the review stays visible; resolve/reopen, Markdown copy and clearing resolved were added.
- Change 206: a long note wraps and does not hold the keyboard after it is saved.
- Change 222: selecting a changed file filters the diff in place, so notes are written where the files are listed.
- Change 261: hunk actions are limited to the project checkout and, with notes, to HEAD ↔ local.
- Change 307: comparison with a past revision, side-by-side layout, change navigation and blame were added; they never offer hunk buttons or notes.
- Change 006: word-level marks on rewritten lines are computed at parse time and shown on both sides in side-by-side.
- Change 277: re-reading a diff must not blank the pane.
