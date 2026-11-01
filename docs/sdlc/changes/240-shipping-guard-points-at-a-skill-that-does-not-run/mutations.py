#!/usr/bin/env python3
"""The mutation sweep for change 240. Each mutation undoes one part of the
landing notice — the policy key, a suppression of the ship-worktree text, a
substitution — and the Stop-notice test must refuse to pass. The runner is
scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/240-shipping-guard-points-at-a-skill-that-does-not-run")

RUN_SECONDS = 1800

TESTS = ["the_stop_notice_names_this_repositorys_landing"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

POLICY = ROOT / ".claude/config/worktree-policy.json"
GUARD = ROOT / ".cli/hooks/worktree-shipping-guard.mjs"

MUTATIONS = {
    "the policy names no landing": (POLICY, '"landing_steps": [', '"landing_steps_unread": ['),
    "the PLAN.md tag printed with a landing": (
        GUARD,
        "c.plan_missing && !landingSteps ? ' [PLAN.md missing",
        "c.plan_missing ? ' [PLAN.md missing",
    ),
    "the landing steps not printed": (
        GUARD,
        "lines.push(...buildLandingSection(projectDir, unlanded, landingSteps));",
        "",
    ),
    "the ship-worktree eligibility line printed with a landing": (
        GUARD,
        "  if (landingSteps) {\n    // The steps above are the whole landing",
        "  if (false) {\n    // The steps above are the whole landing",
    ),
    "the worktree not substituted": (GUARD, ".replaceAll('{worktree}', rel)", ""),
    "the branch not substituted": (GUARD, ".replaceAll('{branch}', c.branch)", ""),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
