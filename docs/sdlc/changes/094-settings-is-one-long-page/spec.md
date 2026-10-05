# Spec: settings in five sections

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. A new in-memory `SettingsSection` enum in `src/app.rs` — `Appearance`,
   `Agents`, `Notifications`, `Data`, `Keys` — with `all()` in that order,
   `label()` (「外観」, 「エージェント」, 「通知」, 「データと復旧」,
   「キー割り当て」) and `summary()` (the one line in `./screen.md`), all
   through `tr`. `OperonApp` gains `settings_section: SettingsSection`,
   `Appearance` at start.
2. `OperonApp::open_settings(&mut self, section)` sets `page = Page::Settings`
   and `settings_section = section`. The three places that send a person to
   settings because something is missing — Projects' 「セットアップ手順を開く」,
   the launch sheet's 「設定を開く」, the launch-progress modal's
   `open_settings` — call `open_settings(SettingsSection::Agents)`. The
   toolbar's ⚙, the palette's 「設定」 and ⌘, keep setting only the page, so
   they show the last section chosen.
3. `ui_settings` draws a left column 200 px wide — 「設定」, then one
   selectable row per section — and the chosen section to its right, headed by
   `page_header(ui, palette, label, Some(summary), |_| {})`. Each row is a
   `nav_tab(ui, palette, "", label, chosen)` — the toolbar's current-page look,
   `row_selected` fill and `accent_text` ink. The エージェント row ends with 「準備完了」 in `palette.success` when
   `tools.tmux && tools.available_agent_count() > 0`, else 「要対応」 in
   `palette.accent_soft`.
4. 外観: today's theme, font, and language rows and their persist-or-revert
   handling, moved unchanged into `ui_settings_appearance`.
