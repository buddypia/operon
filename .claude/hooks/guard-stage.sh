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

payload=$(cat)
[ "${OPERON_SKIP_STAGE_GATE:-0}" = "1" ] && exit 0

path=$(printf '%s' "$payload" | jq -r '.tool_input.file_path // empty')
[ -z "$path" ] && exit 0

cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
relative=${path#"$PWD/"}

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
# At most one. Two open changes is its own problem and not this hook's to solve,
# so the first is taken and named in the message.

in_flight=""
for state_file in docs/sdlc/changes/*/state.yaml; do
  [ -f "$state_file" ] || continue
  case "$(field "$state_file" status)" in
    planning | ready | in-progress | reviewing) in_flight="$state_file"; break ;;
  esac
done
[ -z "$in_flight" ] && exit 0

change=$(dirname "$in_flight")
readiness=$(field "$in_flight" readiness)
screen=$(field "$in_flight" screen)
risk=$(field "$in_flight" risk)

# --- gate 1: the readiness verdict ----------------------------------------------

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
  set -- $tuple
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
