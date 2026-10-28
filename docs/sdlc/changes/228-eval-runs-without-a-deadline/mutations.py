#!/usr/bin/env python3
"""Portable eval budgets, invalid limits, and the claim in measured conditions."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 600
CHECKS = [["cargo", "test", "--locked", "the_eval_runner_"]]
SCRIPT = ROOT / "scripts/run-evals.sh"
MUTATIONS = {
    "agent-runs-without-group-deadline": (
        SCRIPT, 'bash "$repo_root/scripts/run-bounded.sh" "$limit" "$agent"', '"$agent"'),
    "invalid-limit-starts-paid-agent": (
        SCRIPT, 'if [ "$limit_valid" -ne 1 ] || [ "${#limit}" -gt 10 ] || [ "$limit" -gt 2147483647 ]; then', 'if false; then'),
    "conditions-hide-the-enforced-deadline": (
        SCRIPT, 'conditions="$conditions · timeout ${limit}s"', 'conditions="$conditions · timeout none"'),
    "timeout-counts-an-early-result-as-success": (
        SCRIPT, 'if [ "$agent_status" -eq 124 ]; then', 'if false; then'),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
