# Intent: Launching an agent CLI from a folder lacks visible progress and feedback

- **Status**: approved
- **Opened**: 2026-09-12

## Problem

When a user specifies a target folder and starts an agent CLI (or terminal) in Operon, the application only displays a brief toast notice at the bottom of the window while the session prepares in the background. The user cannot see what directory was targeted, which CLI process and model are executing, what startup phase is in flight, or how long it has been running. If the launch hangs or fails, the user is left guessing whether the click registered, whether the folder path was valid, or why the process failed.

## Who feels it, and when

Anyone who launches an agent CLI session from the overview screen or worktrees tab, especially on slower machines or during first-time launches where tmux session allocation, command line assembly, and workspace checks take noticeable time.

## Desired outcome

When a launch is initiated, the application presents a clear modal dialog displaying the target folder path, the chosen CLI agent and command, startup progress status with an active spinner, and elapsed time. The user can dismiss the modal to let it run in the background or stay informed. If launch succeeds, the interface transitions smoothly to the active terminal. If launch fails, the modal clearly explains the failure cause and offers actionable next steps (such as opening settings or retrying). In addition, the overview screen provides clearer folder visibility and inspection controls.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese; code, comments, and docs are English.
- All user-facing strings must have entries in `src/i18n_tables.rs`.
- Do not introduce blocking loops on the main rendering frame.

## Systems likely affected

- `src/app.rs` — `OperonApp` state, launch triggers, background task completion handling, `ui_launch_modal`.
- `src/app/screens.rs` — overview launch section folder display and actions.
- `src/ui/widgets.rs` — reusable launch modal components.
- `src/i18n_tables.rs` — translations for new Japanese message keys.
- `src/tests.rs` — modal rendering, state management, and failure tests.

## Open questions

None. The modal pattern follows the existing `ui_restore_modal` structure and integrates with `BackgroundKey::SessionStart` and `BackgroundResult::SessionStarted`.

## Not in scope

- Changing how tmux sessions are spawned or managed in `src/tmux.rs`.
- Adding remote SSH folders or non-local execution.
