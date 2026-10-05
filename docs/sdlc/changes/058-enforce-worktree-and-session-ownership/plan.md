# Plan: Enforce worktree isolation and session ownership across all agent CLIs

- **Spec**: `./spec.md`
- **Approved**: 2026-09-18
- **Status**: done

## Files that change

| File | Change |
|---|---|
| `.cli/hooks/destructive-git-guard.mjs` | Add pattern blocking `git config [.*] core.worktree` |
| `.cli/hooks/worktree-policy-guard.mjs` | Add Bash command classification on trunk; block non-whitelisted development commands |
| `.cli/hooks/worktree-session-owner-guard.mjs` | Extend Bash inspection to extract worktree targets and block foreign-session execution |
| `.cli/lib/hook-registry.mjs` | Register `worktree-policy-guard` on Bash; refine `worktree-session-owner-guard` matcher |
| `.claude/settings.json` | Register `worktree-policy-guard.mjs` for Claude Code |
| `.codex/hooks.json` | Register `worktree-policy-guard.mjs` for Codex CLI |
| `.claude/hooks.json` | Register `worktree-policy-guard.mjs` for Antigravity CLI |
| `.claude/scripts/worktree-new.mjs` | Record `.session-owner` during worktree initialization with `--session-id` |
| `src/git.rs` | Provide `git_command(path)` scrubbing `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_PREFIX` |
| `src/sys.rs` | Delegate `git_output` through `git_command` |
| `src/app.rs` | Use `git_command` for worktree operations |
| `src/tests.rs` | Replace `Command::new("git")` with `git_command` and add `git_command_scrubs_inherited_git_environment_variables` test |
| `.gitignore` | Ignore local CLI config files (.claude/config.json, .codex/config.toml, .agents/config.json, .claude/settings.local.json) |
| `docs/sdlc/changes/**` | Unbacktick uncommitted/dead paths per lesson 003 |

## Order of work

1. Provide centralized `git_command` in `src/git.rs` that scrubs inherited git environment pointers, update all git commands in `src/sys.rs`, `src/app.rs`, and test fixtures in `src/tests.rs`, and add a unit test verifying env scrubbing.
2. Update `destructive-git-guard.mjs` to block `git config core.worktree` and add tests for this pattern.
3. Update `worktree-policy-guard.mjs` to enforce worktree creation for development commands on trunk, allowing safe read-only queries and worktree lifecycle commands.
4. Update `worktree-session-owner-guard.mjs` to block foreign-session command execution targeting other worktrees.
5. Update `worktree-new.mjs` to ensure `.session-owner` is recorded on creation.
6. Synchronize hook registries across Claude Code, Codex, and Antigravity.
7. Verify all three gates (`cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| False-positive blocking of valid read-only commands on trunk | Developer running `git status` or `ls` gets blocked | Unit tests covering allowed trunk commands and regression testing |
| Session ID format discrepancy across CLIs | Layer 2 ownership check fails to recognize session owner | Unit tests verifying session ID extraction and matching across Claude, Codex, and Antigravity shapes |
| Test suite failure when `GIT_DIR` is scrubbed | Tests expecting repository context fail | `cargo test --locked` runs the entire 460+ test suite |

## Proof of completion

- `cargo fmt --check` — silent pass (exit code 0).
- `cargo test --locked` — `test result: ok. 461 passed; 0 failed; 6 ignored; finished in 17.00s`.
- `cargo clippy --locked -- -D warnings` — silent pass (exit code 0, 0 warnings).
- `destructive-git-guard.mjs` blocks `git config core.worktree ...` (verified via node test suite).
- `worktree-policy-guard.mjs` blocks development commands like `cargo build` on trunk while allowing `make wt.new` and `git status` (verified via node test suite).
- `worktree-session-owner-guard.mjs` blocks commands targeting foreign-owned worktrees (verified via node test suite).

## Departures from the plan

- Scrubbing expanded from test fixtures to production git callers (`src/git.rs`, `src/sys.rs`, `src/app.rs`) using a centralized `git_command` helper to ensure complete isolation against parent environment pollution.
- Unbackticked uncommitted scripts and gitignored paths in documentation (`038`, `040`, `051`) adhering to lesson 003 so `harness_documents_only_name_paths_that_exist` passes in clean worktree checkouts.
