#!/usr/bin/env bash
# Run the eval suite: real prompts, deterministic checks, throwaway worktrees.
# Stage 4 Play 2 of the pipeline in docs/sdlc/README.md.
#
# The suite in src/tests.rs says whether the code is right. These say whether the
# *steering* is right — whether an agent handed a real prompt, with only what this
# repository tells it, does the thing the repository wants. Every eval is seeded
# from a mistake that actually happened; see docs/sdlc/lessons.md.
#
#   bash scripts/run-evals.sh --list
#   bash scripts/run-evals.sh                      # all of them, once each
#   bash scripts/run-evals.sh --only 003
#   bash scripts/run-evals.sh --only 003 --keep     # leave the worktree to inspect
#   bash scripts/run-evals.sh --from main           # judge a ref other than HEAD
#   bash scripts/run-evals.sh --runs 3 --recheck    # a baseline: see evals/README.md
#   bash scripts/run-evals.sh --split train         # only the evals a hillclimb reads
#   bash scripts/run-evals.sh --model haiku --effort low
#
# Exit 0 every run passed, 1 a run that started did not pass, 3 nothing could
# start.
#
# One agent run is one sample, and agent runs vary. `--runs N` repeats each eval
# in a fresh worktree and the summary reports a pass rate with a 95% Wilson
# interval, so a change can be told apart from noise. A run ends in one outcome:
#
#   pass      the check accepted the agent's work
#   fail      the check rejected it — this is what the steering is judged on
#   infra     setup failed, the checks could not be hidden or put back, the
#             agent timed out, or its transcript carries no successful result.
#             Not a verdict on the steering; not in the rate, but counted.
#   unstable  with --recheck, the check said two different things about the same
#             tree. The check is broken; not in the rate.
#
# Every run leaves its transcript and check output under
# target/eval-results/<stamp>-<sha>/, with results.tsv and summary.md beside them.
#
# Each eval runs in a worktree of a committed ref, on a throwaway `eval/`
# branch that is deleted afterwards (kept with --keep), so what is being
# judged is the steering as committed — not whatever is in the working tree.
# While the agent works, every eval file in that worktree is cut down to its
# prompt: the check, the setup, and the `guards:` line naming the test the check
# leans on are the answer, and an agent that can read the answer is not being
# measured. A hide that fails makes the run `infra`. The worktrees live under
# $TMPDIR, not in the repository, so this checkout is not an ancestor of them.
# A transcript whose tool calls mention `evals/` anyway is flagged `peeked`. The cargo cache is shared with the main checkout through
# CARGO_TARGET_DIR, because a cold build per eval costs more than the eval.
#
#   EVAL_AGENT    the agent command, default `claude`. The runner's own tests
#                 point it at a stub.
#   EVAL_TIMEOUT  seconds one agent run may take, default 1800. Applied when
#                 `timeout` or `gtimeout` exists.
set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

only=""
keep=0
ref="HEAD"
list=0
runs=1
recheck=0
split=""
model=""
effort=""
while [ $# -gt 0 ]; do
  case "$1" in
    --list) list=1; shift ;;
    --only) only="${2:?--only needs an eval id}"; shift 2 ;;
    --from) ref="${2:?--from needs a ref}"; shift 2 ;;
    --keep) keep=1; shift ;;
    --runs) runs="${2:?--runs needs a count}"; shift 2 ;;
    --recheck) recheck=1; shift ;;
    --split) split="${2:?--split needs train or test}"; shift 2 ;;
    --model) model="${2:?--model needs a model}"; shift 2 ;;
    --effort) effort="${2:?--effort needs a level}"; shift 2 ;;
    -h|--help) sed -n '2,48p' "$0"; exit 0 ;;
    *) echo "usage: run-evals.sh [--list] [--only ID] [--split train|test] [--from REF] [--runs N] [--recheck] [--model M] [--effort E] [--keep]" >&2; exit 2 ;;
  esac
done
case "$runs" in
  ''|*[!0-9]*|0) echo "--runs needs a positive count, not '$runs'" >&2; exit 2 ;;
esac
case "$split" in
  ''|train|test) ;;
  *) echo "--split is train or test, not '$split'" >&2; exit 2 ;;
