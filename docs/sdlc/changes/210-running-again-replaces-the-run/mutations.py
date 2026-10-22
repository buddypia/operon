#!/usr/bin/env python3
"""Change 210's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/210-running-again-replaces-the-run")

RUN_SECONDS = 1800

TESTS = [
    "tests::a_running_run_script_is_not_started_twice",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "a second run starts beside the first": (
        APP,
        "        if self.running_run_session(worktree).is_some() {\n",
        "        if false {\n",
    ),
    "a starting run is not running": (
        APP,
        "                        SessionStatus::Starting | SessionStatus::Active\n                    )",
        "                        SessionStatus::Active\n                    )",
    ),
    "a run in another worktree counts": (
        APP,
        "                    && session.worktree_path.as_deref() == Some(worktree)\n",
        "",
    ),
    "an agent session counts as the run": (
        APP,
        "                session.agent == RUN_SCRIPT_AGENT\n                    && ",
        "                ",
    ),
    "the menu offers no stop": (
        SCREENS,
        "                                    self.stop_session(running);\n",
        "                                    let _ = running;\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
