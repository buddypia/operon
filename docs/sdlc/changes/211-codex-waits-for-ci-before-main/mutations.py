#!/usr/bin/env python3
"""Change 211's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/211-codex-waits-for-ci-before-main")

RUN_SECONDS = 1800

TESTS = [
    "tests::codex_runs_the_merge_gate_on_every_shell_call",
    "tests::merge_gate_refuses_branch_with_in_flight_state_yaml",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

CODEX = ROOT / ".codex/hooks.json"
GATE = ROOT / ".claude/hooks/gate-merge.sh"
HANDLER = (
    '          },\n'
    '          {\n'
    '            "type": "command",\n'
    '            "command": "bash \\"$(dirname \\"$(git rev-parse --path-format=absolute --git-common-dir || echo /nonexistent/nogit)\\")/.claude/hooks/gate-merge.sh\\"",\n'
    '            "timeout": 600\n'
)
MUTATIONS = {
    "Codex runs no merge gate": (CODEX, HANDLER, ""),
    "the gate times out before the suite's limit": (
        CODEX,
        '/.claude/hooks/gate-merge.sh\\"",\n            "timeout": 600\n',
        '/.claude/hooks/gate-merge.sh\\"",\n            "timeout": 60\n',
    ),
    "a worktree's own gate runs": (
        CODEX,
        '--git-common-dir || echo /nonexistent/nogit)\\")/.claude/hooks/gate-merge.sh',
        '--show-toplevel || echo /nonexistent/nogit)\\")/.claude/hooks/gate-merge.sh',
    ),
    "the gate cannot read Codex's command": (
        GATE,
        "jq -r '.tool_input.command // ",
        "jq -r '",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
