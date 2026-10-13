# Branch Completion Options

> Scope: `create-pr` Mode B branch-finishing UX principles.

## Completion Options

Confirm verification evidence first, then present completion options:

| Option | Action | Worktree Handling | Safety Rule |
|---|---|---|---|
| **Create PR** | `create-pr ship-worktree` | Auto-cleanup (or `--no-cleanup`) | Halts if uncommitted changes exist |
| **Local Merge** | Outside `create-pr` flow | Manual cleanup after merge | Sync `main` and pass checks first |
| **Keep / Archive** | No cleanup | Retain worktree as-is | Record status and next actions |
| **Discard** | Destructive | Deletes worktree and branch | Requires typed user confirmation |

## Pre-Check & Prompt

Verify current branch, worktree path, clean status, and test results before prompting:

```text
The worktree is verified. Choose an action:
1. Create PR: Push + create PR via create-pr Mode B
2. Keep: Preserve worktree and branch as-is
3. Discard: Delete worktree/branch after typed confirmation
```

## Do Not

- Do NOT present options before verifying tests.
- Do NOT discard implicitly or use `git reset --hard`, `git clean`, or `git checkout --`.

| Step | Automation |
|---|---|
| Verify tests | `make q.check` or task proving command |
| Push and create PR | `node .claude/scripts/create-pr/ops.mjs ship-worktree ...` |
