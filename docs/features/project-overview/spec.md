# Spec: プロジェクトの概要とエージェント設定

## Behaviour
1. `ProjectTab::Overview` (「概要」) is the first tab and the one a project opens on (project list click, `try_add_project_path`). `ui_overview` draws it. Pinned by `the_project_page_has_six_tabs`.
2. On first draw for a project, `ui_overview` requests the git changes snapshot and the worktree list if their caches have no entry for it; it does not re-request each frame.
3. Three tiles in three columns: 「セッション」 (count of every session record of the project, any status), 「未コミットの変更」 (number of files in the changes snapshot) and 「worktree」 (number of worktrees, main included). A tile whose cache is missing or errored shows 「—」. Pinned by `the_overview_tab_summarises_the_project`.
4. 「新しいセッション」 calls `open_project_session_setup(project.id)`, opening the launch sheet with this project chosen.
5. 「このプロジェクトのセッション」 lists the project's sessions newest first (`created_at` descending), at most 8, each drawn with `home_session_row(.., "project")`. With none: 「セッションはまだありません。」.
6. `ProjectTab::AgentSettings` (「エージェント設定」) draws `ui_agent_settings`: the skills section, a gap, then the rules section. Pinned by `agent_settings_shows_skills_and_rules`, `agent_settings_sections_use_section_headings`.
7. 「スキル」 lists every file named exactly `SKILL.md` found under the project folder, absolute paths in monospace, with a refresh button (「更新」) and the line 「プロジェクトと各エージェントのフォルダで見つかった SKILL.md です。」. Search limit 80 results. Empty: 「SKILL.md は見つかりませんでした。」. While running: 「スキルを検索しています…」.
8. 「ルールと指示」 lists files named `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `CODEX.md` or `INSTRUCTIONS.md` under the project folder, limit 200. Empty: 「プロジェクトの指示ファイルは見つかりませんでした。」. While running: 「指示ファイルを検索しています…」.
9. Each listed file has an icon button 「「ファイル」タブで開く」 that sets `selected_file` to it and switches to `ProjectTab::Files`.
10. Both lists are fetched once per project (cached in `skill_cache` / `rule_cache`) and again on 「更新」.
11. When a scan was truncated or skipped unreadable directories, a one-line warning (`file_scan_warning`) is drawn above the list: 「スキャンの上限内で見つかったプロジェクトを表示しています。…」. Pinned by `file_scan_is_sorted_and_discloses_result_truncation`.

## States
- Caches not yet filled: tiles show 「—」; lists show the searching line.
- Git error or non-repository folder: 「未コミットの変更」 and 「worktree」 show 「—」.
- No sessions: the empty line in item 5.
- Scan truncated: warning line, results still shown.
- Documented mismatch: the skills caption mentions each agent's own folders, but `request_skills` scans only the project folder.

## Invariants
- The overview never starts work itself except the two read-only requests in item 2.
- The session list is capped at 8 rows; the tile counts all sessions.
- Skill and rule scans run in the background (`BackgroundKey::Skills` / `Rules`), never in the draw path, and are bounded by `FILE_SCAN_VISIT_LIMIT` and their result limits (80, 200).
- Tab counts exist only for 変更, worktree, PR and ナレッジ; 概要, ファイル and エージェント設定 have none.
- A CLI-restore control does not live on the overview; it is in history.

## Code and tests
- Symbols:
- `src/app/screens.rs::ui_overview`: the overview tiles, start button and session list.
- `src/app/screens.rs::ui_agent_settings`: skills then rules.
- `src/app/screens.rs::ui_skills`: skill list, refresh, open-in-Files button.
- `src/app/screens.rs::ui_rules`: instruction-file list.
- `src/app.rs::request_skills`: background scan for `SKILL.md`, limit 80.
- `src/app.rs::request_rules`: background scan for the five instruction file names, limit 200.
- `src/app.rs::ProjectTab`: tab order, labels, `project_tab_count`.
- `src/files.rs::find_named_files` and `find_named_files_any`: bounded file-name search.
- `src/files.rs::file_scan_warning`: the truncation warning text.
- Tests:
- `the_overview_tab_summarises_the_project`: tiles and session list for a seeded project.
- `cli_restore_is_in_history_not_in_the_overview`: restore is not offered on the overview.
- `agent_settings_shows_skills_and_rules`: both sections draw.
- `agent_settings_sections_use_section_headings`: section headings in the source.
- `the_project_page_has_six_tabs`: tab labels and order (historical name, seven labels).
- `tab_counts_do_no_work`: the count function reads caches only.
- `file_scan_is_sorted_and_discloses_result_truncation`: sorted results and truncation disclosure.

## Decisions inherited
- Change 092: the launch form left the Overview tab for its own sheet, so the overview stays a summary.
- Change 093: Skills and Rules became one 「エージェント設定」 tab, and tab counts were added, computed from caches.
- Change 245: one project is the hero in one place only.
- Change 237: the session column shows each project's session status, so the overview does not repeat it.
- Change 005: agent-authored files (skills, instructions) open in the editor and keep their format.
- Change 158: an empty project offers to start a session.
- Change 171: knowledge is its own tab with its own count, not part of the overview.
