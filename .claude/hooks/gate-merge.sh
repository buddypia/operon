#!/usr/bin/env bash
# PreToolUse(Bash): the merge gate. Change 128.
#
# The full suite used to run on this machine at every commit. It now runs in
# GitHub Actions on the pushed branch, and this is where its verdict is waited
# for: a `git merge` into main is refused until scripts/ci-verified.sh says the
# CI workflow passed on the branch head. The commit gate keeps fmt and clippy;
# this keeps "no code reaches main without the whole suite having passed on it".
#
# The branch has to contain main. Then the --no-ff merge commit has exactly the
# branch's tree, which is the tree CI ran — a merge over a main that moved would
# be a tree nobody tested.
#
# When CI cannot be asked (offline, gh missing or logged out), the suite runs
# here instead, in the branch's own worktree, under a limit shorter than this
# hook's own: a hook that times out is a non-blocking error, so running out of
# time has to be a refusal this script makes, not one it is killed before.
#
# One spelling is judged, and every other spelling of a merge is refused rather
# than read. Review found the reader of "any merge command" letting through
# `-m msg` after the branch, a second merge chained behind a first, `git -C`,
# and `cd <main> &&` from a worktree session. A merge into main is written
#
#   git merge --no-ff --no-edit <branch>        (or --ff-only; --no-edit optional)
#
# alone, from a session whose working directory is the main checkout.
# `git merge --abort` and `--quit` pass. In a worktree session, a merge with no
# redirection (`-C`, `cd`, `--git-dir`, `GIT_*=`) is not into main and passes.
set -uo pipefail

payload=$(cat)
command=$(printf '%s' "$payload" | jq -r '.tool_input.command // empty')
running_in=$(printf '%s' "$payload" | jq -r '.cwd // empty')

