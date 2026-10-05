# Plan: Remember and Restore Recent Agent Launch Settings

- **Spec**: ./spec.md
- **Approved**: 2026-09-23
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Add `RECENT_AGENT_SETTINGS_FILE_NAME` constant. |
| `src/agents.rs` | Add `AgentLaunchSettings`, `RecentAgentSettings`, loading/saving, path resolver, and flag sanitizer. |
| `src/app.rs` | Add `recent_agent_settings` to `OperonApp`. In `from_state`, load saved settings and restore recent options. In `select_project`, stop clearing agent launch settings. In `launch_session`, persist current settings. |
| `src/app/screens.rs` | In `select_agent`, stash departing agent options and restore incoming agent options. In `set_launch_mode`, `toggle_launch_flag`, and `ui_launch_options`, synchronize recent settings. |
| `src/tests.rs` | Add unit tests for persistence, project switching retention, agent switching isolation, sanitize logic, and update existing assertions where appropriate. |

## Order of work

1. Define `RECENT_AGENT_SETTINGS_FILE_NAME` in `src/config.rs`.
2. Implement `AgentLaunchSettings`, `RecentAgentSettings`, `load_recent_agent_settings`, and `save_recent_agent_settings` in `src/agents.rs`.
3. Integrate settings restoration into `OperonApp::from_state` in `src/app.rs`.
4. Update `select_project` to retain agent options and `launch_session` to persist them.
5. Update `select_agent`, `set_launch_mode`, `toggle_launch_flag`, and option pickers in `src/app/screens.rs`.
6. Write unit tests in `src/tests.rs` asserting persistence, round-trip restoration, project-switch retention, and flag sanitization.
7. Run format, tests, clippy, and package macOS application.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Stored flags become invalid after CLI definition change | CLI launch command rejection | Sanitizer drops unknown flags during load |
| Incompatible flag with newly selected mode | Launch options conflict | `set_launch_mode` and `toggle_launch_flag` exclusivity guards |
| Test suite assumptions on empty launch inputs after project switch | `switching_projects_clears_project_scoped_launch_state` failure | Explicitly audit and align project-switch test assertions |

## Proof of completion

- `bash scripts/check-readiness.sh docs/sdlc/changes/072-a-new-session-forgets-the-agent-launch-settings` — Go.
- `cargo fmt --check` — no output.
- `cargo test --locked` — all tests pass (0 failed).
- `cargo clippy --locked -- -D warnings` — no warnings.
- User verification: setting effort or permissions in one session carries forward automatically into subsequent sessions and across app restarts.

## Departures from the plan

None.
