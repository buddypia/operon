#!/usr/bin/env python3
"""Change 213's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/213-a-landed-record-is-judged-where-it-landed")

RUN_SECONDS = 1800

TESTS = [
    "tests::the_review_gate_refuses_what_it_says_it_refuses_after_a_landing",
    "tests::every_judging_script_reads_the_real_objects",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

GATE = ROOT / "scripts/check-review.sh"
LANDED = 'landed_on=$(git rev-parse --verify --quiet "${landing}^1" 2>/dev/null) && base=$landed_on'
MUTATIONS = {
    "a landed record is judged against today's main": (
        GATE,
        LANDED,
        'landed_on=$(git rev-parse --verify --quiet "${landing}^1" 2>/dev/null) && true',
    ),
    "the base is the landing itself, not its parent": (
        GATE,
        LANDED,
        'landed_on=$(git rev-parse --verify --quiet "${landing}" 2>/dev/null) && base=$landed_on',
    ),
    "the base is the merge's parent, not the main the branch took in": (
        GATE,
        'landing=$(git rev-list --first-parent refs/heads/main "^$commit" 2>/dev/null | tail -n 1)',
        'landing=$(git rev-list --first-parent --merges refs/heads/main "^$commit" 2>/dev/null | tail -n 1)',
    ),
    "an unlanded record is judged where it forked": (
        GATE,
        '    if [ "$mode" = commit ] && [ "$base" = refs/heads/main ] \\\n'
        '      && git merge-base --is-ancestor "$commit" refs/heads/main 2>/dev/null; then\n',
        '    if [ "$mode" = commit ] && [ "$base" = refs/heads/main ]; then\n',
    ),
    "the stop gate prints the graft hint again": (
        ROOT / ".claude/hooks/gate-stop.sh",
        "export GIT_ADVICE=0\n",
        "",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
