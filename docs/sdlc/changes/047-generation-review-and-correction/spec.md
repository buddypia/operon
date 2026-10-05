# Spec: Generation Review, Evaluation, and Correction Suite

- **Intent**: ./intent.md
- **Status**: approved

## Requirements

1. Diff comments maintain lifecycle state (`sent` and `resolved` boolean flags) persisted in the sidecar storage file, defaulting to `false` for legacy comments.
2. Delivering review comments to an active session sets their `sent` flag to `true` instead of immediately deleting the annotations, ensuring the review feedback remains visible for verification.
3. Each comment card in the diff view provides controls to toggle resolution status ("解決" / "再開") and delete the comment individually.
4. The diff review footer provides a "Markdownをコピー" button that copies all review notes formatted as structured Markdown with file paths, line ranges, and comment text to the clipboard.
5. The diff review footer provides an action to clear resolved comments ("解決済みを消去") while retaining active unaddressed comments, as well as clearing all comments ("すべて消す").
6. The diff header provides an "AIレビュー" button that runs a local agent CLI (Claude Code, Codex, or Antigravity) in read-only background mode to evaluate the current diff for edge cases, bugs, regressions, and test omissions.
7. The AI review output is displayed in a collapsible findings panel above the diff, with actions to dismiss, copy, or refresh the review.
8. When a git commit fails (such as pre-commit hooks, linting, or gate refusals), Operon sanitizes the failure output, extracts meaningful failure context, and surfaces a "AIで修正" button in the commit panel.
9. Clicking "AIで修正" constructs a structured recovery prompt containing the branch, staged files, commit message, and sanitized error trace, sending it directly to the active session's terminal.
10. All user-facing strings are in Japanese and registered in `src/i18n_tables.rs`.

## Behaviour

- **Review notes persistence**: When the user adds comments on diff lines and clicks "セッションにまとめて送る", the comments are formatted and delivered to the active agent session as before. Instead of disappearing, the comment cards remain in place marked with a subtle badge "送信済み" (Sent).
- **Verification & Resolution**: When the agent revises code and the diff updates, the user re-examines each comment. Clicking "解決" collapses or marks the note with a "解決済み" (Resolved) badge. Clicking "削除" removes that note.
- **Copying Notes**: Clicking "Markdownをコピー" in the diff review footer serializes all current comments into a clean Markdown note summary and writes it to the system clipboard, displaying a confirmation notice "レビューノートをクリップボードにコピーしました。".
- **AI Diff Review**: Clicking "AIレビュー" in the diff header triggers a background task that inspects the staged or unstaged diff and prompts the configured CLI agent. While running, a spinner and label "AIが変更をレビュー中…" are shown. On completion, an expandable panel displays the structured findings (e.g. Overview, Potential Bugs, Missing Tests, Security Considerations).
- **Commit Failure Recovery**: If `git commit` fails (e.g. `.claude/hooks/gate-commit.sh` fails with test errors), the commit section renders an alert with the error summary and an actionable button "AIで修正". Clicking it sends a specialized prompt to the agent instructing it to resolve the specific failure, fix code/tests, and verify status.

## Design

- `src/git/comments.rs`:
  - Extend `DiffComment` with `pub(crate) sent: bool` and `pub(crate) resolved: bool` using `#[serde(default)]` for backwards-compatible serialization.
  - Implement `format_diff_comments_markdown(comments: &[(DiffComment, bool)]) -> String` for clipboard export.
  - Implement methods on `DiffAnnotations`: `mark_sent(&mut self, project: Uuid)`, `toggle_resolved(&mut self, project: Uuid, file: &str, line: usize)`, `remove(&mut self, project: Uuid, file: &str, line: usize)`, and `clear_resolved(&mut self, project: Uuid)`.
- `src/git.rs`:
  - Add `ai_diff_review_prompt(branch: &str, files: &[ChangedFile], patch: &str) -> String` to generate an objective, senior-level code review prompt.
  - Add `clean_generated_ai_review(raw: &str) -> String` to sanitize reviewer response.
  - Add `summarize_commit_failure(error: &str) -> String` and `build_fix_commit_failure_prompt(branch: &str, files: &[String], commit_message: &str, error: &str) -> String` following the usual recovery prompt patterns.
