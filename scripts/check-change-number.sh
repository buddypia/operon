#!/usr/bin/env bash
# Refuse a change number another change already holds. Change 162.
#
#   bash scripts/check-change-number.sh <branch>
#
# Exit 0 when the branch carries no number, or its number is free, or the only
# holder is the same change (the same slug, under `fix/` or `feature/` alike).
# Exit 1, naming the holders and the next free number, when the number is held
# under another slug. `make wt.new` runs this before creating anything.
#
# not-wired: no hook or workflow calls this, and none should — a number is
# chosen once, when a worktree is opened, and `make wt.new` is that moment.
#
# Why: a session picks the next number from what `docs/sdlc/changes/` on main
# holds, and a parallel session's change is not on main until it lands. Four
# numbers on main already name two changes each (059, 060, 098, 128), and 161
# was taken twice in one afternoon. The branches are where an in-flight change
# shows: local ones (every worktree shares them) and the ones other clones
# pushed, so origin is fetched first. Offline, the check goes on with the refs
# it has rather than blocking the work.
set -uo pipefail

branch="${1:-}"
[ -n "$branch" ] || { echo "usage: check-change-number.sh <branch>" >&2; exit 2; }
[[ "$branch" =~ ^[a-z]+/([0-9]{3})-(.+)$ ]] || exit 0
number=${BASH_REMATCH[1]}
slug=${BASH_REMATCH[2]}
here=$(cd "$(dirname "$0")" && pwd)

if git remote get-url origin >/dev/null 2>&1; then
  bash "$here/run-bounded.sh" 20 git fetch --quiet --prune origin >/dev/null 2>&1 \
    || echo "⚠ origin を取得できませんでした。手元の情報だけで番号を確かめます。"
fi

# Every `NNN-slug` in use, one per line, a tab, and where it was seen.
in_use() {
  local ref
  for ref in main origin/main; do
    git ls-tree --name-only "$ref:docs/sdlc/changes" 2>/dev/null |
      grep -E '^[0-9]{3}-' | sed "s|^\\(.*\\)\$|\\1	docs/sdlc/changes/\\1 ($ref)|"
  done
  git for-each-ref --format='%(refname:lstrip=2)' refs/heads refs/remotes 2>/dev/null |
    sed 's|^origin/||' | grep -E '^[a-z]+/[0-9]{3}-' |
    sed -E 's|^([a-z]+/)([0-9]{3}-.*)$|\2	\1\2|'
}

records=$(in_use | sort -u)
holders=$(printf '%s\n' "$records" |
  awk -F'\t' -v n="$number" -v s="$slug" 'substr($1, 1, 3) == n && substr($1, 5) != s { print $2 }' |
  sort -u | paste -sd, - | sed 's/,/, /g')
[ -z "$holders" ] && exit 0

highest=$(printf '%s\n' "$records" | cut -c1-3 | grep -E '^[0-9]{3}$' | sort -n | tail -1)
next=$(printf '%03d' $((10#${highest:-0} + 1)))
echo "❌ 変更番号 ${number} は使用中です: ${holders}。"
echo "次の空き番号は ${next} です。"
exit 1
