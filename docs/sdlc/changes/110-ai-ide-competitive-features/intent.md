# Intent: AI agent development loop lacks local worktree landing, run scripts, checkpoints, and IDE integrations

- **Status**: approved
- **Opened**: 2026-10-04

## Problem

Operon provides agent supervision across tmux sessions with Git worktree isolation and diff reviews. However, key workflow and development integration features are missing:
1. Worktree landing: Completed work in a worktree has no one-click path to merge back to the local mainline branch and safely clean up the worktree and branch.
2. Run scripts: No dedicated one-click Run button exists to launch project or worktree development servers and tests in dedicated tmux sessions with approval protection.
3. Pull Request creation: While PR listing and CI checks exist via `gh`, creating a PR from a worktree with AI-drafted title and description is missing.
4. Checkpoints & rollback: When an agent produces undesired changes across turns, there is no quick turn-level snapshot or one-click rollback mechanism.
5. External editor launch: Opening a worktree or project directly in VS Code, Cursor, Zed, or Finder requires leaving the app.
6. Prompt templates & Magic Commands: Frequently used workflows (review diff, fix CI, resolve conflicts) cannot be triggered from templates.
7. GitHub Issue onboarding: Starting a session with context from a repository issue requires manual copy-pasting.

## Who feels it, and when

Anyone using Operon to run autonomous agent workflows on repositories, especially:
- Repositories without a GitHub remote where pushing is impossible and local landing is required.
- Developers needing to verify work with local dev servers or tests before landing.
- Developers supervising agents across multiple turns who need to roll back regressions.
- Developers switching between Operon and full-featured editors like VS Code/Cursor.

## Desired outcome

1. One-click "Land Worktree" action merges clean worktrees into the mainline branch with conflict protection and optional cleanup.
2. One-click "Run" button launches `.operon/run.sh` in a managed tmux session with SHA digest approval.
3. "Create PR" action submits a pull request via `gh pr create` with auto-drafted content.
4. Checkpoints are automatically captured on prompt turns to allow one-click rollback and diff viewing.
5. Quick actions open project or worktree folders in external editors (VS Code, Cursor, Zed, Finder).
6. Prompt templates (`.operon/prompts/*.md`) can be selected and populated into prompts.
7. GitHub issues can be imported to populate session requests and branch names.

## Constraints this change inherits

- macOS only; `eframe`/`egui` 0.31 immediate-mode GUI.
- Local-first: no telemetry, no accounts, no cloud calls.
- User-facing text is Japanese (via `tr()` and tables in `src/i18n_tables.rs`); code, comments, and docs are English.
- No force pushes; all Git operations must be safe and abort on conflict without corrupting worktrees.

## Systems likely affected

- `src/git.rs` (worktree landing, checkpoints, PR creation, issues)
- `src/git/setup.rs` (run script approval and discovery)
- `src/app.rs` / `src/app/screens.rs` (UI buttons, modals, actions)
- `src/models.rs` (data structures for checkpoints, run scripts, issues)
- `src/i18n_tables.rs` (Japanese, English, Korean translations)
- `src/tests.rs` (unit and integration tests)

## Open questions

None. The user explicitly approved implementing all identified workflow features.

## Not in scope

- Cloud sandboxes and cloud synchronization (Operon remains strictly local-first).
- Full WYSIWYG rich-text document editors.
