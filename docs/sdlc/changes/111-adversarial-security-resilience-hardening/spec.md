# Spec: Adversarial Security, Resilience, and Performance Hardening

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **TOCTOU Elimination in Run Scripts**: `decide_run_script_run(worktree, shown_digest)` must verify that the file on disk matches `shown_digest` at the exact instant the user clicks "承認して実行". If the file changed, return `SetupRunDecision::Changed(new_script)` and update the preview modal without launching. If unreadable, return `SetupRunDecision::Unreadable`.
2. **Subprocess Git Isolation**: Every Git operation in `src/git.rs` must use `git_command(path)` and `git_output(path, args)` to ensure `forget_inherited_repository` removes `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_PREFIX`, and `GIT_COMMON_DIR`.
3. **CLI Option Injection Defense**: All Git commands accepting ref, commit, or branch arguments must pass `--` before user-derived arguments to prevent option injection. Ref names must be validated by `is_safe_ref_name` against leading `-` and control characters.
4. **Mainline Checkout Verification**: In `git_land_worktree`, before attempting `git merge`, verify that `current_branch == mainline`. If the primary checkout is detached or on another branch, refuse with a clear Japanese error instructing the user to checkout mainline first.
5. **Isolated Index Checkpointing**: `git_create_checkpoint` must use a temporary isolated index (`GIT_INDEX_FILE = .git/index_checkpoint_<session>_<turn>`) with RAII guard deletion. Run `git add -A -- .` into that temporary index and `git write-tree` to capture all working tree changes without disturbing the user's `.git/index`.
6. **Bounded Subprocess Diff Output**: `git_diff_checkpoint` must use `git_output_allowing_limited` capped at 1 MB to prevent memory exhaustion on large diffs.
7. **Zero Sync Disk I/O in Draw Loop**: In `ui_launch_form`, prompt templates must not scan disk synchronously on every frame. Templates are evaluated only on menu expansion or cached outside the continuous 60fps render path.
8. **macOS GUI Fallback for External Editors**: `open_in_external_editor` must first attempt the direct binary command, and if that fails (e.g. `code` not in minimal GUI `PATH`), fall back to `open -a <ApplicationName>` (`Visual Studio Code`, `Cursor`, `Zed`).

## Behaviour

- When running `.operon/run.sh`, if the file is edited on disk after the modal opens, clicking "承認して実行" displays a notice informing the user that the content changed and presents the updated preview, refusing execution until re-approved.
- When landing a worktree, if the main repository is checked out to any branch other than the mainline branch (e.g. `main`), the operation is refused before any merge command is invoked.
- When creating a checkpoint, all edited files in the worktree are accurately preserved in the checkpoint commit, even if they were never staged with `git add`, and the user's active staging area remains 100% untouched.
- When opening an external editor from Finder-launched Operon, editors installed in standard `/Applications` locations open reliably even without shell PATH configuration.

## Design

- `src/git.rs`:
  - `is_safe_ref_name(name: &str) -> bool`: Validates against leading hyphens, control characters, spaces, and invalid Git ref components.
  - `git_land_worktree`: Check `git_output(project, &["rev-parse", "--abbrev-ref", "HEAD"])? == mainline` before merging. Add `--` before branch arguments in `rev-list`, `merge`, `worktree remove`, and `branch -d`.
  - `git_create_checkpoint`: Create a scoped temporary index file `.git/index_checkpoint_<session>_<turn>`, execute `git add -A -- .` and `git write-tree` targeting the temporary index, and delete the temporary index via RAII drop.
  - `git_diff_checkpoint`: Use `git_output_allowing_limited(worktree, &["diff", "--", commit_sha], &[0, 1], 1024 * 1024)`.
- `src/git/setup.rs`:
  - `decide_run_script_run(worktree: &Path, shown_digest: &str) -> SetupRunDecision`: Verify digest match before execution.
- `src/util.rs`:
  - `open_in_external_editor`: Check path existence, attempt direct binary spawn with 5s timeout, and on failure fall back to `open -a <ApplicationName> <path>`.
- `src/app/screens.rs`:
  - In `ui_run_script_modal`, call `decide_run_script_run`. On `Changed`, update `pending.script` and show notice; on `Unreadable`, clear pending and notify.
  - In `ui_launch_form`, lazily read prompt templates on menu button click or reference cached list to eliminate per-frame filesystem calls.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Subprocess safety | Yes | Strips all inherited repository pointers via `git_command`; applies `--` option terminators; bounds child output. |
| Durability | Yes | Temporary index files use RAII Drop guards to prevent disk leakage; user's `.git/index` is untouched. |
| Budgets | Yes | Checkpoint diffs bounded to 1 MB; script preview bounded to 16 KiB; prompt templates cached. |
| Local-first | Yes | No external network or cloud services; zero `unsafe`. |
| i18n | Yes | All new error messages and notifications provided via `tr()` and `tf!`. |

## Flagged concerns

None.

## Acceptance

- `cargo fmt --check` clean.
- `cargo test --locked` passes, with new tests verifying TOCTOU defense, mainline branch mismatch refusal, isolated checkpoint index capture, and `--` option defense.
- `cargo clippy --locked -- -D warnings` clean.
- Zero per-frame disk I/O in `ui_launch_form`.
- External editors open via bundle name fallback when PATH lacks symlinks.

## Rejected alternatives

- Staging directly with `git add` in the user's `.git/index`: Rejected because it destroys the user's manual staging selections and disrupts work in progress.
- Ignoring TOCTOU on run scripts: Rejected because race-condition payload replacement allows arbitrary unauthorized code execution.
- Blindly merging without checking main checkout HEAD: Rejected because merging into non-mainline branches causes silent repository corruption.
