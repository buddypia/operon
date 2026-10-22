#!/usr/bin/env bash
# PreToolUse(Edit|Write): the stage boundary, enforced rather than requested.
#
# Change 008 introduced the orchestrator's data — docs/sdlc/routes.yaml,
# docs/sdlc/risk.yaml, docs/sdlc/templates/state.yaml — and its readiness script,
# and wired none of it to anything that could refuse. `scripts/check-readiness.sh`
# was called by its own test and nothing else; `risk.yaml`'s `paused` tier was
# read by nobody. That is the failure `docs/sdlc/README.md` opens by naming: prose
# in AGENTS.md is a promise, a hook is a gate. Tables without a hook
# leave compliance to good intentions: they carry the vocabulary, not the
# mechanism.
#
# So this refuses an edit to src/ when the change in flight has not passed the
# gate that stage 3 is supposed to sit behind.
#
# It stays quiet unless the pipeline is actually in use: with no in-flight change
# directory there is nothing to check, and a one-line fix is not meant to have
# one. That is deliberate — a gate that fires on every edit in a repository where
# most edits are small is a gate that gets switched off.
#
#   OPERON_SKIP_STAGE_GATE=1   proceed anyway (say why in the report)
set -uo pipefail
export GIT_NO_REPLACE_OBJECTS=1
export GIT_GRAFT_FILE=/dev/null
# The graft deprecation hint git 2.56 prints on every call; scripts/check-review.sh.
export GIT_ADVICE=0

payload=$(cat)
[ "${OPERON_SKIP_STAGE_GATE:-0}" = "1" ] && exit 0

path=$(printf '%s' "$payload" | jq -r '.tool_input.file_path // .tool_input.TargetFile // .toolCall.args.TargetFile // .toolCall.args.file_path // empty')
[ -z "$path" ] && exit 0

unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
running_in=$(printf '%s' "$payload" | jq -r '.tool_input.Cwd // .toolCall.args.Cwd // .cwd // .session.cwd // .workspacePaths[0] // empty')
case "$path" in
  /*) target_dir=$(dirname -- "$path" 2>/dev/null) ;;
  *) target_dir="" ;;
esac
probe_dir=$target_dir
while [ -n "$probe_dir" ] && [ "$probe_dir" != "/" ] && [ ! -d "$probe_dir" ]; do
  probe_dir=$(dirname -- "$probe_dir" 2>/dev/null)
done
if [ -n "$probe_dir" ] && [ -d "$probe_dir" ]; then
  resolved_probe_dir=$(cd -- "$probe_dir" 2>/dev/null && pwd -P)
  if [ -n "$resolved_probe_dir" ]; then
    suffix=${path#"$probe_dir/"}
    path="$resolved_probe_dir/$suffix"
    target_dir=$resolved_probe_dir
  fi
else
  target_dir=${running_in:-${CLAUDE_PROJECT_DIR:-.}}
fi
working_tree=$(cd -- "$target_dir" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)
if [ -z "$working_tree" ] && [ -n "$running_in" ] && [ -d "$running_in" ]; then
  working_tree=$(cd -- "$running_in" 2>/dev/null && pwd -P)
fi
cd -- "${working_tree:-${CLAUDE_PROJECT_DIR:-.}}" || exit 0
pwd_phys=$(pwd -P 2>/dev/null || pwd)
relative=${path#"$pwd_phys/"}
[ "$relative" = "$path" ] && relative=${path#"$PWD/"}

# Only source edits are gated. The artifacts, the harness, and the documents are
# how a change gets itself *to* stage 3; gating those would deadlock the pipeline
# against itself.
case "$relative" in
  src/*.rs | src/*/*.rs) ;;
  *) exit 0 ;;
esac

block() {
  printf 'Edit blocked at the stage boundary: %s\n\n%s\n' "$1" "$2" >&2
  exit 2
}

field() { sed -n "s/^  $2: \"\(.*\)\"$/\1/p" "$1" | head -1; }

# --- which change is in flight? -------------------------------------------------
#
# Scoped to the change directories touched in this worktree/branch against
# refs/heads/main (matching gate-stop.sh), so a record another change left open
# never shadows the change in flight. Thirty did until change 202 closed them;
# gate-merge.sh now refuses a landing while any record on the branch is open.

if git merge-base refs/heads/main HEAD >/dev/null 2>&1; then
  in_play=$(
    {
      git diff -z --no-renames --name-only HEAD -- docs/sdlc/changes 2>/dev/null
      git ls-files -z --others --exclude-standard -- docs/sdlc/changes 2>/dev/null
      git diff -z --no-renames --name-only "$(git merge-base refs/heads/main HEAD 2>/dev/null)" HEAD -- docs/sdlc/changes 2>/dev/null
    } | tr '\0' '\n' | sed -n 's|^docs/sdlc/changes/\([^/]*\)/.*|\1|p' | sort -u
  )
  candidates=""
  for dir_name in $in_play; do
    candidates="$candidates docs/sdlc/changes/$dir_name/state.yaml"
  done
