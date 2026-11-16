#!/usr/bin/env bash
# Claude Code WorktreeRemove hook. Change 299.
#
# Removes a tree .claude/hooks/worktree-create.sh made, when Claude Code cleans
# one up (leaving a `--worktree` session, ExitWorktree). Only a
# `<main checkout>/.worktrees/agent-<name>` tree of this repository is touched:
# a person's `.worktrees/fix/…` or anything else is refused, and stays.
#
# stdin: {"worktree_path": "<dir>", ...}. Exit 0 once the directory is gone.
set -uo pipefail

fail() { echo "worktree-remove: $1" >&2; exit 1; }

path=$(jq -r '.worktree_path // empty') || fail "the payload is not JSON"
[ -n "$path" ] || fail "no worktree_path"
[ -e "$path" ] || exit 0

common=$(git -C "$path" rev-parse --path-format=absolute --git-common-dir 2>/dev/null) ||
  fail "$path is not a git worktree"
main_root=$(cd "$(dirname "$common")" && pwd -P)
real=$(cd "$path" && pwd -P)
# What is left after the prefix is the name alone: a path without the prefix is
# left whole, and holds a slash.
case "${real#"$main_root"/.worktrees/agent-}" in
  '' | */*) fail "refusing to remove $path: not a .worktrees/agent-* tree" ;;
esac

git -C "$main_root" worktree remove --force "$real" >&2 || fail "git worktree remove failed for $real"
# operon: end of .claude/hooks/worktree-remove.sh