5. エージェント (`ui_settings_agents`):
   a. A card headed 「ツール」, `tf!("{p0}/6 使用可能", …)` counting tmux, the
      three agents, Git, and GitHub CLI (success when 6, else `accent_soft`),
      and today's 「ツールを再確認」 button calling `request_tool_status`.
   b. Six rows through `tool_status_row`, which now draws 「使用可能」 /
      「未検出」 instead of AVAILABLE / NOT FOUND and takes an `extra` closure
      drawn before the verdict. For codex, claude, gemini the closure draws:
      while `hooks_enabled` and `hook_installs` holds that provider, its
      state as 「フック 導入済み」 (success), 「フック 未導入」 (`text_faint`)
      or 「フック 失敗」 (danger), the last two with their reason on hover
      (today's `hook_status_row` shows both reasons); no entry, or hooks off,
      draws no hook cell;
      for agents that `agent_supports_accounts`, 「アカウント {n}」 counting
      `accounts.accounts` of that agent; and when the agent is missing, a
      quiet 「手順」 button opening that CLI's setup URL through today's
      `request_system_action`. For tmux when missing, 「brew install tmux」 in
      monospace. Git and GitHub CLI draw nothing extra.
   c. A card 「起動時の動作」: today's hooks checkbox with its explanation and
      side effects (bind listener, `request_hook_apply`), and today's
      auto-approve checkbox with the label 「初回起動時のフォルダ信頼プロンプト
      を自動承認する」 and 「フォルダ信頼のみ」 beside it in `text_faint`.
   d. Today's `ui_agent_accounts`, unchanged, below the cards.
   e. The PATH note, when tmux or every agent is missing, as today.
   `hook_status_row` and today's separate セットアップ手順 frame are removed:
   their content now lives in (b).
6. 通知 (`ui_settings_notifications`): today's notification checkbox and its
   persist-or-revert and authorization request, unchanged.
7. データと復旧 (`ui_settings_data`): today's ローカルデータ label and text, the
   data path, 「データフォルダを表示」, and today's ターミナル復旧 row with its
   orphan cards, unchanged.
8. キー割り当て (`ui_settings_keys`): for each of `KEYMAP_ACTIONS`, a row with
   `tr(action.label)` and `self.keymap.label_for(action.id)`, or 「なし」 when
   that is empty; then 「キー割り当てファイルを開く」 calling today's
   `open_keymap_file`.
9. Message ids: new EN and KO rows for every new string (section labels and
   summaries, 「準備完了」, 「要対応」, 「ツール」, 「{p0}/6 使用可能」,
   「使用可能」, 「未検出」, the three hook words, 「アカウント {p0}」, 「手順」,
   「起動時の動作」, 「フォルダ信頼のみ」, the new auto-approve label, 「なし」).
   Rows left without a caller are removed: the old auto-approve label,
   「セッションの準備状況」, 「セットアップ手順」, and any other the move
   orphans, found with a search per removed call.
10. Nothing persisted changes. No new background request, child process, or
    file-system call in the draw path.

## Behaviour

See `./screen.md`.

## Design

- `src/app.rs`: `SettingsSection`; the field; `open_settings`; the modal's
  entry point.
- `src/app/screens.rs`: `ui_settings` as the frame and the five section
  functions; the two entry points.
- `src/ui/widgets.rs`: `tool_status_row` in Japanese with the `extra`
  closure; `hook_status_row` removed.
- `src/i18n_tables.rs`, `src/tests.rs`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: `row_selected`, `accent_text` (through `nav_tab`), `success`, `accent_soft`, `warning`, `danger`, `text_faint` exist. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Existing constants only. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | no | No new identifier. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | `settings_section` is in-memory; every persisted setting keeps its own persist-or-revert code, moved unchanged. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | no | The 手順 buttons reuse `request_system_action` with today's URLs; no new child. |
| Documentation — user-facing docs change in all three languages together | no | The three READMEs name Settings only as a place ("one switch in Settings", "switched live from Settings"), which stays true; none names its layout. Checked with a search. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | UI only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No new scan. |
| Draw path — no I/O per frame | yes | Every value drawn is in memory: `tools`, `hook_installs`, `accounts`, `keymap`. The one file write reachable from the page is `open_keymap_file`, on a click, as from the palette today. |

## Flagged concerns

- **A setting lost in the move** — each of today's controls must land in
  exactly one section. Resolved by `every_setting_is_in_one_section`, which
  draws each section and looks for each control's label.
- **Sent to settings and landing on 外観** — a person refused a launch for
  a missing tool, pressing 「設定を開く」, would otherwise land on whatever
  section they last chose and have to find the tools themselves. Resolved by
  req 2: every missing-tool entry point opens エージェント, held by
  `a_missing_tool_opens_the_agents_section`.
- **Hook state shown while hooks are off** — `hook_installs` keeps its last
  value; resolved by drawing it only while `hooks_enabled`, as today, with
  `the_hook_cell_follows_the_hooks_switch`.

## Acceptance

- `cargo test --locked` passes, including new tests:
  `settings_has_five_sections` (labels in order),
  `every_setting_is_in_one_section` (each section drawn headless; the theme,
  font, and language labels only in 外観; ツールを再確認, the hook and
  auto-approve checkboxes, and エージェントのアカウント only in エージェント; the
  notification checkbox only in 通知; データフォルダを表示 and tmux をスキャン only
  in データと復旧; キー割り当てファイルを開く only in キー割り当て),
  `a_missing_tool_opens_the_agents_section` (`open_settings(Agents)` from
  the Keys section on Home lands on Settings / Agents; and, read from source,
  `self.page = Page::Settings` is assigned in `src/app.rs` and
  `src/app/screens.rs` in exactly three places — the ⚙
  button, the palette's action, and `open_settings` — while
  `open_settings(SettingsSection::Agents)` appears exactly three times, so
  reverting any missing-tool entry point, or making ⚙ reset the section,
  fails),
  `the_agents_badge_says_whether_sessions_can_start` (tmux and one agent:
  準備完了; no tmux and one agent: 要対応; tmux and no agent: 要対応),
  `tool_rows_speak_japanese` (the agents section drawn: 使用可能 and 未検出
  present; the quoted literals `"AVAILABLE"`, `"NOT FOUND"`, `"INSTALLED"`
  absent from `src/ui/widgets.rs`),
  `the_hook_cell_follows_the_hooks_switch` (an `Installed` claude entry in
  `hook_installs`: with `hooks_enabled` 「フック 導入済み」 is drawn; with it
  off, no text beginning 「フック」),
  `the_keys_section_lists_every_action` (every `KEYMAP_ACTIONS` label drawn,
  and ⌘K's chord label).
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the installed app the settings page matches `./screen.md`.

## Rejected alternatives

- Tabs across the top instead of a left list — the mock and macOS System
  Settings both use a list, and five labels plus a badge fit a column better.
- Persisting the chosen section — nothing asks for it; the page is opened on
  purpose, and 「設定を開く」 chooses the section itself.
