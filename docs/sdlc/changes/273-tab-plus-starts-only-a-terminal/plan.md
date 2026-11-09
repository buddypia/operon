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
