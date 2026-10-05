#!/usr/bin/env bash
# What the pipeline actually did, read out of git history. Stage 6 of the
# pipeline in docs/sdlc/README.md — the measuring half of it.
#
#   bash scripts/pipeline-indicators.sh              # every change, as JSON
#   bash scripts/pipeline-indicators.sh --steering   # steering_bytes per commit
#   bash scripts/pipeline-indicators.sh --breaches   # when each band was crossed
#
# The AI-native SDLC playbook gives every play a mechanism and a pair of
# indicators. Change 019 built the mechanisms; nothing computed the indicators,
# so the pipeline could not say whether it was getting faster or slower. These
# are the ones the playbook names that are honestly available here: it asks for
# PR metadata and an incident tracker for the lagging half, and this repository
# is local-first with neither. What it does have is thirty-six change
# directories and the git history that made them, which is enough for every
# leading indicator in stages 1, 2 and 3.
#
# Nothing here is a gate. It reports; a person decides.
set -uo pipefail
# `git replace` and the graft file both make a later `git` call read different
# objects, and every number below is read from history. Entry in REVIEW.md.
export GIT_NO_REPLACE_OBJECTS=1
export GIT_GRAFT_FILE=/dev/null

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode=changes
case "${1:-}" in
  --steering) mode=steering ;;
  --breaches) mode=breaches ;;
  --lessons) mode=lessons ;;
  --workflows) mode=workflows ;;
  "") ;;
  *) echo "usage: pipeline-indicators.sh [--steering | --breaches | --lessons | --workflows]" >&2; exit 2 ;;
esac

# The commit that first added a path, as a unix timestamp. Empty when the path
# was never committed — a change whose intent.md is still only in the working
# tree has no elapsed time yet, and saying nothing is right.
added_at() {
  git log --diff-filter=A --follow --format=%at --reverse -- "$1" 2>/dev/null | head -1
}

# How many commits touched a path after a given timestamp. This is the
# playbook's rework indicator: a spec rewritten after the build began.
commits_after() {
  local path=$1 since=$2
  [ -n "$since" ] || { echo 0; return; }
  git log --format=%at -- "$path" 2>/dev/null | awk -v s="$since" '$1 > s' | wc -l | tr -d ' '
}

days_between() {
  local from=$1 to=$2
  if [ -z "$from" ] || [ -z "$to" ]; then echo null; return; fi
  awk -v a="$from" -v b="$to" 'BEGIN { printf "%.2f", (b - a) / 86400 }'
}

