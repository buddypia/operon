# Plan: The tab-strip + chooses what to start, and the tabs are compact

- **Spec**: `./spec.md`
- **Approved**: 2026-11-09
- **Status**: approved

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | `workspace_pane_place`, `PaneAgentLaunch`, `workspace_agent_launch`, `launch_workspace_agent`; `launch_workspace_terminal` uses `workspace_pane_place`. |
| `src/app/screens.rs` | `WORKSPACE_LAUNCH_CHOICES`, `ui_workspace_launch_menu`; both + call it; tab margins, spacings, strip height. |
| `src/ui/widgets.rs` | `launch_choice_button`; `session_status_icon` at `TAB_STATUS_MARK_SIZE`. |
| `src/glyphs.rs` | `SESSION_MARK_SIZE` 14; `TAB_STATUS_MARK_SIZE`. |
| `src/i18n_tables.rs` | EN and KO rows for 「ターミナルや AI を起動」. |
| `DESIGN.md` | Tab mark sizes; Controls row for `launch_choice_button`. |
| `README.md`, `README.ja.md`, `README.ko.md` | The + opens the four-way menu. |
| `src/tests.rs` | `session_273_*`; the 254/260 tests press ターミナル after the +; the status-circle finder reads the new size. |
| `docs/sdlc/changes/273-tab-plus-starts-only-a-terminal/` | State, screen, verification, mutation sweep. |

## Order of work

1. Commit intent, spec, screen and plan after the evaluator's verdicts and a
   readiness Go; no `src/` edit before.
2. Write the `session_273_` tests and the updated 254/260 tests; watch them fail
   on the unmodified source for the stated reason (no menu; tabs too large).
3. `src/glyphs.rs`, `src/ui/widgets.rs`: sizes and `launch_choice_button`.
4. `src/app.rs`: place helper, decision, launch. Tree compiles.
5. `src/app/screens.rs`: menu and tab geometry. Tests pass.
6. Strings, `DESIGN.md`, READMEs.
7. Render fixture images (temporary renderer, not committed): tabs before and
   after, the open menu with few tabs and at the right edge, a disabled row,
   the empty workspace; look at them against `screen.md`.
8. `make q.fast`; mutation sweep; `verification.yaml`; an independent empirical
   verifier tries to break it; rust-reviewer for the craft review.
9. Close `state.yaml`, commit, push, CI, merge on main, clean up, package and
   install.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A click on a menu row also lands on the terminal under it | Terminal takes focus or selection instead of the launch | `session_273_agent_choice_reaches_the_launch` presses the row over a live pane; fixture check |
| The quick launch bypasses the danger acknowledgement | A permission-skipping agent starts with no consent | `session_273_dangerous_options_open_the_sheet`; mutation that drops the check |
| The quick launch overwrites the remembered options or the sheet's draft | Sheet opens with changed options; typed request lost | `session_273_agent_launch_uses_pane_and_remembered_options` compares draft and `recent_agent_settings` before and after |
| Smaller marks become unreadable | State glyph a smudge at 14px | Fixture images at 1x and 2x in dark and light themes; DESIGN.md's size note |
| Tighter tabs clip the underline, the rename editor or the close button | Rule missing, editor cut, close unclickable | `session_254_selected_tab_rule_is_visible`, `session_253_` tests, `session_267_` tests |
| The danger fallback rewrites the remembered options or the file | Next sheet or next quick launch uses the pin's options for another CLI | `session_273_dangerous_options_open_the_sheet` compares `recent_agent_settings` and the file's bytes before and after; it never calls `select_agent` |
| The terminal row is drawn disabled (its value is not an agent `tools.agent_available` knows) | ターミナル greyed out, the old + behaviour unreachable | `session_273_plus_opens_launch_menu` presses ターミナル and sees the launch; `session_273_missing_cli_is_disabled` with every CLI missing |
| The quick launch runs under a login the person did not pick | Agent starts on another account | `session_273_agent_launch_uses_pane_and_remembered_options`: no login without a pin for this CLI, the pin's login with one |
| The new session lands in another pane | Tab appears in the focused pane, not the clicked one | `session_273_agent_launch_uses_pane_and_remembered_options` uses a split |
| The menu runs off the window at the right edge | Rows cut off | egui `Area` constrain; fixture at 640px |

## Proof of completion

- `make q.fast` — passes.
- `cargo test --locked session_273_`, `session_254_`, `session_260_`,
  `session_267_`, `session_253_` — all pass; the new ones failed before step 3.
- Mutation sweep — every mutation caught.
- Fixture images match `screen.md`.
- CI green on the branch head; the installed app is the build from main.

## Departures from the plan

- The tab measured 148×36, not the 138×32 the spec had computed: the close
  button inherits the shared 10/5 button padding and was 34×28, and egui's
  `horizontal` row takes the shared 28px interact height. Inside the tab frame
  the row height is `CONTROL_HEIGHT_SMALL` and the button padding is zero, so
  the close button is its documented 24px and the tab is the 28px strip. The
  spec's Requirement 7 and Tab geometry now give the measured figures.
- `workspace_agent_launch` returns `Start`/`Confirm` instead of a struct with a
  flag, and decides the danger acknowledgement before tmux: the sheet needs no
  tmux to open, and the test guarding consent can then run without tmux, so a
  mutation that skips the check meets the missing-tmux refusal instead of
  starting a real CLI. Recording the session moved into
  `open_workspace_agent_session`, so a test can check the record without
  starting the agent. The spec's Design and the No-tmux state say so.
- The menu's rows are 4px apart instead of the default 8, after the rendered
  fixture showed four 28px rows reading as separate buttons.
- egui shows a tooltip only for a pointer at rest that has not moved since a
  click within 0.1s, so `session_273_missing_cli_is_disabled` lets frames pass
  before and after moving onto the disabled row.
- Rendered fixture images (menu open in dark and light, a disabled row with its
  hover, the + at the right edge with 18 tabs, the empty workspace, the compact
  tabs) came from a temporary renderer removed before the commit.
- Review found that the Escape closing the menu also reached the focused
  terminal, which would send ESC to a running agent and interrupt its turn:
  egui's popup only reads the key. `ui_tab_workspace` now consumes Escape
  before any pane draws while one of the + menus is open, checked by
  `session_273_plus_opens_launch_menu` and the `menu-escape-passed-on` mutation.
  Other popups (the sheet's combo boxes) are left alone: the check names the
  + menus' ids rather than asking whether any popup is open.
- The bands check found `src/app.rs` (9968 → 10111 lines) and
  `src/app/screens.rs` (9972 → 10067) past the 10 000-line
  `largest_module_lines` band, which asks for a change of its own. The change's
  new code — the place helper, the launch decision and recording, the menu and
  its Escape — moved instead into a child module, `src/app/launch_menu.rs`,
  listed in `CLAUDE.md`; `src/app.rs` ends at 9885 lines and
  `src/app/screens.rs` at 9995. The spec's Design names the new file; the sweep
  points its mutations there.
- The empirical verifier ran in this worktree rather than an isolated one: the
  worktree guard judges the Agent tool's worktree, outside `.worktrees/`, to be
  the trunk, and refuses a second worktree from this session. It changed no
  file and reproduced nothing.
- Merging main brought change 272's ⌘T, labelled 「空のターミナルを開く」 as
  "the + in the strip". ⌘T still opens a plain terminal, so it keeps that
  name and its translation rows come back; `docs/MANUAL*.md` and the keymap
  comment now call it the + menu's ターミナル row without the menu, and the
  manuals' tab paragraph says the + asks what to start.
