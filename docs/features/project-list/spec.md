# Spec: プロジェクトの追加と一覧

## Behaviour
1. The Projects page (`Page::Projects`, no project selected) draws `ui_projects`. With `store.projects` empty it is the first-run view: heading 「まずはプロジェクトを開いてください」, the buttons 「プロジェクトフォルダを選択…」 and 「ワークスペースをスキャン…」, a drag hint, and a checklist 「最初のセッションを始める前に」 with two rows (tmux; at least one coding agent among Codex, Claude, Antigravity). When either is missing a 「セットアップ手順を開く」 button goes to Settings → Agents. Pinned by `starts_with_onboarding_until_a_project_exists`.
2. With projects, the page header 「プロジェクト」 carries 「フォルダを選択…」 (primary-chord ⌘O) and 「ワークスペースをスキャン…」. 「フォルダを選択…」, the first-run 「プロジェクトフォルダを選択…」, ⌘O and the palette's project action open the folder picker (behaviour 14), not the macOS dialog. Cards are listed newest-added first (`added_at` descending). Each card shows the name and the home-abbreviated path. Pinned by `folder_317_every_entry_opens_the_picker`.
3. Clicking a card calls `select_project(Some(id))` and sets `project_tab = ProjectTab::Overview`.
4. A card has two icon buttons (「Finder で表示」, 「プロジェクトを削除」) and the same two entries in its context menu.
5. Choosing delete sets `pending_project_removal`; that row is replaced by a danger-bordered card 「「{name}」を Operon から削除しますか？」 with 「記録だけを削除します。ファイル・ブランチ・worktree は残ります。」, 「Operon から削除」 and 「キャンセル」. The same confirmation appears on the opened project's page header via 「削除…」. Pinned by `the_project_list_offers_inline_deletion_and_confirmation`.
6. 「パスを手入力」 toggles a group with two single-line inputs: the project path (button 「追加」, Enter submits) and the workspace path (「Git リポジトリを含むワークスペース（スキャン深さ: 3）」, button 「スキャン」).
7. Adding a folder (`register_project_folder`): the path must be a directory, is canonicalised, and the name is its last component. A folder already registered (same canonical path) is not duplicated: that project is selected, the page becomes Projects and the notice is 「そのプロジェクトは追加済みのため、開きました。」. A new project is persisted; on success it is selected, the Overview tab opens, notice 「プロジェクトを追加しました。」. Pinned by `a_projects_stored_path_is_canonical`.
8. Dropping folders from Finder (`add_dropped_folders`) adds each one; non-folders are counted and ignored. With more than one folder, or any rejected/duplicate one, the page returns to the list and the notice states how many were added, how many were registered/unsaveable and how many were not folders. Dropping no folder says 「Finder からフォルダをドロップすると追加できます。」. Pinned by `accepts_only_finder_folders_from_a_drop`.
9. Workspace scan (`import_workspace`): the input must be a directory (else 「ワークスペースフォルダを選んでください。」); it runs as a background task keyed `WorkspaceScan`, depth 3, and finishes in `finish_workspace_import`. New repositories are named by their path relative to the scanned folder; repositories already registered are skipped. The notice reports the count imported, or that none was found within 3 levels, with a truncation wording when the visit limit was hit. Pinned by `workspace_scan_stops_at_its_global_visit_limit`.
10. `scan_git_repositories` returns sorted repository roots. A directory holding `.git` is a result and is not descended into; symlinks are not followed; `.git`, `node_modules`, `target`, `.next` are skipped. Pinned by `scans_only_git_repositories_in_workspace`.
11. `remove_project` deletes the project, its session records, native-session references and CLI handoffs from the store, then persists. It never touches files, branches or worktrees on disk. Selection and caches that referenced the project are cleared. Pinned by `test_remove_project_cleans_session_selection_and_path_cache`, `project_removal_waits_for_an_inflight_worktree_mutation`.
12. The launch sheet (「新しいセッション」) shares `register_project_folder` for its 「新しいプロジェクト…」/「フォルダを選択…」 entries and for dropped folders; it neither navigates nor opens the project page (`launch_projects`).
13. Project tabs, in order, with labels: 「概要」, 「変更」, 「worktree」, 「PR」, 「ファイル」, 「エージェント設定」, 「ナレッジ」 (`ProjectTab::all`). Counts on 変更, worktree, PR and ナレッジ come from caches only. Pinned by `the_project_page_has_six_tabs`, `tab_counts_come_from_the_caches`.
14. The folder picker (`ui_folder_picker`, sheet 「プロジェクトフォルダを開く」, change 317) holds a path field, a location row with 「上のフォルダへ」, and the child folders of the folder the text points into. Text ending in `/` lists that folder; otherwise the last component is a partial name and the parent's folders that start with it (ignoring case) are listed. Only `/…`, `~` and `~/…` are paths. Names starting with `.` show only when the partial starts with `.`; a symlink to a folder is listed. A row click makes that folder the text; Tab extends the partial to what the matches share and closes a sole match with `/`. Pinned by `folder_317_input_splits_into_a_folder_and_a_partial_name`, `folder_317_listing_holds_child_folders_only`, `folder_317_rows_descend_and_tab_completes`.
15. In the picker, 「開く」 (Enter) on an existing folder registers it as behaviour 7 does and closes the picker. On a path that does not exist the button is 「フォルダを作成…」: it asks 「「{path}」を作成しますか？」, and only 「作成して開く」 makes the folder and its missing parents (on a background thread, the sheet showing 「作成中…」) and registers it. 「Finder で選ぶ…」 opens the macOS dialog instead; 「キャンセル」, ✕ and Esc close with no notice. A folder dropped while the picker shows becomes its text. Pinned by `folder_317_opening_a_typed_folder_adds_the_project`, `folder_317_a_missing_folder_is_created_only_after_confirmation`, `folder_317_finder_is_still_reachable`, `folder_317_cancel_changes_nothing`, `folder_317_a_dropped_folder_fills_the_path`.
16. The picker opens beside the most recently added project, or at `~/` when there is none or macOS guards that folder; it does not open while a launch or restore progress modal shows. Pinned by `folder_317_opens_at_home_rather_than_in_a_guarded_folder`, `folder_317_opens_over_no_other_modal`.
17. The session page's column opens the picker with `FolderPickerTarget::SessionColumn` (its + menu, a right-click on its empty space or a project name, and its empty state's 「プロジェクトを追加…」). A folder opened there is registered as behaviour 7 does, but the page stays the session page, and the project is unfolded in the column and made the workspace's project (`apply_column_project_folder`, `docs/features/session-column/spec.md` behaviour 16). Pinned by `column_319_a_project_added_from_the_column_stays_on_the_session_page`, `column_319_a_registered_folder_is_unfolded_and_chosen`.

