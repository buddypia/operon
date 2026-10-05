#!/usr/bin/env bash
# Compare the current harness reading against the control bands, and name the
# tier of any breach. Stage 6 of the pipeline in docs/sdlc/README.md.
#
#   bash scripts/check-bands.sh                    # read the repository now
#   bash scripts/check-bands.sh --gates            # also time the gate run
#   bash scripts/check-bands.sh --metrics f.json   # judge a saved reading
#
# Exit 0 when nothing breached or the worst breach is warn/diagnose; exit 1 when
# a metric reached its `propose` threshold, which means: write an intent.md and
# re-enter the pipeline at stage 1.
#
# Bands live in docs/sdlc/bands.yaml, and the format is described there.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

bands="docs/sdlc/bands.yaml"
metrics_file=""
gates=""
while [ $# -gt 0 ]; do
  case "$1" in
    --gates) gates="--gates"; shift ;;
    --metrics) metrics_file="${2:?--metrics needs a file}"; shift 2 ;;
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "usage: check-bands.sh [--gates] [--metrics FILE]" >&2; exit 2 ;;
  esac
done

if [ -z "$metrics_file" ]; then
  metrics_file=$(mktemp)
  trap 'rm -f "$metrics_file"' EXIT
  bash scripts/harness-metrics.sh $gates > "$metrics_file"
fi

# --- the bands ---------------------------------------------------------------
#
# One `key: "direction baseline warn diagnose propose"` per line under
# `metrics:`. Parsed here rather than by a YAML library so the file cannot drift
# from its parser.

read_bands() {
  awk '
    /^metrics:/ { inside = 1; next }
    /^[^ #]/    { inside = 0 }
    inside && /^  [a-z_]+: *"/ {
      key = $1; sub(/:$/, "", key)
      if (match($0, /"[^"]*"/)) {
        print key, substr($0, RSTART + 1, RLENGTH - 2)
      }
    }
  ' "$bands"
}

tier_of() {
  local direction=$1 value=$2 warn=$3 diagnose=$4 propose=$5
  if [ "$direction" = "max" ]; then
    if   [ "$value" -ge "$propose"  ]; then echo propose
    elif [ "$value" -ge "$diagnose" ]; then echo diagnose
    elif [ "$value" -ge "$warn"     ]; then echo warn
    else echo ok
    fi
  else
    if   [ "$value" -le "$propose"  ]; then echo propose
    elif [ "$value" -le "$diagnose" ]; then echo diagnose
    elif [ "$value" -le "$warn"     ]; then echo warn
    else echo ok
    fi
  fi
}

action_for() {
  case "$1" in
    warn)     echo "recorded, no action" ;;
    diagnose) echo "read-only investigation: what moved, and when" ;;
    propose)  echo "write an intent.md and re-enter at stage 1" ;;
  esac
}

