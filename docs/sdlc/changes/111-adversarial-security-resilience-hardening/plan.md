# Plan: Adversarial Security, Resilience, and Performance Hardening

- **Spec**: `./spec.md`
- **Approved**: 2026-10-04
- **Status**: in progress

## Files that change

| File | Change |
|---|---|
| `src/git.rs` | Refactor `git_land_worktree` (mainline checkout check, `--` arg separation), `git_create_checkpoint` (isolated temporary index, RAII guard), `git_diff_checkpoint` (1MB limit), replace raw `command_output_in_dir("git", ...)` with `git_output`/`git_command` |
| `src/git/setup.rs` | Add `decide_run_script_run` implementing cryptographic TOCTOU check against previewed digest |
| `src/util.rs` | Implement `open -a` fallback in `open_in_external_editor` for macOS GUI environment |
| `src/app/screens.rs` | Enforce TOCTOU check in `ui_run_script_modal`; eliminate per-frame sync disk I/O in `ui_launch_form` |
| `src/i18n_tables.rs` | Add any new error/notice messages in JA/EN/KO preserving UTF-8 byte ordering |
| `src/tests.rs` | Add unit tests verifying mainline checkout check, TOCTOU defense, isolated checkpoint capture, and `--` option defense |

## Order of work

1. Implement `decide_run_script_run` in `src/git/setup.rs`.
2. Refactor `src/git.rs` with defensive Git operations, isolated temporary index for checkpoints, and strict ref name validation.
3. Update `src/util.rs` with macOS application bundle fallback for external editors.
4. Update `src/app/screens.rs` to integrate TOCTOU verification in `ui_run_script_modal` and cache prompt templates in `ui_launch_form`.
5. Update `src/i18n_tables.rs` with translations for any new notices.
6. Add unit tests in `src/tests.rs` covering all hardened security and resilience invariants.
7. Run and pass all quality gates (`cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked`).

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Main checkout not on mainline | Worktree changes merged into wrong branch | `git_land_worktree` checks `rev-parse --abbrev-ref HEAD == mainline` before merge |
| Run script modified during approval | Rogue code executed behind preview | `decide_run_script_run` compares sha256 before launch and blocks mismatch |
| Unstaged edits lost in rollback | Checkpoint fails to capture uncommitted files | Isolated temporary index `git add -A` captures entire working tree |
| Slow storage freezes UI | Stutter in launch form | Prompt templates loaded on menu click or cached outside draw loop |

## Proof of completion

- `cargo fmt --check` — passes.
- `cargo test --locked` — all tests pass (0 failures).
- `cargo clippy --locked -- -D warnings` — 0 warnings.
- New tests in `src/tests.rs` assert TOCTOU refusal, mainline branch mismatch refusal, and isolated index checkpoint creation.

## Departures from the plan

None.