# Can the CI plays run at all? A GitHub Actions workflow runs when GitHub gets a
# push, a pull request, or a schedule tick for a repository it hosts. With no
# remote, none of those can happen, and a workflow file is a description of an
# intention that reads — to anyone opening the repository, including the next
# agent — exactly like a working CI. Entry 003 in lessons.md is a doc comment
# naming a script that was never committed; this is that shape at the scale of
# four plays.
if [ "$mode" = workflows ]; then
  remote=$(git remote 2>/dev/null | head -1)
  count=0
  for workflow in .github/workflows/*.yml .github/workflows/*.yaml; do
    [ -f "$workflow" ] || continue
    count=$((count + 1))
  done
  if [ "$count" = 0 ]; then
    echo "no workflows"
    exit 0
  fi
  if [ -n "$remote" ]; then
    printf '%s workflow(s), remote `%s` — they can run\n' "$count" "$remote"
    exit 0
  fi
  printf '%s workflow(s) and no git remote — none of them has ever run or can run:\n' "$count"
  for workflow in .github/workflows/*.yml .github/workflows/*.yaml; do
    [ -f "$workflow" ] || continue
    added=$(git log --diff-filter=A --format=%ad --date=short -- "$workflow" 2>/dev/null | tail -1)
    printf '  %-28s added %s\n' "$(basename "$workflow")" "${added:-uncommitted}"
  done
  printf '\nEither the local gates are the CI for this tool, and these should say so or\ngo, or this repository is meant to have a remote and does not. See\ndocs/sdlc/changes/038-the-harness-is-built-but-not-measured/intent.md\n'
  exit 1
fi

# Stage 6's lagging indicator, in the only form available here: the share of
# recorded findings that became something that runs. `docs/sdlc/lessons.md` says
# an entry is unfinished until its Guard column names what now catches it, so the
# question is whether each Guard names a test or an eval that actually exists.
# The playbook asks for findings-that-became-merged-fixes against a PR history;
# this is that ratio, read from what a local-first repository does keep.
if [ "$mode" = lessons ]; then
  lessons=docs/sdlc/lessons.md
  [ -f "$lessons" ] || { echo "no $lessons" >&2; exit 2; }
  total=0
  landed=0
  missing=""
  while read -r entry; do
    [ -n "$entry" ] || continue
    total=$((total + 1))
    guard=$(awk -v e="$entry" '
      $0 == e { inside = 1; next }
      inside && /^## / { exit }
      inside && /^\*\*Guard\.\*\*/ { collecting = 1 }
      collecting { print }
    ' "$lessons")
    # Every backticked identifier in the Guard paragraph that looks like a test
    # name or an eval path, and whether the repository holds it.
    names=$(printf '%s\n' "$guard" | grep -o '`[A-Za-z0-9_/.-]*`' | tr -d '`' \
      | grep -E '^([a-z0-9_]{8,}|evals/[0-9].*\.md)$' | sort -u)
    held=0
    for name in $names; do
      case "$name" in
        evals/*) [ -f "$name" ] && held=1 ;;
        *) grep -F "fn $name(" src/tests.rs >/dev/null 2>&1 && held=1 ;;
      esac
    done
    if [ "$held" = 1 ]; then
      landed=$((landed + 1))
    else
      missing="$missing
  $entry"
    fi
  done <<EOF
$(grep '^## [0-9]' "$lessons")
EOF
  printf 'lessons: %s of %s name a guard this repository holds\n' "$landed" "$total"
  if [ -n "$missing" ]; then
    printf '\nno guard found for:%s\n' "$missing"
    exit 1
  fi
  exit 0
fi

if [ "$mode" = changes ]; then
  printf '{\n  "changes": [\n'
  first=1
  for dir in docs/sdlc/changes/*/; do
    [ -f "$dir/state.yaml" ] || continue
    change=$(basename "$dir")
    route=$(sed -n 's/^  route: "\(.*\)"$/\1/p' "$dir/state.yaml" | head -1)
    stage=$(sed -n 's/^  stage: "\(.*\)"$/\1/p' "$dir/state.yaml" | head -1)
    status=$(sed -n 's/^  status: "\(.*\)"$/\1/p' "$dir/state.yaml" | head -1)
    intent_at=$(added_at "$dir/intent.md")
    spec_at=$(added_at "$dir/spec.md")
    plan_at=$(added_at "$dir/plan.md")
    # Stage 2's leading indicator: intent committed to spec committed.
    intent_to_spec=$(days_between "$intent_at" "$spec_at")
    # Stage 3's: spec committed to plan committed.
    spec_to_plan=$(days_between "$spec_at" "$plan_at")
    # Stage 2's lagging indicator, the only one available here: spec commits
    # dated after the first plan commit — requirements rework after build began.
    spec_rework=$(commits_after "$dir/spec.md" "$plan_at")
    plan_rework=$(commits_after "$dir/plan.md" "$plan_at")
    [ $first = 1 ] || printf ',\n'
    first=0
    printf '    {"change": "%s", "route": "%s", "stage": "%s", "status": "%s",' \
      "$change" "$route" "$stage" "$status"
    printf ' "intent_to_spec_days": %s, "spec_to_plan_days": %s,' \
      "$intent_to_spec" "$spec_to_plan"
    # Stage 5's leading indicator: time to first review. `review.yaml` is the
    # record a reviewer's verdict lands in, so its first commit is the moment
    # this change was first judged rather than merely written.
    review_at=$(added_at "$dir/review.yaml")
    plan_to_review=$(days_between "$plan_at" "$review_at")
    printf ' "spec_commits_after_plan": %s, "plan_revisions": %s,' \
      "$spec_rework" "$plan_rework"
    printf ' "plan_to_review_days": %s}' "$plan_to_review"
  done
  printf '\n  ]\n}\n'
  exit 0
