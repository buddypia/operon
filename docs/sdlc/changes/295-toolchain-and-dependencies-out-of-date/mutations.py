#!/usr/bin/env python3
"""The mutation sweep for change 295: Rust 1.100.0 everywhere, egui 0.37, and
the newest direct dependencies. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "every_toolchain_pin_names_one_version",
    "output_limited_command_behaves_as_before",
    "knowledge_map_joins_only_near_meanings",
    "knowledge_map_groups_related_entries_and_is_deterministic",
    "launch_294_recent_projects_come_first",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MUTATIONS = {
    # R1, the claim "every workflow pin": a bump that misses harness.yml.
    "harness-workflow-left-on-the-old-compiler": (
        ROOT / ".github/workflows/harness.yml",
        "          toolchain: 1.100.0",
        "          toolchain: 1.88.0",
    ),
    # rust-toolchain.toml is not mutated: any other channel makes rustup
    # install that compiler and rebuild the world inside the sweep.
    # R1: the resolver ceiling disagrees with the compiler.
    "manifest-msrv-disagrees": (
        ROOT / "Cargo.toml",
        'rust-version = "1.100.0"',
        'rust-version = "1.88.0"',
    ),
    # Lint fix in src/exec.rs: the while-let read loop must still stop at the
    # byte limit.
    "output-limit-no-longer-truncates": (
        ROOT / "src/exec.rs",
        "        let retained = remaining.min(count);",
        "        let retained = count;",
    ),
    # Lint fix in src/knowledge/map.rs: as_chunks must still pair the lanes.
    "dot-product-drops-the-chunked-lanes": (
        ROOT / "src/knowledge/map.rs",
        "            lanes[lane] += a[lane] * b[lane];",
        "            lanes[lane] += 0.0 * a[lane] * b[lane];",
    ),
    # Lint fix in src/app/launch_projects.rs: sort_by_key must stay newest first.
    "launch-projects-oldest-first": (
        ROOT / "src/app/launch_projects.rs",
        "choices.sort_by_key(|choice| std::cmp::Reverse((choice.0, choice.1)));",
        "choices.sort_by_key(|choice| (choice.0, choice.1));",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
