# Spec: a launch sheet over any page, a real 概要, restore in 履歴

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. `OperonApp` gains `launch_sheet_open: bool` (in-memory, false at start).
   `open_project_session_setup(project_id)` selects the project and sets it
   true; it no longer changes `page` or `project_tab`. Every entry that
   reaches it today — toolbar 「新しいセッション」, ⌘N, the palette row, the
   sidebar's per-project "+", Home's プロジェクトから始める, `open_new_session`
   — therefore opens the sheet over the current page. The no-project path of
   `open_new_session` is unchanged (notice, Projects page, no sheet).
   The worktree tab's 「この場所で新規セッション」 keeps setting
   `session_path_input` / `session_path_project` as today and then calls
   `open_project_session_setup(project.id)` instead of switching to the 概要
   tab, so it opens the sheet with the worktree's branch chip (「ワークツリー: {branch}」) as its 作業場所.
1a. Opening the sheet takes keyboard focus away from a terminal pane: on the
   first frame after `launch_sheet_open` becomes true (a
   `launch_sheet_needs_focus` flag set by `open_project_session_setup`), the
   goal field (`TextEdit` with `id_salt("launch-goal")`) requests focus. The
   pane forwards keys only while it has focus, so nothing typed in the sheet
   reaches an agent, and Esc reaches the sheet.
2. `ui_launch_sheet(&mut self, ctx)` draws the sheet when `launch_sheet_open`
   and a project is selected, from `update` right before `ui_restore_modal`
   and `ui_launch_modal` (so a launch-progress modal draws over it). A scrim
   `Area` (`Order::Middle`, `palette.scrim`, senses clicks and does nothing
   with them) covers the screen; the sheet `Area` (`Order::Foreground`) is a
   frame like the launch modal's, 560 px wide, centred, at most 80 % of the
   screen height.
3. Sheet header: 「新しいセッション」, a project `ComboBox` listing every project
   (choosing one calls `select_project(Some(id))`), and a ✕ `small_icon_button`
   (hover 「閉じる」) that sets `launch_sheet_open = false`.
4. Sheet body, in a vertical `ScrollArea`, is `ui_launch_form(ui, project)`:
   a. 「エージェント」, then one horizontal row of three choices (codex,
      claude, gemini), each a selectable frame with `agent_icon`, the name from
      `agent_choice_copy`, and `ICON_AVAILABLE` (success) or `ICON_UNAVAILABLE`
      (`accent_soft`) with hover 「この Mac で使用可能」 / 「未準備（設定で確認）」;
      the selected one has a 1.5 px stroke in `palette.agent_accent(agent)`;
      clicking calls `select_agent`. Then a quiet 「別のコマンドを使う」 that
      calls `select_agent("custom")`; with custom selected, today's
      「起動コマンド（必須）」 row.
   b. The account row, unchanged and unfolded, for agents with accounts.
   c. 「依頼（任意）」 and today's multiline goal field, 3 rows.
   d. 「作業場所」: today's folder chip, 「プロジェクト直下に戻す」 (clears
      `session_path_input` and `session_path_project`) only when a worktree is
      chosen, and today's Finder icon.
   e. The folded section, `id_salt("launch-detail")`, titled
      `launch_detail_title()` whose prefix becomes 「詳しい設定（任意）」:
      session name, model, `ui_launch_options`, and today's
      「ほかのセッションの後に開始」 combo.
5. Sheet footer, outside the scroll area, in order: `self.notice` when set,
   in `accent_soft` (a refusal from `launch_session` is drawn in the central
   panel under the scrim, so the sheet repeats it where it can be read); today's notices (data may
   leave the Mac; Claude opens a terminal; not a Git repository; worktree
   operation running), the danger warning and 「理解したうえで起動する」 when
   `launch_needs_acknowledgement`, and the blocking reason with 「設定を開く」 as
   today. Then one row: quiet 「ターミナルだけ開く」 (enabled as today) at the
   left, and at the right the launch button — agent-coloured fill, text
   `tf!("{p0} を起動  ⌘↩", p0 = agent name)`, enabled exactly when today's
   `launch_environment_ready` is true.
6. While the sheet is open and the palette is not: Esc closes it; ⌘↩
   (`Modifiers::COMMAND` + `Key::Enter`, consumed) calls `launch_session`
   when the button would be enabled, and does nothing otherwise. The
   readiness test is one function, `launch_ready(&self, project) -> bool`,
   used by both the button and the key.
7. When `launch_session` or `launch_empty_session` adds a session record
   (the store's session count grows), the sheet closes. A refusal (notice,
   no record) leaves it open.
