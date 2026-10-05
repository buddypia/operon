# Plan: AI IDE Workflow Features

- **Spec**: `./spec.md`
- **Approved**: 2026-10-04
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/config.rs` | Declare `.operon/run.sh` path and checkpoint git ref prefix constants |
| `src/models.rs` | Add `LandResult`, `Checkpoint`, `ExternalEditor`, `GitHubIssue`, `PromptTemplate` |
| `src/git.rs` | Implement `git_land_worktree`, `git_create_pull_request`, `git_create_checkpoint`, `git_restore_checkpoint`, `git_diff_checkpoint`, `list_github_issues` |
| `src/git/setup.rs` | Add `RunScript`, `read_run_script`, and approval persistence |
| `src/util.rs` | Add `open_in_external_editor` helper |
| `src/glyphs.rs` | Verify and export required Phosphor icons |
| `src/i18n_tables.rs` | Add translations for all new Japanese, English, and Korean UI messages |
| `src/app.rs` | Add background task handlers and state tracking for land, run scripts, checkpoints, PRs, and issues |
| `src/app/screens.rs` | Render UI buttons, confirmation sheets, and external editor menu |
| `src/tests.rs` | Add comprehensive unit and integration tests |
| `README.md`, `README.ja.md`, `README.ko.md` | Document new workflow features across all 3 languages |

## Order of work

1. Update `src/config.rs` and `src/models.rs` with necessary types and constants.
2. Implement backend Git functions in `src/git.rs` (`git_land_worktree`, checkpoints, PR creation, issue listing).
3. Implement `src/git/setup.rs` run script discovery and approval.
4. Add `open_in_external_editor` in `src/util.rs`.
5. Add i18n strings in `src/i18n_tables.rs`.
6. Integrate UI handlers and screens in `src/app.rs` and `src/app/screens.rs`.
7. Write unit and regression tests in `src/tests.rs`.
8. Update documentation in `README.md`, `README.ja.md`, `README.ko.md`.
9. Verify with the quality gate: `cargo fmt --check`, `cargo test --locked`, `cargo clippy --locked -- -D warnings`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Dirty working tree merge | Git merges uncommitted edits into mainline | `git_land_worktree` explicitly checks `git status --porcelain` and rejects dirty trees |
| Merge conflict leaves corrupted state | Partial merge state in main | Conflict executes `git merge --abort` immediately |
| Editor command failure | Nothing happens when clicking editor | Checks exit code and returns a user notice |
| Checkpoint ref clutter | Stray refs in repo | Checkpoints namespaced cleanly in `refs/operon/checkpoints/` |

## Proof of completion

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 656+ passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no output past the compile lines.
- Unit tests verify worktree landing, checkpoints, PR creation, and run script detection.

## Departures from the plan

None.
