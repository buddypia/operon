#!/usr/bin/env bash
# Run again, on this tree, every piece of evidence the changes on this branch
# recorded. Change 242.
#
# A verification.yaml is a list of claims and the commands that check them. The
# commit gate (scripts/check-review.sh) binds the list to its diff and judges
# its shape; it does not run anything, because a ref update has seconds and a
# mutation sweep has minutes. This runs them, in CI, where
# .claude/hooks/gate-merge.sh waits for the result: a recorded obligation that
# does not hold is a red workflow, and a red workflow does not merge. That is
# what turns a record from something believed into something re-checked.
#
#   bash scripts/run-obligations.sh <base-ref>     every change dir the branch
#                                                  touches since <base-ref>
#   bash scripts/run-obligations.sh --dir <dir>    one change directory
#
# The commands it runs are only the ones `check-review.sh --obligations`
# validated — a method's literal prefix plus one plain name — so a record
# cannot make CI run anything else.
#
# Ceilings: OBLIGATION_SECONDS per obligation (900), JOB_SECONDS for the whole
# run (3600), and OUTPUT_TAIL lines (40) of a failing obligation's output in the
# log, each cut to OUTPUT_BYTES (64 KiB). The full output goes to a scratch file
# rather than through `head -c`, which would SIGPIPE a chatty command into a
# failure it did not have.
#
#   exit 0  every obligation held, or there were none to run
#   exit 1  an obligation failed, timed out, or proved nothing
#   exit 2  the records could not be read
set -uo pipefail

OBLIGATION_SECONDS=${OBLIGATION_SECONDS:-900}
JOB_SECONDS=${JOB_SECONDS:-3600}
OUTPUT_TAIL=40
OUTPUT_BYTES=65536

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || { echo "not a git repository"; exit 2; }
cd "$repo_root" || exit 2

dirs=""
case "${1:-}" in
  --dir) dirs=${2%/} ;;
  "") echo "usage: run-obligations.sh <base-ref> | --dir <change-dir>"; exit 2 ;;
  *)
    base=$(git merge-base "$1" HEAD 2>/dev/null) || { echo "cannot find a merge base with $1"; exit 2; }
    dirs=$(git -c core.quotePath=false diff --name-only "$base" HEAD -- docs/sdlc/changes |
      sed -n 's|^\(docs/sdlc/changes/[^/]*\)/.*|\1|p' | sort -u)
    ;;
esac

started=$SECONDS
failed=0
ran=0
scratch=$(mktemp -d) || exit 2
trap 'rm -r "$scratch"' EXIT

for dir in $dirs; do
  [ -f "$dir/verification.yaml" ] || { echo "· ${dir}: 検証記録なし（旧方式の変更か、決定的な検査のみ）"; continue; }
  list=$(bash scripts/check-review.sh --obligations "$dir") || { echo "❌ ${dir}: 検証記録が読めません"; exit 1; }
  while IFS=$'\t' read -r id method command; do
    [ -n "$id" ] || continue
    left=$((JOB_SECONDS - (SECONDS - started)))
    if [ "$left" -le 0 ]; then
      echo "❌ 時間切れ: 全体の上限 ${JOB_SECONDS} 秒を超えました。${dir} ${id} 以降は実行していません"
      exit 1
    fi
    limit=$OBLIGATION_SECONDS
    [ "$left" -lt "$limit" ] && limit=$left
    out="$scratch/${id}.log"
    echo "▶ $(basename "$dir") ${id} ${method} :: ${command}"
    set -f
    # shellcheck disable=SC2086 # validated: a prefix and one plain name, no metacharacters
    bash scripts/run-bounded.sh "$limit" $command >"$out" 2>&1
    status=$?
    set +f
    ran=$((ran + 1))
    # `cargo test` with a filter that matched nothing exits 0. The gate already
    # asks that the name exists; this asks that it ran.
    if [ "$status" -eq 0 ]; then
      case "$command" in
        "cargo test "*)
          grep -E '^test .* \.\.\. ok$' "$out" >/dev/null || { status=3; echo "  実行されたテストが 0 件です。証拠になりません"; } ;;
      esac
    fi
    if [ "$status" -eq 0 ]; then
      echo "  ✓ ${id}"
      continue
    fi
    failed=1
    [ "$status" -eq 124 ] && echo "  ❌ ${id}: 時間切れ（${limit} 秒）" || echo "  ❌ ${id}: 終了コード ${status}"
    tail -n "$OUTPUT_TAIL" "$out" | tail -c "$OUTPUT_BYTES" | sed 's/^/    /'
  done <<<"$list"
done

echo "検証 ${ran} 件を実行、$([ "$failed" -eq 0 ] && echo すべて成立 || echo 不成立あり)（$((SECONDS - started)) 秒）"
exit "$failed"
# operon: end of scripts/run-obligations.sh