# Whether the thing `propose` prescribes actually exists, and what state it is
# in. Nothing used to connect the breach to its answer, and the cost of that was
# measured before this was written: change 030 wrote the prescribed intent.md on
# 2026-09-06 and then sat at `awaiting-user`, while fifteen changes recorded
# `steering_bytes` at propose and waived it. Each waiver was reasonable on its
# own and the aggregate was a band firing into nothing, because a response that
# is written and parked reads — change after change — exactly like a response
# nobody wrote. The difference is a fact on disk, so the report states it rather
# than leaving the next session to go and look.
#
# Deliberately not a refusal. This script's exit code already says `propose` was
# reached; a second gate on whether the answer is progressing would be a gate on
# a person's availability, which is what `awaiting-user` exists to record.
#
# The match is a declaration and not a mention, and the first cut got that
# wrong: searching every intent.md for the metric name answered seven changes
# for `steering_bytes`, of which one was the response and the rest name the
# number in passing — including two at `done`, which is precisely the reading
# that would tell a session the band had been answered when it had not. So a
# change that answers a band says so, in one line its own template documents:
#
#   **Answers band**: steering_bytes
#
# A declaration can be wrong, but only by being written; a mention is wrong by
# accident, which is the failure mode this repository keeps finding.
# The three states are not two. The first cut of this split a declaring change
# into "parked" and "live", and `done` fell on the live side — so the moment the
# answer shipped, the report named it with no further comment and a session read
# that as an answer in progress. Measured on this repository the day it was
# written: change 030 is the only declarer of `steering_bytes`, it closed at
# `done`, and a forced breach printed `prescribed response: 030-... (status
# done)` and nothing else, while bands.yaml two files over says in its own
# comment that the cut did not reach the old tier. A finished answer and a
# running answer call for opposite things, which is the whole point of entry 025
# — so `done` and `archived` are their own case and say so.
prescribed_response_for() {
  local metric=$1
  # Overridable so the branches below can be exercised against fixtures instead
  # of against whatever this repository's change directories happen to hold
  # today — the report is the thing under test and it has four outcomes, three
  # of which the real tree cannot produce at will. It changes what is REPORTED
  # and never the exit code, which is set from the tier alone.
  local changes="${OPERON_CHANGES_DIRECTORY:-docs/sdlc/changes}"
  if [ ! -d "$changes" ]; then
    echo "  prescribed response: cannot tell — $changes is not in this tree"
    return
  fi
  local found=0 live=0 closed=0 parked=0 intent dir status
  for intent in "$changes"/*/intent.md; do
    [ -f "$intent" ] || continue
    # Tokenised rather than matched inside the line, the same way the guard in
    # src/tests.rs tokenises it: strip the prefix, split on whitespace, strip
    # backticks off the ends. Two spellings of the split have already been
    # wrong in opposite directions. A regex requiring a separator after the
    # colon made `**Answers band**:steering_bytes` invisible here and visible
    # there; splitting on `[^A-Za-z0-9_]+` instead made the hyphen a separator,
    # so `**Answers band**: not-steering_bytes` answered a band it does not
    # declare — while Rust's `split_whitespace` reads that as one token and
    # does not. Either way it is two acceptance sets for one rule, which is
    # lesson 001 in a new place, so the split is whitespace in both.
    awk -v metric="$metric" '
      /^\*\*Answers band\*\*:/ {
        rest = substr($0, index($0, ":") + 1)
        count = split(rest, parts, /[[:space:]]+/)
        for (index_ = 1; index_ <= count; index_++) {
          token = parts[index_]
          gsub(/^`+|`+$/, "", token)
          if (token == metric) { found = 1 }
        }
      }
      END { exit found ? 0 : 1 }
    ' "$intent" || continue
    found=$((found + 1))
    dir=$(dirname "$intent")
    status=$(awk '/^ *status:/ { gsub(/["'"'"',]/, "", $2); print $2; exit }' \
      "$dir/state.yaml" 2>/dev/null || true)
    [ -n "$status" ] || status="no status recorded"
    case "$status" in
      done|archived)                closed=$((closed + 1)) ;;
      awaiting-user|blocked|failed) parked=$((parked + 1)) ;;
      *)                            live=$((live + 1)) ;;
    esac
    echo "  prescribed response: ${dir##*/} (status $status)"
  done
  if [ "$found" -eq 0 ]; then
    echo "  prescribed response: none — no intent.md under $changes declares"
    echo "                       \`**Answers band**: $metric\`, which is what this tier asks for"
  elif [ "$live" -gt 0 ]; then
    : # An answer is being worked on. The line above already names it.
  elif [ "$closed" -gt 0 ] && [ "$parked" -gt 0 ]; then
    echo "  every change answering $metric has closed or is parked, so this breach"
    echo "  is against an answer that is finished or waiting, not one in progress"
  elif [ "$closed" -gt 0 ]; then
    echo "  every change answering $metric has already closed, so this is a NEW"
    echo "  breach against an answer that has shipped — read what it achieved"
    echo "  before opening another one"
  else
    echo "  every change answering $metric is parked, so this breach has nowhere to go"
  fi
}

# --- judge -------------------------------------------------------------------

worst=ok
checked=0
skipped=()
# Two counts, not one: `breaches` is what gets printed and a `propose` breach
# adds explanatory lines to it, so counting its length would report more
# breached metrics than there are. The first version of this did exactly that —
# two metrics over, "4 of 20 metrics breached" in the header.
breached=0
breaches=()

while read -r key spec; do
  if [ -z "${key:-}" ]; then continue; fi
  # shellcheck disable=SC2086
  set -- $spec
  direction=$1 baseline=$2 warn=$3 diagnose=$4 propose=$5

  value=$(jq -r --arg k "$key" '.[$k] // "null"' "$metrics_file")
  if ! printf '%s' "$value" | grep -E '^-?[0-9]+$' >/dev/null; then
    skipped+=("$key (not measured in this reading)")
    continue
  fi
  checked=$((checked + 1))

  tier=$(tier_of "$direction" "$value" "$warn" "$diagnose" "$propose")
  if [ "$tier" = ok ]; then continue; fi

  case "$tier" in
    warn)     threshold=$warn ;;
    diagnose) threshold=$diagnose ;;
    propose)  threshold=$propose ;;
  esac
  breached=$((breached + 1))
  breaches+=("$(printf '%-22s %-8s value %s, baseline %s, %s tier at %s (%s) — %s' \
    "$key" "$tier" "$value" "$baseline" "$tier" "$threshold" "$direction" \
    "$(action_for "$tier")")")
  if [ "$tier" = propose ]; then
    while IFS= read -r response_line; do
      [ -n "$response_line" ] && breaches+=("$response_line")
    done <<< "$(prescribed_response_for "$key")"
  fi

  case "$tier" in
    propose)
      worst=propose ;;
    diagnose)
      if [ "$worst" != propose ]; then worst=diagnose; fi ;;
    warn)
      if [ "$worst" = ok ]; then worst=warn; fi ;;
  esac
done <<< "$(read_bands)"

# --- report ------------------------------------------------------------------

if [ "$breached" -eq 0 ]; then
  echo "bands: $checked metrics within their bands"
else
  echo "bands: $breached of $checked metrics breached"
  printf '  %s\n' "${breaches[@]}"
fi
if [ ${#skipped[@]} -gt 0 ]; then
  printf 'skipped: %s\n' "${skipped[@]}"
fi

if [ "$worst" = propose ]; then
  echo "a metric reached its propose threshold — see stage 6 in docs/sdlc/README.md" >&2
  exit 1
fi
exit 0
