# Spec: compare a file with a past revision, side by side, and annotate it

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

Each names the test in `src/tests.rs` that fails without it.

1. `git blame --porcelain` output is parsed into one entry per line — commit
   (short and full), author, author time, summary, line text — with
   uncommitted lines (all-zero commit) marked, and each line flagged when it
   starts a run of lines from the same commit, so only that line shows the
   gutter text. Commit metadata that porcelain writes only on a commit's first
   appearance is carried to its later lines.
   `diff_307_blame_porcelain_parses_runs_and_uncommitted`
2. The file's history (`git log --follow`, unit-separator fields) is parsed into
   revisions (full and short hash, author, time, summary), at most
   `FILE_HISTORY_MAX_ENTRIES` (200), skipping malformed lines.
   `diff_307_file_log_parses_fields_and_caps`
3. A parsed file's lines are laid out as side-by-side rows: context on both
   sides, a run of removals followed by a run of additions paired line by line
   (the longer run's remainder faces an empty filler), a hunk header or note
   spanning both columns. Every diff line appears in exactly one row, in order.
   `diff_307_side_by_side_pairs_rewrites_and_fills_gaps`
4. The change blocks of the rows are found (a block is a maximal run of
   non-context, non-header rows), and "next" / "previous" from a viewport row
   picks the block after / before it, stopping at the ends; the position shown
   is `n / total`.
   `diff_307_change_blocks_step_forward_and_back`
5. The git arguments for each comparison are built by one function:
   HEAD ↔ local is today's working-tree diff; a revision ↔ local is
   `diff --no-ext-diff --unified=3 <sha> -- <file>`; one commit's own change is
   `show --no-ext-diff --unified=3 --format= --diff-merges=first-parent <sha> -- <file>`.
   A revision that is not a full commit id is refused before any child starts.
   `diff_307_revision_diff_arguments_per_base`
6. The chosen layout (左右 / 統合) is kept in the store; a store written before
   this change loads as 左右. `diff_307_diff_layout_round_trips_and_old_stores_default_to_side_by_side`
7. Staging / discarding a hunk and writing a note are offered only while the
   comparison is HEAD ↔ local; any other comparison draws neither the hunk
   buttons nor the note marker. `diff_307_only_head_comparison_offers_hunk_actions_and_notes`
8. Below `DIFF_SIDE_BY_SIDE_MIN_WIDTH` (560px) the pane draws 統合 whatever the
   setting, and the setting is not changed.
   `diff_307_side_by_side_falls_back_below_min_width`
9. The side-by-side pane draws, in one frame at 900px, the left label, the right
   label, both sides' line numbers of a rewrite pair, and survives a file with
   only additions (no left lines).
   `diff_307_side_by_side_pane_draws_both_columns`
10. The blame of a file the repository has never committed is reported as that
    state, not as an error; output past `GIT_BLAME_MAX_BYTES` is reported as too
    large, never shown partially. `diff_307_blame_reports_untracked_and_too_large`

## Behaviour

`screen.md` is the layout, approved by the person. Strings:

- View switch gains 「注釈」 (icon `ICON_ANNOTATE`) after 「差分」, shown where 差分
  is shown (a project-rooted text document).
- Compare bar (差分 view): 「比較:」, picker button text 「HEAD（最新コミット）」
  or the short hash; 「↔ ローカル」; for one commit's change
  「{short} の変更（親 ↔ {short}）」 and the button 「HEAD と比較に戻る」. Layout
  toggle 「左右」/「統合」. Navigation icon buttons with hovers
  「前の変更 (⇧F7)」「次の変更 (F7)」, and 「{n} / {total} 変更」 (「変更なし」 at 0).
- Column labels: HEAD ↔ local: 「HEAD」/「ローカル」; revision ↔ local:
  「{short}」/「ローカル」; commit: 「{short} の親」/「{short}」.
- Picker popup title 「比較するリビジョン」; rows 「{short}  {date}  {author}  {summary}」;
  while loading 「履歴を読み込んでいます…」; on error 「履歴を取得できません: {error}」.
- 注釈 view: gutter 「{short} {YYYY-MM-DD} {author}」 on a run's first line,
  「未コミット」 for uncommitted lines; hover 「{short} · {author} · {date time} · {summary}」.
  Click menu: 「このコミットの変更を表示」「このリビジョンとローカルを比較」
  「コミットハッシュをコピー」; on an uncommitted line only 「HEAD とローカルを比較」.
  Copy shows the brief notice 「コミットハッシュをコピーしました。」.
- States: 「注釈を読み込んでいます…」, 「注釈を取得できません: {error}」,
  「このファイルはまだコミットされていません。」,
  「大きすぎるため注釈を表示できません（{limit} KiB まで）。」,
  「このリビジョンとローカルに違いはありません。」 (revision ↔ local, empty),
  「このコミットはこのファイルを変更していません。」 (commit, empty);
  HEAD ↔ local empty keeps today's string; diff errors keep
  「差分を取得できません: {error}」; truncation keeps today's notice.
- F7 / ⇧F7 act while the 差分 view is shown and no text field has focus.
- Switching to another document returns the comparison to HEAD ↔ local.
  Selecting 注釈 or 差分 reloads its data (the file may have changed); the
  refresh button in the editor toolbar does too.

## Design

Placement: `EXTEND` for the diff view (`src/ui/diff.rs`, the 差分 view in
`src/app/screens.rs`), `NEW_IN_MODULE` for blame and file history in `src/git.rs`
(git is the concern the map names). Route `feature` because 注釈 and revision
comparison do not exist. Evidence: `git grep -n blame src` finds nothing;
`git_working_tree_diff` (`src/git.rs`) is the only diff source of the editor.

- `src/git.rs`: `BlameLine`, `FileRevision`, `DiffBase { Head, Revision(String), Commit(String) }`,
  `parse_blame_porcelain`, `parse_file_log`, `revision_diff_arguments(base, file) -> Result<Vec<String>>`,
  `git_blame(path, file) -> Result<BlameOutcome>` (`Lines(Vec<BlameLine>)`,
  `Untracked`, `TooLarge`), `git_file_history`, `git_revision_diff(path, file, base)`.
  Children through `git_output_allowing_limited` (20 s timeout, byte ceiling):
  blame `GIT_BLAME_MAX_BYTES` = 4 MiB, history 64 KiB (200 entries × short
  lines; past it the entries read so far are kept; a cut-off last line is dropped), revision diff `GIT_DIFF_MAX_BYTES` with today's truncation notice.
  Untracked is recognised from git's `no such path` stderr (English tool
  output, kept as emitted).
