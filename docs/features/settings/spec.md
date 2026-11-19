# Spec: 設定

## Behaviour
1. The Settings page (`Page::Settings`) is reached from the toolbar gear, the palette row 「設定を開く」 and the `page.settings` chord (default ⌘,). Those entries keep the last chosen section.
2. A 200 px left column shows 「設定」 and five rows in this order: 「外観」, 「エージェント」, 「通知」, 「データと復旧」, 「キー割り当て」. The chosen section is drawn to the right under a page header whose one-line summary is fixed per section. The app starts on 「外観」.
3. The 「エージェント」 row carries a word: 「準備完了」 (success ink) when tmux is present and at least one agent CLI is present, otherwise 「要対応」.
4. Places that send a person here because a tool is missing (the project page setup link, the launch sheet's 「設定を開く」, the launch-progress modal) call `open_settings(SettingsSection::Agents)`, so they land on 「エージェント」.
5. 「外観」 has three pickers, each persisted at once: 「テーマ」 (ダーク default, ライト, ハイコントラスト（ダーク）) with colour swatches and a one-line detail; 「ターミナルとコードのフォント」 (Sarasa Mono J default, Sarasa Mono K, Sarasa Term J, Sarasa Fixed J) with the cell-width note; 「言語」 (English, 日本語, 한국어, each in its own script).
6. A theme change is applied on the next frame by `apply_appearance`: metrics and visuals are reset, cached terminal layouts are cleared so open terminals re-colour. A font change reloads the font definitions and clears the terminal render cache.
7. Choosing a language switches the process-wide active language immediately and saves it. On a fresh install the language comes from `LC_ALL`, `LC_MESSAGES`, `LANG`, then `defaults read -g AppleLanguages`, then `AppleLocale`; if none maps to en/ja/ko it is Japanese.
8. 「エージェント」 shows a 「ツール」 card: 「{n}/6 使用可能」 counting tmux, Codex, Claude Code, Antigravity, Git and GitHub CLI, plus 「ツールを再確認」 (re-detects PATH). Each of the three agent rows shows its availability, a 「導入手順」 button when missing (opens the vendor setup URL), 「アカウント {n}」 for agents that support accounts, and the hook cell (「フック 導入済み」, 「フック 未導入」, 「フック 失敗」, reason on hover) while hooks are enabled. A hook install that skipped a registered account folder that is gone shows a warning beside it.
9. tmux, Git and GitHub CLI have their own rows (tmux shows 「brew install tmux」 when missing). When tmux or every agent is missing, a hint says that an app started from Finder also searches Homebrew, `~/.local/bin` and `~/.cargo/bin`.
10. 「起動時の動作」 holds two checkboxes: 「エージェント CLI のフックで状態を検知する」 (binds the hook listener and installs or removes the managed hook entries) and 「初回起動時のフォルダ信頼プロンプトを自動承認する」 (default on, persisted, marked 「フォルダ信頼のみ」).
11. 「エージェントのアカウント」 lists registered accounts (agent, name, folder, a remove button 「このアカウントを削除」) or 「まだ登録されていません」, and a row to add one: an agent picker limited to agents that support accounts (Codex, Claude Code), a name field (hint 「例: 仕事」) and 「フォルダを選んで追加」, which opens a folder picker.
12. Registering an account requires a supporting agent, a non-empty trimmed name, an existing directory, a directory not already registered, and a name unused for that agent. Success saves the accounts file, clears the name field, shows 「アカウント「{name}」を追加しました。」 and re-runs the hook install so the new folder gets the managed hooks. Cancelling the picker shows 「アカウントの選択をキャンセルしました。」.
13. Removing an account deletes it and every session-to-account record pointing at it, and clears the launch form's selection if it was that account. The folder and its hook entries are left alone.
14. A launch under an account sets that agent's login-folder variable (`CLAUDE_CONFIG_DIR` or `CODEX_HOME`) and nothing else; a session reopens under the account it ran under; launching under the machine's own login records nothing.
15. 「通知」 has one checkbox 「ターンの完了・入力待ち・終了を通知する」 (default off). Turning it on asks macOS for notification permission.
16. 「データと復旧」 shows the data file path, 「データフォルダを表示」 (reveals the folder in Finder), the semantic-search model block, and 「tmux をスキャン」, which lists tmux sessions (safe name, existing working directory) that the store does not know. Each result shows its name and working directory and a 「セッションを引き継ぐ」 button, enabled only when a registered project contains that directory.
17. 「キー割り当て」 lists every remappable action with its current chord (or 「なし」) and a 「キー割り当てファイルを開く」 button (see the key bindings feature).

## States
- A setting that cannot be saved is reverted on screen (theme, font, language, notifications, folder-trust) and a notice 「データを保存できませんでした。直前の変更は適用されていません: …」 is shown.
- A write that committed but could not be confirmed on disk keeps the change in memory and raises a durability warning; the store write is retried automatically.
- No project registered: the recovery rows say 「引き継ぐ前に、このプロジェクトを登録してください。」 instead of the button.
- An already-registered tmux session: 「そのターミナルセッションは既に登録されています。」. No project contains the directory: 「先に、このターミナルの作業ディレクトリを含むプロジェクトを登録してください。」.
- Account refusals: 「{path} はフォルダではありません。」, 「このフォルダは「{name}」として既に登録されています。」, 「「{name}」という名前は既にあります。」, 「アカウントの名前を入力してください。」, 「選んだアカウントが見つかりません。」, 「アカウント「{name}」のフォルダが見つかりません。」.
- An accounts file over 64 KiB or unparseable is treated as empty (no error shown).
- English and Korean fall back to the Japanese message id for any string without a table row.

## Invariants
- Message ids are the Japanese strings; Japanese is the identity mapping and needs no table. Every table is sorted by message id, keeps the placeholder names of its message id, and no `tr` id carries a placeholder (use `tf!`).
- The language picker's own labels are never translated.
- `Language`, `AppTheme` and `TerminalFont` variants are persisted: they are added, never renamed or removed (old font names keep a serde alias).
- Under test the detected language is always Japanese, so a test cannot switch the language of tests running beside it.
- A new colour is a role on all three palettes; every ink must meet WCAG 2.1 AA against every surface in every theme.
- Accounts file ceiling: `AGENT_ACCOUNTS_FILE_MAX_BYTES` = 64 KiB; it is written atomically.
- An account is a name and a folder; Operon never reads the login contents.
- Pruning drops session-account records whose session or account no longer exists.
- Every child process (for example `defaults`) goes through `run_command_with_timeout` (5 s here).

## Code and tests
- `src/app/screens.rs::ui_settings`: the left list, section header and dispatch to the five section drawers.
- `src/app/screens.rs::ui_settings_appearance`, `ui_settings_agents`, `ui_settings_notifications`, `ui_settings_data`, `ui_settings_keys`, `ui_agent_accounts`: the section bodies.
- `src/app.rs::SettingsSection`: the five sections with `label()` and `summary()`; `OperonApp::open_settings` selects one.
- `src/app.rs::apply_appearance`: applies theme, metrics and font when they differ from what is applied.
- `src/app.rs::persist`, `apply_persist_result`: save and the revert/retry outcome.
- `src/app.rs::add_agent_account`, `forget_agent_account`, `choose_agent_account_folder`, `scan_orphaned_tmux_sessions`, `adopt_orphaned_tmux_session`: the actions behind the buttons.
- `src/agents/accounts.rs::register_account`, `account_variables`, `account_ready_to_launch`, `AgentAccounts::prune`, `load_agent_accounts`, `save_agent_accounts`: account rules and file.
- `src/theme.rs::AppTheme`, `TerminalFont`: theme and font choices, labels and details.
- `src/i18n.rs::Language`, `detected_language`, `tr`, `tf!`, `set_active_language`: language model and lookup.
- `src/i18n_tables.rs`: the per-language message tables.
- `src/config.rs::AGENT_ACCOUNTS_FILE_NAME`, `AGENT_ACCOUNTS_FILE_MAX_BYTES`: the accounts file name and ceiling.
- `src/store.rs::Store`: carries `theme`, `language`, `terminal_font`, `notifications_enabled`, `auto_approve_workspace_prompts`.
- Tests: `settings_has_five_sections` pins the section order and labels.
- Tests: `every_setting_is_in_one_section` pins that nothing is lost or duplicated across sections.
- Tests: `a_missing_tool_opens_the_agents_section` pins the redirect to 「エージェント」.
- Tests: `the_keys_section_lists_every_action` pins the key list.
- Tests: `settings_sidebar_label_is_a_panel_label` pins the 「設定」 label's style.
- Tests: `switching_themes_recolours_open_terminals` pins the terminal re-colour.
- Tests: `every_theme_stays_readable` pins WCAG AA for every ink on every surface.
- Tests: `only_the_light_theme_builds_egui_from_a_light_base` pins the light/dark base.
- Tests: `bundled_terminal_fonts_keep_cjk_at_exactly_two_ascii_cells` pins the 1:2 cell rule of all four fonts.
- Tests: `bundled_ui_font_has_english_japanese_and_korean_glyphs` pins glyph coverage.
- Tests: `every_translation_table_is_sorted_by_message_id`, `every_translation_keeps_the_placeholder_names_of_its_message_id`, `no_tr_call_carries_a_placeholder` pin the i18n tables.
- Tests: `an_account_is_one_variable_and_the_launch_carries_it`, `the_same_directory_cannot_be_registered_twice`, `a_session_reopens_under_the_account_that_had_it`, `an_account_record_for_a_session_that_is_gone_is_dropped`, `the_hooks_follow_every_account_that_is_registered`, `the_account_row_is_on_the_screen_without_unfolding_anything` pin the account rules.
- Tests: `recovery_scan_failure_is_reported_instead_of_as_zero_results`, `recovery_scan_distinguishes_no_server_from_a_real_tmux_failure` pin the scan outcomes.

## Decisions inherited
- Change 094: settings is five sections, not one long page; a missing tool lands on 「エージェント」, and verdicts are Japanese words rather than AVAILABLE/NOT FOUND.
- Change 031: a login is a name plus a folder handed to the CLI through its own variable; sessions already running keep their account and reopen under it.
- Change 203: the usage reader and the managed hooks follow every registered account folder, not only the machine's own.
- Change 002: user text goes through `tr`/`tf!` with the Japanese string as message id; `format!` is not used for user-facing text.
- Change 013: other-language tables are backfilled, and a missing row silently falls back to Japanese.
- Change 274: wording in all three languages should read as written by a person, not machine-made.
- Change 041: body-size text must clear 4.5:1 and focus marks 3:1 in all themes; the contrast suite enforces it.
- Change 136: one focus look for every button, drawn by the shared helpers in `src/ui/widgets.rs`.
- Change 295: the UI runs on egui 0.37, where `logic` and `ui` are separate; appearance is applied in `logic`.
