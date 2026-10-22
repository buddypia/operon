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
export GIT_NO_REPLACE_OBJECTS=1
export GIT_GRAFT_FILE=/dev/null
# The graft deprecation hint git 2.56 prints on every call; scripts/check-review.sh.
export GIT_ADVICE=0

payload=$(cat)
command=$(printf '%s' "$payload" | jq -r '.tool_input.command // .tool_input.CommandLine // .toolCall.args.CommandLine // .toolCall.args.command // empty')
running_in=$(printf '%s' "$payload" | jq -r '.tool_input.Cwd // .toolCall.args.Cwd // .cwd // .session.cwd // .workspacePaths[0] // empty')

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
current=$(git branch --show-current 2>/dev/null)
[ "$current" = main ] && on_main=1

# A merge run in the main checkout is a landing, but it lands on whatever branch
# the checkout holds. The trunk guard refuses switching it away from main, so a
# switch comes from outside these hooks — a person, another tool — and the
# merge would go into their branch and pass below as "not into main". Change
# 200 met it once, stopped only by that branch's uncommitted Cargo.lock.
# A payload with no usable cwd is judged in CLAUDE_PROJECT_DIR, the main
# checkout, and refused here too. `--path-format` needs git 2.31; an older git
# leaves git_dir empty and the check off.
git_dir=$(git rev-parse --path-format=absolute --git-dir 2>/dev/null)
if [ "$on_main" = 0 ] && [ -n "$git_dir" ] &&
  [ "$git_dir" = "$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" ]; then
  block "the main checkout is on ${current:-a detached HEAD}, not main" \
"A merge here would land on ${current:-that detached HEAD}, which somebody else
switched to. Leave the main checkout as it is. Land from your own worktree:
\`git switch main\` there, the merge, \`git push origin main\`, then \`git switch\`
back to your branch at once — also when the merge or the push failed — so the
main checkout can return to main. If git refuses the switch because main is
checked out in another worktree, somebody else is landing: wait for them."
fi

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

touched_changes=$(git diff --no-renames --name-only main "$sha" -- 'docs/sdlc/changes/' 2>/dev/null |
  sed -n 's|^docs/sdlc/changes/\([^/]*\)/.*|\1|p' | sort -u)
routes_blob=$(git show "$sha:docs/sdlc/routes.yaml" 2>/dev/null || cat "$root/docs/sdlc/routes.yaml" 2>/dev/null || true)
for change_dir in $touched_changes; do
  [ -n "$(git ls-tree --name-only "$sha" "docs/sdlc/changes/$change_dir/" 2>/dev/null)" ] || continue
  changed_state="docs/sdlc/changes/$change_dir/state.yaml"
  state_blob=$(git show "$sha:$changed_state" 2>/dev/null) ||
    block "$changed_state is missing on $branch" \
"Every change directory under docs/sdlc/changes/ must carry a state.yaml file."
  state_status=$(printf '%s\n' "$state_blob" | sed -n 's/^  status: "\(.*\)"$/\1/p' | head -1)
  case "$state_status" in
    archived) ;;
    done)
      pending_contract=$(printf '%s\n' "$state_blob" | awk '
        /^contract:/ { inside = 1; next }
        inside && /^[a-z]/ { inside = 0 }
        inside && /"machine pending / { print $1 }')
      if [ -n "$pending_contract" ]; then
        block "$changed_state is \`done\` with a \`machine pending\` contract on $branch" \
"Run each pending contract command and record \`machine passed\` (or \`machine waived <reason>\`)
in $changed_state on $branch before merging."
      fi
      state_screen=$(printf '%s\n' "$state_blob" | sed -n 's/^  screen: "\(.*\)"$/\1/p' | head -1)
      if [ "$state_screen" = pending ]; then
        block "$changed_state is \`done\` with \`screen: \"pending\"\` on $branch" \
"Settle screen approval (\`approved\` or \`n/a\`) in $changed_state on $branch before merging."
      fi
      state_route=$(printf '%s\n' "$state_blob" | sed -n 's/^  route: "\(.*\)"$/\1/p' | head -1)
      state_readiness=$(printf '%s\n' "$state_blob" | sed -n 's/^  readiness: "\(.*\)"$/\1/p' | head -1)
      route_stages=""
      if [ -n "$routes_blob" ] && [ -n "$state_route" ]; then
        route_stages=$(printf '%s\n' "$routes_blob" | sed -n "s/^  $state_route: \"\([^ ]*\).*\"$/\1/p" | head -1)
      fi
      case ",${route_stages:-design}," in
        *,design,*)
          case "$state_readiness" in
            go | conditional) ;;
            *)
              block "$changed_state is \`done\` with \`readiness: \"$state_readiness\"\` on $branch" \
"Route \`$state_route\` includes the design stage; pass scripts/check-readiness.sh and record \`readiness: \"go\"\` (or \`\"conditional\"\`) in $changed_state on $branch before merging."
              ;;
          esac
          ;;
      esac
      ;;
    *)
      block "$changed_state has status \`$state_status\` on $branch" \
"A change that lands on main must record its terminal status (\`done\` or \`archived\`)
in $changed_state on $branch before merging — main itself refuses direct edits."
      ;;
  esac
done

# And every other record on the branch, by status alone. The loop above reads
# the directories this branch touched, so a record that reached main open some
# other way — thirty arrived in the squashed import of 2026-10-05 — was never
# read again, and sat at `reviewing` for weeks while guards learned to skip it
# (sdlc 202). Every state.yaml is listed first, so one whose status cannot be
# read is open rather than absent; then the first `  status:` line of each,
# which is the state block's, ahead of the closed set in `sets:`. A git that
# fails here refuses: an unread list would pass as an empty one.
record_files=$(git ls-tree -r --name-only "$sha" -- docs/sdlc/changes) ||
  block "the records on $branch could not be listed" "git ls-tree failed on ${sha:0:12}."
record_statuses=$(git grep -n -E '^  status: "' "$sha" -- 'docs/sdlc/changes/*/state.yaml')
[ "$?" -le 1 ] ||
  block "the records on $branch could not be read" "git grep failed on ${sha:0:12}."
open_records=$(
  {
    printf '%s\n' "$record_files" | grep -E '^docs/sdlc/changes/[^/]+/state\.yaml$' | sed 's/^/file:/'
    printf '%s\n' "$record_statuses" | sed "s/^$sha://"
  } | awk '
    /^file:/ { path = substr($0, 6); order[++n] = path; next }
    {
      path = $0; sub(/:.*/, "", path)
      if (path in status) next
      value = $0; sub(/^[^:]*:[0-9]+:  status: "/, "", value); sub(/".*/, "", value)
      status[path] = value
    }
    END {
      for (i = 1; i <= n; i++) {
        path = order[i]
        value = (path in status) ? status[path] : "no readable status"
        if (value == "done" || value == "archived") continue
        dir = path; sub(/^docs\/sdlc\/changes\//, "", dir); sub(/\/state\.yaml$/, "", dir)
        print "  " dir " (" value ")"
      }
    }')
if [ -n "$open_records" ]; then
  block "records on $branch that never closed" \
"$open_records
Each of these is on $branch without a terminal status, and this branch did not
touch it. Read each against the code, then record \`done\` in its state.yaml on
$branch, or \`archived\` if it was abandoned or superseded — one commit per
change directory — before merging."
fi

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
# Under this hook's 600 seconds in .claude/settings.json and .codex/hooks.json,
# with room to refuse.
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
