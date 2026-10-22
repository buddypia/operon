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
ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/201-what-the-final-verification-left")

# 2. HOW LONG one check may take before the sweep stops waiting for it. Two
#    orders of magnitude above a healthy run, because this bound is here to end
#    a hang and not to measure anything.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN. One command per list; the first non-zero
#    exit ends the check. Name the guards rather than running the whole suite —
#    a sweep runs this once per mutation.
TESTS = [
    "--exact",
    "tests::a_new_change_cannot_take_a_number_already_in_use",
    "tests::test_fast_subprocess_completes_without_25ms_polling_floor",
    "tests::a_session_that_ends_gives_back_only_its_own_worktree_leases",
    "tests::fixtures_take_their_root_from_one_place",
    "tests::the_commit_gate_leaves_the_suite_to_ci_and_the_merge_gate",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
TESTS_RS = ROOT / "src/tests.rs"
MUTATIONS = {
    "the number check reads main's change directories only": (
        ROOT / "scripts/check-change-number.sh",
        "for ref in $(git for-each-ref --format='%(refname:short)' refs/heads refs/remotes/origin 2>/dev/null); do",
        "for ref in main origin/main; do",
    ),
    "the runner's first poll sleeps 25 ms": (
        ROOT / "src/exec.rs",
        "let mut poll_sleep = Duration::from_millis(1);",
        "let mut poll_sleep = Duration::from_millis(25);",
    ),
    "the runner's poll sleeps a flat 25 ms after its first": (
        ROOT / "src/exec.rs",
        "poll_sleep = (poll_sleep * 2).min(Duration::from_millis(25));",
        "poll_sleep = Duration::from_millis(25);",
    ),
    "the session-end release waits 3 s again": (
        ROOT / ".cli/hooks/worktree-owner-tracker.mjs",
        "resolveProjectDir(data), { timeout: 8000 });",
        "resolveProjectDir(data), { timeout: 3000 });",
    ),
    "node fixtures go back to the session's $TMPDIR": (
        TESTS_RS,
        '        .env("TMPDIR", temporary_root());',
        "        ;",
    ),
    "the node hooks' git has no patience floor": (
        TESTS_RS,
        '        .env("HARNESS_EXEC_TIMEOUT_FLOOR_MS", "60000")\n',
        "",
    ),
    "a fixture takes its root from temp_dir again": (
        TESTS_RS,
        'let root = temporary_root().join(format!("operon-history-{}", Uuid::new_v4()));',
        'let root = std::env::temp_dir().join(format!("operon-history-{}", Uuid::new_v4()));',
    ),
    "a commit that moves a path no longer runs the record tests": (
        ROOT / ".claude/hooks/gate-commit.sh",
        'if [ -n "$({ git diff --cached --diff-filter=DR --name-only; git diff --diff-filter=D --name-only; } 2>/dev/null)" ] ||\n',
        "if false ||\n",
    ),
}

# 5. Nothing below this line is filled in.

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
