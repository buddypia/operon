# Spec: worktree とセットアップ

## Behaviour
1. The 「worktree」 tab (`ProjectTab::Worktrees`, `ui_worktrees`) has a branch-name input (hint 「新しいブランチ名（例: feature/my-change）」), the button 「worktree を作成」, a refresh button, and the list from `git worktree list --porcelain`. Rows are labelled 「メインの worktree」 or 「追加の worktree」 with the branch, or 「detached HEAD（ブランチ未設定）」. Each row offers 「この場所で新規セッション」, Finder, 「エディタで開く」 and, for non-main rows, 「main にマージ…」 and 「削除…」. Pinned by `lists_main_and_linked_git_worktrees`.
2. Creating (`create_worktree_from_mainline`) resolves the mainline in order: `refs/remotes/origin/HEAD` target, `refs/remotes/origin/main`, `refs/remotes/origin/master`, `refs/heads/main`, `refs/heads/master`, each verified to resolve to a commit. The base is passed fully qualified with `--no-track`. The destination is `<project parent>/<project name>-worktrees/<branch>` with characters other than ASCII alphanumerics, `-`, `_`, `.` replaced by `-`. Pinned by `detects_the_base_ref_from_origin_head`, `refuses_to_create_a_worktree_with_no_detectable_base`, `qualifies_the_base_ref_so_a_tag_cannot_win`.
3. A name is taken when the branch exists, the destination exists or a registered worktree holds the path; the next free `name-2`, `name-3`… is used. After creation `branch.<name>.base` is written to git config and `push.autoSetupRemote` is set when unset. The result names the created branch and its base. Pinned by `numbers_the_second_worktree_of_the_same_name`, `skips_a_name_whose_branch_exists_without_a_worktree`, `records_the_creation_base_in_git_config`, `leaves_an_existing_push_autosetupremote_alone`.
4. Removal (`remove_worktree`) refuses while any session reserving that worktree is starting, active or queued; otherwise runs `git worktree remove <path>` (30 s), after an in-place confirmation 「この worktree を削除しますか？」 (button 「worktree を削除」). Git's refusal on uncommitted changes is shown (「Git が削除を拒否しました: …」). The main worktree is never removed. The branch is kept.
5. Landing (`git_land_worktree`, dialog 「worktree を main にマージ」 with 「マージ元:」, 「マージ先:」, a checkbox 「マージ成功後に worktree とブランチを自動で削除する」 on by default, and 「マージを実行する」) runs `git merge --no-ff --no-edit -- <branch>` in the main checkout. On success the notice states branch, mainline and commit count, and, if chosen, `git worktree remove` and `git branch -d` follow. Pinned by `refuses_to_land_over_a_merge_already_in_progress`, `landing_and_the_projects_other_git_writes_wait_for_each_other`, `landing_waits_for_a_session_in_the_main_checkout`.
6. The same landing is offered in the session terminal menu as 「worktree を main にマージ…」.
7. A failed merge of any kind (non-zero exit, timeout, spawn error, a conflict) takes one path (`undo_failed_landing`): the merge is aborted and the mainline restored to its prior commit. A conflict names up to 10 files plus 「ほか {n} 件」. Pinned by `a_conflicting_landing_is_aborted_and_names_the_files`, `a_landing_stopped_by_a_hook_is_undone`, `a_landing_that_cannot_be_undone_says_so`, `a_landing_that_left_the_tree_changed_is_not_called_unchanged`.
8. After creating a worktree, if `.operon/setup.sh` exists in it (≤ 64 KiB) a card 「このリポジトリのセットアップを実行しますか？」 shows the first 8 lines and 「… 全 {n} 行」 with the checkbox 「内容が変わるまで、今後は確認しない」 and buttons 「実行しない」/「実行する」. 「実行する」 re-reads the file; if the digest equals the shown one, the bytes are saved under the approved-scripts directory beside the store, named by digest, and run in a dedicated session through `bash -c '. "$approved"'` with `$0` set to the repository path. A changed file asks again; a vanished or oversized one does not run (「{path} を読み直せませんでした。実行していません。」). Pinned by `reads_a_setup_script_and_its_digest`, `treats_a_missing_setup_script_as_nothing_to_do`, `refuses_a_setup_script_past_its_byte_ceiling`, `rechecks_the_setup_script_at_the_moment_of_running`, `a_setup_script_runs_end_to_end_in_a_path_with_a_space`.
9. A remembered approval is the pair (project, digest) kept in the setup-trust sidecar; any edit changes the digest and asks again. An unreadable trust file means nothing is approved. Pinned by `approves_a_setup_script_per_project_and_per_content`, `an_edited_setup_script_is_not_the_one_that_was_approved`.
10. If the repository commits `.operon/run.sh`, the session terminal menu offers 「実行スクリプト (.operon/run.sh) を起動…」, which opens 「実行スクリプトの確認」 (「.operon/run.sh を実行しようとしています。」, 「内容を確認して実行を承認してください:」, 「キャンセル」, 「承認して実行」). Approval re-reads the file, and a changed digest refuses (「{path} の内容が変わったため実行しませんでした。もう一度確認してください。」). The approved copy runs in a terminal with agent kind `script`; 「実行スクリプトを停止」 stops it; it is not started twice while running. Every run asks again. Pinned by `test_run_script_discovery_and_preview`, `a_running_run_script_is_not_started_twice`, `the_run_script_and_the_templates_are_read_off_the_frame`.
11. Checkpoints: the terminal menu section 「チェックポイント」 has 「手動チェックポイントを作成」, the newest 5 entries each with a 「差分」 copy action, and 「チェックポイントへロールバック」. A checkpoint is a commit of the worktree built with a temporary index, stored under `refs/operon/checkpoints/<session id>/<turn>`; the user's index is untouched. With the status hooks on, one is captured at each request. Rollback runs `git checkout <full commit id> -- .` and leaves later-created files; the diff copy is bounded to 1 MB. Only a full commit id is accepted. Pinned by `test_checkpoints_create_list_restore_diff`, `a_checkpoint_is_named_by_a_whole_object_id`, `a_checkpoint_diff_is_copied_from_a_background_thread`.
12. Listening ports (`ui_session_ports`): the session header shows a badge counting listeners in this session's folder (「{n} ポート稼働中」, or the project's count 「プロジェクトで {n} ポート稼働中」), and a menu with sections 「この作業フォルダ」 and 「プロジェクト内のほかの場所」, each port with 「ブラウザで開く」 and a URL copy. 「停止する」 opens 「ポートを停止」 (「:{port} {process} を停止しますか？」, PID, 「フォルダ」, 「終了シグナル (SIGTERM) を送ります。…」). Pinned by `ports_262_menu_sections_name_folders_and_offer_stop`.
13. A port is credited to the deepest of a session worktree, a project worktree or the project root that contains the listener's working directory (from `lsof`). Pinned by `attributes_a_port_to_the_deepest_worktree_that_contains_it`.
14. Stopping re-checks that the same process still listens on that port, sends SIGTERM, then confirms it is gone; the notice is one of 「{process} を停止しました（:{port}）。」, 「…まだ :{port} で動いています。」, 「…止まったか確かめられませんでした。」, 「:{port} はもう … 何もしませんでした。」, 「ポートの持ち主を確かめられなかったため、停止しませんでした。」, 「このプロセスは停止できません。」, or 「… を停止できませんでした: …」. Pinned by `ports_262_stop_sequence_checks_before_signalling`, `ports_262_stop_never_reports_stopped_without_seeing_it_gone`, `ports_262_stop_refuses_operon_and_init`.