esac

# --- eval file format ---------------------------------------------------------
#
# Front matter, then `## Setup` (optional), `## Prompt`, `## Check`, each
# followed by one fenced block. every_eval_states_a_prompt_and_a_check in
# src/tests.rs is what keeps an eval from being half-written.

field() { sed -n "s/^$2: *//p" "$1" | head -1; }

section() {
  awk -v want="## $2" '
    $0 == want { in_section = 1; next }
    in_section && /^```/ { if (in_block) exit; in_block = 1; next }
    in_section && in_block { print }
  ' "$1"
}

# What the agent is allowed to see of an eval: who it is and what was asked.
# The placeholder check keeps the file well-formed for the suite's own tests,
# which an agent may run.
hidden_form() {
  printf -- '---\nid: %s\ntitle: %s\nsplit: %s\n---\n\n## Prompt\n\n```text\n' \
    "$(field "$1" id)" "$(field "$1" title)" "$(field "$1" split)"
  section "$1" Prompt
  printf '```\n\n## Check\n\n```sh\n# Hidden by scripts/run-evals.sh while an agent runs.\nexit 0\n```\n'
}

all=$(ls evals/[0-9][0-9][0-9]-*.md 2>/dev/null || true)
if [ -z "$all" ]; then
  echo "no evals under evals/" >&2
  exit 3
fi

evals=""
for file in $all; do
  if [ -n "$only" ] && [ "$(field "$file" id)" != "$only" ]; then continue; fi
  if [ -n "$split" ] && [ "$(field "$file" split)" != "$split" ]; then continue; fi
  evals="$evals $file"
done
if [ -z "$evals" ]; then
  echo "no eval matched${only:+ --only $only}${split:+ --split $split}" >&2
  exit 3
fi

if [ "$list" -eq 1 ]; then
  for file in $evals; do
    printf '%-6s %-6s %-38s guards: %s\n' \
      "$(field "$file" id)" "$(field "$file" split)" "$(field "$file" title)" "$(field "$file" guards)"
  done
  exit 0
fi

agent="${EVAL_AGENT:-claude}"
if ! command -v "$agent" >/dev/null 2>&1; then
  echo "skipped: no '$agent' on PATH, so no eval could run. The suite judges" >&2
  echo "steering by running an agent against it; there is no offline form." >&2
  exit 3
fi

limit="${EVAL_TIMEOUT:-1800}"
bound=""
if command -v timeout >/dev/null 2>&1; then
  bound="timeout $limit"
elif command -v gtimeout >/dev/null 2>&1; then
  bound="gtimeout $limit"
else
  echo "note: no timeout command, so agent runs are unbounded" >&2
fi

agent_options=(--permission-mode acceptEdits --output-format stream-json --verbose)
if [ -n "$model" ]; then agent_options+=(--model "$model"); fi
if [ -n "$effort" ]; then agent_options+=(--effort "$effort"); fi

# --- run ----------------------------------------------------------------------

export CARGO_TARGET_DIR="$repo_root/target/eval"
base=$(git rev-parse --short "$ref")
stamp=$(date -u +%Y%m%dT%H%M%SZ)
results="$repo_root/target/eval-results/$stamp-$base"
mkdir -p "$results"
printf 'id\trun\toutcome\tpeeked\tcost_usd\tduration_ms\n' > "$results/results.tsv"
# Outside the repository. A worktree under it would have this checkout's own
# evals one `../` away, and the agent would load this checkout's CLAUDE.md as
# an ancestor of the one at $ref — judging the working tree, not the commit.
worktrees=$(mktemp -d "${TMPDIR:-/tmp}/operon-evals.XXXXXX")

check_passes() {
  ( cd "$worktree" && eval "$check" ) > "$1" 2>&1
}

