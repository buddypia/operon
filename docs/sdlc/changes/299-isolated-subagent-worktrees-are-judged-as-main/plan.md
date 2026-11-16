# Plan: isolated subagent worktrees are judged as main

- **Spec**: n/a (`bugfix` route — the bug report is in `./state.yaml`)
- **Approved**: 2026-11-16
- **Status**: approved

## Root cause

Claude Code creates a subagent's isolated worktree at
`<repo>/.claude/worktrees/<name>`. This repository's convention, and every
guard that decides "is this a worktree", is `<repo>/.worktrees/<…>`:
`isWorktreeRelPath` and `worktreeHolding` in
`.cli/hooks/worktree-policy-guard.mjs`, `splitWorktreePath` in
`.cli/lib/worktree-path.mjs`, `resolveProjectDir` in `.cli/lib/utils.mjs`, and
the session-owner, commit and destructive-git guards beside them. A path under
`<repo>/.claude/worktrees/<name>` is therefore inside the main checkout and outside every
worktree; the guard asks the main checkout's branch (CLAUDE_PROJECT_DIR), reads
`main`, and refuses the subagent's shell commands and edits as trunk work.

## The fix

Make the tree land where the guards already look, rather than teach six guards
a second convention. Claude Code's `WorktreeCreate` hook replaces its own
worktree creation for subagents, workflows, `--worktree` and `EnterWorktree`:
the hook prints the path and Claude Code works there.

- `.claude/hooks/worktree-create.sh` reads the payload's `name` and `cwd`,
  refuses a name that is not `[A-Za-z0-9._-]`, and runs
  `git worktree add --detach <main checkout>/.worktrees/agent-<name> HEAD` from
  `cwd`, so the tree starts at the head the parent is on — a verifier spawned
  from a change's worktree has that change checked out. It seeds `target/`
  with `scripts/seed-target.sh`, as `make wt.new` does, and prints the path.
  Detached: no branch is created, so nothing is left to delete.
- `.claude/hooks/worktree-remove.sh` removes the tree at `worktree_path` when it
  is a `.worktrees/agent-*` tree of this repository, and nothing else.
- `.claude/settings.json` registers both.
- Claude Code does not run `WorktreeRemove` for a subagent's tree, so
  `sweepAgentWorktrees` in `.claude/scripts/create-pr/ops.mjs` — the landing
  cleanup's sweep of agy's leftover trees (change 296) — also sweeps
  `.worktrees/agent-*`, under its existing rules: clean, untouched for six
  hours, HEAD already in main, `git worktree remove` without `--force`.

No guard is edited and no check is narrowed: a `.worktrees/` tree was already
worktree work; this only puts subagent trees there.

## Files that change

| File | Change |
|---|---|
| `.claude/hooks/worktree-create.sh` | new |
| `.claude/hooks/worktree-remove.sh` | new |
| `.claude/settings.json` | `WorktreeCreate`, `WorktreeRemove` |
| `.claude/scripts/create-pr/ops.mjs` | the sweep also takes `.worktrees/agent-*` |
| `src/tests.rs` | `isolated_subagent_worktrees_land_where_the_guards_look`, `isolated_subagent_worktree_work_is_not_judged_as_main`, the sweep test extended |
| `docs/sdlc/changes/299-…/` | state, plan, approvals, mutations, verification |

## Order of work

1. Tests, watched failing without the hooks.
2. The hooks and the settings entry.
3. The sweep.
4. `mutations.py`: the tree put back under `.claude/worktrees/<name>`, a name with a
   slash or `..` let through, the tree started at main rather than the parent's
   head, removal of a path outside `.worktrees/agent-*`, and the sweep skipping
   agent trees — each turns a test red.
5. After landing: a real `isolation: "worktree"` subagent runs `git status`,
   edits a file and runs a named test in its tree.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The hook fails | Claude Code refuses to start the subagent | failures print a reason on stderr and exit non-zero; the create test |
| A name escapes `.worktrees/` | a tree written over something else | the name check; the create test's `../x` and `a/b` cases |
| The remove hook deletes a person's worktree | lost work | it acts only on `.worktrees/agent-*`; the remove test's `fix/x` case |
| Agent trees pile up | disk | the landing sweep, under change 296's rules |
| The parent is on main | the tree starts at main | allowed: a tree under `.worktrees/` is worktree work whatever its HEAD |

## Proof of completion

- `cargo test --locked isolated_subagent` passes; it failed before the hooks.
- The extended sweep test passes.
- `python3 docs/sdlc/changes/299-…/mutations.py` — every mutation caught.
- After landing, an isolated subagent's shell and edit are not refused.

## Departures from the plan

- **The guard test uses `touch`, not `cargo test`.** `cargo test --locked` is
  on main's allowlist, so it passes in either tree and cannot tell them apart;
  a command off the allowlist does.
- **The remove hook checks once.** It first checked the prefix and then the
  name; the sweep showed the first check could go without a test noticing,
  because the second alone refuses anything outside `.worktrees/agent-*`. Only
  the second is kept.
- **Lesson 115** records the departure pattern that hid this: 296 and 297 each
  noted a verifier that could not run, and neither was read as a defect.
- **The live check runs after landing.** Claude Code reads hooks from the main
  checkout's settings, so a real isolated subagent can use the hook only once
  this is on main; its result is reported to the person.