else
  candidates=$(ls -r docs/sdlc/changes/*/state.yaml 2>/dev/null || true)
fi

in_flight=""
for state_file in $candidates; do
  [ -f "$state_file" ] || continue
  case "$(field "$state_file" status)" in
    planning | ready | in-progress | reviewing | awaiting-user | blocked | failed)
      in_flight="$state_file"
      break
      ;;
  esac
done
[ -z "$in_flight" ] && exit 0

change=$(dirname "$in_flight")
in_flight_status=$(field "$in_flight" status)
case "$in_flight_status" in
  awaiting-user | blocked | failed)
    block "$change has status \`$in_flight_status\`" \
"This change is currently parked at \`$in_flight_status\`. Resolve the blocker or user question and set \`status: \"in-progress\"\` in $change/state.yaml before editing source files."
    ;;
esac
route=$(field "$in_flight" route)
readiness=$(field "$in_flight" readiness)
screen=$(field "$in_flight" screen)
risk=$(field "$in_flight" risk)

# --- gate 1: the readiness verdict (for routes with a design stage) -------------

route_stages=""
if [ -f docs/sdlc/routes.yaml ] && [ -n "$route" ]; then
  route_stages=$(sed -n "s/^  $route: \"\([^ ]*\).*\"$/\1/p" docs/sdlc/routes.yaml | head -1)
fi
case ",${route_stages:-design}," in
  *,design,*)
    case "$readiness" in
      go | conditional) ;;
      *)
        block "$change has readiness \`$readiness\`" \
"Stage 3 sits behind the readiness gate, and this change has not passed it.

  bash scripts/check-readiness.sh $change

Then record the verdict in $change/state.yaml as \`readiness\`. A No-Go is fixed
in the spec — answering a document problem in the implementation is what the gate
exists to prevent."
        ;;
    esac
    ;;
esac

# --- gate 2: the screen, when the change paints ---------------------------------

if [ "$screen" = "pending" ]; then
  block "$change has screen \`pending\`" \
"This change touches a surface docs/sdlc/risk.yaml marks \`screen yes\`, so its
screen is agreed before its code. Read
.claude/skills/sdlc/references/screen-approval.md, present an ASCII layout at a
realistic width with the Japanese strings and the empty and error states, commit
it as screen.md, and set \`screen: approved\` once the approver
.claude/skills/sdlc/references/approval.md names — a person for a new structure,
the evaluator for a small change — has approved it in approvals.log.

Lesson 005 was a layout bug that shipped under a green widget test. A widget test
does not prove a layout."
fi

# --- gate 3: the change classified the surface it is editing --------------------
#
# risk.yaml says what a surface is worth; state.yaml says what this change thinks
# it is doing. When those disagree the state is wrong, and every autonomy decision
# downstream was made against the wrong tier.

matched_tier=""
matched_surface=""
while IFS= read -r line; do
  surface=${line%%:*}
  surface=$(printf %s "$surface" | tr -d " ")
  tuple=$(printf '%s' "$line" | sed -n 's/.*"\(.*\)".*/\1/p')
  set -f
  set -- $tuple
  set +f
  tier=$1
  paths=$4
  IFS=',' read -ra candidates <<<"$paths"
  for candidate in "${candidates[@]}"; do
    case "$relative" in
      "$candidate" | "$candidate"/*)
        # high beats medium beats low; take the worst match.
        case "$tier" in
          high) matched_tier=high; matched_surface=$surface ;;
          medium) [ "$matched_tier" = high ] || { matched_tier=medium; matched_surface=$surface; } ;;
          low) [ -n "$matched_tier" ] || { matched_tier=low; matched_surface=$surface; } ;;
        esac
        ;;
    esac
  done
done < <(grep -E '^  [a-z-]+: "' docs/sdlc/risk.yaml)

if [ -n "$matched_tier" ] && [ "$matched_tier" != "$risk" ]; then
  # Only a *lower* recorded tier is a problem. A change that classified itself
  # higher than the file it is touching is being careful, not wrong.
  worse=0
  case "$matched_tier:$risk" in
    high:medium | high:low | medium:low) worse=1 ;;
  esac
  if [ "$worse" -eq 1 ]; then
    block "$relative is on the \`$matched_surface\` surface (\`$matched_tier\`), and $change records \`risk: $risk\`" \
"docs/sdlc/risk.yaml classifies this file higher than the change classifies
itself, so every autonomy decision made so far was made against the wrong tier.

Set \`risk: $matched_tier\` in $change/state.yaml, and take the autonomy that tier
allows — \`paused\` means stop and ask before editing, and record
\`status: awaiting-user\` while you wait."
  fi
fi

exit 0
