# Spec: 変更・コミット・PR

## Behaviour
1. The 「変更」 tab (`ProjectTab::Git`) draws `ui_git`: a row of three views, `GitView::Changes` 「変更ファイル」, `GitView::Diff` 「差分」, `GitView::History` 「コミット履歴」, and a refresh button. Refresh reloads the changes and the diff (Changes), the diff (Diff) or the history (History).
2. In 「変更ファイル」 the pane is two columns: left (width min(280, 40% of the pane, at least 200)) with 「変更されたファイル」, the file rows, the commit box and the push row; right with the diff pane. Each row has a checkbox (hover 「次のコミットに含めます」), a status badge, the path (click selects it and filters the diff, hover 「このファイルの差分だけを表示します」) and an edit button 「エディタで開く」. Selecting a file keeps the view on 「変更ファイル」.
3. When the changes arrive, `ensure_staged_selection` ticks tracked modifications and leaves untracked (`??`) files unticked; paths that stopped being changed are dropped from the selection.
4. The diff pane 「作業ツリーの差分」 shows the file count and `+added −removed`, 「すべて表示」 when one file is selected, and 「AIレビュー」. The diff is `git diff --no-ext-diff --unified=3 HEAD` (or `--cached` before the first commit) plus each untracked file as a no-index diff against `/dev/null`.
5. The diff is parsed once (`parse_unified_diff`) into files with kind (added, removed, modified, renamed), hunks and lines; a rewritten line pairs removed/added lines and marks only the changed words (`mark_rewritten_words`). Files fold individually. Rows are virtualised.
6. Each hunk header shows 「ステージ」 (`git apply --cached` of that hunk alone) and 「破棄」 (reverse-apply on the working tree). Discard has no confirmation. After either, changes and diff are re-read; after a stage, the file is ticked and remembered as hunk-staged; after a discard, open unmodified editor documents of that file reload. Notices: 「ハンクをステージしました。」, 「ハンクの変更を破棄しました。」, or 「ハンクを操作できませんでした: …」.
7. The commit box has a multiline 「コミットメッセージ」 per project and the button 「{N} ファイルをコミット」 (enabled when N>0 and no git write is running). Commit stages exactly the ticked files (`git_stage_for_commit`: index reset, then `git add` of ticked files; files staged by Operon's hunk buttons are kept as they are) and runs `git commit -m <message>` with the message as one argv element. An empty message says 「コミットメッセージを入力してください。」; no ticked file says 「コミットするファイルを選んでください。」.
8. 「AI に書かせる」 (drawn only when `commit_message_agent` finds a usable CLI, with the agent's short label beside it) stages the ticked files, summarises the staged files and patch, asks the agent for a message and writes it into the box; it never commits. While running the label is 「書いています…」.
9. A failed commit draws 「コミット失敗: {summary}」 with 「閉じる」 and 「AI で修正」 (`fix_commit_failure_with_ai`).
10. The push row states the branch against its upstream: 「このブランチは origin にまだありません。」, 「{upstream} と同じ状態です。」, 「… コミット先行しています。」, 「… コミット遅れています。」 or both. 「push」 is enabled only when behind is 0 and (no upstream, or ahead>0). A branch without upstream is pushed with `--set-upstream origin <branch>`.
11. 「PR を作成…」 (drawn only when `gh` is available, also in the session terminal menu as 「プルリクエストを作成…」) opens 「プルリクエストを作成」 with 「タイトル:」 (default `feat: changes from <branch>`), 「説明:」, 「AI でドラフト生成」 (fills the body with the first 500 characters of the diff) and 「下書き (Draft PR) として作成」, then runs `gh pr create --title --body [--draft]`.
12. The 「PR」 tab lists open PRs (`gh pr list --state open --limit 30`) with number, title, branch, 「下書き」 mark, review decision and a check summary, and 「ブラウザで開く」. Without `gh`: 「GitHub CLI を利用できません。次のコマンドでインストールしてください: brew install gh」.
13. 「AIレビュー」 runs the local agent read-only on the current diff, shows the result in 「AIコードレビュー評価」 with 「コピー」 and 「閉じる」, and is disabled as 「AIレビュー中…」 while running.
14. 「コミット履歴」 shows the recent log as a code block.
15. The tab shows the changed-file count as its badge, emphasised because uncommitted changes ask to be looked at.

## States
- Loading: 「Git の状態を更新しています…」, 「差分を読み込んでいます…」, 「履歴を読み込んでいます…」.
- Clean tree: 「作業ツリーに変更はありません。」 and 「変更はありません。」 in the diff.
- Git error: 「Git の状態を取得できません: …」, 「Git の差分を取得できません: …」, 「Git の履歴を取得できません: …」.
- A hand-made partial staging (outside Operon) refuses the commit, naming the files, and tells the person to commit in a terminal or `git reset` first.
- Remote ahead: push refused, 「リモートが進んでいます。先に pull してください。」.
- Git write already running: 「Git の操作が既に実行中です。」.
- Diff too large: truncated with 「… 差分を {n} KiB で切り詰めました」.
- No diff to review: 「レビュー対象の差分がありません。」; agent without a non-interactive mode: 「このエージェントには下書きの実行方法がありません。」.
- Binary or rename-only change: a line 「内容の変更なし」 where there are no hunks.

## Invariants
- No force push in any spelling (`no_source_file_can_force_a_push` scans the source).
- The index is only reset with `git reset` without `--hard`; a hand-made partial staging is never overwritten.
- The diff captured for display is bounded by `GIT_DIFF_MAX_BYTES` (180 000) across tracked and untracked content together.
- AI review times out at 45 s and returns at most 32 KiB; the commit-message agent times out at 120 s and 8 KiB; commit 60 s; push 120 s; `gh pr create` 60 s.
- Commit messages travel as an argv element, never through a shell.
- Word marks are bounded by `DIFF_WORD_TOKEN_LIMIT` (128 tokens per side) and computed during parsing, not per frame.
- Selection of ticked files is settled when the changes arrive, not while drawing.
- Every child process goes through the `exec` helpers with a timeout.

## Code and tests
- Symbols:
- `src/app/screens.rs::ui_git`: the three views, file list, split layout.
- `src/app/screens.rs::ui_git_diff_view`: diff header, AI review panel, pane, hunk actions.
- `src/app/screens.rs::ui_commit_box` and `ui_push_row`: commit box, failure card, upstream sentence, push and PR buttons.
- `src/app/screens.rs::ui_pull_requests` and `ui_create_pr_modal`: PR list and the creation dialog.
- `src/app.rs::record_staged_change`, `push_project`, `ensure_staged_selection`, `apply_diff_hunk_action`: the actions behind them.
- `src/git.rs::git_working_tree_diff`: the bounded working-tree diff.
- `src/git.rs::parse_porcelain_changed_files`, `parse_unified_diff`, `mark_rewritten_words`: status and diff parsing, word marks.
- `src/git.rs::git_stage_exactly`, `git_stage_for_commit`, `git_record_staged`, `git_push`, `git_upstream_state`: staging, commit, push.
- `src/git.rs::git_stage_hunk`, `git_discard_hunk`, `extract_single_hunk_patch`: per-hunk actions.
- `src/git.rs::git_create_pull_request`, `list_pull_requests`, `summarize_checks`: PR calls.
- `src/git.rs::commit_message_prompt`, `ai_diff_review_prompt`, `summarize_commit_failure`: agent prompts.
- `src/ui/diff.rs::diff_pane_with_actions`: the virtualised diff pane and hunk buttons.
- `src/config.rs::GIT_DIFF_MAX_BYTES`, `AI_REVIEW_TIMEOUT_SECONDS`, `COMMIT_MESSAGE_TIMEOUT_SECONDS`: the limits.
- Tests:
- `parses_porcelain_changes_including_untracked_and_renames`: status parsing.
- `git_changes_include_untracked_renamed_and_unborn_repository_content`: untracked and unborn cases.
- `parses_the_diff_shapes_git_writes_for_a_working_tree`: diff parser shapes.
- `a_rewritten_line_marks_only_the_words_that_changed`: word marks.
- `a_line_past_the_word_comparison_ceiling_still_marks_its_middle`: the token ceiling.
- `git_diff_enforces_one_capture_budget_for_large_tracked_and_untracked_content`: diff byte budget.
- `reads_how_far_a_branch_is_from_its_upstream`: ahead and behind.
- `stages_exactly_the_files_that_were_ticked`: commit stages only ticked files.
- `commits_a_message_that_would_break_a_shell`: message passed intact.
- `refuses_to_push_over_a_remote_that_moved`: push refused when behind.
- `pushes_a_new_branch_with_its_upstream_and_never_with_force`: first push.
- `no_source_file_can_force_a_push`: no force anywhere.
- `a_hunk_staged_by_hand_stops_the_commit_rather_than_being_overwritten`: partial staging refused.
- `the_staged_selection_is_settled_when_the_changes_arrive`: default ticks.
- `a_commit_message_stays_with_the_project_it_was_written_for`: per-project message.
- `summarizes_various_git_commit_failures`: failure summaries.
- `builds_fix_commit_failure_prompt_with_clean_context`: the 「AI で修正」 prompt.
- `ai_diff_review_prompt_and_clean_response`: review prompt and cleanup.
- `summarizes_pull_request_checks`: CI check summary.
- `the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane`: two-column layout.
- `clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab`: file selection.
- `ide_261_extract_and_apply_single_hunk_stage_and_discard`: hunk stage and discard.
- `ide_261_diff_pane_hunk_buttons_emit_stage_and_discard`: the buttons emit the actions.
- `diff_277_open_diff_stays_on_screen_while_it_is_read_again`: no blink on re-read.

## Decisions inherited
- Change 006: mark only words that changed in a rewritten pair, bounded and computed at parse time, so a rewrite reads as a rewrite.
- Change 020: reading a diff and handing corrections back are one screen; notes anchor to what a line said.
- Change 022: the loop up to the commit stays inside Operon.
- Change 047: AI review is read-only and advisory; review notes carry sent and resolved states.
- Change 205: the find bar and the commit box do not overlap.
- Change 222: the file list and commit box stay beside the live diff; selecting a file filters instead of switching view.
- Change 261: hunk-level stage and discard act only on the project checkout; discard is immediate.
- Change 277: an open diff stays on screen while it is re-read.
- Change 307: other comparisons (revision, one commit) offer neither hunk buttons nor notes.
- Change 110: landing, run scripts, checkpoints and IDE hand-offs round the loop out locally.
