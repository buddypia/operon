#!/usr/bin/env bash
# Measure the harness an agent works inside, so a loop can see it move.
#
# Every number here is read out of the repository, never estimated. The point is
# the *trend*: run it before and after a change, diff the two, and the diff says
# whether the harness got cheaper to work in or more expensive. The tests in
# `src/tests.rs` are the pass/fail gates; this is the dial beside them.
#
# `docs/sdlc/bands.yaml` puts a control band around these numbers and
# `scripts/check-bands.sh` judges a reading against it, which is what turns a
# dial into a signal. Every key added here that is worth watching wants a band.
#
#   bash scripts/harness-metrics.sh            # fast, no compilation
#   bash scripts/harness-metrics.sh --gates    # also time the full gate run
#
# Output is one JSON object on stdout. Diagnostics go to stderr, so
# `bash scripts/harness-metrics.sh > metrics.json` stays clean.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

with_gates=0
case "${1:-}" in
  --gates) with_gates=1 ;;
  "") ;;
  *) echo "usage: harness-metrics.sh [--gates]" >&2; exit 2 ;;
esac

# --- how large the thing an agent has to hold in its head is -----------------

modules=$(grep -cE '^mod [a-z_]+;$' src/main.rs)
source_lines=$(find src -name '*.rs' ! -name 'tests.rs' -exec cat {} + | wc -l | tr -d ' ')
test_lines=$(wc -l < src/tests.rs | tr -d ' ')
largest_module=$(find src -name '*.rs' ! -name 'tests.rs' -exec wc -l {} + \
  | grep -v ' total$' | sort -rn | head -1 | awk '{print $2}')
largest_module_lines=$(find src -name '*.rs' ! -name 'tests.rs' -exec wc -l {} + \
  | grep -v ' total$' | sort -rn | head -1 | awk '{print $1}')

# --- what the gates actually cover -------------------------------------------

tests_total=$(grep -c '^#\[test\]' src/tests.rs)
tests_ignored=$(grep -c '^#\[ignore' src/tests.rs)
tests_in_ci=$((tests_total - tests_ignored))

# Guards that exist only because nothing else could see the failure. Their count
# rising is the harness learning; it falling without a deletion is a regression.
invariant_tests=$(grep -cE '^pub\(crate\) fn (harness_|claude_md_|every_document_that_names|every_icon_resolves|design_md_documents|every_theme_stays|ansi_numbers_map|the_session_prefix_|every_name_the_app|the_pre_rename_|the_transcript_archive_|the_review_policy_|the_control_bands_|every_eval_|every_hook_|every_rule_|every_route_|the_stop_gate_|every_state_|every_risk_|the_readiness_gate_|the_review_gate_|the_commit_gate_|every_reference_|every_gate_script_|every_judging_script_|every_table_split_|every_change_directory_|the_release_|the_approval_)' src/tests.rs)

# --- what an agent pays before it reads a single line of code -----------------

