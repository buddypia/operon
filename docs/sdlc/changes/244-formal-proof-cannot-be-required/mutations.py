#!/usr/bin/env python3
"""Change 244's mutation sweep: the Kani harness has to stay invisible to every
build that is not `cargo kani`. Also the change's `empirical-mutation`
obligation, so CI runs it again — ROOT is found from this file.
"""
import sys
from pathlib import Path

# docs/sdlc/changes/<this>/mutations.py -> the checkout.
ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 3600

# Warnings are errors here as they are in the CI verify job, so an undeclared
# cfg is a red build and not a line nobody reads.
CHECKS = [[
    "env", "RUSTFLAGS=-D warnings",
    "cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--",
    "--exact", "tests::model_check_safe_agent_command_admits_no_shell_metacharacter",
]]

SUITE = ROOT / "src/tests.rs"
MANIFEST = ROOT / "Cargo.toml"

MUTATIONS = {
    # The harness compiled into the suite: `kani::` exists only under cargo kani.
    "harness-outside-cfg-kani": (
        SUITE,
        "#[cfg(kani)]\n#[kani::proof]",
        "#[kani::proof]",
    ),
    # cfg(kani) undeclared: rustc warns on the harness's attribute.
    "cfg-kani-undeclared": (
        MANIFEST,
        'check-cfg = ["cfg(kani)"]',
        "check-cfg = []",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