for file in $evals; do
  id=$(field "$file" id)
  title=$(field "$file" title)
  setup=$(section "$file" Setup)
  prompt=$(section "$file" Prompt)
  check=$(section "$file" Check)

  run=1
  while [ "$run" -le "$runs" ]; do
    out="$results/$id/$run"
    mkdir -p "$out"
    worktree="$worktrees/$id-$run"
    git worktree prune
    echo "=== eval $id run $run/$runs — $title (worktree of $base)"

    outcome=""
    peeked=no
    cost=""
    duration=""
    # A named branch, not --detach: the agent runs under this repository's
    # hooks, and the worktree-policy guard refuses every edit on a branch it
    # cannot name (sdlc 104). eval/ is not a protected branch.
    branch="eval/$stamp-$$-$id-$run"
    if ! git worktree add -b "$branch" "$worktree" "$ref" >/dev/null 2>&1; then
      echo "  could not create a worktree at $ref" >&2
      outcome=infra
    fi

    if [ -z "$outcome" ] && [ -n "$setup" ]; then
      if ! ( cd "$worktree" && eval "$setup" ) > "$out/setup.log" 2>&1; then
        echo "  setup failed — $out/setup.log" >&2
        outcome=infra
      fi
    fi

    hidden=""
    if [ -z "$outcome" ]; then
      # Hide the answers. skip-worktree keeps the rewrite out of `git status`
      # and `git diff`, which is what every check reads. A hide that did not
      # take is not a run: the agent would work with the answer in front of it.
      hidden=$(cd "$worktree" && git ls-files 'evals/[0-9]*.md')
      if ! ( cd "$worktree" && git update-index --skip-worktree $hidden \
          && for each in $hidden; do hidden_form "$each" > "$each.hidden" && mv "$each.hidden" "$each" || exit 1; done ) \
          > "$out/hide.log" 2>&1; then
        echo "  could not hide the checks — $out/hide.log" >&2
        outcome=infra
      fi
    fi

    if [ -z "$outcome" ]; then
      ( cd "$worktree" && $bound "$agent" -p "$prompt" "${agent_options[@]}" ) \
        > "$out/transcript.jsonl" 2> "$out/agent.stderr"
      agent_status=$?
    fi

    # Put the real files back whether or not the agent ran. A check run over
    # the hidden form would judge the wrong tree.
    if [ -n "$hidden" ] \
        && ! ( cd "$worktree" && git update-index --no-skip-worktree $hidden && git checkout -- $hidden ) \
          >> "$out/hide.log" 2>&1; then
      echo "  could not restore the evals — $out/hide.log" >&2
      outcome=infra
    fi

    if [ -z "$outcome" ]; then
      result=$(grep '"type":"result"' "$out/transcript.jsonl" | tail -1)
      cost=$(printf '%s' "$result" | sed -n 's/.*"total_cost_usd":\([0-9.eE+-]*\).*/\1/p')
      duration=$(printf '%s' "$result" | sed -n 's/.*"duration_ms":\([0-9]*\).*/\1/p')
      if grep '"type":"assistant"' "$out/transcript.jsonl" | grep '"tool_use"' | grep 'evals/' >/dev/null; then
        peeked=yes
      fi
      if [ "$agent_status" -eq 124 ]; then
        echo "  the agent timed out after ${limit}s" >&2
        outcome=infra
      elif [ -z "$result" ]; then
        echo "  the transcript has no result (agent exit $agent_status) — $out/agent.stderr" >&2
        outcome=infra
      elif printf '%s' "$result" | grep '"is_error":true' >/dev/null; then
        echo "  the agent reported an error — $out/transcript.jsonl" >&2
        outcome=infra
      fi
    fi

    if [ -z "$outcome" ]; then
      if check_passes "$out/check.log"; then outcome=pass; else outcome=fail; fi
      if [ "$recheck" -eq 1 ]; then
        if check_passes "$out/check-2.log"; then again=pass; else again=fail; fi
        if [ "$again" != "$outcome" ]; then outcome=unstable; fi
      fi
    fi

    echo "  $(printf '%s' "$outcome" | tr '[:lower:]' '[:upper:]')$( [ "$peeked" = yes ] && echo ' (peeked at evals/)')"
    if [ "$outcome" = fail ] || [ "$outcome" = unstable ]; then
      tail -20 "$out/check.log" | sed 's/^/    /'
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$run" "$outcome" "$peeked" "$cost" "$duration" \
      >> "$results/results.tsv"

    if [ "$keep" -eq 1 ]; then
      echo "  worktree kept at $worktree"
      echo "  branch kept: $branch"
    else
      git worktree remove --force "$worktree" >/dev/null 2>&1 || rm -rf "$worktree"
      git worktree prune
      git branch -D "$branch" >/dev/null 2>&1
    fi
    run=$((run + 1))
  done
