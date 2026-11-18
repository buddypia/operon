# Spec: 新しいセッション

## Behaviour
1. 「新しいセッション」 (⌘N, or the buttons that open it) draws the launch sheet over the current page without navigating. Esc or 「閉じる」 closes it without launching (an open popup takes the Esc first); ⌘↩ presses the launch button when it is enabled. A stray click on the scrim does nothing.
2. The sheet header carries a project picker. Projects are listed with the latest-started first, then never-used ones by newest added, and a filter 「プロジェクトを絞り込み」 appears from 8 projects (`LAUNCH_PROJECT_FILTER_MIN`). The list ends with 「新しいプロジェクト…」, which opens a folder picker. A folder dropped on the sheet becomes its project. A folder that is already a project is switched to, not duplicated; a path that is not a folder changes nothing.
3. With no project at all the sheet shows 「まだプロジェクトがありません。」 and 「フォルダを選択…」 instead of closing.
4. 「エージェント」 offers Codex, Claude Code and Antigravity (persisted kind `"gemini"` launches `agy`) with a mark for those available on this Mac, and 「別のコマンドを使う」, which asks for 「起動コマンド（必須）」.
5. 「アカウント」 appears only for Claude Code and Codex with a registered account; the default is 「このマシン」.
6. 「依頼（任意）」 takes the first message. 「テンプレートを挿入」 offers four built-in requests and the files in the project's `.operon/prompts/`. 「GitHub Issue から取得…」 (only when `gh` is available) lists open issues; choosing one fills the request and the session name. `@` opens candidates `@diff`, `@knowledge:<title>` and project files (up to 8); they are inserted as text and not expanded by Operon.
7. 「作業場所」 is 「プロジェクト直下」 or 「ワークツリー: {branch}」 (a worktree made in the project's worktree tab), with 「プロジェクト直下に戻す」 and 「作業フォルダを Finder で表示」.
8. 「詳しい設定（任意）」 is folded by default and holds 「セッション名（任意）」, 「モデル（任意）」, the CLI's own mode (labelled with that CLI's word), 「推論の深さ」, the CLI's switches, 「追加の引数（任意）」 (not for the custom command), 「再開する会話 ID（任意）」, and 「ほかのセッションの後に開始」. Mode, effort and switches are chosen only from the CLI's own catalogue; `resolve_agent_flags` enforces switches that exclude each other, and setting a mode that invalidates a ticked switch unticks it.
9. 「追加の引数（任意）」 accepts whitespace-separated switches that start with `-`, use only the characters allowed for agent commands, are written `--name=value`, and are at most 1,024 characters. A switch the sheet owns (`--model`, `--effort`, `--permission-mode`, `--sandbox`, `--mode`, `--session-id`, `--resume`, `--conversation`, `--new-project`, or any ticked-switch flag) is refused. They are appended after the ticked options.
10. A conversation ID resumes that conversation with every merged option (`claude <options> --resume <id>`, `codex resume <options> <id>`, `agy <options> --conversation <id>`). It is refused with a message for an unsafe ID, a non-empty request, or a ticked switch the resume cannot honour. The ID is empty at app start, cleared after a launch and when the agent changes.
11. 「複数エージェントで並走（別 worktree で比較）」 is drawn only when the project is a Git repository. With two or more of Claude Code, Codex, Antigravity ticked (only available ones by default), a launch gives each its own new worktree and branch named `<base>-<agent>`; fewer than two ticked means no arena.
12. A dangerous switch or mode shows 「危険」 and 「{label}（危険）」; the launch button stays disabled until 「理解したうえで起動する」 is ticked. The same gate applies to a custom command and to typed arguments carrying such a flag, and editing the arguments withdraws the tick.
13. 「この設定を毎回使う」 pins the current agent, model, mode, effort, switches, command, arguments and account. Each opening of the sheet starts from the pin. Editing the form does not change the pin; 「今の設定で固定し直す」 does. A pinned account that no longer exists opens as 「このマシン」; unchanged pin keeps its acknowledgement.
14. Without a pin, the last-used model, mode, effort, switches, custom command and arguments per agent, and the last selected agent, are written to `recent-agent-settings.json` and loaded on the next opening and on app start.
15. The footer shows 「ターミナルだけ開く」 (plain `tmux` shell, no agent) and 「{agent} を起動  ⌘↩」. The launch button is disabled, with a reason shown, for 「起動コマンドを入力してください」, 「tmux が必要です」, 「選んだ AI が見つかりません」 (with 「設定を開く」), 「worktree 操作の完了待ち」, or 「上の確認にチェックを入れてください」.
16. 「ほかのセッションの後に開始」 queues the session until every selected predecessor is done; a waiting session offers 「今すぐ開始」. A failed or cancelled predecessor leaves the waiting session blocked and never starts it.
17. A launch shows a progress modal naming 「作業フォルダ」, 「プロジェクト」, 「エージェント」, 「起動コマンド」 and the request; on success it opens the terminal workspace with the new session selected, on failure it shows the reason with 「再試行」, 「設定を開く」 and 「閉じる」. A successful launch clears the request.
18. The terminal workspace's tab-strip 「+」 opens a menu with the plain terminal and Codex / Claude Code / Antigravity (an unavailable CLI is disabled with 「未準備（設定で確認）」). Choosing an agent starts it in that pane's project and folder with no request, using the pin when it is for the same CLI, else the last-used options. It writes neither the sheet's draft nor `recent-agent-settings.json`. If those options need the danger acknowledgement and the pin does not carry it, nothing starts and the sheet opens on that CLI and folder.

