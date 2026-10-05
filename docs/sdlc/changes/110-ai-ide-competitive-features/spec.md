# Spec: AI IDE Workflow Features (Worktree Landing, Run Scripts, Checkpoints, PR Creation, External Editors, Issue Import, Prompt Templates)

- **Intent**: `./intent.md`
- **Status**: approved

## Requirements

1. **Worktree Landing**: `git_land_worktree` must merge a clean worktree branch into the project's mainline using `git merge --no-ff`. It must refuse if the worktree has uncommitted changes, if the mainline checkout is dirty, if there are 0 commits ahead, or if the agent is actively working. If merge conflicts occur, it must execute `git merge --abort` and return the conflict details. On successful merge, it offers to remove the worktree and delete the merged branch with `git branch -d`.
2. **Run Scripts**: The app must discover `.operon/run.sh` in the project or worktree. A dedicated Run button in the UI requests approval showing preview and digest (following the `.operon/setup.sh` pattern) and launches the script in an isolated tmux session. Clicking while running stops the session.
3. **PR Creation**: `git_create_pull_request` runs `gh pr create` with specified or drafted title, body, and draft status.
4. **Checkpoints & Rollback**: `git_create_checkpoint` saves a snapshot of the worktree state to `refs/operon/checkpoints/<session_id>/<turn_index>` on prompt submission or turn start. `git_restore_checkpoint` restores files from a chosen checkpoint. `git_diff_checkpoint` shows changes since that checkpoint.
5. **External Editor Launch**: Provide actions to open the worktree or project folder in external applications: VS Code (`code`), Cursor (`cursor`), Zed (`zed`), or macOS Finder (`open`).
6. **Prompt Templates**: Scan `.operon/prompts/*.md` (and built-in default templates) and allow quick insertion into the session prompt composer.
7. **GitHub Issue Import**: Query repository issues via `gh issue list --json number,title,body,labels` and allow selecting an issue to prefill the session request and branch name.

## Behaviour

- In worktree sessions and the Project Worktrees view, a "main にマージ" (Land) button is visible. Clicking it opens a confirmation dialog showing the target mainline branch and the worktree branch. If requirements are not met (e.g. uncommitted changes exist), a clear Japanese notification explains the refusal.
- If `.operon/run.sh` exists, a "スクリプト実行" / "Run" button appears in the toolbar/header. On first click, the approval dialog presents the script preview and line count. Once approved, it runs in a tmux session named `operon-run-<project/worktree>`.
- In diff review or worktree headers, a "PR を作成" (Create PR) button opens a modal allowing AI drafting of PR title/body and one-click submission via `gh`.
- In the prompt timeline navigation, each turn displays a "復元" (Restore) and "差分" (Diff) option for turn-level checkpoints.
- In session and project menus, an "エディタで開く" (Open in Editor) submenu lists available editors (VS Code, Cursor, Zed, Finder).
- In the new session sheet, an "Issue から作成" (Create from Issue) button allows picking from open GitHub issues.
- All strings are provided in Japanese via `tr()` with entries in `src/i18n_tables.rs` for ja, en, and ko.

## Design

- `src/git.rs`:
  - `git_land_worktree(project: &Path, worktree: &Path, branch: &str) -> Result<LandResult>`
  - `git_create_pull_request(path: &Path, title: &str, body: &str, draft: bool) -> Result<String>`
  - `git_create_checkpoint(path: &Path, session_id: Uuid, turn: usize, msg: &str) -> Result<String>`
  - `git_restore_checkpoint(path: &Path, commit_sha: &str) -> Result<()>`
  - `git_diff_checkpoint(path: &Path, commit_sha: &str) -> Result<String>`
  - `list_github_issues(path: &Path) -> Result<Vec<GitHubIssue>>`
- `src/git/setup.rs`:
  - Add `RUN_SCRIPT_RELATIVE_PATH = ".operon/run.sh"` and `read_run_script` mirroring `SetupScript`.
- `src/util.rs`:
  - `open_in_external_editor(path: &Path, editor: ExternalEditor) -> Result<()>`
- `src/models.rs`:
  - Data structures: `LandResult`, `GitHubIssue`, `Checkpoint`, `ExternalEditor`, `PromptTemplate`.
- `src/app.rs` / `src/app/screens.rs`:
  - Integrate UI buttons, confirmation modals, background tasks for land, run script, PR creation, and issue fetching.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | No | Reuses existing palette semantic roles (Accent, Subtle, Foreground, Background). |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Yes | Uses existing Phosphor glyphs in `src/glyphs.rs` (`ICON_GIT_MERGE`, `ICON_PLAY`, `ICON_STOP`, `ICON_CHECKPOINT`, `ICON_EXTERNAL_LINK`, `ICON_ISSUE`). |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Yes | Script paths (`.operon/run.sh`) and ref namespaces (`refs/operon/checkpoints`) declared in `src/config.rs`. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Yes | Checkpoints are saved in Git references (`refs/operon/checkpoints/`), leaving the SQLite/JSON metadata store untouched, avoiding schema migration risk. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Yes | All `git` and `gh` subprocess invocations use `command_output_in_dir` or `run_command_with_timeout` from `src/exec.rs`. |
| Documentation — user-facing docs change in all three languages together | Yes | `README.md`, `README.ja.md`, and `README.ko.md` updated in tandem. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Yes | Zero network/cloud calls (except standard user-initiated `gh` CLI commands); strictly zero `unsafe`. |
| Budgets — any new scan or output path states its byte and item ceiling | Yes | Issue scans capped at 30 items; diffs capped at existing diff byte budgets; script preview capped at 16 KiB. |

## Flagged concerns

None.

## Acceptance

- `cargo fmt --check` clean.
- `cargo test --locked` passes, including new tests covering `git_land_worktree`, checkpoints, run script approval, external editors, and issue parsing.
- `cargo clippy --locked -- -D warnings` clean.
- All new features operate cleanly without panics, respecting all safety and confirmation guardrails.

## Rejected alternatives

- Cloud agent orchestration: Rejected to maintain strict 100% local-first privacy.
- Automatic force push or blind merge: Rejected to prevent any risk of data loss.
