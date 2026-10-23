# Plan: parallel worktrees skip redundant hook git queries, share warm targets, and leave packaging to main

- **Spec**: none — bugfix route; reported when parallel multi-CLI worktree sessions timed out or queued on redundant hook and CI checks.
- **Approved**: 2026-10-23
- **Status**: approved

## The defect

When multiple AI CLI sessions (Claude Code, Codex CLI, Antigravity CLI) work in parallel across `.worktrees/<branch>` checkouts, five redundant or mis-sized checks compound:

1. **Unconditional trunk `git branch --show-current` on every `Bash` and `Edit`/`Write` call in `.cli/hooks/worktree-policy-guard.mjs`.**
   `runTrunkBash` calls `queryBranch(projectDir)` (spawning `/bin/sh -c "git branch --show-current"` in the trunk checkout) before `findDisallowedTrunkSegment`. Yet `findDisallowedTrunkSegment` is pure and returns `null` for every command running inside `.worktrees/<branch>` and every command on `trunk_bash_allowlist`. Likewise, `run` for `Edit`/`Write`/`MultiEdit`/`apply_patch` calls `queryBranch(projectDir)` before checking whether the edited paths are inside `.worktrees/` (`isWorktreeRelPath`) or Tier 1 (`classifyTier !== 3`). Every allowed worktree or trunk tool call therefore spawns an unnecessary `git` subprocess in the trunk repository.
2. **Repeated heredoc lexing, eager registry deep-cloning, and redundant trunk `git` on worktree commits.**
   - `.cli/hooks/commit-guard.mjs` calls `safeGit('branch --show-current', projectDir)` in `resolveDecisionContext` even when `isWorktree` or `isCreatePrActive` is true, where `decide` returns `passthrough` without reading `branch`.
   - `commit-guard.mjs` and `.cli/hooks/destructive-git-guard.mjs` re-run `stripHeredocData`/`inspectableCommand` 7–8 times on the same `command` string during one hook call, and `destructive-git-guard.mjs` reads the lease file on disk before checking `isCreatePrSafeCommand(command)`.
   - `.cli/lib/hook-registry-overlay.mjs` (`loadMergedHookRegistry`) deep-clones the 1,062-line `MANAGED_HOOK_REGISTRY` via `JSON.parse(JSON.stringify(...))` at module import time in every hook process even when the optional local overlay (`hook-registry.local.json`) is absent.
3. **5-second hook timeouts kill `.cli/hooks/` guards under parallel multi-CLI build load.**
   `.claude/hooks.json`, `.codex/hooks.json`, `.claude/settings.json`, and `.cli/lib/hook-registry.mjs` configure `5` seconds on `.cli/hooks/` `PreToolUse`, `PostToolUse`, and `PermissionRequest` entries, and `.claude/hooks.json`/`.codex/hooks.json` keep `5` seconds on `SessionEnd` even after change 201 raised `releaseLeases`'s git timeout inside `worktree-owner-tracker.mjs` to 8,000 ms (and `.claude/settings.json`'s `SessionEnd` to 10 s). Under parallel `cargo` builds, four concurrent `node` hooks exceed 5 s and are killed.
4. **Cold worktree builds when `main/target/debug/deps` is absent, and unpruned/locked clones.**
   `scripts/seed-target.sh` only checks `$main_root/target/debug/deps`. When `main` has no debug cache (or was cleaned) while a sibling worktree under `$main_root/.worktrees/` already has a warm `target/debug/deps`, the new worktree compiles all of `Cargo.lock` cold (16m29s). And when `main/target/debug` is cloned, `seed-target.sh` does not prune obsolete test binaries from `main/target` first and copies `.cargo-lock` into the new worktree.