# Is any simple command a git whose subcommand is `merge`? The subcommand is
# the first word past git's own options (`-C dir`, `-c k=v`, `--git-dir …`), so
# `git -C x merge` and `/usr/bin/git merge` are merges and `git commit -m "a
# merge"` or `git log --grep merge` are not — review of 128 found a reader of
# "the word merge anywhere" refusing ordinary commits. Words split on any
# whitespace: a tab is a separator to the shell, and was a way past a reader
# that split on spaces. The git is the first `git` word anywhere in the
# segment, not only its first word: `bash -c 'git merge …'`, `env git …`,
# `then git …` and `$(git …)` put another word in front, and review of 128
# found each of them passing a reader that looked only at the front. Leading
# quotes, backslashes, `$(`, backticks and brackets are stripped from a word
# before it is compared. `echo git merge x` is read as a merge, and refused
# with an explanation rather than let through.
#
# A heredoc that feeds text is not a command: a commit message written through
# `cat <<EOF` that says "refuse a git merge" is not a merge (review of 128). But
# a heredoc that feeds a shell is a program — `bash <<EOF`, `cat <<EOF | sh`,
# `eval "$(cat <<EOF …)"` — and dropping its body hid a merge inside it (same
# review). So a body is dropped only when the line that opens it reads into a
# text sink (`cat`, `tee`, `git commit`) and names no interpreter. Every other
# body is read as commands. A body with no terminator is not a body, and the
# raw text is read instead, so nothing unread is let through.
readable=$(printf '%s\n' "$command" | awk '
  inbody { t = $0; sub(/^\t+/, "", t); if (t == term) inbody = 0; next }
  {
    print
    sink = ($0 ~ /(^|[^[:alnum:]_])(cat|tee|commit)([[:space:]]|$)/) &&
      $0 !~ /(^|[^[:alnum:]_.\/-])(bash|sh|zsh|dash|ksh|eval|source|xargs|exec)([[:space:]]|$)/ &&
      $0 !~ /(^|[[:space:]])\.[[:space:]]/
    if (sink && match($0, /<<-?[[:space:]]*[\047"]?[A-Za-z_][A-Za-z0-9_]*/)) {
      term = substr($0, RSTART, RLENGTH)
      sub(/^<<-?[[:space:]]*[\047"]?/, "", term)
      inbody = 1
    }
  }
  END { if (inbody) exit 1 }') || readable=$command
merges=$(printf '%s\n' "$readable" | tr ';&|' '\n\n\n' | awk '
  {
    n = split($0, w, /[[:space:]]+/)
    for (j = 1; j <= n; j++) gsub(/^[\047"\\`$({]+/, "", w[j])
    i = 1
    while (i <= n && !(w[i] == "git" || w[i] ~ /\/git$/)) i++
    if (i > n) next
    i++
    while (i <= n) {
      if (w[i] ~ /^(-C|-c|--git-dir|--work-tree|--namespace|--exec-path|--config-env)$/) { i += 2; continue }
      if (w[i] ~ /^-/) { i++; continue }
      break
    }
    sub(/[\047"`)}]+$/, "", w[i])
    if (w[i] == "merge") print "merge"
  }')
[ -n "$merges" ] || exit 0

block() {
  printf 'Merge blocked by the gate: %s\n\n%s\n' "$1" "$2" >&2
  exit 2
}
note() { jq -n --arg m "$1" '{systemMessage: $m}'; exit 0; }

trimmed=$(printf '%s' "$command" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')
single=0
[ "$(printf '%s\n' "$trimmed" | grep -c .)" = 1 ] && single=1
if [ "$single" = 1 ] && printf '%s' "$trimmed" | grep -E '^git[[:space:]]+merge[[:space:]]+--(abort|quit)$' >/dev/null; then
  exit 0
fi

[ -n "$running_in" ] && [ -d "$running_in" ] || running_in=${CLAUDE_PROJECT_DIR:-.}
cd -- "$running_in" 2>/dev/null || exit 0
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_COMMON_DIR GIT_PREFIX
on_main=0
[ "$(git branch --show-current 2>/dev/null)" = main ] && on_main=1

canonical='^git[[:space:]]+merge[[:space:]]+--(no-ff|ff-only)([[:space:]]+--no-edit)?[[:space:]]+[A-Za-z0-9][A-Za-z0-9._/-]*$'
if [ "$single" = 0 ] || ! printf '%s' "$trimmed" | grep -E "$canonical" >/dev/null; then
  redirected=0
  printf '%s' "$command" |
    grep -E '(^|[^[:alnum:]_])(cd|pushd)([[:space:]]|$)|(^|[[:space:]])-C|--git-dir|--work-tree|GIT_[A-Z_]+=' >/dev/null &&
    redirected=1
  [ "$on_main" = 0 ] && [ "$redirected" = 0 ] && exit 0
  block "a merge written in a form this gate does not read" \
"A merge into main is judged only as the whole command
  git merge --no-ff --no-edit <branch>
run from a session whose working directory is the main checkout. Options after
the branch, chained commands, -C, cd and GIT_* variables are refused rather
than guessed at."
fi
[ "$on_main" = 1 ] || exit 0

# The canonical form matched, so the branch is its last word.
branch=$(printf '%s\n' "$trimmed" | awk '{ print $NF }')
root=$(git rev-parse --show-toplevel 2>/dev/null) ||
  block "the main checkout" "Its top directory could not be resolved."
sha=$(git rev-parse --verify --quiet "refs/heads/$branch^{commit}") ||
  block "$branch is not a local branch" \
"The merge gate judges a branch by its head, so the name has to be a branch here."

git merge-base --is-ancestor main "$sha" ||
  block "$branch does not contain main" \
"main moved after $branch was branched or last updated, so the merge would be a
tree CI never ran. Merge main into $branch in its worktree, push it, wait for CI
(gh run watch), then merge."

verdict=$(bash "$root/scripts/ci-verified.sh" "$sha" 2>&1)
status=$?
case "$status" in
  0) note "Merge gate: $verdict." ;;
  1) block "CI has not passed on $branch (${sha:0:12})" \
"$verdict
Push the branch (git push -u origin $branch) from its worktree, wait for the run
(gh run watch), then merge. A failed run is evidence: fix it on the branch." ;;
esac

# CI could not be asked. Run the suite where the branch is checked out.
worktree=$(git worktree list --porcelain | awk -v ref="branch refs/heads/$branch" '
  index($0, "worktree ") == 1 { path = substr($0, 10) }
  $0 == ref { print path; exit }')
[ -n "$worktree" ] ||
  block "CI could not be asked, and $branch has no worktree to test in" "$verdict"
[ "$(git -C "$worktree" rev-parse HEAD)" = "$sha" ] &&
  [ -z "$(git -C "$worktree" status --porcelain)" ] ||
  block "CI could not be asked, and $worktree is not exactly $branch" \
"$verdict
The fallback tests the branch in its worktree, which has changes or another
commit checked out, so it would be testing something else."
command -v cargo >/dev/null 2>&1 || PATH="$HOME/.cargo/bin:$PATH"
# Under this hook's 600 seconds in .claude/settings.json, with room to refuse.
limit=${OPERON_MERGE_SUITE_TIMEOUT:-540}
output=$(cd -- "$worktree" && bash "$root/scripts/run-bounded.sh" "$limit" cargo test --locked 2>&1)
case $? in
  0) note "Merge gate: $verdict; cargo test --locked passed locally in $worktree instead." ;;
  124) block "cargo test --locked in $worktree did not finish in ${limit}s (CI could not be asked)" \
"$verdict
$(printf '%s' "$output" | tail -20)" ;;
  *) block "cargo test --locked in $worktree (CI could not be asked)" \
"$verdict
$(printf '%s' "$output" | tail -60)" ;;
esac
# operon: end of .claude/hooks/gate-merge.sh
