# Plan: Launching a CLI opens its terminal screen immediately without a blocking progress modal

- **Spec**: `./spec.md`
- **Approved**: 2026-10-24
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `src/app.rs` | Start `LaunchProgress` with `dismissed: true` and immediately navigate to `Page::Sessions` with the session selected in `launch_session`, `launch_empty_session`, and `start_session`; in `BackgroundResult::SessionStarted` and `BackgroundResult::EmptySessionStarted`, arm immediate output capture (`request_session_output`), 33 ms echo polling (`last_terminal_input`), and terminal focus (`terminal_focus_request`) on `Ok`, and set `dismissed = false` with `failure = Some(error)` on `Err` |
| `src/tmux.rs` | Spawn `auto_approve_workspace_trust_prompt` asynchronously after `start_tmux_agent_session` in `launch_stored_session` so session start returns immediately, and exit `auto_approve_workspace_trust_prompt` early with `Ok(false)` when `tmux_capture_visible_screen` reports `TmuxState::Gone` |
| `src/tests.rs` | Add four tests covering immediate terminal navigation without the progress modal, immediate output/echo/focus arming on `SessionStarted`, failure modal display on launch error, and non-blocking `launch_stored_session` when workspace trust auto-approval is enabled |

## Order of work

1. Run `bash scripts/check-readiness.sh docs/sdlc/changes/219-launch-modal-hides-starting-terminal` and obtain evaluator approvals for `intent.md`, `spec.md`, `screen.md`, and `plan.md`; commit the planning artifacts.
2. Add the four new tests in `src/tests.rs` (`launch_opens_terminal_screen_immediately_without_progress_modal`, `session_started_arms_immediate_terminal_output_polling_and_focus`, `launch_failure_surfaces_diagnostic_modal_after_starting_in_terminal`, and `launch_stored_session_does_not_block_on_workspace_trust_polling`) and run each one with `cargo test --locked <test_name>` to observe it fail before production code changes.
3. Update `src/app.rs` and `src/tmux.rs` to implement Requirements 1–4.
4. Run `cargo test --locked` on each of the four new tests plus existing launch modal, workspace trust prompt, and handoff tests, then run `make q.fast` and `bash scripts/check-bands.sh`.
5. Write and run `docs/sdlc/changes/219-launch-modal-hides-starting-terminal/mutations.py` to verify every mutation is caught by the named tests.
6. Stage changes, run `rust-reviewer` and `subprocess-safety-reviewer`, record verdicts in `review.yaml`, settle `state.yaml` to `done` and `plan.md` to `done`, commit, push, verify CI, merge to `main` with `--no-ff`, clean up the worktree, and package `/Applications/Operon.app`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Calling `request_session_output` before `tmux new-session` finishes produces a transient capture error that overwrites `self.notice` | Handoff notice tests or launch notice assertions fail | `start_session` only selects the session and switches to `Page::Sessions`; `request_session_output` runs on `SessionStarted` / `EmptySessionStarted` (`cargo test --locked handoff` and `session_started_arms_immediate_terminal_output_polling_and_focus`) |
| Removing or breaking `ui_launch_modal` or its i18n strings breaks existing modal and i18n table guards | `the_launch_modal_*` or `every_message_row_is_one_the_code_shows` fails | `cargo test --locked the_launch_modal_` and `cargo test --locked every_message_row_is_one_the_code_shows` |
| Background workspace-trust thread keeps polling for 8s after a test's temporary `tmux` session is killed | Unnecessary background `tmux capture-pane` calls on dead sessions | `auto_approve_workspace_trust_prompt` checks `TmuxState::Gone` and returns `Ok(false)` immediately; tested by `launch_stored_session_does_not_block_on_workspace_trust_polling` |
| Launch failure goes unnoticed if `dismissed` remains `true` on `Err` | Error modal does not render when `tmux` or CLI binary is missing | `launch_failure_surfaces_diagnostic_modal_after_starting_in_terminal` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- `cargo test --locked launch_opens_terminal_screen_immediately_without_progress_modal` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked session_started_arms_immediate_terminal_output_polling_and_focus` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked launch_failure_surfaces_diagnostic_modal_after_starting_in_terminal` — `test result: ok. 1 passed; 0 failed`.
- `cargo test --locked launch_stored_session_does_not_block_on_workspace_trust_polling` — `test result: ok. 1 passed; 0 failed`.
- `python3 docs/sdlc/changes/219-launch-modal-hides-starting-terminal/mutations.py` — all mutations caught.

## Departures from the plan

- Added `"can't find pane"` to `tmux_error_state` in `src/tmux.rs` so `auto_approve_workspace_trust_prompt` recognizes a missing target pane (`tmux capture-pane -t <name>`) as `TmuxState::Gone` when the `tmux` server is already running.
