#!/usr/bin/env bash
# PreToolUse(Bash): the approval gates. Stage 5 Play 2 of the AI-native SDLC —
# an agent may act right up to a gate and may not pass it.
#
# `deny` is for commands that destroy a record no rerun can rebuild. `ask` is for
# a boundary whose answer is outside this working directory. Every decision
# explains itself, because a block with no reason gets worked around.
#
# The release boundary used to be an `ask` too, putting three questions to a
# person that are all facts about the tree at that instant. Change 032 replaced
# the questions with the checks: scripts/check-release-preconditions.sh
# establishes them, and only its silence produces `allow`. The `ask` is still
# here, and is what a failed check falls back to.
#
# The environment tiers this enforces are written down in docs/sdlc/README.md.
set -uo pipefail

command=$(cat | jq -r '.tool_input.command // empty')
[ -z "$command" ] && exit 0

decide() {
  jq -n --arg decision "$1" --arg reason "$2" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: $decision,
      permissionDecisionReason: $reason
    }
  }'
  exit 0
}

has() { printf '%s' "$command" | grep -F -- "$1" >/dev/null; }

# --- deny: records a rerun cannot rebuild ------------------------------------

if printf '%s' "$command" | grep -E '\brm\b' >/dev/null && has '.operon-package.lockfile'; then
  decide deny "Never delete dist/.operon-package.lockfile. It is an advisory flock the kernel releases on exit or crash, so a leftover file means another packaging run is live — find that run instead. See .claude/skills/ship/SKILL.md."
fi

if printf '%s' "$command" | grep -E '\brm\b' >/dev/null && has '/Applications/Operon.app'; then
  decide deny "Never rm /Applications/Operon.app. Replacing the installed app is a transaction owned by scripts/replace-macos-bundle.sh, which keeps a recoverable backup; rm leaves the user with no app and nothing to roll back to."
fi

# --- the release gate: checked, and only asked when the check cannot pass ------
#
# The script exits 0 only when all four conditions hold, and prints the failed one
# otherwise. Its output becomes the reason either way, so the transcript records
# what was established rather than merely that something was.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if has 'replace-macos-bundle.sh'; then
  if verdict=$(bash "$root/scripts/check-release-preconditions.sh" "$command" 2>&1); then
    decide allow "Release gate — $verdict"
  fi
  decide ask "Release gate — this swaps this project's production bundle, /Applications/Operon.app, and it may not proceed until the release preconditions hold. $verdict"
fi

# --- ask: boundaries outside this working directory ---------------------------

if has 'gh release'; then
  decide ask "Release gate — a GitHub release is published to users and cannot be quietly withdrawn. Confirm the tag points at a tree that passed the three gates, and that the attached bundle is the verified dist/Operon.app."
fi

# A plain push of one branch to origin is how a change reaches CI, which runs the
# suite the merge into main waits for (change 128), so it is part of landing and
# not a question. Plain means the whole command is that push and nothing else:
# no force, no `+` or `:` refspec, no second command. Anything else still asks.
# The branch word starts with a letter or digit, so `git push origin --force`,
# `-f` or `--mirror` — options git reads after the remote too — are not a branch.
plain_push='^git +push( +(-u|--set-upstream))? +origin +[A-Za-z0-9][A-Za-z0-9._/-]*$'
if printf '%s' "$command" | grep -E '\bgit +push\b' >/dev/null; then
  pushed=$(printf '%s' "$command" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')
  if [ "$(printf '%s\n' "$pushed" | grep -c .)" = 1 ] \
    && printf '%s' "$pushed" | grep -E "$plain_push" >/dev/null; then
    exit 0
  fi
  decide ask "Publishing gate — this push is not a plain push of one branch to origin, so confirm the destination and the refspec are the ones you mean. Force-pushing main rewrites the audit trail the SDLC pipeline depends on (docs/sdlc/README.md)."
fi

if printf '%s' "$command" | grep -E '\bgit +tag\b' >/dev/null \
  && ! printf '%s' "$command" | grep -E '\bgit +tag +(-l\b|--list\b|-n)' >/dev/null; then
  decide ask "Release gate — a tag is the name a release is cut from. Confirm the commit it points at passed the three gates."
fi

exit 0
