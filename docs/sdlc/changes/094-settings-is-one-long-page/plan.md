# Plan: settings in five sections

- **Spec**: `./spec.md`
- **Approved**: 2026-09-27
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `SettingsSection` (`all`, `label`, `summary`); `settings_section` field and init; `open_settings`; the launch-progress modal calls it |
| `src/app/screens.rs` | `ui_settings` becomes the nav frame; `ui_settings_appearance`, `ui_settings_agents`, `ui_settings_notifications`, `ui_settings_data`, `ui_settings_keys` hold today's code, moved; Projects' and the sheet's entry points call `open_settings(Agents)` |
| `src/ui/widgets.rs` | `tool_status_row` in Japanese with an `extra` closure; `hook_status_row` removed |
| `src/i18n_tables.rs` | EN and KO rows for new ids; orphaned rows removed |
| `src/tests.rs` | seven new tests; any test drawing today's settings re-pointed |

## Order of work

1. `SettingsSection`, the field, `open_settings`, the three entry points.
   Compiles.
2. `ui_settings` split into the frame and five sections, moving code without
   editing it; then the エージェント section's cards and rows.
3. `tool_status_row` in Japanese; `hook_status_row` removed.
4. i18n rows; orphan search per removed call.
5. Tests; gates; one mutation per new test.
6. rust-reviewer and subprocess-safety-reviewer on the staged diff; commit;
   package; install; look at the screen.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A setting lost or doubled in the move | a control unreachable, or in two sections | `every_setting_is_in_one_section` |
| A persisted setting's revert-on-failure lost | a setting shown changed that was not saved | code moved unchanged; reviewer compares the moved blocks |
| 「設定を開く」 lands on 外観 | a person sent to fix a tool sees fonts | `a_missing_tool_opens_the_agents_section` |
| English verdicts survive | AVAILABLE on screen | `tool_rows_speak_japanese` |
| Readiness badge wrong | 準備完了 without tmux | `the_agents_badge_says_whether_sessions_can_start` |
| I/O in the draw path | stutter on the settings page | spec req 10; reviewer. The one file write reachable from the page is `open_keymap_file`, on a click, as the palette already does |
| Hook state shown while hooks are off | a stale 導入済み | `the_hook_cell_follows_the_hooks_switch` |
| The nav uses the wrong colour role | text-selection blue for a chosen section | req 3 draws rows with `nav_tab`, the toolbar's own widget; reviewer |
| A key binding missing from the list | a chord a person cannot find | `the_keys_section_lists_every_action` |
| An orphaned message id | a table row with no caller | a search per removed call in step 4, recorded in Departures |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. N passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- New tests watched failing with their rule inverted.
- Installed app: the settings page as in `./screen.md`.

## Departures from the plan

- `hook_status_row` became `hook_cell`, a cell in the tool row. Its
  installed-state detail 「この CLI が自分で状態を報告します」 is gone with it;
  the word 「フック 導入済み」 says the same in the row.
- The old page kept showing hook rows for the frame in which hooks were being
  switched off (`previous_hooks_enabled || hooks_enabled`); the cell follows
  `hooks_enabled` alone, as the spec says.
- Orphan search per removed call (plan step 4): nine rows removed — the three
  setup-button labels, 「Homebrew で tmux をインストール:」, the no-agent
  sentence, 「セットアップ手順」, 「セッションの準備状況」, the old auto-approve
  label, and the installed-hook detail. Seven rows already had no caller on
  `main` before this change and are left for their own change.
- The ツール card's header keeps 「ツールを再確認」's full label and PATH hover
  (screen.md said 「再確認」): the shorter word would orphan a row and lose the
  hover that says what is re-checked.
- A hook that is not installed or failed says why only on hover now; the old
  page printed the reason under the row. A failed install still raises a
  notice, a not-installed one (for example, not on `PATH`) does not.
- The hook word does not line up across the three agent rows: アカウント is
  drawn only for Codex and Claude, and 手順 only for a missing agent, so
  Antigravity's hook word sits one cell further right than the mock shows.
