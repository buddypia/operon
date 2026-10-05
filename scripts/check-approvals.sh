#!/usr/bin/env bash
# Is every approval a change claims backed by a line in its ledger, and which
# kinds of automatic approval has an escape taken back to a person?
#
# Change 082. `.claude/skills/sdlc/references/approval.md` is the rule: an
# artifact that is not a significant decision is approved by an evaluator that
# did not write it, and every verdict — automatic, a person's, or an escape — is
# one line in `approvals.log` in the change directory:
#
#   <utc> <by> <category> <artifact> <digest> <verdict> <reason...>
#
# The trust is earned rather than declared. An escape (a defect a person found in
# something approved `auto`) demotes its category to a person; the category comes
# back after `clean_run` consecutive approvals where the person agreed with the
# evaluator. That state is computed from every ledger on each run and never
# stored, so it cannot drift from the lines that decide it.
#
# Only changes numbered above 082 are checked: older ones predate the ledger, and
# a number cannot be dodged the way a missing file can.
#
#   bash scripts/check-approvals.sh <change-dir>...    findings; exit 1 on any
#   bash scripts/check-approvals.sh --summary <change-dir>...   one line, or nothing
#   bash scripts/check-approvals.sh --metrics          three counts for harness-metrics
#   bash scripts/check-approvals.sh --list [category]  every ledger line, for sampling
#
#   exit 0 clean, 1 findings, 64 usage
set -uo pipefail

readonly clean_run=5
readonly first_checked=83

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root" || exit 64
# git resolves these before the directory it is run in, so a session that
# exported them would have this read another repository's branch (sdlc 080).
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR

changes_root=docs/sdlc/changes

# The paused surfaces are categories too, read from risk.yaml rather than
# retyped, and they start demoted: trust in them is earned in shadow first.
paused=$(sed -n 's/^  \([a-z-]*\): "[a-z]* paused .*/\1/p' docs/sdlc/risk.yaml 2>/dev/null | tr '\n' ' ')
categories="intent spec plan screen contract $paused"
lessons=$(sed -n 's/^## \([0-9][0-9][0-9]\) .*/\1/p' docs/sdlc/lessons.md 2>/dev/null | tr '\n' ' ')

# What an approval is bound to. The `- **Status**:` line is not part of it —
# flipping a draft to approved is the approval, not a new artifact — and a plan
# stops at its departures, which are written after it was judged.
digest() {
  awk '/^## Departures from the plan/ { exit } /^- \*\*Status\*\*:/ { next } { print }' "$1" |
    shasum -a 256 | cut -c1-16
}

