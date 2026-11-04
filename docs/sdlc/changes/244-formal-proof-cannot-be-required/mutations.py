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
    "model_check_safe_agent_command_admits_no_shell_metacharacter",
    "a_launch_gate_change_must_name_a_proof_harness_that_exists",
    "the_verify_job_installs_kani_before_it_runs_a_recorded_proof",
]]

SUITE = ROOT / "src/tests.rs"
MANIFEST = ROOT / "Cargo.toml"
GATE = ROOT / "scripts/check-review.sh"
TABLE = ROOT / "docs/sdlc/verification-methods.yaml"
WORKFLOW = ROOT / ".github/workflows/ci.yml"

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
    # A change to the launch gates asks for no proof.
    "launch-gate-asks-for-nothing": (
        TABLE,
        '  launch-gate: "formal-proof"\n',
        "",
    ),
    # Any harness name is accepted, one that matches nothing included.
    "gate-accepts-any-harness-name": (
        GATE,
        """END { print found + 0 }')" != 1 ]; then""",
        """END { print found + 0 }')" = 9 ]; then""",
    ),
    # A `#[kani::proof]` anywhere above counts for the next function.
    "proof-attribute-carries-over": (
        GATE,
        '&& proof) found = 1; proof = 0 }',
        '&& proof) found = 1 }',
    ),
    # CI runs a recorded proof with no Kani installed.
    "verify-job-never-installs-kani": (
        WORKFLOW,
        "            cargo install --locked kani-verifier --version 0.68.0\n",
        "",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
