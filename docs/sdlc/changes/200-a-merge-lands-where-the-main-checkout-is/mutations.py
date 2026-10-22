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
ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/200-a-merge-lands-where-the-main-checkout-is")

# 2. HOW LONG one check may take before the sweep stops waiting for it. Two
#    orders of magnitude above a healthy run, because this bound is here to end
#    a hang and not to measure anything.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN. One command per list; the first non-zero
#    exit ends the check. Name the guards rather than running the whole suite —
#    a sweep runs this once per mutation.
TESTS = ["--exact", "tests::the_merge_gate_waits_for_ci_and_falls_back_to_the_local_suite"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
GATE = ROOT / ".claude/hooks/gate-merge.sh"
MUTATIONS = {
    "a merge in the main checkout on another branch passes": (
        GATE,
        'if [ "$on_main" = 0 ] && [ -n "$git_dir" ] &&',
        'if false &&',
    ),
    "a linked worktree counts as the main checkout": (
        GATE,
        '  [ "$git_dir" = "$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" ]; then',
        "  true; then",
    ),
    "the refusal does not name the branch the checkout holds": (
        GATE,
        'block "the main checkout is on ${current:-a detached HEAD}, not main"',
        'block "the main checkout is not on main"',
    ),
}

# 5. Nothing below this line is filled in.

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
