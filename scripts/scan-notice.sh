#!/usr/bin/env bash
# Say so when the last weekly scan failed. Change 195.
#
#   bash scripts/scan-notice.sh
#
# .github/workflows/scheduled-scan.yml runs on GitHub every Monday, and its
# failures on 2026-10-12 and 10-19 were seen by no one: nothing local reads
# GitHub's result, and stage 6 of docs/sdlc/README.md only works if a breach
# reaches somebody who can open a change for it. `make wt.new` runs this, so
# the next change started hears of it.
#
# Messages go to stderr: stdout of `make wt.new` is worktree-new.mjs's JSON.
# Fail-open: no gh, no network, no login, or no answer within 10 seconds
# prints nothing, since a new worktree must not wait on GitHub.
set -uo pipefail

here=$(cd "$(dirname "$0")" && pwd)
last=$(bash "$here/run-bounded.sh" 10 gh run list --workflow scheduled-scan.yml --limit 1 \
  --json conclusion,createdAt,url 2>/dev/null) || exit 0
failed=$(jq -r '.[0] | select(.conclusion == "failure") | "\(.createdAt[0:10]) \(.url)"' <<< "$last" 2>/dev/null) || exit 0
[ -n "$failed" ] || exit 0

echo "scan-notice: 週次スキャンが ${failed%% *} に失敗しています: ${failed#* }" >&2
echo "  見つかったものは新しい変更として扱ってください（docs/sdlc/README.md の stage 6）。" >&2
exit 0
# operon: end of scripts/scan-notice.sh
