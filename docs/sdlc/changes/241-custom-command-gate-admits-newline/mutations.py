#!/usr/bin/env python3
"""Change 241's mutation sweep: put each line break back into the custom-command
gate and require a guard to go red. Also the change's `empirical-mutation`
obligation, so CI runs it again — ROOT is found from this file.
"""
import sys
from pathlib import Path

# docs/sdlc/changes/<this>/mutations.py -> the checkout.
ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

# The model runs first and alone, so a caught line names the model: each line
# break put back must be found by the exhaustive walk, not only by the examples.
CARGO = ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"]
CHECKS = [
    [*CARGO, "--exact", "tests::model_check_safe_agent_command_admits_no_shell_metacharacter"],
    [*CARGO, "custom_command_with_a_line_break_inside_is_refused", "allows_custom_agent_flags_but_rejects_shell_operators"],
]

GATE = ROOT / "src/agents.rs"
ARM = "                    ' ' | '\\t' | '-' | '_' | '.' | '/' | '=' | ':' | '@' | '+'"

MUTATIONS = {
    # The original defect.
    "gate-admits-all-ascii-whitespace": (
        GATE,
        "            character.is_ascii_alphanumeric()\n                || matches!(",
        "            character.is_ascii_alphanumeric()\n                || character.is_ascii_whitespace()\n                || matches!(",
    ),
    "gate-admits-a-newline": (GATE, ARM, ARM.replace("'\\t' |", "'\\t' | '\\n' |")),
    "gate-admits-a-carriage-return": (GATE, ARM, ARM.replace("'\\t' |", "'\\t' | '\\r' |")),
    "gate-admits-a-form-feed": (GATE, ARM, ARM.replace("'\\t' |", "'\\t' | '\\u{0c}' |")),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
