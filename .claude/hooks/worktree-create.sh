#!/usr/bin/env bash
# Claude Code WorktreeCreate hook. Change 299.
#
# Claude Code puts an isolated worktree — a subagent's `isolation: "worktree"`,
# a workflow agent's, `--worktree`, EnterWorktree — at
# `<repo>/.claude/worktrees/<name>`. Every worktree guard in this repository
# knows a worktree only under `<repo>/.worktrees/`, so work in the default
# place was judged as work on main and refused. With this hook registered,
# Claude Code runs it instead of creating the tree itself and works in the path
# it prints: `<main checkout>/.worktrees/agent-<name>`, where the guards already
# allow worktree work.
#
# The tree is detached at the HEAD of the directory the session is in, so a
# verifier spawned from a change's worktree has that change checked out, and
# no branch is created that would need deleting. `target/` is seeded from the
# main checkout as `make wt.new` does. Claude Code does not run WorktreeRemove
# for a subagent's tree; the landing cleanup sweeps them
# (sweepAgentWorktrees in .claude/scripts/create-pr/ops.mjs).
#
# stdin: {"name": "<slug>", "cwd": "<dir>", ...}. stdout: the path, alone.
# Anything else goes to stderr; a non-zero exit makes Claude Code refuse to
# create the worktree rather than fall back to the place the guards refuse.
set -uo pipefail

fail() { echo "worktree-create: $1" >&2; exit 1; }

payload=$(cat)
name=$(printf '%s' "$payload" | jq -r '.name // empty') || fail "the payload is not JSON"
from=$(printf '%s' "$payload" | jq -r '.cwd // empty')
[ -n "$from" ] || from=$PWD

# A slug only: a slash, `..` or a leading dot would put the tree somewhere
# other than directly under .worktrees/.
case "$name" in
  '' | .* | *[!A-Za-z0-9._-]*) fail "refusing worktree name '$name'" ;;
esac

common=$(git -C "$from" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) ||
  fail "$from is not inside a git repository"
main_root=$(dirname "$common")
worktree="$main_root/.worktrees/agent-$name"
[ -e "$worktree" ] && fail "$worktree already exists"

git -C "$from" worktree add --quiet --detach "$worktree" HEAD >&2 ||
  fail "git worktree add failed for $worktree"
bash "$main_root/scripts/seed-target.sh" "agent-$name" >&2 || true

printf '%s\n' "$worktree"
# operon: end of .claude/hooks/worktree-create.sh
