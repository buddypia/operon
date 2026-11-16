#!/usr/bin/env python3
"""The mutation sweep for change 299: Claude Code's isolated worktrees are made
under .worktrees/agent-<name>, where the guards count worktree work, detached
at the parent's head; only such trees are removed; and the landing sweep takes
the leftovers. Each mutation takes one of those away and a test must refuse to
pass. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1200

TESTS = [
    "isolated_subagent_worktrees_land_where_the_guards_look",
    "isolated_subagent_worktree_work_is_not_judged_as_main",
    "cleanup_sweeps_leftover_subagent_worktrees_and_nothing_else",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

CREATE = ROOT / ".claude/hooks/worktree-create.sh"
REMOVE = ROOT / ".claude/hooks/worktree-remove.sh"
OPS = ROOT / ".claude/scripts/create-pr/ops.mjs"

MUTATIONS = {
    # The claim of the change: the tree lands where the guards count worktree work.
    "the tree goes back under .claude/worktrees": (
        CREATE,
        'worktree="$main_root/.worktrees/agent-$name"',
        'worktree="$main_root/.claude/worktrees/$name"',
    ),
    "a name with a slash or dots is let through": (
        CREATE,
        "  '' | .* | *[!A-Za-z0-9._-]*) fail",
        "  '') fail",
    ),
    "the tree starts at main, not the parent's head": (
        CREATE,
        'worktree add --quiet --detach "$worktree" HEAD',
        'worktree add --quiet --detach "$worktree" main',
    ),
    "the tree gets a branch": (
        CREATE,
        'worktree add --quiet --detach "$worktree" HEAD',
        'worktree add --quiet -b "agent-$name" "$worktree" HEAD',
    ),
    "the path is printed with other output": (
        CREATE,
        'git -C "$from" worktree add --quiet --detach "$worktree" HEAD >&2 ||',
        'git -C "$from" worktree add --detach "$worktree" HEAD ||',
    ),
    "removal takes any worktree": (
        REMOVE,
        "  '' | */*) fail \"refusing to remove",
        "  '') fail \"refusing to remove",
    ),
    "removal does nothing": (
        REMOVE,
        'git -C "$main_root" worktree remove --force "$real" >&2 ||',
        'true ||',
    ),
    "the sweep skips Claude Code's agent trees": (
        OPS,
        "      if (!real.startsWith(agentRoot) && !claudeAgent) continue;",
        "      if (!real.startsWith(agentRoot)) continue;",
    ),
    "the sweep takes every .worktrees tree": (
        OPS,
        "claudeAgentPrefix = join(canonicalPath(join(PROJECT_DIR, '.worktrees')), 'agent-');",
        "claudeAgentPrefix = canonicalPath(join(PROJECT_DIR, '.worktrees')) + sep;",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
