#!/usr/bin/env python3
"""Change 305's mutation sweep: each mutation lets a CLI tab, or no tab at all,
reach the plain-shell fallback, and one of the tests must go red. Runner:
scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "cli_305_unresumable_cli_tab_keeps_its_kind_and_conversation",
    "pane_303_unresumable_open_tab_falls_back_to_plain_terminal_with_cooldown",
    "pane_303_lost_or_failed_open_tab_from_previous_run_recovers_on_load",
    "pane_303_f2_refused_relaunch_preserves_resumable_cli_metadata_instead_of_shell_fallback",
    "model_check_pane_303_missing_pane_recovery_and_output_guard",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

RELAUNCH = ROOT / "src" / "app" / "relaunch.rs"
CONDITION = "            && cli_provider_for_agent(&session.agent).is_none()\n"
MUTATIONS = {
    "cli-tab-falls-back-to-shell-again": (RELAUNCH, CONDITION, ""),
    "only-cli-tabs-fall-back": (
        RELAUNCH,
        CONDITION,
        "            && cli_provider_for_agent(&session.agent).is_some()\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