## States
- Empty: first-run view above; the manual-entry group is still offered.
- Not a folder: 「既存のフォルダを選んでください。」 and nothing is added.
- macOS dialog cancelled (workspace scan, or a project through 「Finder で選ぶ…」): 「ワークスペースの選択をキャンセルしました。」 / 「プロジェクトの選択をキャンセルしました。」. The in-app picker cancelled: nothing is said.
- Picker, text that is not a path: 「/ または ~ で始まるパスを入力してください。」, no listing asked, primary off. A file: 「これはフォルダではありません。」, primary off. Pinned by `folder_317_a_relative_path_or_a_file_opens_nothing`.
- Picker, listing not back: 「読み込み中…」. No child folder: 「フォルダはありません。」 or 「一致するフォルダはありません。」. Denied: 「このフォルダは読み取れません。」. Listed folder missing: 「この場所はまだありません。」. Pinned by `folder_317_listing_says_what_the_typed_path_is`.
- Picker, a folder with more than 5 000 entries: the rows read so far and 「項目が多いため、先頭の {count} 件だけを読みました。パスを最後まで入力すれば開けます。」. Pinned by `folder_317_listing_stops_at_its_entry_limit`.
- Picker, a listing for text no longer in the field is dropped and the current text asked. Pinned by `folder_317_a_stale_listing_is_not_shown`.
- Folder cannot be created: 「「{path}」を作成できませんでした: {error}」, the picker stays open, nothing is registered.
- Picker, a path the disk will not answer for (under a file, or under a folder that may not be looked into): 「このパスは確認できません。」, primary off, creation not offered. Pinned by `folder_317_a_path_that_cannot_be_checked_is_not_created`.
- Picker, Japanese input: while a conversion is open Enter, Esc and Tab are the IME's, and the Enter that confirms it opens nothing. Pinned by `folder_317_an_ime_enter_does_not_open_the_folder`.
- Picker, Tab on `~` completes under the home folder (`~/name/`), and does nothing on a listing cut at the ceiling. Pinned by `folder_317_tab_on_a_bare_tilde_keeps_a_path`.
- Scan already running: 「ワークスペースのスキャンは実行中です。」.
- Scan found nothing: 「3 階層以内に新しい Git リポジトリは見つかりませんでした。」; when also truncated, a variant asking to narrow the folder.
- Scan truncated with results: the import notice adds that some folders may have been omitted.
- Removal refused while a session of the project is Starting, Active, Queued or Unknown: 「実行中のセッションを停止または完了してください。」.
- Removal refused while a worktree mutation of the project runs: 「worktree 操作の完了を待ってください。」.
- Persist failure on add, import or remove: the in-memory store is restored to its previous value.

