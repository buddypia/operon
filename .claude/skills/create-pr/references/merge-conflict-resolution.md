# Merge Conflict Resolution (5-Step Protocol)

> **Scope**: `create-pr` Mode A/B multi-worktree environments and superset warning reconciliation. Retains the "never abort" principle without proposing commands blocked by `.cli/hooks/destructive-git-guard.mjs`.

## 5-Step Protocol

### 1. Understand Current State

```bash
git status
git diff --name-only --diff-filter=U
git log --oneline --graph -10
```

### 2. Investigate Original Intent for Each Conflict Hunk

Do NOT guess. Inspect commit history to understand why each side changed:

```bash
git log -p --follow -- <conflicted-file>
git log <ours-branch>..<theirs-branch> --oneline -- <conflicted-file>
git show <commit-sha>
```

### 3. Resolve Each Hunk — Preserve Both Intents

- **Preserve both intents** whenever possible rather than picking one side mechanically.
- If mutually exclusive, select the approach aligned with the stated goal and document the trade-off in the commit message.
- **Never invent new behavior** unrelated to the original changes.
- **Always resolve; never flee** via `git merge --abort`.

### 4. Execute Automated Verification Checks

Run the project's verification suite (`make q.fast` / `make q.check`) and ensure existing tests pass.

### 5. Complete Merge

```bash
git add <resolved-files>
git commit
```

## Caveats & Multi-Worktree Reconciliation

- `.cli/hooks/destructive-git-guard.mjs` blocks `git rebase`, `git reset --hard`, `git checkout .`, `git clean -f`, and `git push --force`.
- When reconciling with `origin/main`:
  1. `git fetch origin`
  2. `git merge origin/main` into the worktree branch and resolve conflicts with the 5-step protocol above.
  3. Re-run verification checks after resolution.
