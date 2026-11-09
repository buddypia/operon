# Spec: The tab-strip + chooses what to start, and the tabs are compact

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. Pressing a pane's + opens a menu directly below it and starts nothing by
   itself. The menu has four rows, in this order, each a 28px control with its
   mark and its name: ターミナル (Operon's terminal mark), Claude Code, Codex CLI,
   Antigravity (each vendor's mark). Pressing the + again, pressing Escape, or
   clicking outside closes it. Test: `session_273_plus_opens_launch_menu`.
2. Choosing ターミナル starts a plain terminal in that pane exactly as the + did
   before: the pane's project and the current tab's folder, the launch sheet's
   draft untouched, the missing-tmux notice when tmux is absent. Tests:
   `session_254_terminal_context`, `session_254_terminal_action` (both now press
   the + and then ターミナル).
3. Choosing an agent creates a session for that CLI in the pane's project and
   the current tab's folder, named like any new session (`セッションN`), with no
   request text, with that CLI's remembered launch options — the pinned
   combination and its login when 「この設定を毎回使う」 is pinned to that same
   CLI, otherwise the options the launch sheet last used for it and no chosen
   login (the CLI's own default, as an unpinned sheet starts) — and starts it
   with the same launch, notices and failure reporting as the launch sheet. The
   new tab opens in that pane. The launch sheet's draft (agent, options, request,
   name, folder) and `recent-agent-settings.json` are untouched. Tests: `session_273_agent_launch_uses_pane_and_remembered_options`,
   `session_273_agent_choice_reaches_the_launch`.
4. When those remembered options need the danger acknowledgement and the pin
   does not carry it for that CLI, choosing the agent starts nothing and opens
   the launch sheet on that CLI with the pane's project and folder, where the
   existing warning and acknowledgement apply. The sheet opens the way every
   other entry opens it (`open_project_session_setup`: a different project's
   draft is cleared and the form starts from the pin); if the form is then on
   another CLI it is switched to this one with this CLI's remembered options
   and no login, without recording the form's options and without writing
   `recent-agent-settings.json`. Test:
   `session_273_dangerous_options_open_the_sheet` (also compares the remembered
   options and the file before and after).
5. A CLI this Mac does not have is drawn disabled in the menu, cannot be
   chosen, and says 「未準備（設定で確認）」 on hover. The ターミナル row is never
   disabled for this reason (a missing tmux is reported when it is chosen, as
   before). Test: `session_273_missing_cli_is_disabled`.
6. The empty workspace's + opens the same menu and stays disabled without a
   project; ターミナル there keeps the missing-tmux notice. Tests:
   `session_254_empty_workspace`, `session_260_empty_plus_sits_where_the_first_tab_appears`.
7. Session tabs are compact: a tab is at most `CONTROL_HEIGHT` (28px) tall, a
   tab whose title fits the 40px minimum is at most 116px wide (138px before:
   16 margin + 18 + 16 + 40 + 24 + 3×8; after: 12 + 14 + 14 + 40 + 24 + 3×4),
   the state mark and the CLI mark are drawn at 14px, and the + still follows
   the last tab. Tests: `session_273_tabs_are_compact`,
   `session_260_plus_follows_last_tab`, `session_254_compact_tabs`.

## Behaviour

Strings a person reads:

- The + names itself 「ターミナルや AI を起動」 on hover and to assistive
  technology (was 「空のターミナルを開く」, which no longer says what it does).
- Menu rows: 「ターミナル」 (`agent_short_label("terminal")`, the word the left
  column uses), then 「Claude Code」, 「Codex CLI」, 「Antigravity」 — the names the
  launch sheet already uses (`agent_choice_copy`).
- A disabled row's hover: 「未準備（設定で確認）」 (existing string).

States:

- **Happy, agent**: the menu closes, a new tab with the starting state mark
  appears in that pane and is selected, and the existing notice
  「セッションを起動しています…」 shows; the tab moves to running, or to failed
  with the existing launch diagnostic.
- **No tmux**: any choice gives 「tmux が必要です。「設定」を開いてください。」 and
  creates nothing.
- **worktree operation running**: 「worktree 操作の完了を待ってください。」, nothing
  created.
- **Login not ready** (pinned login gone or signed out): the existing
  `account_ready_to_launch` reason, nothing created.
- **Remembered options no longer valid** (a typed switch the gate now refuses):
  the reason from `build_agent_launch_command`, nothing created.
- **Danger acknowledgement needed**: the launch sheet opens on that CLI over
  the workspace (Requirement 4); nothing is created until the person launches
  from it.
- **Empty workspace without a project**: the + is disabled, as before.
- **Too many tabs**: unchanged — tabs scroll under a clip ending before the +,
  and the menu opens below the + at the pane's right edge, kept on screen by
  egui's area constraint.
- **Long names**: still truncate at 132px with the full name on hover.

Tab geometry: frame margin 6px across and 2px down (was 8 and 4), 4px between
items inside a tab and between tabs (was 8), state mark 14px (was 18), CLI mark
14px (was 16), title 12pt unchanged, close button 24px unchanged, selected
underline unchanged. The strip row is `CONTROL_HEIGHT` tall (was 32px).

## Design

Placement: `EXTEND`. The + and the strip are `ui_tab_workspace` in
`src/app/screens.rs` (changes 254, 260, 267); the pane-scoped terminal launch is
`launch_workspace_terminal` in `src/app.rs`; the agent launch is
`launch_session` in `src/app.rs`. Route `modify`.

- `src/app.rs`:
  - `workspace_pane_place(group) -> Option<(Project, String)>`: the project and
    folder text `launch_workspace_terminal` already derives from the pane's
    current tab; that function now calls it.
  - `workspace_agent_launch(group, agent) -> UiResult<PaneAgentLaunch>`: the
    whole decision, with no side effect — project and resolved folder, the
    settings (pin for this CLI, else `recent_agent_settings.agents[agent]`, else
    empty), the account (the pin's when the pin is for this CLI and the login
    still exists, else none), the built command (`build_agent_launch_command`, the
    existing gate) and whether the danger acknowledgement is missing
    (`launch_needs_acknowledgement` against the pin's `acknowledged`). Refusals
    (no project, worktree operation, no tmux, CLI missing, login, bad options,
    folder) are its `Err` strings, the same strings `launch_session` uses.
  - `launch_workspace_agent(group, agent)`: on `Err` sets `notice`; when the
    acknowledgement is missing calls `open_project_session_setup`, then
    — when the form is on another CLI — sets `selected_agent`, calls
    `apply_agent_launch_settings(agent)`, clears `acknowledged_launch`, the
    conversation ID and `agent_account_input` (never `select_agent`, which
    records the form into the remembered options and writes the file), and
    sets the sheet's folder to the pane's; otherwise
    focuses the pane, pushes the `Session` (managed native id via
    `command_with_managed_native_session_id`, tracking token via
    `goal_with_native_tracking_token` on an empty request, `origin: None`, no
    dependency), persists (restoring the store on failure, as `launch_session`
    does), `remember_session_account`, sets `launch_progress`, and calls
    `start_session`. It never calls `persist_recent_agent_settings`, which
    records the sheet's draft.
- `src/app/screens.rs`: `WORKSPACE_LAUNCH_CHOICES`, the menu's four agent
  values in order. `ui_workspace_launch_menu(ui, add, group)` toggles an
  `egui::popup_below_widget` (`PopupCloseBehavior::CloseOnClick`, id per pane)
  holding the four rows — the terminal row labelled by `agent_short_label` and
  always enabled, each agent row by `agent_choice_copy` and enabled by
  `tools.agent_available` — and dispatches the choice after the popup closes —
  `launch_workspace_terminal` or `launch_workspace_agent`. Both the pane + and
  the empty-workspace + call it. Tab margins, spacings and the strip height
  change as in Behaviour.
- `src/ui/widgets.rs`: `launch_choice_button(ui, texture, label, enabled)` — a
  frameless-at-rest `egui::Button::image_and_text` at `CONTROL_HEIGHT`, the mark
  at `SESSION_MARK_SIZE`; `session_status_icon` allocates and paints at
  `TAB_STATUS_MARK_SIZE`.
- `src/glyphs.rs`: `SESSION_MARK_SIZE` 16 → 14; new `TAB_STATUS_MARK_SIZE = 14.0`
  (the 18-point grid `STATUS_ICON_SIZE` stays the geometry's unit, so the marks
  scale rather than being redrawn).
- No persisted type changes. No new process or argument: the launch goes
  through `start_session` → `launch_session_record` → `launch_stored_session`
  like every sheet launch. Nothing new runs per frame while the menu is closed;
  while open it reads four booleans and four cached textures.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | no | No colour is added; rows use the button visuals, the popup the existing popup frame. Read `DESIGN.md` Colours to confirm. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | yes | No glyph is added: the rows use the existing marks (`session_mark_texture`); the + keeps `ICON_ADD`. `DESIGN.md` lines on the tab marks' sizes (18px status, 16px CLI) change to 14px, and the Controls table gains a row for `launch_choice_button` (a CLI's mark and its name, 28px): the existing rows do not fit it — `small_icon_button` is an icon alone at 24px and `icon_text_button` takes a glyph, not a picture. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | yes | No new identifier. The menu's order is one list, `WORKSPACE_LAUNCH_CHOICES` in `src/app/screens.rs`: `"terminal"` (the existing plain-terminal agent value) then `"claude"`, `"codex"`, `"gemini"`. `session_273_plus_opens_launch_menu` puts every choice but `"terminal"` through `build_agent_launch_command`, so an agent choice that is not in `AGENTS` fails the round trip; `"terminal"` is checked against `session_mark_texture`, which names it. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | yes | No shape change. The new session is pushed and persisted with the same restore-on-failure step `launch_session` uses; `recent-agent-settings.json` is read, never written, by this path. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | yes | The command is built only by `build_agent_launch_command` (which applies `is_safe_agent_option` and the typed-argument gate) and launched by the existing `launch_session_record`; the danger acknowledgement is enforced (Requirement 4). `src/agents.rs` is not edited. |
| Documentation — user-facing docs change in all three languages together | yes | `README.md`, `README.ja.md`, `README.ko.md` describe the + as opening a plain terminal; all three change to the menu. New strings get EN and KO rows in `src/i18n_tables.rs`. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | yes | None added. |
| Budgets — any new scan or output path states its byte and item ceiling | no | No scan or output path. |

## Flagged concerns

None open. The menu's shape and the amount of shrinking were put to the person
with previews; the recommended defaults were adopted when no answer came, and
both are a revert away.

## Acceptance

- `make q.fast` passes; `cargo test --locked session_273_`, `session_254_`,
  `session_260_`, `session_267_` pass.
- The mutation sweep catches: the + launching a terminal directly, an agent
  row launching the terminal, the danger check skipped, a missing CLI enabled,
  and the old tab margins.
- Rendered fixture images show the open menu under the + (few tabs and at the
  pane's right edge) and the compact tabs beside the old ones.
- CI passes before landing; package from main and install.

## Rejected alternatives

- Opening the launch sheet preselected to the chosen CLI: the person asked for
  the choice to start it.
- Launching with the CLI's bare defaults: every model or effort the person set
  in the sheet would be lost on the quickest path.
- Launching through `launch_session` with the sheet's fields swapped and
  restored (as the arena does): it reads and clears the person's draft and
  records the sheet's options as remembered ones.
- A custom CLI row: it has no command until one is typed in the sheet.
- Shrinking the close button or the + below the documented 24/28px heights.