5. **Three macOS CI jobs per branch push exhaust the 5-runner concurrency limit across parallel worktrees.**
   `.github/workflows/ci.yml` runs `package` (`bash scripts/package-macos.sh`, which re-runs `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, and a full `cargo build --release --locked`) on every `feature/**` and `fix/**` push. Two parallel worktree pushes start 6 macOS jobs and queue behind GitHub's 5-runner macOS limit, even though `checks` and `test` already run fmt, clippy, and the packager tests on every branch and only `main` ships the `.app` bundle.

## The fix

1. In `.cli/hooks/worktree-policy-guard.mjs`:
   - In `runTrunkBash`, evaluate `findDisallowedTrunkSegment` first and return `{ exitCode: 0 }` when it returns `null`; query `branch = queryBranch(projectDir)` and `shouldEvaluateTiers(branch, policy)` only when a disallowed segment is found.
   - In `run` (`Edit`/`Write`/`MultiEdit`/`apply_patch`), when `isPolicyUsable(policy)` is true, classify `filePaths` first and return `{ exitCode: 0 }` when no path is a non-worktree Tier-3 file inside `projectDir`; call `queryBranch(projectDir)` and `shouldEvaluateTiers(branch, policy)` only when a Tier-3 file is found or the policy is unusable.
2. In `.cli/hooks/commit-guard.mjs`, `.cli/hooks/destructive-git-guard.mjs`, and `.cli/lib/hook-registry-overlay.mjs`:
   - In `commit-guard.mjs`, compute `isCreatePrActive` and `isWorktree` before `branch`, and skip `safeGit('branch --show-current', projectDir)` when `isForeignRepo || isCreatePrActive || isWorktree`.
   - Memoize the last `(command -> result)` in `stripHeredocData` and `inspectableCommand` in `commit-guard.mjs` and `destructive-git-guard.mjs`, and check `isCreatePrSafeCommand(command)` before `isCreatePrLeaseActive(projectDir)` in `destructive-git-guard.mjs`.
   - In `loadMergedHookRegistry` (`.cli/lib/hook-registry-overlay.mjs`), return `registry: managed` without `cloneRegistry(managed)` when `!read.overlay` (`mergeHookRegistryOverlay` itself still clones for direct callers).
3. In `.claude/hooks.json`, `.codex/hooks.json`, `.claude/settings.json`, and `.cli/lib/hook-registry.mjs`:
   - Raise `.cli/hooks/` `PreToolUse`, `PostToolUse`, and `PermissionRequest` timeouts from `5` to `30` seconds, and raise `SessionEnd` in `.claude/hooks.json`, `.codex/hooks.json`, and `.cli/lib/hook-registry.mjs` from `5` to `10` seconds to match `.claude/settings.json`'s 10-second `SessionEnd` budget over `releaseLeases`'s 8,000 ms git timeout.
4. In `scripts/seed-target.sh`:
   - When `$main_root/target/debug/deps` is absent, fall back to a warm sibling worktree's `target/debug` under `$main_root/.worktrees/` that has `deps/`.
   - When cloning from `$main_root/target/debug`, run `scripts/prune-target.sh` in `$main_root` first so obsolete test binaries are not cloned, and remove `$worktree/target/debug/.cargo-lock` after cloning.
5. In `.github/workflows/ci.yml`:
   - Add `if: github.ref == 'refs/heads/main'` to the `package` job so feature and fix branches run `checks` and `test` (2 macOS jobs per branch instead of 3), while `main` continues to run `checks`, `test`, and `package`.

## Files that change

| File | Change |
|---|---|
| `.cli/hooks/worktree-policy-guard.mjs` | check `findDisallowedTrunkSegment` and Tier-3 paths before `queryBranch(projectDir)` |
| `.cli/hooks/commit-guard.mjs` | skip trunk `branch --show-current` when `isWorktree` or `isCreatePrActive` is true; memoize `stripHeredocData` and `inspectableCommand` |
| `.cli/hooks/destructive-git-guard.mjs` | memoize `stripHeredocData` and `inspectableCommand`; check `isCreatePrSafeCommand` before `isCreatePrLeaseActive` |
| `.cli/lib/hook-registry-overlay.mjs` | return `managed` without deep-cloning in `loadMergedHookRegistry` when `!read.overlay` |
| `.claude/hooks.json`, `.codex/hooks.json`, `.claude/settings.json`, `.cli/lib/hook-registry.mjs` | 30s timeout on `.cli/hooks/` tool guards; 10s timeout on `SessionEnd` |
| `scripts/seed-target.sh` | fall back to a warm sibling worktree's `target/debug`, prune `main/target` before cloning `main`, and remove `.cargo-lock` from the clone |
| `.github/workflows/ci.yml` | run `package` job on `refs/heads/main` only |
| `src/tests.rs` | guard tests for zero-git fast paths on worktree/allowlisted calls, hook timeouts across all three CLIs, sibling worktree `seed-target.sh` fallback and `.cargo-lock` removal, and `ci.yml` `package` job gating |
| `docs/sdlc/lessons.md` | entry 100 |

## Order of work

1. `state.yaml` and `plan.md`; fresh evaluator subagent review and `approvals.log`.
2. Guard tests in `src/tests.rs`, watched failing on the unedited tree.
3. Implement the fixes across `.cli/`, `.claude/`, `.codex/`, `scripts/seed-target.sh`, `.github/workflows/ci.yml`, and `docs/sdlc/lessons.md`.
4. Run the named tests and `mutations.py` (`scripts/mutation_sweep.py`).
5. Run required reviewers (`rust-reviewer` and `subprocess-safety-reviewer`), record `review.yaml`, commit, push, verify CI, merge into `main`, and clean up the worktree.

## What this is not

- **A weakening of any guard verdict.** Every command or file edit that was denied, asked about, or failed-closed before is still denied, asked about, or failed-closed with the identical message. Calling `queryBranch(projectDir)` after `findDisallowedTrunkSegment` / Tier-3 classification only skips `git branch --show-current` when the outcome is `allow` on every branch (including `main` and `null`).
- **A skip of CI tests on branches.** `checks` (`cargo fmt --check`, `cargo clippy --locked -- -D warnings`, hook checks, harness metrics, control bands) and `test` (`cargo test --locked` with `RUSTFLAGS: -D warnings`, which includes the packager and channel tests) still run on every branch push; only the redundant `package` release build is restricted to `main`.

## Departures from the plan

- Memoized `splitHeredocs` and `expandingHeredocBodies` inside `.cli/lib/heredoc-strip.mjs` (the shared module imported by both `commit-guard.mjs` and `destructive-git-guard.mjs`) rather than duplicating a wrapper in each hook.
