#!/usr/bin/env bash
# Delete from the remote every branch whose work has landed on main (change 140).
#
#   bash scripts/prune-landed-branches.sh            delete
#   bash scripts/prune-landed-branches.sh --dry-run  say what would go, delete nothing
#
# Run by .github/workflows/prune-landed-branches.yml on every push to main, which
# is the last step of a landing. Landing here is `git merge --no-ff`, so a landed
# branch's tip is the second parent of a merge commit on main: reachable from
# main, and not on main's first-parent line. That is the whole rule.
#
# Reachable from main alone would be lossless — every commit is already in main —
# but it would also take a branch pushed from main's tip before its first commit,
# which is another session's work about to start. Such a tip sits on main's
# first-parent line, so that line is excluded. A branch fast-forwarded into main
# also sits there and is kept: left behind, never wrongly deleted.
#
# The deletion carries a lease on the tip that was judged, so a commit pushed in
# the seconds between the fetch and the delete keeps the branch. A deletion that
# fails — another run got there first — is reported and the rest go on.
set -euo pipefail

base="${PRUNE_BASE:-main}"
remote=origin
dry_run=0
# Anything but no argument or --dry-run is refused: a typo of the safe flag
# must not run the deleting form.
case "$#:${1:-}" in
  0:) ;;
  1:--dry-run) dry_run=1 ;;
  *) echo "usage: $0 [--dry-run]" >&2; exit 2 ;;
esac

git fetch --quiet --prune "$remote"
mainline=$(git rev-list --first-parent "refs/remotes/$remote/$base")

git for-each-ref --format='%(refname:strip=3) %(objectname)' "refs/remotes/$remote/" |
  while read -r name sha; do
    if [ "$name" = "$base" ] || [ "$name" = "HEAD" ]; then
      continue
    fi
    if ! git merge-base --is-ancestor "$sha" "refs/remotes/$remote/$base"; then
      echo "kept $name (not landed)"
      continue
    fi
    if grep -qx "$sha" <<<"$mainline"; then
      echo "kept $name (on $base's line)"
      continue
    fi
    if [ "$dry_run" -eq 1 ]; then
      echo "would prune $name"
    elif git push --quiet "--force-with-lease=refs/heads/$name:$sha" "$remote" --delete "refs/heads/$name"; then
      echo "pruned $name"
    else
      echo "could not prune $name"
    fi
  done
