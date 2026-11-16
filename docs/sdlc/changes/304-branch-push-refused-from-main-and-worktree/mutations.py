#!/usr/bin/env python3
"""The mutation sweep for change 304: on the trunk the allowlist admits a
plain branch push (the Publishing gate's form) and nothing wider — no force,
no refspec or forced ref, no other remote, nothing after the branch. Each
mutation takes one of those away, or the claim itself, and the trunk test must
refuse to pass. The `> out` deny comes from the guard's own redirect rule, not
from the pattern, so no pattern mutation here targets it.
Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = [
    "the_trunk_allowlist_and_the_ownership_check_are_switched_on_here",
    "a_plain_push_to_origin_is_not_asked_about",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

POLICY = ROOT / ".claude/config/worktree-policy.json"
GATE = ROOT / ".claude/hooks/guard-bash.sh"
PUSH = '"git\\\\s+push(?:\\\\s+(?:-u|--set-upstream|-q|--quiet))*\\\\s+origin\\\\s+[A-Za-z0-9][A-Za-z0-9._/-]*\\\\s*$",'

MUTATIONS = {
    # The claim of the change: a branch is pushed from the main checkout.
    "back to main only": (POLICY, PUSH, '"git\\\\s+push\\\\s+origin\\\\s+main\\\\s*$",'),
    "the -u flag is not admitted": (
        POLICY,
        "(?:-u|--set-upstream|-q|--quiet)",
        "(?:--set-upstream|-q|--quiet)",
    ),
    "--force is admitted as a flag": (
        POLICY,
        "(?:-u|--set-upstream|-q|--quiet)",
        "(?:-u|--set-upstream|-q|--quiet|--force)",
    ),
    "a refspec colon is admitted in the branch": (
        POLICY,
        "[A-Za-z0-9][A-Za-z0-9._/-]*",
        "[A-Za-z0-9:][A-Za-z0-9._/:-]*",
    ),
    "a forced ref is admitted": (
        POLICY,
        "[A-Za-z0-9][A-Za-z0-9._/-]*",
        "[A-Za-z0-9+][A-Za-z0-9._/-]*",
    ),
    "any remote is admitted": (
        POLICY,
        "\\\\s+origin\\\\s+[A-Za-z0-9]",
        "\\\\s+\\\\w+\\\\s+[A-Za-z0-9]",
    ),
    "the end anchor is dropped": (
        POLICY,
        "[A-Za-z0-9._/-]*\\\\s*$\",",
        "[A-Za-z0-9._/-]*\",",
    ),
    # The second layer: the Publishing gate refuses what the trunk allowlist
    # alone lets through. Here it stops reading a push written after git's
    # options or behind an assignment as a push, and stops requiring the
    # whole line to be the push.
    "the Publishing gate admits a plain push anywhere in the line": (
        GATE,
        "  if [ \"$(printf '%s\\n' \"$command\" | grep -c '[^[:space:]]')\" = 1 ] \\\n"
        "    && printf '%s' \"$line\" | grep -E \"$lone_push\" >/dev/null; then\n"
        "    printf '%s' \"$line\" | grep -E \"$plain_push\" >/dev/null && exit 0",
        "  if [ \"$(printf '%s\\n' \"$command\" | grep -c '[^[:space:]]')\" = 1 ]; then\n"
        "    printf '%s' \"$line\" | grep -E \"${plain_push#^}\" >/dev/null && exit 0",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
