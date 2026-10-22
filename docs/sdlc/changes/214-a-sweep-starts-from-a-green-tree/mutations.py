#!/usr/bin/env python3
"""Change 214's mutation sweep. The runner and its seven rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/214-a-sweep-starts-from-a-green-tree")

RUN_SECONDS = 1800

TESTS = [
    "tests::the_mutation_sweep_template_ends_a_run_that_will_not_finish",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

RUNNER = ROOT / "scripts/mutation_sweep.py"
MUTATIONS = {
    "the sweep starts without a baseline": (
        RUNNER,
        "        green, output = run_checks()\n        if not green:\n",
        "        green, output = True, \"\"\n        if not green:\n",
    ),
    "a red baseline is reported without saying it was red before": (
        RUNNER,
        "SETUP FAILED — 変異の前からチェックが通りません。",
        "SETUP FAILED — チェックが通りません。",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
