#!/usr/bin/env python3
"""The mutation sweep for change 296: the landing cleanup reaps the worktrees
Antigravity leaves for its subagents, and only those that lose nothing. Each
mutation drops one condition of the sweep, or the sweep itself, and the test
must refuse to pass. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = ["cleanup_sweeps_leftover_subagent_worktrees_and_nothing_else"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

OPS = ROOT / ".claude/scripts/create-pr/ops.mjs"

MUTATIONS = {
    "the cleanup never sweeps": (
        OPS,
        "  for (const swept of sweepAgentWorktrees()) warnings.push",
        "  for (const swept of []) warnings.push",
    ),
    "a worktree outside agy's root is swept": (
        OPS,
        "if (!existsSync(path) || !canonicalPath(path).startsWith(agentRoot)) continue;",
        "if (!existsSync(path)) continue;",
    ),
    "a recently touched worktree is swept": (
        OPS,
        "if (now - statSync(path).mtimeMs < AGENT_WORKTREE_IDLE_MS) continue;",
        "",
    ),
    # `worktree remove` without --force is the second line of defence; the
    # mutation takes both away, or git alone would still refuse.
    "a worktree with uncommitted files is swept, forced": (
        OPS,
        "      if (gitFn(['status', '--porcelain'], { cwd: path }).length > 0) continue;\n"
        "      gitFn(['merge-base', '--is-ancestor', 'HEAD', baseBranch], { cwd: path });\n"
        "      const branch = gitFn(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: path });\n"
        "      gitFn(['worktree', 'remove', path],",
        "      gitFn(['merge-base', '--is-ancestor', 'HEAD', baseBranch], { cwd: path });\n"
        "      const branch = gitFn(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: path });\n"
        "      gitFn(['worktree', 'remove', '--force', path],",
    ),
    "a worktree with commits outside main is swept": (
        OPS,
        "gitFn(['merge-base', '--is-ancestor', 'HEAD', baseBranch], { cwd: path });",
        "",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
