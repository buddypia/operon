# Intent: enforce worktree isolation and session ownership across all agent CLIs

- **Status**: approved
- **Opened**: 2026-09-18

## Problem

Operon relies on isolated Git worktrees to allow concurrent work by multiple AI agents (Claude Code, Codex CLI, Antigravity CLI). While `AGENTS.md` prescribes worktrees and `.cli/hooks/commit-guard.mjs` blocks direct commits on `main`, several critical isolation loopholes remain:

1. **AI sessions on trunk (`main`) can execute arbitrary development commands**:
   `trunk-start-warning.mjs` only emits an advisory warning. `worktree-policy-guard.mjs` only guards `Edit` and `Write` tools. An AI session launched on `main` can execute build, test, and shell commands without creating a worktree, polluting the trunk working tree and causing confusion.
2. **Commands targeting foreign-session worktrees are not blocked**:
   `worktree-session-owner-guard.mjs` only inspects `tool_name === 'Bash'` when the command is a `git commit`. If an agent on `main` or in another worktree executes commands against a worktree owned by another session (e.g., `cd <foreign-worktree>`, `git -C <foreign-worktree>`, `--manifest-path <foreign-worktree>/...`), the command passes through unblocked.
3. **Repository corruption via `git config core.worktree`**:
   `destructive-git-guard.mjs` does not block `git config core.worktree`. When a test fixture or command sets `core.worktree`, it mutates `.git/config` for the whole repository, pointing `main` to a worktree and breaking tree resolution for every session.
4. **Git test fixture leaks**:
   Test fixtures running git commands do not scrub inherited environment pointers (`GIT_DIR`, `GIT_WORK_TREE`), allowing tests to pollute parent `.git/config`.
5. **Worktree session owner tracking**:
   `worktree-new.mjs` relies solely on `worktree-owner-tracker.mjs` PostToolUse hook to record `.session-owner`. If worktrees are created without that hook firing, `.session-owner` is missing, weakening Layer 2 ownership guards.

## Who feels it, and when

Anyone using multiple AI sessions in parallel (Claude Code, Codex CLI, Antigravity CLI) on the same repository. Sessions collide, modify each other's worktrees, corrupt git config, and run on `main` instead of dedicated worktrees.

## Desired outcome

A permanent, hardened multi-layer enforcement mechanism:
1. **Worktree Enforcement**: `worktree-policy-guard.mjs` enforces that on trunk branches (`main`/`master`), development commands are blocked. Only worktree management (`make wt.*`, `git worktree`), read-only inspection, and landing/release gates are permitted on trunk.
2. **Session Ownership Enforcement**: `worktree-session-owner-guard.mjs` blocks any command or edit targeting a worktree owned by another session.
3. **`core.worktree` Guard**: `destructive-git-guard.mjs` blocks any mutation to `core.worktree` in git config.
4. **Test Pointer Scrubbing**: Git fixtures in `src/tests.rs` explicitly scrub `GIT_DIR`, `GIT_WORK_TREE`, and `GIT_INDEX_FILE`.
5. **Automatic Owner Recording**: `worktree-new.mjs` records `.session-owner` reliably.
6. All guards verified and active across Claude Code, Codex CLI, and Antigravity CLI.

## Constraints this change inherits

- Multi-CLI support: Claude Code, Codex CLI, Antigravity CLI must share identical guard semantics.
- Local-first: No network dependencies, no external services.
- Minimal friction for valid operations: Allowed trunk operations (worktree creation, queries, landing merges) must pass seamlessly.

## Systems likely affected

- `.cli/hooks/worktree-policy-guard.mjs`
- `.cli/hooks/worktree-session-owner-guard.mjs`
- `.cli/hooks/destructive-git-guard.mjs`
- `.claude/scripts/worktree-new.mjs`
- `src/tests.rs`
- Hook configurations: `.claude/settings.json`, `.codex/hooks.json`, `.claude/hooks.json`