ledger() {
  for log in "$changes_root"/*/approvals.log; do
    [ -f "$log" ] || continue
    awk -v change="$(basename "$(dirname "$log")")" \
      '!/^[[:space:]]*(#|$)/ { print change "\t" NR "\t" $0 }' "$log"
  done
}

# One pass over every ledger, in time order: ERR lines for findings, SEEN for
# each valid line, DEMOTED for each category a person is deciding at the end,
# and the counts.
judge() {
  ledger | awk -F'\t' -v categories="$categories" '
    BEGIN { n = split(categories, c, " "); for (i = 1; i <= n; i++) known[c[i]] = 1 }
    {
      change = $1; line = $2; split($3, f, " ")
      reason = $3; for (i = 1; i <= 6; i++) sub(/^[^ ]+ +/, "", reason)
      where = change "/approvals.log:" line
      if (f[1] !~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T[0-9][0-9]:[0-9][0-9]:[0-9][0-9]Z$/) { bad = "時刻が UTC の ISO 8601 ではありません" }
      else if (f[2] != "auto" && f[2] != "person" && f[2] != "escape") { bad = "by は auto / person / escape のいずれかです" }
      else if (!(f[3] in known)) { bad = "category `" f[3] "` は定義されていません" }
      else if (f[4] == "") { bad = "artifact がありません" }
      else if (f[5] !~ /^[0-9a-f]+$/ || length(f[5]) != 16) { bad = "digest は 16 桁の hex です" }
      else if (f[2] == "escape" && f[6] !~ /^lesson-[0-9][0-9][0-9]$/) { bad = "escape の verdict は lesson-NNN です" }
      else if (f[2] != "escape" && f[6] !~ /^(approve|revise|reject|escalate)$/) { bad = "verdict は approve / revise / reject / escalate のいずれかです" }
      else if (f[7] == "") { bad = "理由がありません" }
      else { bad = "" }
      if (bad != "") { print "ERR\t" change "\t" where ": " bad; next }
      rank = (f[2] == "auto") ? 0 : (f[2] == "person") ? 1 : 2
      print "REC\t" f[1] "\t" rank "\t" change "\t" f[2] "\t" f[3] "\t" f[4] "\t" f[5] "\t" f[6] "\t" where
    }' | sort -t "$(printf '\t')" -k1,1 -k2,2 -k3,3n |
    awk -F'\t' -v paused="$paused" -v categories="$categories" -v lessons=" $lessons" -v clean_run="$clean_run" '
      BEGIN { n = split(paused, p, " "); for (i = 1; i <= n; i++) demoted[p[i]] = 1 }
      $1 == "ERR" { print; next }
      {
        change = $4; by = $5; cat = $6; artifact = $7; dig = $8; verdict = $9; where = $10
        # An automatic verdict in a demoted category is a shadow: the person
        # decides, and it counts only as evidence toward the clean run.
        print "SEEN\t" change "\t" by "\t" cat "\t" verdict "\t" (by == "auto" && demoted[cat] ? "shadow" : "")
        key = change SUBSEP artifact SUBSEP dig SUBSEP cat
        if (by == "auto") {
          auto[key] = verdict
          if (verdict == "approve") {
            if (!demoted[cat]) approvals++
            if (demoted[cat] && !(key in person)) owed[key] = change "\t" where ": `" cat "` は人の判断に戻っています — 自動承認には同じ digest の person 行が要ります"
          }
        } else if (by == "person") {
          person[key] = verdict; delete owed[key]
          if (demoted[cat] && (key in auto)) {
            if (verdict == "approve" && auto[key] == "approve") {
              if (++run[cat] >= clean_run) { demoted[cat] = 0; run[cat] = 0 }
            } else if (verdict == "revise" || verdict == "reject") run[cat] = 0
          }
        } else {
          escapes++
          traced = artifact; slash = index(traced, "/")
          source = substr(traced, 1, slash - 1); inner = substr(traced, slash + 1)
          if (slash == 0 || auto[source SUBSEP inner SUBSEP dig SUBSEP cat] != "approve")
            print "ERR\t" change "\t" where ": escape が指す自動承認 " artifact " @ " dig " が台帳にありません"
          if (index(lessons, " " substr(verdict, 8, 3) " ") == 0)
            print "ERR\t" change "\t" where ": escape が指す " verdict " が docs/sdlc/lessons.md にありません"
          demoted[cat] = 1; run[cat] = 0
        }
      }
      END {
        for (k in owed) print "ERR\t" owed[k]
        count = 0
        m = split(categories, order, " ")
        for (i = 1; i <= m; i++) if (demoted[order[i]]) { print "DEMOTED\t" order[i]; count++ }
        print "COUNT\tauto_approvals\t" approvals + 0
        print "COUNT\tapproval_escapes\t" escapes + 0
        print "COUNT\tdemoted_categories\t" count
      }'
}

# The number a change directory starts with, as a base-10 integer.
number() {
  local digits
  digits=$(basename "$1" | sed -n 's/^\([0-9][0-9]*\)-.*/\1/p')
  [ -n "$digits" ] || { echo 0; return; }
  echo $((10#$digits))
}

# Which paused surfaces this branch's files touch. The category on a ledger
# line is its writer's word; the files are not, so a plan that edits a paused
# surface still demoted needs a person whatever category it was logged under.
# Measured against the fork from main plus what is uncommitted, the same
# "this tree's work" the Stop gate uses.
touched_paused() {
  local fork
  fork=$(git merge-base refs/heads/main HEAD 2>/dev/null) || return 0
  {
    # --no-renames: a file moved off a surface is a deletion from it, and rename
    # detection would list only the new path, which matches no prefix.
    git -c core.quotePath=false diff --no-renames --name-only "$fork" 2>/dev/null
    git -c core.quotePath=false ls-files --others --exclude-standard 2>/dev/null
  } | awk '
    NR == FNR {
      if ($0 ~ /^  [a-z-]+: "[a-z]+ paused [a-z]+ /) {
        name = $1; sub(/:$/, "", name)
        paths = $0; sub(/.* paused [a-z]+ /, "", paths); sub(/".*/, "", paths)
        count[name] = split(paths, list, ",")
        for (i = 1; i <= count[name]; i++) prefix[name, i] = list[i]
      }
      next
    }
    { for (s in count) for (i = 1; i <= count[s]; i++) if (index($0, prefix[s, i]) == 1) hit[s] = 1 }
    END { for (s in hit) print s }' docs/sdlc/risk.yaml -
}

# An artifact that calls itself approved, with no approve line at its digest.
# A person's verdict at that digest outranks the evaluator's, and the last one
# written stands: an approve a person later revised is not an approval.
unbacked_claims() {
  local change=$1 dir="$changes_root/$1" artifact status want need_person held surface
  for artifact in intent.md spec.md plan.md screen.md; do
    [ -f "$dir/$artifact" ] || continue
    case "$artifact" in
      screen.md)
        grep -E '^  screen: "approved"' "$dir/state.yaml" >/dev/null 2>&1 || continue ;;
      plan.md)
        status=$(sed -n 's/^- \*\*Status\*\*: *//p' "$dir/$artifact" | head -1)
        if [ -z "$status" ] || [ "$status" = draft ]; then continue; fi ;;
      *)
        grep -E '^- \*\*Status\*\*: *approved' "$dir/$artifact" >/dev/null || continue ;;
    esac
    want=$(digest "$dir/$artifact")
    need_person=0
    held=""
    if [ "$artifact" = plan.md ]; then
      for surface in $touched; do
        printf '%s\n' "$verdicts" | grep -xF "$(printf 'DEMOTED\t%s' "$surface")" >/dev/null && held="$held $surface"
      done
      [ -n "$held" ] && need_person=1
    fi
    # The same lines ledger() reads: a commented-out approval is a retracted one.
    awk -v a="$artifact" -v d="$want" -v need_person="$need_person" '
      !/^[[:space:]]*#/ && $4 == a && $5 == d {
        if ($2 == "person") { person = $6; seen = 1 } else if ($2 == "auto") auto = $6
      }
      END { exit !(seen ? person == "approve" : (!need_person && auto == "approve")) }' \
      "$dir/approvals.log" 2>/dev/null && continue
    if [ "$need_person" = 1 ]; then
      printf '%s/%s は人の判断に戻っている paused 面 (%s) を編集します。digest %s に person の approve が要ります。\n' \
        "$change" "$artifact" "${held# }" "$want"
    else
      printf '%s/%s は approved を名乗っていますが、現在の digest %s に approve の行がありません。\n' \
        "$change" "$artifact" "$want"
    fi
  done
  if grep -E '^  screen: "approved"' "$dir/state.yaml" >/dev/null 2>&1 && [ ! -f "$dir/screen.md" ]; then
    printf '%s は screen: approved ですが、承認された layout の screen.md がありません。\n' "$change"
  fi
}

