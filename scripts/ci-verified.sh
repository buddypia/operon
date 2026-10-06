#!/usr/bin/env bash
# Did the CI workflow pass on this tree? Change 128.
#
# The full suite runs in GitHub Actions, not on the local machine, so the gates
# that used to run it — the merge into main (.claude/hooks/gate-merge.sh) and the
# release check (scripts/check-release-preconditions.sh) — ask this instead. One
# question, asked in one place, so the two cannot drift on what "passed" means.
#
#   bash scripts/ci-verified.sh [commit]              default HEAD
#   bash scripts/ci-verified.sh --worktree [commit]   and the working tree is
#                                                     exactly that commit
#
#   exit 0  the CI workflow completed with success on this commit, or on a
#           parent with the identical tree (a --no-ff merge main had not moved
#           past: the merge commit is new, its tree is the one CI ran)
#   exit 1  it has not: failed, still running, or never ran — or, with
#           --worktree, the working tree holds something CI never saw. The
#           reason is printed
#   exit 2  CI could not be asked: gh missing, not logged in, no network, no
#           GitHub remote. The caller runs the suite locally instead
#
# Only an identical tree counts. A green run on an earlier push of the same
# branch says nothing about the commit on top of it, and a green commit says
# nothing about an edit or an untracked file beside it: cargo reads a build.rs
# or a .cargo/config.toml whether git tracks it or not.
set -uo pipefail

workflow=ci.yml
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

worktree=0
if [ "${1:-}" = --worktree ]; then
  worktree=1
  shift
fi
commit=${1:-HEAD}

sha=$(git rev-parse --verify --quiet "$commit^{commit}") ||
  { echo "$commit is not a commit"; exit 1; }
if [ "$worktree" -eq 1 ]; then
  [ "$(git rev-parse HEAD)" = "$sha" ] ||
    { echo "the working tree is not at $commit"; exit 1; }
  changed=$(git status --porcelain)
  [ -z "$changed" ] ||
    { echo "the working tree has changes CI never saw: $(printf '%s\n' "$changed" | head -n 5 | tr '\n' ' ')"; exit 1; }
fi
tree=$(git rev-parse "$sha^{tree}")
candidates=$sha
for parent in $(git rev-list --parents -n 1 "$sha" | cut -d' ' -f2-); do
  [ "$(git rev-parse "$parent^{tree}")" = "$tree" ] && candidates="$candidates $parent"
done

command -v gh >/dev/null 2>&1 || { echo "gh is not on PATH"; exit 2; }

# Bounded: a gh waiting on a dead network would otherwise hold the hook that
# called this until its own timeout.
ask() {
  bash "$here/run-bounded.sh" "${OPERON_CI_TIMEOUT:-20}" \
    gh run list --workflow "$workflow" --commit "$1" --limit 20 \
    --json status,conclusion --jq '.[] | .status + "/" + .conclusion'
}

seen=""
for candidate in $candidates; do
  if ! runs=$(ask "$candidate" 2>&1); then
    echo "CI could not be asked: $(printf '%s\n' "$runs" | tail -n 3 | tr '\n' ' ')"
    exit 2
  fi
  if printf '%s\n' "$runs" | grep -x 'completed/success' >/dev/null; then
    echo "CI passed on $candidate"
    exit 0
  fi
  if [ -z "$runs" ]; then
    seen="$seen ${candidate:0:12}: no run"
  else
    seen="$seen ${candidate:0:12}: $(printf '%s\n' "$runs" | tr '\n' ' ')"
  fi
done
echo "CI has not passed on this tree:$seen"
exit 1
# operon: end of scripts/ci-verified.sh
