# Intent: guard updates must not erase Operon's own declarative policies

- **Status**: approved
- **Opened**: 2026-09-23

## Problem

The worktree and commit guards separate standardized harness mechanisms from
project-specific configurations. Changes 058 and 071 had previously edited ten
guard files in place. The trunk command allowlist, the ownership check on every
shell command, the no-remote cleanup, the Stop notice for a merged worktree that
was never removed, the `core.worktree` refusal, and two document fixes all lived
inside individual script files.

To avoid configuration drift, the guard infrastructure now supports declarative
policy switches in `.claude/config/worktree-policy.json`. What is needed is to
move these behaviors cleanly into that policy file and keep the guard mechanisms
standardized.

## Who feels it, and when

Whoever updates the guards, and every agent session after it: a session
on `main` could build and test there again, and a session could run commands in
another session's worktree, with nothing saying the guard had changed.

## Desired outcome

- The bundle is current, and no managed file differs from what the receipt
  records.
- Every behaviour 058 and 071 added still holds, and Operon's specific policies
  live declaratively in `.claude/config/worktree-policy.json`.
- Something in the suite goes red if that configuration is lost.

## Constraints this change inherits

- Standardized guard files maintain consistent policy evaluation across CLIs.
- Claude Code, Codex CLI and Antigravity CLI keep the same guard semantics.
- Local-first: the guards run locally under node, with no network.

## Systems likely affected

The synced `.cli/` hooks and libraries, `.claude/scripts/`, the create-pr
skill, the generated `.claude/hooks.json` and `.codex/hooks.json`,
`.claude/config/worktree-policy.json`, the receipt, and `src/tests.rs`.

## Open questions

None left. The user approved the whole task, including the landing merge on
`main`, before it started.

## Not in scope

- Changing what the guards decide beyond what 058 and 071 already decided.
- Re-trusting the changed Codex hook commands on this machine.