## Invariants
- Adding or removing a project never writes inside a project folder. A folder that does not exist is created, empty, only after the person confirms in the picker (change 317).
- The picker reads the disk only on background threads (`list_folder`, at most `FOLDER_PICKER_ENTRY_LIMIT` = 5 000 entries of one folder, never recursive) and lists a macOS-guarded folder only when the person typed or clicked into it.
- Two projects never share a canonical path.
- The workspace scan depth is 3 and its visit budget is `FILE_SCAN_VISIT_LIMIT` (100 000 directory entries); it never follows symlinks.
- Folder pickers are not opened from inside a draw pass on the launch sheet; the choice is applied the next frame.
- Removing a project waits for in-flight worktree work and for live sessions; it is never forced.
- Tab counts do no work in the frame (they read caches).

## Seen in the running app
`python3 scripts/verify-app.py run project-list` starts the built application apart from the person's data, follows these steps, and keeps a picture and the window's tree for each (change 314). One `verb words` per line; the words are what the screen shows.

    start project
    click プロジェクト
    see verify-demo
    shot projects
    click フォルダを選択…
    see プロジェクトフォルダを開く
    see Finder で選ぶ…
    shot folder-picker

## Code and tests
- Symbols:
- `src/app/screens.rs::ui_projects`: first-run view, list, cards, delete confirmation, manual entry, then the opened project's header and tab row.
- `src/app.rs::try_add_project_path`: add or open an existing project and set the notice.
- `src/app.rs::add_dropped_folders`: drop handling and the summary notice.
- `src/app.rs::import_workspace` and `finish_workspace_import`: scan task and its import and notice.
- `src/app.rs::remove_project`: guarded removal of a project's records.
- `src/app.rs::ProjectTab`: tab order, labels, `project_tab_count`.
- `src/app/launch_projects.rs::register_project_folder`: shared folder registration (`RegisteredProject`).
- `src/app/folder_picker.rs::ui_folder_picker`: the picker sheet; `list_folder`, `split_folder_input`, `folder_completion`, `folder_picker_start` are its pure parts; `submit_folder_picker` and `confirm_folder_creation` its two presses.
- `src/config.rs::FOLDER_PICKER_ENTRY_LIMIT`: listing ceiling.
- `src/git.rs::scan_git_repositories_with_limit`: the bounded repository walk.
- `src/models.rs::Project`: persisted project (id, path, name, added_at).
- `src/config.rs::FILE_SCAN_VISIT_LIMIT`: scan visit budget.
- Tests:
- `starts_with_onboarding_until_a_project_exists`: first-run state until a project exists.
- `reports_supported_agent_readiness_from_cached_tool_state`: the agent readiness the checklist reads.
- `accepts_only_finder_folders_from_a_drop`: drops keep folders and count the rest.
- `scans_only_git_repositories_in_workspace`: scan results and depth.
- `workspace_scan_stops_at_its_global_visit_limit`: the visit ceiling and truncation flag.
- `the_project_list_offers_inline_deletion_and_confirmation`: delete button, context menu, in-place confirmation.
- `project_removal_waits_for_an_inflight_worktree_mutation`: removal refused during worktree work.
- `test_remove_project_cleans_session_selection_and_path_cache`: selection and caches cleared on removal.
- `a_projects_stored_path_is_canonical`: stored path is canonical.
- `the_project_page_has_six_tabs`: pins the seven tab labels in order (name is historical).
- `launch_294_registered_folder_is_switched_to`: an already registered folder is selected, not duplicated, from the sheet.
- `launch_294_not_a_folder_leaves_the_sheet_as_it_was`: a non-folder changes nothing.

## Decisions inherited
- Change 044: delete lives on the card and in its context menu, confirmed in place, because a confirmation that leaves the list loses where the person was.
- Change 093: seven tabs, counts only where a count is information (changes, worktrees, PRs, knowledge), computed from caches.
- Change 158: a project with no sessions offers to start one instead of an empty page.
- Change 166: an empty project says the start once, not twice.
- Change 294: a folder that is not yet a project can be registered from the launch sheet through the same registration as this page.
- Change 317: one in-app picker for typing, walking and creating a project folder, opened from this page and the launch sheet; one level listed rather than a tree, creation confirmed and never `git init`; the macOS dialog kept behind 「Finder で選ぶ…」.
- Change 311: the session column opens projects folded; this page stays the full list.
- Change 034: documents belong to a root, not to a project, so a project's folder is not the only place a file may live.
- Change 092: the launch form moved out of the Overview tab into its own sheet.
