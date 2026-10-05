# Intent: A new session forgets the agent launch settings

- **Status**: approved
- **Opened**: 2026-09-23

## Problem

When launching an agent session in Operon, users often configure specific CLI launch options tailored to their workflow — such as reasoning depth (effort), approval and permission flags (e.g. bypassing confirmations, sandbox switches, web search), target model, or execution mode. However, the application forgets these preferences immediately:
1. Switching to a different project clears all CLI launch settings back to empty defaults.
2. Launching a session retains the inputs in memory for the current screen, but closing and reopening Operon discards them completely.
3. Switching between agents discards the previously configured options for the prior agent.

As a result, a user who deliberately chooses a specific reasoning depth or permission level must painstakingly re-enter or re-toggle those options every time they launch a new session or switch projects. The prompt (`goal_input`) is the only field that legitimately varies on every new session; the underlying CLI parameters represent the user's ongoing execution preferences.

## Who feels it, and when

Anyone launching CLI agent sessions in Operon, particularly users who configure non-default reasoning depth (e.g. `high` effort), relaxed permission flags (e.g. `bypass-approvals`), or custom models, across project switches and application restarts.

## Desired outcome

Operon remembers the most recently configured launch options per agent (model, mode, effort, and flags, plus custom command) and persists them in a machine-local file (`recent-agent-settings.json`) beside the store.
- When creating a session, the prompt is consumed and cleared, but the configured CLI launch options remain active for subsequent launches.
- When switching projects, the active agent's launch settings are preserved rather than blindly wiped.
- When switching agents, the prior agent's settings are saved in memory and on disk, and the newly selected agent's most recent settings are restored.
- When Operon restarts, the last selected agent and its recent launch settings are automatically restored, allowing immediate one-click launching with preferred options.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, accounts, or cloud synchronization.
- User-facing text is Japanese; code, comments, and docs are English.
- The persisted store schema (`src/store.rs`, `src/models.rs`) is a high-risk paused surface: machine-local working state must live in a dedicated JSON file beside the store (`recent-agent-settings.json`), without bumping `STORE_SCHEMA_VERSION`.
- Atomicity: file persistence must use `write_file_atomically`.
- Safety: remembered flags must still be validated against the agent's known catalog (`agent_flag_options`, `resolve_agent_flags`), pruning any invalid or conflicting switches upon restoration.

## Systems likely affected

- `src/config.rs` — constant for `RECENT_AGENT_SETTINGS_FILE_NAME`.
- `src/agents.rs` (or `src/agents/settings.rs`) — data structures for `AgentLaunchSettings`, `RecentAgentSettings`, serialization, and loading/saving.
- `src/app.rs` — `OperonApp` state initialization in `from_state`, preserving options in `select_project` and `launch_session`.
- `src/app/screens.rs` — preserving and restoring agent options on agent selection in `select_agent` and updating settings on UI edits.
- `src/tests.rs` — test suite coverage for persistence, restoration across project switch and restart, and flag validation.

## Open questions

None. The persistence pattern precisely mirrors `setup-trust.json` (`src/git/setup.rs`) and `diff-comments.json` (`src/git/comments.rs`).

## Not in scope

- Per-project launch settings overrides (a single machine-wide preference per agent satisfies the user expectation without project configuration complexity).
- Remembering the prompt text (`goal_input`), which should always start blank for new tasks.