- `src/ui/diff.rs`: `SidePair { left, right }` rows, `side_by_side_rows`,
  `change_blocks`, `step_change`, `effective_diff_layout`; a new `DiffRow::Pair`.
  `diff_pane_with_actions` takes a `DiffPaneView` (layout, column labels,
  whether actions and notes are offered, a pending navigation step) and returns
  `DiffPaneOutcome { notes_changed, changes, current }`. Side-by-side rows are
  computed with the rows each frame from the parsed file, as unified rows are
  today (a linear pass over the lines, no allocation per line beyond the row
  vector the pane already builds).
- `src/models.rs`: `DiffLayout { SideBySide (default), Unified }`.
  `EditorView::Annotate`.
- `src/store.rs`: `#[serde(default)] diff_layout: DiffLayout`. Additive; older
  builds ignore the field (no `deny_unknown_fields`); `STORE_SCHEMA_VERSION`
  stays 5 (an additive defaulted field is not a shape a migration has to rewrite).
- `src/app.rs`: `editor_compare: Option<(DocumentId, DiffBase)>`, caches
  `blame_cache`, `file_history_cache`, `revision_diff_cache` keyed by
  (project, file[, base]); `BackgroundKey::{Blame, FileHistory, RevisionDiff}`
  and their `BackgroundResult`s; the draw path only reads the caches.
- `src/app/screens.rs`: compare bar, picker popup, 注釈 view (virtualised rows
  like the viewer), gutter menu, F7 handling.
- `src/glyphs.rs`: `ICON_ANNOTATE`, `ICON_CHANGE_PREVIOUS`, `ICON_CHANGE_NEXT`.
- `src/i18n_tables.rs`: every string above in every table.
- README (three languages): the Git section gains the three abilities.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour | yes | Reuses `diff_added_bg`, `diff_removed_bg`, `raised`, `border_subtle`, `text_muted`; filler uses `raised`. No new role. |
| Icons | yes | Three `ICON_*` constants with `ICON_VOCABULARY` entries; `DESIGN.md` gains a line for 注釈. |
| Identifier SSOT | yes | Byte and entry ceilings and the min width are constants in `src/config.rs`; the git arguments are built in one function and tested there. |
| Durability | yes | One additive `serde(default)` store field; round-trip and old-file test (R6). No write-order, cancellation or migration change. Persisted shape approved by the person (`approvals.log`, store line). |
| Subprocess safety | yes | Three new git invocations, all via `git_output_allowing_limited` → `run_command_with_output_limit`; revision must pass `is_commit_id`; file path after `--`. |
| Documentation | yes | README.md, README.ja.md, README.ko.md together. |
| Local-first | yes | Local git only; no `unsafe`. |
| Budgets | yes | blame 4 MiB (too large → state, not partial), history 64 KiB / 200 entries, revision diff 180 000 bytes with today's truncation notice; README budget line names them. |

## Flagged concerns

- **Rename history** — `--follow` lists revisions from before a rename, and
  comparing such a revision by the current path shows the file as added.
  Answer: accepted for this change; noted under Not in scope in the intent's
  sense (arbitrary-path comparison). No further work.

## Acceptance

- `cargo test --locked diff_307_` passes; `make q.fast` passes; CI full suite green.
- In the running app: open a changed tracked file → 差分 shows two columns with
  HEAD left and ローカル right; F7 moves between changes; 統合 switches back and is
  still chosen after relaunch; 注釈 shows hashes in the gutter; a line's menu
  「このコミットの変更を表示」 shows that commit's change; 「このリビジョンとローカルを比較」
  shows revision ↔ local; the picker lists the file's commits.

## Rejected alternatives

- Annotate as a toggle on the 編集 view — the person chose the separate view.
- Re-implementing a diff algorithm for two revisions' full texts — git already
  produces the hunks the pane parses; pairing them for two columns is enough.
- A separate prefs file for the layout — one more file and lock for one enum;
  the store already holds theme and font preferences.
- Side-by-side in the Git screen's project diff — not in the approved screen.