done
if [ "$keep" -eq 0 ]; then rmdir "$worktrees" 2>/dev/null; fi

# --- report -------------------------------------------------------------------
#
# The rate counts pass and fail only. A 95% Wilson interval rather than a bare
# percentage, because seven evals run three times is twenty-one samples, and a
# gain inside the interval is not a gain.

awk -F '\t' -v base="$base" -v runs="$runs" -v results="$results" '
  NR == 1 { next }
  {
    if (!($1 in seen)) { seen[$1] = 1; order[++ids] = $1 }
    count[$1 SUBSEP $3]++
    total[$3]++
    if ($4 == "yes") peeked++
    if ($5 != "") { cost += $5; costed = 1 }
  }
  END {
    for (i = 1; i <= ids; i++) {
      id = order[i]
      p = count[id SUBSEP "pass"] + 0
      j = p + count[id SUBSEP "fail"]
      line = sprintf("eval %-5s %d/%d passed", id, p, j)
      extra = ""
      if (count[id SUBSEP "infra"]) extra = extra sprintf(", %d infra", count[id SUBSEP "infra"])
      if (count[id SUBSEP "unstable"]) extra = extra sprintf(", %d unstable", count[id SUBSEP "unstable"])
      if (extra != "") line = line "  (" substr(extra, 3) ")"
      print line
      if (j > 0 && p == 0) always[++failing] = id
    }
    passed = total["pass"] + 0
    judged = passed + total["fail"]
    tail = sprintf(" against %s · %d run%s per eval", base, runs, runs == 1 ? "" : "s")
    if (costed) tail = tail sprintf(" · cost $%.2f", cost)
    if (judged == 0) {
      print "evals: nothing was judged" tail
    } else {
      z = 1.96
      rate = passed / judged
      denominator = 1 + z * z / judged
      centre = (rate + z * z / (2 * judged)) / denominator
      half = z * sqrt(rate * (1 - rate) / judged + z * z / (4 * judged * judged)) / denominator
      low = centre - half; if (low < 0) low = 0
      high = centre + half; if (high > 1) high = 1
      printf "evals: %d/%d passed (%.1f%%, 95%% CI %.1f-%.1f%%)%s\n", passed, judged, 100 * rate, 100 * low, 100 * high, tail
    }
    printf "infra %d, unstable %d, peeked %d\n", total["infra"], total["unstable"], peeked
    for (i = 1; i <= failing; i++)
      printf "warning: eval %s failed every judged run — suspect the eval before the steering\n", always[i]
    if (judged > 0 && runs >= 3 && passed / judged >= 0.95)
      print "warning: 95% or more passed — no headroom left to show a gain; the suite needs harder cases from docs/sdlc/lessons.md"
  }
' "$results/results.tsv" > "$results/summary.txt"

{
  echo "# Eval results — $base"
  echo
  echo '```text'
  cat "$results/summary.txt"
  echo '```'
  echo
  echo "| eval | run | outcome | peeked | cost (USD) | transcript | check |"
  echo "|---|---|---|---|---|---|---|"
  awk -F '\t' 'NR > 1 {
    check = "-"
    if ($3 == "pass" || $3 == "fail" || $3 == "unstable") check = "[check](" $1 "/" $2 "/check.log)"
    printf "| %s | %s | %s | %s | %s | [transcript](%s/%s/transcript.jsonl) | %s |\n", $1, $2, $3, $4, ($5 == "" ? "-" : $5), $1, $2, check
  }' "$results/results.tsv"
} > "$results/summary.md"

cat "$results/summary.txt"
echo "results: $results/summary.md"

if awk -F '\t' 'NR > 1 && $3 != "pass" { bad = 1 } END { exit !bad }' "$results/results.tsv"; then
  exit 1
fi
exit 0