## States
- tmux missing: launch and 「ターミナルだけ開く」 are refused with the reason; the + menu still lists 「ターミナル」 and reports the missing tmux when chosen.
- Agent missing: the button is disabled and a shortcut to Settings is shown.
- Worktree mutation running: launches are fenced until it ends.
- Launch failure: the progress modal's error state; no session record is kept for a refused resume ID or refused arguments.
- A settings file with unknown or unsafe values is sanitised on load (strings capped at 1,024 characters, custom command at 4,096) and old files without newer fields load with defaults.
- A pinned launch that no longer holds (missing agent, option, account) drops what is invalid.

## Invariants
- Options come only from each CLI's own catalogue or a guarded typed switch; Operon never invents a flag.
- A launch that includes a permission-skipping flag cannot start without the acknowledgement, whichever way the flag got there (ticked, typed, custom command, remembered, pinned, resume).
- Opening, changing and closing the sheet never navigates away from the current page, and a stray click never discards a request in progress.
- The ID of a conversation to resume belongs to one CLI and is never remembered.
- The + menu never rewrites the sheet's draft or the remembered settings.
- Launch commands pass `is_safe_agent_command` / `is_safe_agent_option`.
- Settings are saved atomically beside the store as `recent-agent-settings.json`.

## Code and tests
- Symbols:
  - `src/app/screens.rs::ui_launch_sheet`: the sheet, scrim, Esc and ⌘↩ handling.
  - `src/app/screens.rs::ui_launch_form`, `ui_launch_options`, `ui_launch_footer`: form, option pickers, footer.
  - `src/app.rs::ui_launch_modal`: the launch progress modal.
  - `src/app/launch_projects.rs::launch_project_choices`: project order and filter; `register_project_folder` adds a folder.
  - `src/app/launch_menu.rs::ui_workspace_launch_menu`: the tab strip's + menu; `workspace_agent_launch` decides start or confirm.
  - `src/agents.rs::build_agent_launch_command`: the command from agent, options and request.
  - `src/agents.rs::resolve_agent_flags`, `agent_flag_options`: switch catalogue and exclusivity.
  - `src/agents.rs::launch_needs_acknowledgement`, `arguments_need_acknowledgement`: the danger gate.
  - `src/agents.rs::typed_launch_arguments`: validation of typed switches.
  - `src/agents.rs::launch_command_resuming`: resume with merged options.
  - `src/agents/settings.rs::RecentAgentSettings`, `PinnedLaunch`, `sanitize_agent_launch_settings`: remembered and pinned settings.
  - `src/util.rs::parallel_arena_agents`, `arena_worktree_branch_name`: the arena's agents and branches.
  - `src/config.rs::PROMPT_TEMPLATES_RELATIVE_DIR`, `LAUNCH_EXTRA_ARGS_MAX_CHARS`: template directory and argument ceiling.
- Tests:
  - `the_launch_screen_offers_every_mode_and_switch_the_agent_has`: catalogue on screen.
  - `a_dangerous_switch_cannot_be_launched_without_the_acknowledgement`: danger gate.
  - `a_dangerous_switch_typed_into_a_custom_command_still_asks`: custom command gate.
  - `typed_launch_arguments_refuse_bare_words_unsafe_characters_and_owned_options`: typed switch rules.
  - `typed_launch_arguments_that_skip_permissions_need_the_acknowledgement`: typed danger.
  - `a_resume_id_reopens_that_conversation_with_every_merged_option`: resume command.
  - `a_resume_id_refuses_malformed_ids_a_request_and_a_new_project`: resume refusals.
  - `recent_agent_settings_save_and_load_round_trip`: remembered settings.
  - `recent_agent_settings_sanitizes_invalid_options`: sanitising.
  - `pinned_launch_fills_the_sheet_each_time_it_opens`: pin behaviour.
  - `pinned_launch_drops_what_no_longer_holds`: stale pin.
  - `ide_261_parallel_arena_agents_and_branch_names`: arena agents and branch names.
  - `queues_until_every_dependency_is_done`: ordering.
  - `failure_or_cancellation_never_releases_a_dependency`: blocked queue.
  - `the_launch_modal_names_what_it_is_doing`: progress modal.
  - `session_273_agent_launch_uses_pane_and_remembered_options`: + menu launch.
  - `session_273_dangerous_options_open_the_sheet`: + menu danger path.
  - `session_273_missing_cli_is_disabled`: unavailable CLI.
  - `launch_294_project_list_ends_with_new_project`: project list.
  - `launch_294_folder_dropped_on_the_sheet_becomes_its_project`: dropped folders.
  - `launch_294_recent_projects_come_first`: project order.
  - `test_prompt_templates_defaults_and_custom`: templates.
  - `test_github_issue_parsing`: issue import.

## Decisions inherited
- Change 042: a launch shows a progress modal and reports failure in plain Japanese with retry.
- Change 072: each agent remembers its own last launch settings across sessions and restarts.
- Change 092: the launch form is a sheet over the current page, not a tab, so starting a session never navigates.
- Change 138: 「この設定を毎回使う」 pins a whole combination; the form's edits do not alter it, so a launch is repeatable.
- Change 156: a person's own switches are accepted only as guarded, non-owned switches, and a conversation ID resumes with every merged option; refusals write no record.
- Change 219: the launching CLI's terminal must not be hidden by the progress modal.
- Change 261: arena launches, `@` mentions and template/issue sources feed the request without Operon expanding them.
- Change 273: the + menu starts a terminal or agent in the pane's place without touching the sheet's draft, and sends dangerous options through the sheet.
- Change 294: adding a project happens inside the sheet, and the sheet never closes just because no project exists.
