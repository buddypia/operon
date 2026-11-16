#!/usr/bin/env python3
"""The mutation sweep for change 298: scripts/wait-ci.sh waits for the push's
CI run to appear, watches that run, and reports how it ended. Each mutation
takes one of those away and a test must refuse to pass.
Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = [
    "the_ci_wait_waits_for_the_run_to_appear_then_watches_it",
    "the_ci_wait_gives_up_when_no_run_appears",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

WAIT = ROOT / "scripts/wait-ci.sh"

MUTATIONS = {
    # The claim of the change: it does not watch before the run exists.
    "the wait asks once and gives up on no run": (
        WAIT,
        "  [ -n \"$run\" ] && break\n",
        "  [ -n \"$run\" ] && break\n  echo \"no run; push the branch\"; exit 1\n",
    ),
    "the wait never gives up": (
        WAIT,
        "  if [ \"$SECONDS\" -ge \"$deadline\" ]; then\n",
        "  if false; then\n",
    ),
    "a failed run is reported as passed": (
        WAIT,
        "gh run watch \"$run\" --exit-status >/dev/null 2>&1\nstatus=$?\n",
        "gh run watch \"$run\" --exit-status >/dev/null 2>&1\nstatus=0\n",
    ),
    "the wait asks about every commit, not this one": (
        WAIT,
        "--commit \"$sha\" --limit 1",
        "--limit 1",
    ),
    "the wait watches without the run's id": (
        WAIT,
        "gh run watch \"$run\" --exit-status",
        "gh run watch --exit-status",
    ),
    "the give-up message does not say to push": (
        WAIT,
        "in ${appear}s: push the branch holding it (git push -u origin <branch>)",
        "in ${appear}s",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
