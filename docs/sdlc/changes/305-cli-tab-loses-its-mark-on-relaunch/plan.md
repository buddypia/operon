# Plan: a CLI tab loses its mark on relaunch

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-17
- **Status**: done

## Root cause

`can_fallback_open_tab_to_shell` in `src/app/relaunch.rs` (change 303) admits
every interactive session with an open tab, CLIs included. When
`relaunch_lost_session_attempt` answers `ConversationUnavailable` — always, for
a CLI typed into a terminal once its process is gone —
`recover_session_or_fallback_shell` calls
`reopen_exited_session_as_shell_attempt`, which sets `agent = "terminal"` and
clears `native_session_id` / `native_session_path`. The tab's vendor mark and
state dot go with it (`session_mark_texture`, `session_mark_speaks`).

## The fix

`can_fallback_open_tab_to_shell` also requires
`cli_provider_for_agent(&session.agent).is_none()`. A CLI tab whose
conversation cannot be resumed then settles in `Lost`/`Failed` as it did before
303: kind, conversation pointers, mark, dot, and 「再試行」 kept; the 303
capture-loop guards keep its pane quiet. Custom and existing-terminal tabs keep
the shell fallback and its toast.

## Files that change

| File | Change |
|---|---|
| `src/app/relaunch.rs` | the CLI condition in `can_fallback_open_tab_to_shell` |
| `src/tests.rs` | `cli_305_unresumable_cli_tab_keeps_its_kind_and_conversation`; the 303 fallback tests now expect a CLI tab to stay a CLI and use non-CLI tabs for the shell fallback |
| `README*.md` | the lifecycle bullet says only a tab with no CLI reopens as a terminal |
| `docs/sdlc/changes/305-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. The new test, watched failing on the current code.
2. The condition.
3. The 303 tests and READMEs brought in line.
4. `mutations.py`: drop the condition, invert it — the new test goes red.
5. Land, then the ship skill.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| A CLI tab left dead on screen | a stopped tab with 「再試行」 | intended; 303's capture-missing guard keeps it from looping |
| Custom tabs lose the fallback | dead custom tab | the 303 fallback test keeps a custom tab falling back |

## Proof of completion

The named tests pass, `mutations.py` catches every mutation, CI green on the
branch head, `/Applications/Operon.app` replaced after landing.

## Departures from the plan

- `model_check_pane_303_missing_pane_recovery_and_output_guard` gained the
  invariant that a CLI session never becomes a terminal, admits an unresumable
  CLI leaving `lost_tabs_on_load` with nothing to persist, and covers 300
  states instead of 305: the five of a CLI turned into a terminal are gone.
  The F2 test now expects an ambiguous Claude tab to stay a stopped Claude tab.