## States
- Loading and error: 「worktree を読み込んでいます…」, 「worktree を取得できません: …」; creation while another worktree action runs says 「worktree の操作が既に実行中です。」 and the create button shows 「worktree 操作の完了待ち」.
- Empty name: 「worktree のブランチ名を入力してください。」.
- No mainline: creation refused with 「プロジェクトのメインラインブランチを検出できませんでした。」.
- Landing refused: uncommitted changes in the worktree or the main checkout; the main checkout on another branch; another merge in progress; same source and target; nothing to merge; a session starting, active, queued or unknown in the worktree or the main checkout (「…停止・完了・キャンセルしてからマージしてください。」).
- Landing that cannot be undone, or that left the tree changed, says so rather than reporting unchanged.
- Setup or run script missing: nothing is drawn. Over 64 KiB: reported as too large, not previewed.
- No `lsof`: no ports are shown and no error.
- Port stop refused for Operon itself and for init.

## Invariants
- A new worktree always branches from the mainline, never from whatever is checked out, and never tracks it.
- No setup or run script runs without the person having been shown those exact bytes; what runs is the approved copy, not the file in the repository.
- Landing, commit, push and draft on one project exclude one another (one `GitMutation`/`WorktreeMutation` at a time).
- Project removal waits for in-flight worktree work.
- Checkpoints never change the user's index; rollback never deletes files.
- Port scan interval 30 s, timeout 8 s, at most 200 entries, 512 KiB of output; a port is signalled only after its holder is re-confirmed.
- The path reaches a shell quoted (a path with a space works).
- No force or remote operation is part of landing; it is local.