- `src/app.rs`:
  - Add state fields:
    - `pub(crate) diff_ai_review: HashMap<Uuid, String>`
    - `pub(crate) last_commit_failure: HashMap<Uuid, (String, Vec<String>, String)>`
  - Implement methods:
    - `request_ai_diff_review(&mut self, project: &Project)`
    - `fix_commit_failure_with_ai(&mut self, project: &Project)`
    - `copy_diff_comments_markdown(&mut self, project: Uuid)`
  - Update `send_diff_comments` to mark annotations as `sent` rather than discarding them.
- `src/app/screens.rs`:
  - Update `ui_diff_comment_footer` to display count breakdowns (total, sent, resolved) and actions ("セッションに送る", "Markdownをコピー", "解決済みを消去", "すべて消す").
  - Update `ui_git` to draw the "AIレビュー" button and findings card in the diff viewer.
  - Update `ui_commit_section` to display the "AIで修正" button when `last_commit_failure` contains an error for the project.
- `src/ui/diff.rs`:
  - In `render_comment_row`, draw badges for "送信済み" and "解決済み", and buttons for resolution toggle and individual deletion.
- `src/i18n_tables.rs`:
  - Register all new Japanese UI strings across the translation tables.

## Policy conformance

| Policy | Applies? | How this change satisfies it |
|---|---|---|
| Colour — a new role means three palette rows plus a `DESIGN.md` entry, and WCAG 2.1 AA against every surface | Applies | Uses existing `Palette` roles (`accent`, `card`, `success`, `warning`, `text_muted`, `text_strong`) without raw color literals. |
| Icons — a new mark means an `ICON_*` constant in `src/glyphs.rs` and an `ICON_VOCABULARY` entry | Applies | Reuses existing registered icons `ICON_COMMENT`, `ICON_CHECK`, `ICON_TRASH`, `ICON_COPY`, `ICON_REFRESH`, and `ICON_WARN`. |
| Identifier SSOT — a string two places must agree on lives in `src/config.rs`, and the round trip is what gets tested | Does not apply | No shared external string identifiers are created; all constants are local to git review modules. |
| Durability — see `.claude/skills/durability-invariants/SKILL.md`; schema change means a `STORE_SCHEMA_VERSION` bump | Does not apply | Diff comments are stored in the sidecar `diff-comments.json` with serde defaults, leaving `store.json` schema untouched. |
| Subprocess safety — new children go through `src/exec.rs`; new launch paths through the gates in `src/agents.rs` | Applies | AI diff review uses `commit_message_agent(&self.tools)` and `run_command_with_output_limit` with standard timeout and memory bounds. |
| Documentation — user-facing docs change in all three languages together | Does not apply | Changes are internal UI enhancements within the diff and review workflows. |
| Local-first — no telemetry, accounts, cloud calls, or new `unsafe` | Applies | Operates strictly locally using local git commands, local tmux sessions, and local CLI tools; zero network traffic. |
| Budgets — any new scan or output path states its byte and item ceiling | Applies | AI diff review output is bounded by `DIFF_COMMENT_MESSAGE_MAX_BYTES` (32 KB); prompt diffs are bounded to 100 KB. |

## Flagged concerns

- **Diff comment sidecar backwards compatibility with older versions** — The existing JSON file `diff-comments.json` contains comments without `sent` or `resolved` keys. By annotating the newly added struct fields with `#[serde(default)]`, any existing file continues to deserialize seamlessly without errors or data loss.
- **Subprocess timeout handling during AI diff review** — Large repository diffs could cause background CLI review execution to stall. We enforce a strict 45-second execution timeout using `run_command_with_output_limit` and truncate diffs exceeding 100 KB so CLI evaluation remains responsive and never freezes the application.

## Acceptance

- `cargo test --locked` passes, including unit tests verifying `DiffComment` serialization defaults, comment lifecycle state changes, Markdown formatting, and failure recovery prompt generation.
- `cargo clippy --locked -- -D warnings` and `cargo fmt --check` pass cleanly.
- Adding a comment on a diff line, sending it, and verifying that the comment stays visible with the "送信済み" badge.
- Clicking "解決" toggles the comment status to "解決済み", and clicking "Markdownをコピー" writes structured Markdown notes to the clipboard.
- Triggering "AIレビュー" runs a local CLI agent in the background and populates the review findings panel.
- Inducing a git commit failure displays the "AIで修正" button which delivers a formatted recovery prompt to the agent session.

## Rejected alternatives

- Automatically deleting review comments immediately upon delivery was rejected because human developers cannot verify whether the agent's revised code satisfies the comment without persistent reference notes.
- Opening an external browser tab for code review was rejected because Operon is a local-first desktop application whose primary value is fast, self-contained iteration within git worktrees.
