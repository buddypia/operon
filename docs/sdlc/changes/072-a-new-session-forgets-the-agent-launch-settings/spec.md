# Spec: Remember and Restore Recent Agent Launch Settings

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. Operon must persist recent launch settings (model, mode, effort, flags, custom command) per agent in a machine-local JSON file named `recent-agent-settings.json` beside the application store file.
2. When launching a session, the user prompt (`goal_input`) is cleared for the next task, but the configured launch parameters (`agent_model_input`, `agent_mode_input`, `agent_effort_input`, `agent_flag_inputs`) are retained in the UI form and saved to disk.
3. Switching projects (`select_project`) must not wipe the active agent's launch settings (model, mode, effort, flags, custom command); it only clears project-specific inputs (session path, branch, dependencies, and prompt goal).
4. Switching agents (`select_agent`) must record the departing agent's current settings, switch the active agent, and restore the arriving agent's most recently saved settings (or defaults if none exist).
5. Application startup (`OperonApp::from_state`) must load `recent-agent-settings.json`, restore the last chosen agent if available, and populate the launch form with that agent's recent settings.
6. Restored flags and mode must be checked against the agent's catalog to ensure obsolete or unsupported options are pruned cleanly without causing runtime failures.
7. Persistence must write atomically via `write_file_atomically` so that crashes or power loss cannot produce truncated settings.

## Behaviour

- **First launch**:
  - If `recent-agent-settings.json` is missing or empty, default options apply (empty model, default CLI mode, default effort, no flags, default agent).
- **Adjusting options on Overview**:
  - The user enters a custom model, selects an execution mode (e.g. `danger-full-access`), chooses reasoning depth (e.g. `high`), or toggles flags (e.g. `bypass-approvals`).
  - These choices are reflected immediately in the form state and recorded into recent settings.
- **Starting a session**:
  - The user types a goal prompt and clicks launch.
  - The session starts. `goal_input` and `session_name_input` are cleared.
  - The model, mode, effort, and flag settings remain populated in the overview form and are saved to `recent-agent-settings.json`.
  - Next time the user begins another session in this or another project, the same settings are already selected.
- **Switching agents**:
  - The user switches from `claude` to `codex`.
  - `claude`'s current options are stored in the memory cache.
  - `codex`'s recent options are restored into `agent_model_input`, `agent_mode_input`, `agent_effort_input`, and `agent_flag_inputs`.
  - The active agent preference is written to `recent-agent-settings.json`.
- **Switching projects**:
  - The user navigates to another project in the sidebar.
  - Project-specific worktree path and goal inputs reset, but the selected agent and its options remain intact.
- **Restarting the application**:
  - The user quits Operon and reopens it.
  - `recent-agent-settings.json` is loaded. The last active agent and all its options are restored.

## Design

- `src/config.rs`:
  - Define `pub(crate) const RECENT_AGENT_SETTINGS_FILE_NAME: &str = "recent-agent-settings.json";`.
- `src/agents.rs`:
  - Define `AgentLaunchSettings`:
    - `model: String`
    - `mode: String`
    - `effort: String`
    - `flags: Vec<String>`
    - `custom_command: String`
  - Define `RecentAgentSettings`:
    - `last_selected_agent: Option<String>`
    - `agents: HashMap<String, AgentLaunchSettings>`
  - Implement helper functions:
    - `recent_agent_settings_path(data_file: &Path) -> PathBuf`
    - `load_recent_agent_settings(data_file: &Path) -> RecentAgentSettings`
    - `save_recent_agent_settings(data_file: &Path, settings: &RecentAgentSettings) -> Result<()>`
    - `sanitize_agent_launch_settings(agent: &str, settings: &mut AgentLaunchSettings)`
- `src/app.rs`:
  - Add `recent_agent_settings: RecentAgentSettings` to `OperonApp`.
  - In `OperonApp::from_state`:
    - Load settings via `load_recent_agent_settings(&data_file)`.
    - Determine `selected_agent`: prefer `recent_agent_settings.last_selected_agent` if the tool is available, falling back to `default_agent(&tools)`.
    - Populate `agent_model_input`, `agent_mode_input`, `agent_effort_input`, `agent_flag_inputs`, and `custom_command` from the selected agent's settings.
  - In `select_project`:
    - Stop clearing `agent_model_input`, `agent_mode_input`, `agent_effort_input`, `agent_flag_inputs`, and `custom_command`.
  - In `launch_session`:
    - Update `recent_agent_settings` with the current settings for `selected_agent` and save to disk.
- `src/app/screens.rs`:
  - In `select_agent`:
    - Stash outgoing agent options into `recent_agent_settings`.
    - Load incoming agent options from `recent_agent_settings` if present, otherwise set empty defaults.
    - Update `last_selected_agent` and call `save_recent_agent_settings`.
  - In `set_launch_mode`, `toggle_launch_flag`, and `ui_launch_options`:
    - Update and persist recent options whenever launch configuration changes.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Reuses existing UI components and palette colors. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | No | No new glyphs introduced. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | `RECENT_AGENT_SETTINGS_FILE_NAME` defined in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | The store schema is unchanged; recent settings are kept in an independent JSON file beside the store using `write_file_atomically`. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | No | Launches continue through existing validated pathways. |
| Documentation — user-facing docs change in all three languages together | No | Purely behavioral retention; no documentation changes needed. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Local disk storage only; fully zero telemetry. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | JSON file is bounded to a few kilobytes for known agent entries. |

## Flagged concerns

- **Carrying obsolete flags across CLI updates**: If a CLI agent deprecates or changes flags, a stored setting could fail command line validation. The design mitigates this by passing restored flags through `agent_flag(agent, id)` validation to prune unrecognized flags before populating the form.
- **Switching projects with different conventions**: Users who use different flags per project can still easily adjust settings on the overview page. Retaining the last-used setting avoids repetitive re-entry for the predominant majority of tasks.

## Acceptance

- `cargo test --locked` passes, including unit tests verifying:
  - `recent-agent-settings.json` is created and read back accurately.
  - Project switching retains agent model, mode, effort, and flags.
  - Agent switching preserves settings for each agent independently.
  - App initialization loads and applies saved settings.
  - Unknown or conflicting flags are sanitized on restoration.
- Manual verification shows non-default effort and permissions persist across session launches and app restarts.

## Rejected alternatives

- Storing recent settings in the project record inside `src/store.rs`: Rejected because `store.rs` is a high-risk paused surface requiring a `STORE_SCHEMA_VERSION` bump, and launch preferences are user/machine-level rather than repository-level attributes.
- Remembering the prompt text: Rejected because new sessions require fresh instructions; keeping old prompts leads to accidental duplicate task submissions.