8. The 概要 tab (`ui_overview`) becomes a summary: three `metric`-style cards
   in `ui.columns(3)` — 「セッション」 (count of this project's sessions),
   「未コミットの変更」 (`files.len()` of `git_changes_cache`), 「worktree」
   (`len()` of `worktree_cache`); a missing cache entry is requested with
   `request_git_changes` / `request_worktrees` and shown as 「—」, as is an
   error. Then a secondary `ICON_ADD` 「新しいセッション」 calling
   `open_project_session_setup(project.id)`. Then 「このプロジェクトのセッション」
   and this project's sessions, newest first, at most 8, as
   `home_session_row(ui, session, "project")`; 「セッションはまだありません。」
   when none.
9. The CLI section that followed the launch form moves unchanged into
   `ui_cli_sessions(&mut self, ui, project)`, drawn in 履歴 under
   `ui_sessions`, below a heading 「CLI セッション」 and a project `ComboBox`
   bound to a new in-memory `history_project: Option<Uuid>` (default: the
   selected project, else the first). No project: not drawn.
10. Message ids left without a caller are removed from every table; new ids
    get EN and KO rows: 「新しいセッション」 (exists), 「エージェント」,
    「依頼（任意）」, 「作業場所」, 「プロジェクト直下に戻す」, 「詳しい設定（任意）」,
    「{p0} を起動  ⌘↩」, 「未コミットの変更」, 「このプロジェクトのセッション」,
    「CLI セッション」 (exists), 「閉じる」 (exists).
11. Nothing persisted changes. `build_agent_launch_command`, the
    `is_safe_agent_*` gates, `launch_needs_acknowledgement`, and
    `launch_session`'s own refusals are unchanged.

## Behaviour

See `./screen.md`.

## Design

- `src/app.rs`: the two fields; `open_project_session_setup`; `launch_ready`;
  the sheet call in `update`; the key handling; `launch_detail_title` prefix.
- `src/app/screens.rs`: `ui_launch_sheet`, `ui_launch_form` (the body of
  today's `ui_overview` launch half, reshaped), `ui_agent_choice` compacted,
  the new `ui_overview`, `ui_cli_sessions` (moved), the 履歴 section.
- `src/i18n_tables.rs`, `src/tests.rs`.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | yes | No new role: `scrim`, `raised`, `border`, `agent_accent`, `success`, `accent_soft` exist. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | Existing constants only. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | The button and ⌘↩ share `launch_ready`; a test drives both. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | no | Both new fields are in-memory. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | No new launch path: the sheet calls the same `launch_session` / `launch_empty_session`; the danger acknowledgement and every refusal are reused, not re-implemented. |
| Documentation — user-facing docs change in all three languages together | yes | README launch paragraph in all three. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | no | UI only. |
| Budgets — any new scan or output path states its byte and item ceiling | no | The 概要 counts reuse the tabs' existing background requests. |
| Draw path — no I/O per frame | yes | Counts read caches; requests are background tasks deduplicated by key. The folder chip's `git_branch` call is today's, moved, not added. |

## Flagged concerns

- **Tests that draw the old 概要** — `overview_labels` and the four tests on it
  (account row, danger acknowledgement, launch options, folded summary) move
  to drawing the sheet; each keeps its assertion. The source-reading half of
  `the_launch_screen_offers_every_mode_and_switch_the_agent_has` re-anchors on
  the new section boundary. Change 091's Home project-start test now asserts
  the sheet opens and the page stays Home.
- **Account row vs the mock** — the mock folds アカウント into 詳しい設定;
  an earlier change moved it out because a person with two logins could not
  see which one a run would use, and a test holds it there. Resolved: it
  stays visible under the agent row, recorded in screen.md as the sheet's one
  departure from the mock.
- **A modal over the terminal** — the pane forwards keys while it has
  keyboard focus, and a scrim takes the pointer, not the keyboard. Resolved by
  req 1a: opening the sheet moves focus to its goal field, with a test.
- **A refusal the sheet cannot show** — `launch_session` can refuse for a
  reason `launch_ready` does not cover (an account not ready). Resolved by req
  5: the footer repeats the notice.

## Acceptance

- `cargo test --locked` passes, including the moved tests and new ones:
  `new_session_opens_the_sheet_over_the_current_page` (from Home and from
  Sessions, `open_new_session` and the palette's 「新しいセッション」 row each
  leave `page` unchanged and `launch_sheet_open` true; with no project, no
  sheet and today's notice),
  `the_sheet_closes_on_escape_and_on_its_close_button` (drawn headless; Esc
  closes; reopened, a click on ✕ closes),
  `cmd_enter_and_the_button_share_one_readiness` (tools all present, a
  dangerous switch on and not acknowledged — the one refusal only the button's
  enabled state enforces, `launch_session` does not: `launch_ready` false, the
  button drawn disabled, ⌘↩ adds no session and leaves the sheet open; after
  the acknowledgement, `launch_ready` true),
  `opening_the_sheet_takes_focus_from_the_terminal` (with another widget
  focused, `open_project_session_setup` then one drawn frame leaves the
  focused id equal to the goal field's),
  `the_worktree_tabs_new_session_opens_the_sheet_there` (the button's effect
  applied through the same method: `launch_sheet_open`, `session_path_input`
  the worktree path, `project_tab` unchanged),
  `the_launch_button_sits_at_the_sheets_bottom_right` (its rect's right edge
  within 40 px of the sheet frame's, and below the goal field),
  `the_overview_tab_summarises_the_project` (caches seeded with 3 changed
  files and 2 worktrees: 「3」 and 「2」 drawn, no 「エージェント」 label drawn),
  `cli_restore_is_in_history_not_in_the_overview` (the library draws
  「ローカル CLI セッションを検出」; 概要 does not).
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` are clean.
- In the installed app the sheet, 概要, and 履歴 match `./screen.md`.

## Rejected alternatives

- A separate window (`egui::Window`) — draggable, can fall behind the page,
  and the launch-progress modal is already an `Area` pair; one pattern.
- Keeping the form on the tab and only opening that tab from everywhere —
  the problem the owner named is leaving the current screen.
