#!/usr/bin/env bash
# Wait for the CI workflow on a pushed commit, and say how it ended. Change 298.
#
# `gh run watch` straight after `git push` finds no run: GitHub registers the
# run a few seconds after the push, and until then `gh run watch` and
# `gh run list --commit` answer "no runs". A session that met that answer
# polled by hand instead, one tool call each time, and change 297's landing
# spent 219 turns there. This script waits for the run to appear, then watches
# that one run to its end, in one call.
#
#   bash scripts/wait-ci.sh [commit]     default HEAD; run it in the background
#
#   exit 0  the CI workflow completed with success on this commit
#   exit 1  it completed otherwise, or no run appeared in time (was the branch
#           pushed? is the commit on it?) — the reason is printed
#   exit 2  CI could not be asked: gh missing, not logged in, no network
#
# The workflow is the one scripts/ci-verified.sh and the merge gate ask about.
#
#   OPERON_CI_APPEAR_SECONDS  how long a run may take to appear (default 180)
#   OPERON_CI_POLL_SECONDS    pause between asks while it has not (default 5)
#   OPERON_CI_WATCH_SECONDS   how long the run may take once it has (default 3600)
set -uo pipefail

workflow=ci.yml
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
appear=${OPERON_CI_APPEAR_SECONDS:-180}
poll=${OPERON_CI_POLL_SECONDS:-5}
watch=${OPERON_CI_WATCH_SECONDS:-3600}

commit=${1:-HEAD}
sha=$(git rev-parse --verify --quiet "$commit^{commit}") ||
  { echo "$commit is not a commit"; exit 1; }
command -v gh >/dev/null 2>&1 || { echo "gh is not on PATH"; exit 2; }

# The newest run of the workflow on this commit: a re-run is listed first.
latest_run() {
  bash "$here/run-bounded.sh" 20 \
    gh run list --workflow "$workflow" --commit "$sha" --limit 1 \
    --json databaseId --jq '.[0].databaseId // empty'
}

deadline=$((SECONDS + appear))
while :; do
  if ! run=$(latest_run 2>&1); then
    echo "CI could not be asked: $(printf '%s\n' "$run" | tail -n 3 | tr '\n' ' ')"
    exit 2
  fi
  [ -n "$run" ] && break
  if [ "$SECONDS" -ge "$deadline" ]; then
    echo "no $workflow run appeared for ${sha:0:12} in ${appear}s: push the branch holding it (git push -u origin <branch>)"
    exit 1
  fi
  sleep "$poll"
done

echo "watching $workflow run $run on ${sha:0:12}"
bash "$here/run-bounded.sh" "$watch" gh run watch "$run" --exit-status >/dev/null 2>&1
status=$?
case "$status" in
  0) echo "CI passed on ${sha:0:12} (run $run)"; exit 0 ;;
  124) echo "CI run $run did not finish in ${watch}s"; exit 1 ;;
  *) echo "CI failed on ${sha:0:12} (run $run): gh run view $run --log-failed"; exit 1 ;;
esac
# operon: end of scripts/wait-ci.sh
