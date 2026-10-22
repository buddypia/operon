#!/usr/bin/env python3
"""Change 202's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/202-finished-changes-read-as-open")

RUN_SECONDS = 1800

TESTS = [
    "--exact",
    "tests::merge_gate_refuses_branch_with_in_flight_state_yaml",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

GATE = ROOT / ".claude/hooks/gate-merge.sh"
MUTATIONS = {
    "the merge gate reads only the records a branch touched": (
        GATE,
        'if [ -n "$open_records" ]; then',
        "if false; then",
    ),
    "the status is read from any line, the closed set included": (
        GATE,
        "      if (path in status) next\n",
        "",
    ),
    "an archived record counts as open": (
        GATE,
        'if (value == "done" || value == "archived") continue',
        'if (value == "done") continue',
    ),
    "a record whose status cannot be read counts as absent": (
        GATE,
        '(path in status) ? status[path] : "no readable status"',
        '(path in status) ? status[path] : "done"',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
