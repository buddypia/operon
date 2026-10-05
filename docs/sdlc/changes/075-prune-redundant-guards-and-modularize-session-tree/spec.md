# Spec: prune redundant harness guards, resolve policy deadlocks, and modularize session tree

- **Status**: approved
- **Opened**: 2026-09-24

## Specification

### 1. Harness Policy & Guard Optimization (R1)
- Add `trash` to `trunk_bash_allowlist.patterns` in `.claude/config/worktree-policy.json`.
- Add `".agents/**"` to `tier1_main_allowed.patterns` in `.claude/config/worktree-policy.json`.
- Remove:
  - `.cli/hooks/milestone-deck-warning.mjs`
  - `.cli/hooks/pre-ship-review-guard.mjs`
  - `.cli/hooks/trunk-start-warning.mjs`
- Clean up registrations in `.claude/hooks.json`, `.claude/settings.json`, `.codex/hooks.json`, and `.cli/lib/hook-registry.mjs`.
- Add tests in `src/tests.rs` ensuring deleted hooks are not referenced and `trash` is permitted on trunk while unlisted commands (`trashcan`) are denied.

### 2. Session File Tree Modularization (R2)
- Create `src/ui/session_tree.rs` containing `SessionColumnsLayout::compute` and `OperonApp::ui_session_file_tree`.
- Delegate layout calculation and tree UI in `src/app/screens.rs` to the new module.
- Retain full functional equivalence: ⌘+click file navigation, line jump, responsive collapse toggle, and error handling.
- Add unit tests for responsive layout thresholds, sub-pixel floats, and IEEE-754 non-finite float boundary conditions.
