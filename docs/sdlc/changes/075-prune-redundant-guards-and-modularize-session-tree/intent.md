# Intent: prune redundant harness guards, resolve policy deadlocks, and modularize session tree

- **Status**: approved
- **Opened**: 2026-09-24

## Problem

1. **Harness over-engineering and policy deadlock**:
   - `AGENTS.md` directs post-build cleanup of obsolete packages and backups using `trash dist/...`. However, `.claude/config/worktree-policy.json`'s `trunk_bash_allowlist` lacked `trash`, causing policy rejection on trunk.
   - Direct edits to agent coordination directories (`.agents/**`) were classified as Tier 3 and blocked on trunk because `.agents/**` was omitted from `tier1_main_allowed.patterns`.
   - Redundant/dormant guards (`.cli/hooks/milestone-deck-warning.mjs`, `.cli/hooks/pre-ship-review-guard.mjs`, `.cli/hooks/trunk-start-warning.mjs`) added cognitive and execution overhead without providing irreplaceable protection.
2. **Code Monolith in screens.rs**:
   - `src/app/screens.rs` expanded beyond 5,200 lines following recent additions of the session file tree UI and 3-column layout computations.

## Desired Outcome

- `trash` is allowed on trunk in `trunk_bash_allowlist.patterns`.
- `.agents/**` is allowed in `tier1_main_allowed.patterns`.
- Redundant guard scripts are safely removed and deregistered across `.claude/hooks.json`, `.claude/settings.json`, `.codex/hooks.json`, and `.cli/lib/hook-registry.mjs`.
- Critical invariants (`destructive-git-guard.mjs`, `commit-guard.mjs`, `worktree-policy-guard.mjs`, and `.claude/hooks/gate-commit.sh`) remain strictly preserved.
- Session file tree rendering, frame caching, and 3-column layout math are cleanly extracted to `src/ui/session_tree.rs`.
- 100% test compatibility (all tests pass).
