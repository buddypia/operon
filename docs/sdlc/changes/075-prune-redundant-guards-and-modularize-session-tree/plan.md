# Plan: prune redundant harness guards, resolve policy deadlocks, and modularize session tree

- **Status**: approved
- **Opened**: 2026-09-24

## Plan

1. Modify `.claude/config/worktree-policy.json` to allow `trash` and `.agents/**`.
2. Delete redundant hooks `.cli/hooks/{milestone-deck-warning,pre-ship-review-guard,trunk-start-warning}.mjs`.
3. Remove registrations in `.claude/hooks.json`, `.claude/settings.json`, `.codex/hooks.json`, and `.cli/lib/hook-registry.mjs`.
4. Extract session file tree to `src/ui/session_tree.rs` and simplify `src/app/screens.rs`.
5. Add unit and regression tests in `src/ui/session_tree.rs` and `src/tests.rs`.
6. Run quality gates (`cargo fmt`, `cargo test`, `cargo clippy`).
