# Plan: a Codex session waits for CI before it lands on main

- **Spec**: none — bugfix route; the report is A18 in
  `docs/sdlc/changes/202-finished-changes-read-as-open/triage.md`.
- **Approved**: 2026-10-22
- **Status**: approved

## The defect

`.claude/hooks/gate-merge.sh` refuses a `git merge` into main until CI passed
on the branch head, and runs the suite itself when CI cannot be asked. Only
`.claude/settings.json` registers it. `.codex/hooks.json` runs the worktree,
commit and destructive-git guards on every shell call, but not this one, so a
Codex session can land on main before CI has judged the branch. The review
gate reaches Codex through git's reference-transaction hook; the CI check has
no such route, because asking CI belongs before the command, not inside a ref
update.

## The fix

One handler, appended last to the PreToolUse group in `.codex/hooks.json` whose
matcher is `^(Bash|shell|run_shell|run_shell_command|exec_command)$`:

    bash "$(dirname "$(git rev-parse --path-format=absolute --git-common-dir || echo /nonexistent/nogit)")/.claude/hooks/gate-merge.sh"

with a 600-second timeout, the same as Claude's registration.

- The script already reads Codex's payload (`tool_input.command`, `cwd`) and
  refuses the way Codex reads a refusal (exit 2, reason on stderr). It needs no
  `CLAUDE_PROJECT_DIR` when the payload names its directory, which Codex's does.
- The path expression is the one every other handler in the file uses, so the
  main checkout's gate runs, never a worktree's copy (change 073).
- Last in the group, because Codex keys a person's trust of a handler by its
  position: inserting earlier would move handlers already trusted.
- Not through `.cli/_cli-dispatch.mjs`: it loads only `.mjs` modules, and its
  adapter fails open on any error, which a gate must not.

## Files that change

| File | Change |
|---|---|
| `.codex/hooks.json` | the handler above |
| `.claude/hooks/gate-merge.sh` | the comment on the suite's limit names both registrations' 600 seconds |
| `src/tests.rs` | a test that `.codex/hooks.json` runs gate-merge.sh on every shell call with at least 600 seconds; the merge-gate test runs the registered Codex command itself, with a Codex payload and no `CLAUDE_PROJECT_DIR`, and expects the refusal |
| `docs/sdlc/changes/202-finished-changes-read-as-open/triage.md` | not edited: it is the report, and later changes name the rows they close |

## Order of work

1. This plan and `state.yaml`; approvals recorded.
2. Tests, watched failing on the unedited `.codex/hooks.json`.
3. The handler and the comment. Green.
4. Mutations: drop the handler; lower its timeout; point it at a worktree's
   copy (`--show-toplevel`); run the Codex command with a payload the script
   cannot read.
5. Gates, review.

## What this is not

- **fmt and clippy for Codex.** `gate-commit.sh` stays Claude's; CI runs both
  on every pushed branch and the merge gate waits for CI, so a Codex commit
  that fails them cannot land. That gap is not A18.
- **A person's trust in Codex.** Codex skips a new project hook until a person
  trusts it in Codex. That is Codex's own protection and stays theirs.
- **Every Codex shell route.** Codex reports simple shell calls to PreToolUse
  as `Bash` with `tool_input.command`, which the gate reads; its newer
  `unified_exec` path is not intercepted yet, by Codex's own account, and no
  handler in this file sees it. This closes the gap the file can close. A
  failed `git rev-parse` makes the path `/nonexistent/nogit`, and bash's exit
  127 lets the call through, as it does for every handler here.
