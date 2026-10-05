# Intent: agent code generation lacks persistent review verification, automated evaluation, and failure recovery

- **Status**: approved
- **Opened**: 2026-09-15

## Problem

When coding agents generate or modify code, human developers must evaluate, review, and correct the outputs. A complete solution provides a review loop (line-by-line annotations that persist across revisions, per-note resolution, markdown export), automated evaluation capabilities, and failure recovery actions ("Fix with AI" when git commit hooks or checks fail).

Today in Operon, the review and correction loop has three major friction points:
1. Sending review comments unconditionally wipes out all annotations immediately, making it impossible to verify whether the agent actually fixed the reported issues upon revision ("Reply, resolve, re-review" loop is broken).
2. Users cannot copy or export review notes into structured Markdown to share or feed across different CLI agents for second opinions.
3. There is no automated evaluation mechanism for diffs, and when a git commit fails (due to linter, clippy, or pre-commit hook refusals), the user is left to manually copy raw error traces and compose a fix prompt by hand.

## Who feels it, and when

Anyone reviewing agent-generated code changes in a worktree, particularly when:
- Reviewing a complex multi-file diff where review comments must remain visible as reference while the agent revises code.
- Wanting an automated quality assessment or second-opinion review on a diff before committing.
- Running into git pre-commit hook failures or test failures during commit and needing the active agent to fix the failure immediately.

## Desired outcome

Developers have a complete, resilient evaluation, review, and correction workflow:
1. Review annotations persist across iterations with clear "sent" and "resolved" states, allowing developers to verify fixes against original notes, resolve individual comments, and export or copy notes as Markdown.
2. An automated AI diff review can evaluate the current changeset in the background, surfacing edge cases, potential regressions, and test gaps.
3. When git commit fails (such as pre-commit hooks or format checks), a single click generates a structured recovery prompt with sanitized error details and delivers it to the agent for immediate remediation.

## Constraints this change inherits

- macOS only; eframe/egui 0.31 immediate-mode GUI.
- Local-first: no telemetry, no external cloud dependencies; uses locally installed agent CLIs (Claude Code, Codex, Antigravity).
- User-facing text is Japanese; code, comments, and docs are English.
- No new external crates.
- Bounded subprocess execution with strict timeouts and output caps.

## Systems likely affected

Git diff review, diff annotations, application screens, review footers, and subprocess execution.

## Open questions

- Should review notes be deleted or kept when sent to the agent?
  Answered: Notes should remain preserved with their status updated to "sent", allowing the user to resolve them individually or clear them when satisfied.
- How should commit failure recovery be triggered?
  Answered: By displaying a prominent "Fix with AI" button in the commit panel whenever a commit operation fails, pre-populating a structured prompt with sanitized failure output.

## Not in scope

- Web browser integration or remote cloud-hosted PR sync (GitHub/GitLab API integration).
- Multi-worktree automated prompt fanout racing (reserved for a dedicated worktree comparison feature).