## Code and tests
- Symbols:
- `src/app/screens.rs::ui_worktrees`: tab body, create row, list, delete confirmation.
- `src/app/screens.rs::ui_land_worktree_modal` and `ui_run_script_modal`: the landing dialog and the run-script confirmation.
- `src/app/screens.rs::ui_session_ports`: ports badge, menu, stop confirmation.
- `src/app.rs::create_worktree`, `remove_worktree`, `land_worktree`, `capture_checkpoint`, `restore_checkpoint`, `ui_setup_prompt`: the actions and the setup card.
- `src/git.rs::detect_worktree_base`, `create_worktree_from_mainline`, `worktree_name_candidate`, `configure_created_worktree`, `worktree_destination`, `list_worktrees`: creation.
- `src/git.rs::git_land_worktree`, `undo_failed_landing`: landing and its undo.
- `src/git.rs::git_create_checkpoint`, `git_list_checkpoints`, `git_restore_checkpoint`, `git_diff_checkpoint`: checkpoints.
- `src/git/setup.rs::read_setup_script`, `decide_setup_run`, `keep_approved_script`, `setup_command`, `SetupTrust`: setup digest, approval and run.
- `src/git/setup.rs::read_run_script`, `decide_run_script_run`: the run-script counterparts.
- `src/git/ports.rs::scan_listening_ports`, `attribute_ports`, `stop_listening_port`, `port_stop_notice`: ports.
- `src/config.rs::SETUP_SCRIPT_RELATIVE_PATH`, `RUN_SCRIPT_RELATIVE_PATH`, `SETUP_SCRIPT_MAX_BYTES`, `CHECKPOINTS_REF_PREFIX`, `PORT_SCAN_MAX_ENTRIES`: names and limits.
- Tests:
- `lists_main_and_linked_git_worktrees`: list parsing, main flag.
- `replaces_unsafe_branch_characters_in_worktree_path`: destination sanitising.
- `detects_the_base_ref_from_origin_head`: mainline order.
- `refuses_to_create_a_worktree_with_no_detectable_base`: no mainline refused.
- `numbers_the_second_worktree_of_the_same_name`: numbering.
- `skips_a_name_whose_branch_exists_without_a_worktree`: a taken branch name.
- `qualifies_the_base_ref_so_a_tag_cannot_win`: fully qualified base.
- `records_the_creation_base_in_git_config`: base recorded.
- `leaves_an_existing_push_autosetupremote_alone`: config not overridden.
- `refuses_to_land_over_a_merge_already_in_progress`: landing refused.
- `a_conflicting_landing_is_aborted_and_names_the_files`: conflict undo.
- `a_landing_stopped_by_a_hook_is_undone`: any failure undone.
- `a_landing_that_cannot_be_undone_says_so`: honest failure.
- `a_landing_that_left_the_tree_changed_is_not_called_unchanged`: honest result.
- `landing_and_the_projects_other_git_writes_wait_for_each_other`: exclusion.
- `landing_waits_for_a_session_in_the_main_checkout`: sessions block landing.
- `project_removal_waits_for_an_inflight_worktree_mutation`: removal waits.
- `reads_a_setup_script_and_its_digest`: digest and preview.
- `treats_a_missing_setup_script_as_nothing_to_do`: no script.
- `refuses_a_setup_script_past_its_byte_ceiling`: 64 KiB ceiling.
- `approves_a_setup_script_per_project_and_per_content`: approval scope.
- `an_edited_setup_script_is_not_the_one_that_was_approved`: edits re-ask.
- `rechecks_the_setup_script_at_the_moment_of_running`: fresh read at run time.
- `the_setup_command_survives_a_worktree_path_with_a_space`: quoting.
- `a_setup_script_runs_end_to_end_in_a_path_with_a_space`: end to end run.
- `test_run_script_discovery_and_preview`: run script discovery.
- `a_running_run_script_is_not_started_twice`: single run.
- `the_run_script_and_the_templates_are_read_off_the_frame`: reads off the draw path.
- `test_checkpoints_create_list_restore_diff`: checkpoint lifecycle.
- `a_checkpoint_is_named_by_a_whole_object_id`: only full ids.
- `a_checkpoint_diff_is_copied_from_a_background_thread`: diff copy off-frame.
- `attributes_a_port_to_the_deepest_worktree_that_contains_it`: attribution.
- `ports_262_stop_sequence_checks_before_signalling`: stop checks first.
- `ports_262_stop_never_reports_stopped_without_seeing_it_gone`: stop confirmation.
- `ports_262_stop_refuses_operon_and_init`: protected processes.
- `ports_262_menu_sections_name_folders_and_offer_stop`: menu sections.

## Decisions inherited
- Change 018: a worktree branches from the project's mainline, fully qualified and untracked, because branching from whatever was checked out gave agents the wrong starting point.
- Change 019: a repository may declare setup, but is never run until the person has seen those exact contents; the approval is of the digest, and the approved bytes are what run.
- Change 023: dev servers are attributed to the worktree that owns them by working directory.
- Change 110: landing, run scripts and checkpoints keep the agent loop local.
- Change 115: landing refuses before it merges when the main checkout has work in it, and any failed landing is undone, so it cannot undo the main checkout's work.
- Change 262: ports are narrowed to the session's folder, told apart from the rest of the project, and stoppable only after the holder is re-confirmed.
- Change 209: the setup script is read once under its ceiling and run from an approved copy, so a swapped file cannot run.
