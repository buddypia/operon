# Spec: Clear current-session navigation

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. The selected sidebar session has the documented neutral selection fill and a 2px orange left rule, following selection when either sidebar or workspace tabs are clicked. Test: `session_254_selection_follows_navigation`.
2. Workspace tab titles use 12px regular text, left alignment, and a content-sized width capped at 132px; the selected tab has neutral selection fill plus its existing orange bottom rule. Long names truncate and rename/close/drag remain available. Test: `session_254_compact_tabs`.
3. Every pane's tab strip reserves a fixed right-edge plus outside horizontal scrolling. Clicking it immediately starts a plain terminal in that pane's active session project and worktree, using the existing bounded background launch. Stale launch-form values cannot redirect it. Missing tmux and invalid paths use existing Japanese notices without creating a session. Tests: `session_254_terminal_action`, `session_254_terminal_context`.

## Behaviour

Existing sidebar status/vendor icons remain. Only selected row and tab surfaces change. The plus hover and accessible name say 「空のターミナルを開く」. Creation shows existing 「ターミナルを起動しています…」 then the normal starting/active/failed session state. No project disables the action; missing tmux uses 「tmux が必要です。「設定」を開いてください。」. Empty workspace retains a tab-strip plus so the first terminal can start; unavailable paths are refused by the existing launch validation. Long names show their full text on hover. No new modal or rearrangement.

## Design

EXTEND: `ui_tab_workspace` and `ui_workspace_projects` in `src/app/screens.rs` already own these controls. Add `launch_workspace_terminal` in `src/app.rs` to select the clicked pane's context and delegate to existing `launch_empty_session`. Use the active session worktree or active document project root, falling back to the displayed workspace project. No persisted type, dependency, new process command, scan or output budget. Background launch stays with `EmptySessionStart`; drawing reads current state only.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — DESIGN.md | yes | Existing row_selected, text_strong, text and accent roles; no new values. |
| Icons — src/glyphs.rs | yes | Existing ICON_ADD, status marks and vendor assets. |
| Identifier SSOT — src/config.rs | no | No shared identifiers introduced. |
| Durability | no | No persistence implementation or schema changes; existing launch path used. |
| Subprocess safety — src/exec.rs, src/agents.rs | yes | Existing validated launch_empty_session background task reused without command changes. |
| Documentation — CONTRIBUTING.md | yes | Internal change records only; no public documentation changes. |
| Local-first — CONTRIBUTING.md | yes | No network, accounts, telemetry, dependency or unsafe changes. |
| Budgets | no | No new scans or output paths. |

## Flagged concerns

None. The screenshot and explicit request authorize these refinements within the existing layout, including the requested plus. App restart remains a final installation decision.

## Acceptance

- `make q.fast` and `cargo test --locked session_254_` pass.
- Mutation sweep catches removed selection fill/rule, enlarged tab text, missing plus or wrong launch context.
- Inspect rendered Japanese fixture output at normal and narrow widths; selected row/tab obvious and plus remains inside pane.
- CI passes before landing; package from main; install when permission to quit the running app is available.

## Rejected alternatives

- A new setup modal adds a step to the requested immediate terminal creation.
- Putting the plus inside the scrolling content makes it disappear as tabs grow.
- Orange text for the whole selected row competes with session state.
