#!/usr/bin/env python3
"""The mutation sweep a change copies. Put it in the change directory as
`mutations.py` and fill in the four blocks below. The runner is
scripts/mutation_sweep.py, and its header holds the six rules every sweep
keeps — read it before writing a mutation. Rule 4 is the one a copy forgets:
mutate the claim, not only the counter. `.claude/skills/sdlc/SKILL.md` step
10 asks for a sweep; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

# 1. WHERE. An absolute path: this runs from anywhere, and a worktree session's
#    `GIT_DIR`/`GIT_WORK_TREE` do not point here.
ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/198-final-verification")

# 2. HOW LONG one check may take before the sweep stops waiting for it. Two
#    orders of magnitude above a healthy run, because this bound is here to end
#    a hang and not to measure anything.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN. One command per list; the first non-zero
#    exit ends the check. Name the guards rather than running the whole suite —
#    a sweep runs this once per mutation.
TESTS = ["--exact", "tests::the_commit_gate_leaves_the_suite_to_ci_and_the_merge_gate"]
# Run from a session whose $TMPDIR links into the main checkout, as this one
# does: that is the only place the fixtures' old root failed.
FIXTURES = [
    "--exact",
    "tests::the_commit_gate_runs_the_gates_in_the_tree_the_commit_lands_in",
    "tests::the_installed_usage_reader_posts_a_reading_end_to_end",
]
CARGO_TEST = ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"]
CHECKS = [CARGO_TEST + TESTS, CARGO_TEST + FIXTURES]

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
GATE = ROOT / ".claude/hooks/gate-commit.sh"
MUTATIONS = {
    "the gate runs no record test": (
        GATE,
        '  output=$(bash scripts/run-bounded.sh "$left" cargo test --locked -q -- --exact "${record_tests[@]}" 2>&1)',
        "  output=",
    ),
    "the gate runs the whole suite instead": (
        GATE,
        'cargo test --locked -q -- --exact "${record_tests[@]}" 2>&1)',
        "cargo test --locked -q 2>&1)",
    ),
    "a record-only commit no longer triggers them": (
        GATE,
        "grep -E '^docs/|^\\.claude/|\\.md$'",
        "grep -E '^\\.claude/'",
    ),
    "the record tests get a limit of their own beside clippy's": (
        GATE,
        "left=$(( limit - SECONDS ))",
        "left=$limit",
    ),
    "a listed test was renamed away": (
        GATE,
        "  tests::change_directory_numbers_are_unique\n",
        "  tests::change_directory_numbers_stay_unique\n",
    ),
    "a run that matched no test passes": (
        GATE,
        'if [ "$status" -eq 0 ] && ! grep -F',
        'if false && ! grep -F',
    ),
    "fixtures go back under $TMPDIR": (
        ROOT / "src/tests.rs",
        "        .unwrap_or_else(std::env::temp_dir)\n    })",
        "        .map(|_| std::env::temp_dir())\n        .unwrap_or_else(std::env::temp_dir)\n    })",
    ),
}

# 5. Nothing below this line is filled in.

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
