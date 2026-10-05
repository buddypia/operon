# Plan: CLI Launch Progress Modal and Folder Visibility

- **Spec**: ./spec.md
- **Approved**: 2026-09-12
- **Status**: done

This is the plan produced in plan mode and accepted before any file was edited.

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | Add `LaunchProgress` struct, `launch_progress` field on `OperonApp`, initialize on launch triggers, handle completion/failure in background event processing, and implement `ui_launch_modal`. |
| `src/ui/widgets.rs` | Add reusable helper `launch_progress_footer` for drawing status and action buttons cleanly. |
| `src/app/screens.rs` | In overview screen folder row, display enhanced folder info badge and 「Finder で表示」 action. |
| `src/i18n_tables.rs` | Add translation entries in alphabetical order for Japanese, English, and Korean. |
| `src/tests.rs` | Add unit and UI layout tests asserting launch modal properties, error state transitions, and translation consistency. |

## Order of work

1. Add `LaunchProgress` type and modal drawing infrastructure to `src/app.rs` and `src/ui/widgets.rs`.
2. Connect `launch_progress` initialization in `launch_session`, `launch_empty_session`, and `start_session`.
3. Handle completion and failure states in `BackgroundResult::SessionStarted` and `EmptySessionStarted`.
4. Enhance overview screen workspace folder visibility and Finder button in `src/app/screens.rs`.
5. Add i18n messages to `src/i18n_tables.rs`.
6. Add unit tests in `src/tests.rs` verifying title, folder display, error rendering, and dismiss logic.
7. Run format, tests, clippy, and verify the build.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Translation table sorting drift | `every_translation_table_is_sorted_by_message_id` fails | `cargo test --locked` |
| Modal dismiss race with background completion | Rapid dismiss followed by unexpected page switch | Completion check for `progress.dismissed` |
| Multiple repaints or flicker on modal spinner | Jittery text or layout jumps | Vertical separation in `launch_progress_footer` |

## Proof of completion

- `bash scripts/check-readiness.sh docs/sdlc/changes/042-cli-launch-progress-modal` — Go.
- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 474 passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- Unit tests pass:
  - `the_launch_modal_names_what_it_is_doing`
  - `the_launch_modal_shows_folder_and_agent_details`
  - `the_launch_modal_draws_error_state_on_failure`
- Visual confirmation of launch modal showing folder path, agent badge, elapsed seconds, spinner, and error diagnostics.

## Departures from the plan

None so far.
