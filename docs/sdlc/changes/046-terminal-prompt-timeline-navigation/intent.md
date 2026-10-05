# Intent: A person looking at terminal output loses track of prompts and cannot jump to them

- **Status**: approved
- **Opened**: 2026-09-15

## Problem

When a coding-agent CLI runs in Operon, it produces large volumes of output: streaming tokens, tool commands, file diffs, and compiler traces. As this output accumulates, the user prompt that initiated the turn scrolls far out of the visible viewport. There is currently no indicator showing what the person asked in the active turn or previous turns, and navigating back to an earlier prompt requires manually scrolling through thousands of lines of terminal output and scanning by eye for prompt glyphs.

## Who feels it, and when

Anyone running multi-turn coding sessions with Claude Code, Codex CLI, or Antigravity agy, especially during long-running tasks or tasks with verbose tool execution.

## Desired outcome

A person can always see the prompt text of the current turn and all previous turns in the session. Clicking any turn in the prompt timeline immediately scrolls the terminal viewport to that exact prompt line, highlighting the row so the context of the instruction is visible without manual searching. When extra terminal width is needed, the timeline can be collapsed with a single click.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store is on real machines — in-memory UI navigation should avoid unnecessary store schema bumps unless persistence is strictly required.

## Systems likely affected

- `src/ui/terminal.rs`
- `src/app.rs`
- `src/app/screens.rs`
- `src/tmux/hooks.rs`
- `src/i18n_tables.rs`
- `src/tests.rs`

## Open questions

None. The investigation and HTML preview prototype verified that `src/tmux/hooks.rs` already captures `UserPromptSubmit` events with prompt payloads, and `src/ui/terminal.rs` already contains the `scroll_to` offset calculation logic in `terminal_rows`.

## Not in scope

- Editing or re-submitting past prompts from the timeline panel (the panel is for navigation and inspection).
- Cloud synchronization of prompt history.
