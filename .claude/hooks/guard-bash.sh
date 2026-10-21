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
# no force, no `+` or `:` refspec, no second command. `-q` only quiets it.
# The branch word starts with a letter or digit, so `git push origin --force`,
# `-f` or `--mirror` — options git reads after the remote too — are not a branch.
#
# A person is asked only about a push they can read whole: one line that is a
# lone `git push` with no shell syntax in it, whose destination or refspec is a
# decision — a force, a refspec, another remote. Anything else that names a
# push is refused with the form that runs, and the session reruns it alone
# (changes 182, 183): joined, piped or redirected, written `git -C dir push`,
# with the remote and branch left implicit, or only text in a commit message or
# a search. Nothing refused runs, so a force joined to a commit still reaches a
# person, as the lone push the session reruns; and the plain form's guarantee,
# that nothing the guard has not read runs with the push, holds.
#
# A push is `git`, then `push`, with any whitespace between. Once an option
# follows `git` (`-C dir`, `-c k=v`, `--no-pager`), any later word `push` on the
# line counts: an option's argument is a shell word, and no pattern reads every
# spelling of one (`-c a="b c"`, `-c a=$(echo b c)`), so the gate does not try,
# and a `git -C dir stash push` is refused with the push. A quote
# may end the word, so `bash -c "git push"` is one. It is read twice: as written,
# and as the shell hands git its words — a backslash-newline joined, quotes and
# backslashes dropped — so `"git" push`, `git pu""sh` and `git -C a\ b push`
# are pushes too. What the guard cannot read, because the shell computes it when
# the command runs, counts as a push: a git subcommand that is an expansion
# (`git $P`, `git $(echo push)`), always; a program that is one (`$G push`,
# `"$x" push`), an alias (`-c alias.p=push`, in any case) or an expansion after
# git's options (`git -C $W $P`), whenever `push` appears — so `git -C "$W"
# status`, the commonest such line in this repository's sessions, runs. A push run from a script file is not read: the gate catches a
# session's slip, not a session set on getting round it.
git_head='(^|[^[:alnum:]_-])git[[:space:]]+(-.*[[:space:]])?'
push_named="$git_head"'push([^[:alnum:]_-]|$)'
computed_subcommand='(^|[^[:alnum:]_-])git[[:space:]]+["'\'']?[$`]'
push_word='(^|[^[:alnum:]_-])push([^[:alnum:]_-]|$)'
computed='(^|[;&|(]|\$\()[[:space:]]*([A-Za-z_][A-Za-z0-9_]*=[^[:space:]]*[[:space:]]+)*["'\'']?[$`]|[Aa][Ll][Ii][Aa][Ss]\.|(^|[^[:alnum:]_-])git[[:space:]]+-.*[$`]'
lone_push='^git push( [A-Za-z0-9._/:+=@^~-]+)*$'
plain_push='^git push( (-u|--set-upstream|-q|--quiet))* origin [A-Za-z0-9][A-Za-z0-9._/-]*$'
implicit_push='^git push( (-u|--set-upstream|-q|--quiet))*( origin)?$'
words=$(printf '%s\n' "$command" | awk '{ if (sub(/\\$/, "")) printf "%s", $0; else print }' | tr -d '\\"'\''')
if printf '%s' "$command" | grep -E "$push_named" >/dev/null \
  || printf '%s' "$words" | grep -E "$push_named" >/dev/null \
  || printf '%s' "$command" | grep -E "$computed_subcommand" >/dev/null \
  || { printf '%s' "$words" | grep -E "$push_word" >/dev/null \
    && printf '%s' "$command" | grep -E "$computed" >/dev/null; }; then
  # Only the characters bash splits words on are folded; anything else stays
  # and fails the lone shape.
  line=$(printf '%s' "$command" | tr '\t\n' '  ' | tr -s ' ' | sed -E 's/^ //; s/ $//')
  if [ "$(printf '%s\n' "$command" | grep -c '[^[:space:]]')" = 1 ] \
    && printf '%s' "$line" | grep -E "$lone_push" >/dev/null; then
    printf '%s' "$line" | grep -E "$plain_push" >/dev/null && exit 0
    printf '%s' "$line" | grep -E "$implicit_push" >/dev/null \
      || decide ask "Publishing gate — this push is not a plain push of one branch to origin, so confirm the destination and the refspec are the ones you mean. Force-pushing main rewrites the audit trail the SDLC pipeline depends on (docs/sdlc/README.md)."
  fi
  decide deny "Publishing gate — run the push as \`git push origin <branch>\` (\`-u\` and \`-q\` are fine) in a Bash call of its own, from the repository, written plainly: no quotes, \`\\\` or \`\$\` in it, nothing before \`push\` such as \`-C <dir>\`, nothing joined, piped or redirected, the other commands as calls of their own. That form runs without asking; a force, a refspec or another remote is asked about then. If \`git push\` is only text here — a commit message, a search pattern, a file body — keep it out of the command: \`git commit -F <file>\`, the Write or Edit tool, or the pattern \`git[[:space:]]push\`."
fi

if printf '%s' "$command" | grep -E '\bgit +tag\b' >/dev/null \
  && ! printf '%s' "$command" | grep -E '\bgit +tag +(-l\b|--list\b|-n)' >/dev/null; then
  decide ask "Release gate — a tag is the name a release is cut from. Confirm the commit it points at passed the three gates."
fi

exit 0