# CLAUDE.md and AGENTS.md are loaded into every turn, so their size is a tax on
# every task in the repository, not a one-off read.
always_loaded_bytes=$(cat CLAUDE.md AGENTS.md | wc -c | tr -d ' ')
steering_bytes=$(cat CLAUDE.md AGENTS.md DESIGN.md CONTRIBUTING.md REVIEW.md \
  .claude/agents/*.md .claude/skills/*/SKILL.md \
  docs/sdlc/*.md docs/sdlc/templates/*.md | wc -c | tr -d ' ')

# What an agent pays only when it opens a file the rule governs. Deliberately a
# separate number from `steering_bytes`: a path-scoped rule is a different kind
# of cost, and folding the two together would collapse the distinction the
# mechanism exists to create. `every_rule_declares_the_paths_it_applies_to`
# keeps it true — a rule with no `paths:` frontmatter loads on every turn, which
# would make `always_loaded_bytes` under-report by exactly this amount.
conditional_steering_bytes=$( ( cat .claude/rules/*.md 2>/dev/null || true ) \
  | wc -c | tr -d ' ')

# A third tier: a skill's SKILL.md is injected when the skill is invoked, but a
# file under its references/ is read only if the skill reaches for it. Counted
# separately for the same reason as conditional_steering_bytes — reference prose
# that nothing counts is hidden cost, and change 008 is mostly reference prose.
reference_bytes=$( ( cat .claude/skills/*/references/*.md 2>/dev/null || true ) \
  | wc -c | tr -d ' ')

# --- the pipeline's own machinery --------------------------------------------

# The mechanisms an agent is steered by, counted so that adding one is visible.
# A count that rises without `invariant_tests` rising means a mechanism was added
# with nothing checking it still exists.
# `|| true` inside the subshell, not after the pipe: `pipefail` is on, so a glob
# that matches nothing would otherwise abort the whole script.
count_matching() { ( "$@" 2>/dev/null || true ) | wc -l | tr -d ' '; }

skills=$(count_matching ls -d .claude/skills/*/)
subagents=$(count_matching ls .claude/agents/*.md)
rules=$(count_matching ls .claude/rules/*.md)
# Changes carrying machine-readable state, and the surfaces that decide autonomy.
# A change without a state file makes the next session infer its position.
states=$(count_matching ls docs/sdlc/changes/*/state.yaml)
risk_surfaces=$(grep -cE '^  [a-z-]+: "' docs/sdlc/risk.yaml)
# Work-type routes. Falling means a kind of request lost the stages that were
# mandatory for it, which reads from the file listing exactly like nothing.
routes=$(grep -cE '^  [a-z]+: "' docs/sdlc/routes.yaml)
hooks=$(grep -c '"type": "command"' .claude/settings.json)
# Commands the release gate stops on. Falling means a boundary was removed.
gated_commands=$(grep -cE '^  decide (ask|deny) ' .claude/hooks/guard-bash.sh)
evals=$(count_matching ls evals/[0-9][0-9][0-9]-*.md)
sdlc_changes=$(count_matching ls -d docs/sdlc/changes/*/)

# Whether automatic approval is earning its trust (sdlc 082): the ledgers read
# by the same script the Stop gate runs, so the two cannot count differently.
approval_counts=$(bash scripts/check-approvals.sh --metrics 2>/dev/null)
approval_count() { printf '%s\n' "$approval_counts" | awk -v k="$1" '$1 == k { v = $2 } END { print v + 0 }'; }
auto_approvals=$(approval_count auto_approvals)
approval_escapes=$(approval_count approval_escapes)
demoted_categories=$(approval_count demoted_categories)

# --- the surfaces the reviewers are pointed at -------------------------------

unsafe_blocks=$(grep -rhcE '\bunsafe [{(]' --include='*.rs' src \
  | paste -sd+ - | bc)
# Production call sites only: the suite drives its own children on purpose.
subprocess_sites=$(grep -rh 'Command::new' --include='*.rs' --exclude='tests.rs' src \
  | grep -cv '^\s*//' || true)
# A one-shot child that skips the budgeted wrappers waits forever and buffers
# without limit. `exec.rs` owns the wrappers; `history.rs` owns the one
# long-lived app-server child, which has its own kill path.
unwrapped_spawns=$(grep -rn '\.output()\|\.spawn()' --include='*.rs' src \
  | grep -v '^src/exec.rs' | grep -v '^src/tests.rs' | grep -cv '^src/history.rs' || true)

# --- optional: what one loop iteration costs in wall-clock --------------------

gate_seconds=null
if [ "$with_gates" -eq 1 ]; then
  echo "running cargo fmt --check / clippy / test ..." >&2
  started=$(date +%s)
  fmt_ok=true;    cargo fmt --check >/dev/null 2>&1 || fmt_ok=false
  clippy_ok=true; cargo clippy --locked -- -D warnings >/dev/null 2>&1 || clippy_ok=false
  test_ok=true;   cargo test --locked >/dev/null 2>&1 || test_ok=false
  gate_seconds=$(( $(date +%s) - started ))
  gates_pass=$([ "$fmt_ok $clippy_ok $test_ok" = "true true true" ] && echo true || echo false)
else
  gates_pass=null
fi

cat <<JSON
{
  "modules": $modules,
  "source_lines": $source_lines,
  "test_lines": $test_lines,
  "largest_module": "$largest_module",
  "largest_module_lines": $largest_module_lines,
  "tests_total": $tests_total,
  "tests_ignored": $tests_ignored,
  "tests_in_ci": $tests_in_ci,
  "invariant_tests": $invariant_tests,
  "always_loaded_bytes": $always_loaded_bytes,
  "steering_bytes": $steering_bytes,
  "conditional_steering_bytes": $conditional_steering_bytes,
  "reference_bytes": $reference_bytes,
  "unsafe_blocks": $unsafe_blocks,
  "subprocess_sites": $subprocess_sites,
  "unwrapped_spawns": $unwrapped_spawns,
  "skills": $skills,
  "subagents": $subagents,
  "rules": $rules,
  "routes": $routes,
  "states": $states,
  "risk_surfaces": $risk_surfaces,
  "hooks": $hooks,
  "gated_commands": $gated_commands,
  "evals": $evals,
  "sdlc_changes": $sdlc_changes,
  "auto_approvals": $auto_approvals,
  "approval_escapes": $approval_escapes,
  "demoted_categories": $demoted_categories,
  "gate_seconds": $gate_seconds,
  "gates_pass": $gates_pass
}
JSON
