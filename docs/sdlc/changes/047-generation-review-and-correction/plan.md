# Plan: Generation Review, Evaluation, and Correction Suite

- **Spec**: ./spec.md
- **Approved**: 2026-09-15
- **Status**: approved

This is the plan produced in plan mode and accepted before any file was edited.
If the implementation departs from it, update this file — an abandoned plan is
worse than no plan, because the next reader trusts it.

## Files that change

| File | Change |
|---|---|
| `src/git/comments.rs` | Add `sent` and `resolved` flags to `DiffComment`, Markdown note formatting, and state manipulation methods (`toggle_resolved`, `mark_sent`, `remove`, `clear_resolved`). |
| `src/git.rs` | Add AI diff review prompt generation, commit failure error parsing and prompt builder. |
| `src/app.rs` | Add state and background handlers for AI diff review, commit failure recovery, updated diff comment lifecycle, and clipboard export. |
| `src/app/screens.rs` | Update diff footer (copy notes, clear resolved, sent/resolved counts), diff header (AI review trigger & findings card), and commit panel ("AIで修正" button). |
| `src/ui/diff.rs` | Draw status badges (`[送信済み]`, `[解決済み]`) and action buttons (解決, 削除) on comment cards in diff. |
| `src/i18n_tables.rs` | Add Japanese translations for new actions and badges across ja, en, ko tables. |
| `src/tests.rs` | Add unit and integration tests covering comment lifecycle, Markdown notes formatting, recovery prompt construction, and AI review flow. |

## Order of work

1. Update `src/git/comments.rs` with `sent`, `resolved`, helper methods, and Markdown serialization.
2. Update `src/git.rs` with AI review prompt generation and commit failure prompt construction.
3. Update `src/i18n_tables.rs` with necessary localized strings.
4. Update `src/app.rs` and `src/app/screens.rs` to integrate review state, footer actions, and commit recovery actions.
5. Update `src/ui/diff.rs` to render comment status badges and per-comment controls.
6. Add unit and integration tests in `src/tests.rs` and verify `cargo test --locked`.
7. Verify `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, and bundle packaging.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| Sidecar deserialization breaks for existing comments | Unreadable comments warning dialog appears on startup | `#[serde(default)]` and `keeps_diff_comments_across_a_reload` test in `src/tests.rs` |
| CLI subprocess hangs during AI review | Background task spinner spins indefinitely | 45-second execution timeout in `run_command_with_output_limit` |
| Prompt size exceeds terminal / CLI buffer limits | Agent ignores prompt or throws buffer overflow error | Error trace sanitization, line budgeting, and prompt output length capping |

## Proof of completion

The commands and observations that say this is done:

- `cargo fmt --check` — no output.
- `cargo test --locked` — `test result: ok. 490+ passed; 0 failed; 6 ignored`.
- `cargo clippy --locked -- -D warnings` — no warnings emitted.
- `bash scripts/check-readiness.sh docs/sdlc/changes/047-generation-review-and-correction` — exits with code 0 (Go).
- Verification in the UI:
  - Adding a diff comment, sending to session: comment is marked "送信済み" and remains visible.
  - Clicking "解決" marks it "解決済み", and "Markdownをコピー" copies structured Markdown to clipboard.
  - Clicking "AIレビュー" runs the evaluation in background and displays the findings card.
  - Committing with a failure reveals the "AIで修正" button, which delivers a focused recovery prompt.

## Departures from the plan

None so far.