mode=check
case "${1:-}" in
  --summary) mode=summary; shift ;;
  --metrics) mode=metrics; shift ;;
  --list) mode=list; shift ;;
  --*|"") echo "usage: check-approvals.sh <change-dir>... | --summary <change-dir>... | --metrics | --list [category]" >&2; exit 64 ;;
esac

verdicts=$(judge)

case "$mode" in
  metrics)
    printf '%s\n' "$verdicts" | awk -F'\t' '$1 == "COUNT" { print $2 " " $3 }'
    exit 0 ;;
  list)
    ledger | awk -F'\t' -v want="${1:-}" '{ split($3, f, " ") } want == "" || f[3] == want { print $1 "  " $3 }'
    exit 0 ;;
esac

wanted=""
for arg in "$@"; do
  change=$(basename "$arg")
  [ -d "$changes_root/$change" ] || { echo "no such change: $arg" >&2; exit 64; }
  [ "$(number "$change")" -ge "$first_checked" ] || continue
  wanted="$wanted $change"
done

if [ "$mode" = summary ]; then
  [ -n "$wanted" ] || exit 0
  printf '%s\n' "$verdicts" | awk -F'\t' -v wanted="$wanted " '
    $1 == "SEEN" && index(wanted, " " $2 " ") {
      if ($3 == "auto" && $5 == "approve" && $6 != "shadow") { auto++; per[$4]++ }
      else if ($3 == "person") person++
      else if ($3 == "escape") escapes++
    }
    $1 == "DEMOTED" { demoted = demoted (demoted == "" ? "" : "・") $2 }
    END {
      if (auto + person + escapes == 0) exit
      split("intent spec plan screen contract", order, " "); kinds = ""
      for (i = 1; i <= 5; i++) if (order[i] in per) { kinds = kinds (kinds == "" ? "" : "・") order[i] " " per[order[i]]; delete per[order[i]] }
      for (k in per) kinds = kinds (kinds == "" ? "" : "・") k " " per[k]
      printf "自動承認 %d 件%s · 人へ %d · 逃れ %d · 降格中 %s — bash scripts/check-approvals.sh --list で抜き取り確認できます\n",
        auto, (kinds == "" ? "" : "（" kinds "）"), person, escapes, (demoted == "" ? "なし" : demoted)
    }'
  exit 0
fi

touched=$(touched_paused)
findings=""
for change in $wanted; do
  findings="$findings$(printf '%s\n' "$verdicts" | awk -F'\t' -v c="$change" '$1 == "ERR" && $2 == c { print $3 }')
$(unbacked_claims "$change")
"
done
findings=$(printf '%s' "$findings" | grep -v '^$')
[ -z "$findings" ] && exit 0
printf '承認台帳:\n%s\n' "$(printf '%s\n' "$findings" | sed 's/^/  /')"
exit 1
