# Spec: プロジェクトの追加と一覧

## Behaviour
1. The Projects page (`Page::Projects`, no project selected) draws `ui_projects`. With `store.projects` empty it is the first-run view: heading 「まずはプロジェクトを開いてください」, the buttons 「プロジェクトフォルダを選択…」 and 「ワークスペースをスキャン…」, a drag hint, and a checklist 「最初のセッションを始める前に」 with two rows (tmux; at least one coding agent among Codex, Claude, Antigravity). When either is missing a 「セットアップ手順を開く」 button goes to Settings → Agents.
2. With projects, the page header 「プロジェクト」 carries 「フォルダを選択…」 (primary-chord ⌘O) and 「ワークスペースをスキャン…」. Cards are listed newest-added first (`added_at` descending). Each card shows the name and the home-abbreviated path.
3. Clicking a card calls `select_project(Some(id))` and sets `project_tab = ProjectTab::Overview`.
4. A card has two icon buttons (「Finder で表示」, 「プロジェクトを削除」) and the same two entries in its context menu.
5. Choosing delete sets `pending_project_removal`; that row is replaced by a danger-bordered card 「「{name}」を Operon から削除しますか？」 with 「記録だけを削除します。ファイル・ブランチ・worktree は残ります。」, 「Operon から削除」 and 「キャンセル」. The same confirmation appears on the opened project's page header via 「削除…」.
6. 「パスを手入力」 toggles a group with two single-line inputs: the project path (button 「追加」, Enter submits) and the workspace path (「Git リポジトリを含むワークスペース（スキャン深さ: 3）」, button 「スキャン」).
7. Adding a folder (`register_project_folder`): the path must be a directory, is canonicalised, and the name is its last component. A folder already registered (same canonical path) is not duplicated: that project is selected, the page becomes Projects and the notice is 「そのプロジェクトは追加済みのため、開きました。」. A new project is persisted; on success it is selected, the Overview tab opens, notice 「プロジェクトを追加しました。」.
8. Dropping folders from Finder (`add_dropped_folders`) adds each one; non-folders are counted and ignored. With more than one folder, or any rejected/duplicate one, the page returns to the list and the notice states how many were added, how many were registered/unsaveable and how many were not folders. Dropping no folder says 「Finder からフォルダをドロップすると追加できます。」.
9. Workspace scan (`import_workspace`): the input must be a directory (else 「ワークスペースフォルダを選んでください。」); it runs as a background task keyed `WorkspaceScan`, depth 3, and finishes in `finish_workspace_import`. New repositories are named by their path relative to the scanned folder; repositories already registered are skipped. The notice reports the count imported, or that none was found within 3 levels, with a truncation wording when the visit limit was hit.
10. `scan_git_repositories` returns sorted repository roots. A directory holding `.git` is a result and is not descended into; symlinks are not followed; `.git`, `node_modules`, `target`, `.next` are skipped.
11. `remove_project` deletes the project, its session records, native-session references and CLI handoffs from the store, then persists. It never touches files, branches or worktrees on disk. Selection and caches that referenced the project are cleared.
12. The launch sheet (「新しいセッション」) shares `register_project_folder` for its 「新しいプロジェクト…」/「フォルダを選択…」 entries and for dropped folders; it neither navigates nor opens the project page (`launch_projects`).
13. Project tabs, in order, with labels: 「概要」, 「変更」, 「worktree」, 「PR」, 「ファイル」, 「エージェント設定」, 「ナレッジ」 (`ProjectTab::all`). Counts on 変更, worktree, PR and ナレッジ come from caches only.

## States
- Empty: first-run view above; the manual-entry group is still offered.
- Not a folder: 「既存のフォルダを選んでください。」 and nothing is added.
- Picker cancelled: 「プロジェクトの選択をキャンセルしました。」 / 「ワークスペースの選択をキャンセルしました。」.
- Scan already running: 「ワークスペースのスキャンは実行中です。」.
- Scan found nothing: 「3 階層以内に新しい Git リポジトリは見つかりませんでした。」; when also truncated, a variant asking to narrow the folder.
- Scan truncated with results: the import notice adds that some folders may have been omitted.
- Removal refused while a session of the project is Starting, Active, Queued or Unknown: 「実行中のセッションを停止または完了してください。」.
- Removal refused while a worktree mutation of the project runs: 「worktree 操作の完了を待ってください。」.
- Persist failure on add, import or remove: the in-memory store is restored to its previous value.

## Invariants
- Adding or removing a project never writes to the project folder.
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

## Code and tests
- Symbols:
- `src/app/screens.rs::ui_projects`: first-run view, list, cards, delete confirmation, manual entry, then the opened project's header and tab row.
- `src/app.rs::try_add_project_path`: add or open an existing project and set the notice.
- `src/app.rs::add_dropped_folders`: drop handling and the summary notice.
- `src/app.rs::import_workspace` and `finish_workspace_import`: scan task and its import and notice.
- `src/app.rs::remove_project`: guarded removal of a project's records.
- `src/app.rs::ProjectTab`: tab order, labels, `project_tab_count`.
- `src/app/launch_projects.rs::register_project_folder`: shared folder registration (`RegisteredProject`).
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
- Change 311: the session column opens projects folded; this page stays the full list.
- Change 034: documents belong to a root, not to a project, so a project's folder is not the only place a file may live.
- Change 092: the launch form moved out of the Overview tab into its own sheet.
