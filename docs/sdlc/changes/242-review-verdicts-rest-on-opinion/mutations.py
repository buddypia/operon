#!/usr/bin/env python3
"""Change 242's mutation sweep: remove each rule the change adds and require a
guard to go red. It is also this change's `empirical-mutation` obligation, so
CI runs it again — which is why ROOT is found from this file rather than
written as this machine's path.
"""
import sys
from pathlib import Path

# docs/sdlc/changes/<this>/mutations.py -> the checkout.
ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "review_gate_requires_a_verification_record_once_the_method_table_exists",
    "an_obligation_command_outside_its_method_prefix_is_refused",
    "every_spec_requirement_has_an_obligation",
    "every_high_surface_names_the_methods_it_requires",
    "exhaustive_check_reports_the_shortest_counterexample",
    "every_model_test_goes_through_exhaustive_check",
    "ci_re_runs_every_obligation_the_branch_records",
    "verification_record_has_no_field_for_an_unreproduced_finding",
    "review_gate_judges_history_before_the_method_table_by_verdicts",
    "verifier_agents_return_evidence_not_verdicts",
    "a_record_satisfies_the_method_an_added_child_requires",
    "an_unknown_route_does_not_land_on_no_evidence",
    "the_evidence_rule_cannot_be_switched_off",
    "the_model_check_count_excludes_its_own_search",
    "a_new_gate_exit_is_pinned_to_its_block",
    "the_review_gate_has_no_bypass",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

GATE = ROOT / "scripts/check-review.sh"
RUNNER = ROOT / "scripts/run-obligations.sh"
SUITE = ROOT / "src/tests.rs"
TABLE = ROOT / "docs/sdlc/verification-methods.yaml"

MUTATIONS = {
    # R1: the evidence rule is switched on by the table in the first parent.
    "gate-never-switches-to-evidence": (
        GATE,
        """if printf '%s\\n' "$methods_before" | grep '^methods:' >/dev/null; then""",
        "if false; then",
    ),
    "gate-ignores-a-stale-record": (
        GATE,
        "      if digest_is_stale; then\n        note \"❌ BLOCK — ${change} · 検証記録が古い",
        "      if false; then\n        note \"❌ BLOCK — ${change} · 検証記録が古い",
    ),
    # Rule 4, the claim: the pass line must report the kinds it counted.
    "pass-line-swaps-empirical-and-formal": (
        GATE,
        "（経験的 ${verified_empirical} · 形式的 ${verified_formal}）",
        "（経験的 ${verified_formal} · 形式的 ${verified_empirical}）",
    ),
    # R2: only a method's prefix and one plain name.
    "gate-accepts-any-command": (
        GATE,
        '      "$prefix"*) rest=${cmd#"$prefix"} ;;',
        "      *) rest=$cmd ;;",
    ),
    "gate-accepts-flags-and-separators": (
        GATE,
        "LC_ALL=C grep -E '^[A-Za-z0-9_][A-Za-z0-9_.-]*$' >/dev/null; then",
        "LC_ALL=C grep -E '.*' >/dev/null; then",
    ),
    "gate-accepts-a-filter-that-matches-no-test": (
        GATE,
        """        if ! printf '%s\\n' "$tests_text" | LC_ALL=C grep -E""",
        """        if false && printf '%s\\n' "$tests_text" | LC_ALL=C grep -E""",
    ),
    # R3, R4: every requirement, every required method.
    "gate-skips-spec-requirements": (GATE, "  for n in $numbers; do", "  for n in ; do"),
    "gate-skips-surface-methods": (GATE, "  for pair in $required_pairs; do", "  for pair in ; do"),
    "store-asks-for-no-formal-model": (
        TABLE,
        'store: "formal-model,empirical-mutation"',
        'store: "empirical-mutation"',
    ),
    # R8: history keeps its rule — the switch reads the parent, not the commit.
    "switch-reads-what-lands": (
        GATE,
        "methods_before=$(landed_before docs/sdlc/verification-methods.yaml)",
        "methods_before=$(landing docs/sdlc/verification-methods.yaml)",
    ),
    # R6: a failing obligation makes CI red, and CI runs the runner.
    "runner-never-fails": (RUNNER, "    failed=1\n", "    failed=0\n"),
    "ci-does-not-run-the-evidence": (
        ROOT / ".github/workflows/ci.yml",
        "run: bash scripts/run-obligations.sh origin/main",
        "run: true",
    ),
    # R5: the checker finds a violation, reports coverage, and every model uses it.
    "checker-never-sees-a-violation": (
        SUITE,
        "        if let Err(why) = invariant(&state) {",
        "        if let Err(why) = invariant(&state).or::<String>(Ok(())) {",
    ),
    "checker-under-reports-coverage": (
        SUITE,
        "        states: parent.len(),",
        "        states: parent.len().saturating_sub(1),",
    ),
    "a-model-that-only-samples": (
        SUITE,
        "#[test]\n/// Change 242, requirement 5. `formal-model` is formal only",
        "#[test]\npub(crate) fn model_check_sampled() {\n    assert!(is_safe_agent_option(\"a\"));\n}\n\n"
        "#[test]\n/// Change 242, requirement 5. `formal-model` is formal only",
    ),
    # R7: no place for an unreproduced finding, and the policy says so.
    "template-gains-carried": (
        ROOT / "docs/sdlc/templates/verification.yaml",
        "\nobligations:\n",
        "\ncarried:\nobligations:\n",
    ),
    "policy-drops-the-reproduction-rule": (
        ROOT / "REVIEW.md",
        "## A finding is a reproduction or nothing",
        "## A finding is a concern a reviewer raises",
    ),
    # F1-F5: the findings the bootstrap review reproduced.
    "added-child-token-splits-on-its-colon": (
        GATE,
        'out="$out 子プロセスやunsafeを足すdiff:$m"',
        'out="$out Command::new/unsafe:$m"',
    ),
    "unknown-route-asks-for-nothing": (
        GATE,
        """  if [ -z "$(field "$(section "$methods_union" routes)" "$route")" ]; then""",
        "  if false; then",
    ),
    "table-can-be-removed": (
        GATE,
        """  if ! printf '%s\\n' "$methods_now" | grep '^methods:' >/dev/null; then""",
        "  if false; then",
    ),
    "runner-is-not-gate-configuration": (
        ROOT / "docs/sdlc/risk.yaml",
        "docs/sdlc/verification-methods.yaml,scripts/run-obligations.sh,",
        "",
    ),
    "model-guard-reads-calls-in-comments": (
        SUITE,
        '                let code = body.split("//").next().unwrap_or_default();',
        "                let code = *body;",
    ),
    "exit-pin-allows-a-second-exit": (
        SUITE,
        '        && code[start + 1..position]\n            .iter()\n            .all(|line| *line != "  exit 0" && *line != "fi")',
        "        && true",
    ),
    # R9: the verifiers return evidence, not verdicts.
    "verifier-returns-an-approval": (
        ROOT / ".claude/agents/formal-verifier.md",
        "No approval, no rating.",
        "Say **approve** when the model passes.",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
