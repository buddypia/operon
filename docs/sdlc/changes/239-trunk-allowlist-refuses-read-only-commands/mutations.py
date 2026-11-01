#!/usr/bin/env python3
"""The mutation sweep for change 239. Each mutation opens one exclusion the
plan names — a spelling that writes or starts a program — and the guard test
must refuse to pass. The runner is scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/239-trunk-allowlist-refuses-read-only-commands")

RUN_SECONDS = 1800

TESTS = ["the_trunk_allowlist_and_the_ownership_check_are_switched_on_here"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

POLICY = ROOT / ".claude/config/worktree-policy.json"
GUARD = ROOT / ".cli/hooks/worktree-policy-guard.mjs"

MUTATIONS = {
    "rg --pre admitted": (POLICY, r"\\s(?:--(?:pre|hostname-bin|search-zip)|", r"\\s(?:--(?:hostname-bin|search-zip)|"),
    "rg -z admitted": (POLICY, r"|search-zip)|-[a-zA-Z]*z))", r"|search-zip)))"),
    "git pull without --ff-only admitted": (
        POLICY,
        r"git\\s+pull(?:\\s+(?:-q|--quiet))*\\s+--ff-only(",
        r"git\\s+pull(?:\\s+(?:-q|--quiet))*(?:\\s+--ff-only)?(",
    ),
    "git pull with a refspec admitted": (
        POLICY,
        r"(?:\\s+origin(?:\\s+main)?)?\\s*$",
        r"(?:\\s+origin(?:\\s+main)?)?",
    ),
    "git ls-remote of any remote admitted": (
        POLICY,
        r"--branches))*\\s+origin(",
        r"--branches))*(?:\\s+\\S+)?(",
    ),
    "gh api with any flag admitted": (POLICY, r"|\\s+(?!-)\\S+)*\\s*$", r"|\\s+\\S+)*\\s*$"),
    "gh api with a value flag beyond --jq admitted": (POLICY, r"(?:--jq|-q)\\s+\\S+", r"-[a-zA-Z-]+\\s+\\S+"),
    "mktemp with a template admitted": (POLICY, r"mktemp(?:\\s+-d)?(?:\\s+-q)?\\s*$", r"mktemp(?:\\s+-d)?(?:\\s+-q)?"),
    "git branch option after the names admitted": (
        POLICY,
        r"no-merged)(?:[ \\t]+(?!-)[\\w./*\\[\\]-]+)+",
        r"no-merged)(?:[ \\t]+[\\w./*\\[\\]-]+)+",
    ),
    "git branch creating a branch admitted": (
        POLICY,
        r"-show-current|-merged|-no-merged)|--format=\\S+))*\\s*$",
        r"-show-current|-merged|-no-merged)|--format=\\S+|\\S+))*\\s*$",
    ),
    "redact filter widened to any secret-guard argument": (
        POLICY,
        r"secret-guard\\.py\\s+redact\\s*$",
        r"secret-guard\\.py",
    ),
    "git options in front of pull and ls-remote unseen": (
        GUARD,
        r"const transport = /^git\s+(?:fetch|pull|ls-remote)(?:\s|$)/;",
        r"const transport = /^git\s+(?:fetch)(?:\s|$)/;",
    ),
    "steering assignments in front of a command unseen": (
        GUARD,
        "return seg.assigns.some((n) => STEERING_ENV_RE.test(n));",
        "return false;",
    ),
    "TMPDIR left out of the steering assignments": (
        GUARD,
        "/^(?:PATH|TMPDIR|PYTHON",
        "/^(?:PATH|PYTHON",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