fi

# steering_bytes is a sum of file sizes, so its whole history is already in git:
# no stored series is needed to see the trend. This answers, with a measurement
# rather than an opinion, the question of whether a metrics series has to be a
# committed file — for this metric it does not.
steering_paths() {
  printf '%s\n' CLAUDE.md AGENTS.md DESIGN.md CONTRIBUTING.md REVIEW.md
  git ls-tree -r --name-only "$1" -- .claude/agents .claude/skills docs/sdlc 2>/dev/null \
    | grep -E '(\.claude/agents/.*\.md|SKILL\.md|docs/sdlc/[^/]*\.md|docs/sdlc/templates/[^/]*\.md)$'
}

steering_at() {
  local rev=$1 total=0 size
  while read -r path; do
    [ -n "$path" ] || continue
    size=$(git cat-file -s "$rev:$path" 2>/dev/null || echo 0)
    total=$((total + size))
  done <<EOF
$(steering_paths "$rev")
EOF
  echo "$total"
}

if [ "$mode" = steering ]; then
  printf '%-12s %-11s %-9s %s\n' commit date bytes subject
  git log --format='%h %at %s' --reverse -- CLAUDE.md AGENTS.md REVIEW.md docs/sdlc 2>/dev/null \
    | while read -r sha when subject; do
        printf '%-12s %-11s %-9s %s\n' "$sha" \
          "$(date -r "$when" +%Y-%m-%d 2>/dev/null || echo "$when")" \
          "$(steering_at "$sha")" "$subject"
      done
  exit 0
fi

# When was a band first crossed, and by which commit? The playbook's stage 6
# indicator is the time from a breach to an intent.md; a breach nothing recorded
# has no start time, so this recovers it from history instead of asking for a
# mechanism that was never built.
bands="docs/sdlc/bands.yaml"
[ -f "$bands" ] || { echo "no $bands" >&2; exit 2; }
set -f
set -- $(sed -n 's/^  steering_bytes: "\(.*\)"$/\1/p' "$bands")
set +f
if [ "$#" -ne 5 ]; then
  echo "steering_bytes band is not the five-field tuple this reads" >&2
  exit 2
fi
baseline=$2 warn=$3 diagnose=$4 propose=$5
printf 'steering_bytes bands: baseline %s  warn %s  diagnose %s  propose %s\n\n' \
  "$baseline" "$warn" "$diagnose" "$propose"
for tier in warn diagnose propose; do
  case "$tier" in
    warn) threshold=$warn ;;
    diagnose) threshold=$diagnose ;;
    propose) threshold=$propose ;;
  esac
  crossed=""
  while read -r sha when subject; do
    [ -n "$sha" ] || continue
    value=$(steering_at "$sha")
    if [ "$value" -ge "$threshold" ]; then
      crossed="$sha $when $value $subject"
      break
    fi
  done <<EOF
$(git log --format='%h %at %s' --reverse -- CLAUDE.md AGENTS.md REVIEW.md docs/sdlc 2>/dev/null)
EOF
  if [ -n "$crossed" ]; then
    set -f
    set -- $crossed
    set +f
    sha=$1; when=$2; value=$3; shift 3
    printf '%-9s crossed at %s on %s (%s bytes) — %s\n' "$tier" "$sha" \
      "$(date -r "$when" +%Y-%m-%d 2>/dev/null || echo "$when")" "$value" "$*"
  else
    printf '%-9s not crossed\n' "$tier"
  fi
done
exit 0
# operon: end of scripts/pipeline-indicators.sh
