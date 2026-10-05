# Intent: Adversarial security, resilience, and performance hardening for AI IDE features

- **Status**: approved
- **Opened**: 2026-10-04

## Problem

Following the implementation of AI IDE workflow features (change 110), an adversarial review identified critical security vulnerabilities, data integrity risks, and performance bottlenecks:
1. TOCTOU in run scripts: `ui_run_script_modal` approves `.operon/run.sh` without verifying that the file on disk still matches the previewed SHA-256 digest at execution time, allowing race-condition arbitrary code execution.
2. Git environment poisoning: Newly added Git operations in `src/git.rs` use raw `command_output_in_dir("git", ...)` rather than `git_command(path)` / `git_output`, failing to strip inherited environment variables (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_COMMON_DIR`).
3. CLI option injection: Git commands (`merge`, `rev-list`, `branch -d`, `checkout`, `diff`) lack `--` double-dash argument separators, allowing hyphen-prefixed branch names or refs to be interpreted as flags.
4. Mainline branch verification gap: `git_land_worktree` does not verify that the primary checkout is currently on the expected mainline branch, risking landing commits onto an unrelated active feature branch.
5. Incomplete checkpoint capture: `git write-tree` only serializes the Git index, omitting unstaged working tree edits; capturing working files without corrupting the user's staging area requires an isolated index.
6. Synchronous disk I/O in UI draw loop: `ui_launch_form` calls `list_prompt_templates` on every frame, performing synchronous filesystem reads in the 60fps draw path.
7. Unbounded subprocess output: Checkpoint diffs and subprocess calls lack upper byte limits, posing memory exhaustion risks.
8. External editor launch failure under macOS GUI: CLI commands (`code`, `cursor`, `zed`) fail when launched from Finder/Dock due to minimal default `PATH`; requires `open -a` application bundle fallback.

## Who feels it, and when

- Users executing workspace run scripts if an agent or background process modifies `.operon/run.sh` while the approval modal is open.
- Users working across multiple repositories or worktrees if inherited Git environment variables leak into subcommands.
- Users landing worktree changes when the main checkout is not currently on the mainline branch.
- Users rolling back checkpoints who lose unstaged work that was never captured.
- Users experiencing UI stutter or frame drops on repositories with slow storage.

## Desired outcome

1. `decide_run_script_run` enforces cryptographic digest re-verification at the exact moment of execution; changed files update the preview and require re-approval.
2. All Git operations consistently use `git_command(path)` and `git_output`, stripping inherited repository pointers.
3. Git commands isolate ref arguments with `--` and validate ref names against unsafe control characters and leading hyphens.
4. `git_land_worktree` verifies that the main checkout's current branch strictly matches the resolved mainline branch before executing merge.
5. `git_create_checkpoint` captures the full working tree state into an isolated temporary index file (`.git/index_checkpoint_*`) with RAII cleanup, preserving user staging state.
6. Prompt templates are loaded lazily or cached outside the continuous frame draw loop.
7. Checkpoint diffs use bounded byte limits (`git_output_allowing_limited`).
8. `open_in_external_editor` falls back to macOS `open -a <Application>` when direct CLI binary invocation fails.

## Constraints this change inherits

- Zero regression and 100% backward compatibility with existing public models and serialized shapes (`LandResult`, `Checkpoint`, `PromptTemplate`, `ExternalEditor`, `GitHubIssue`).
- Strict adherence to Operon durability invariants: bounded subprocess output, no unmanaged background locks, no sync I/O in draw loop.
- All tests pass (`cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`).

## Systems likely affected

- `src/git.rs`
- `src/git/setup.rs`
- `src/util.rs`
- `src/app.rs`
- `src/app/screens.rs`
- `src/tests.rs`

## Open questions

None. The user explicitly directed to proceed with the defensive refactoring.

## Not in scope

- Adding new cloud providers or external network dependencies.
- Changes to persisted SQLite or JSON schemas.
