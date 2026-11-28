#!/usr/bin/env python3
"""The mutation sweep for change 324: `claim` gives back an item a gone
session held and nothing someone may still work on, moves a main-copy claim
into the working copy opened after it, and the skill takes the next item when
the backlog command is given alone. Each mutation takes one of those away and
a `backlog_` test must refuse to pass.
Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

# Grouped as change 323's sweep is, so each group fits the time CI gives one
# obligation. With no argument every group runs, each as its own process.
TESTS = {
    "giving": ["backlog_claim_gives", "backlog_claim_keeps", "backlog_claim_hands"],
    "moving": ["backlog_claim_moves", "backlog_claim_hands"],
    "reach": ["backlog_skill"],
}

SCRIPT = ROOT / "scripts/backlog.py"
SKILL = ROOT / ".claude/skills/backlog/SKILL.md"

LEFT_BEHIND = '''    if copies is None or held["by"] in copies[1]:
        return False
    named = git("log", "--branches", "--remotes", "-F", "--grep", "(#%d)" % number, "-1", "--format=%H")
    return named is not None and not named.strip()'''

GIVING = {
    # Requirements 1 and 4.
    "an item is never given back": (
        SCRIPT,
        "            if left_behind(number, claims[0], copies):",
        "            if False:",
    ),
    "the old claim is left in place": (
        SCRIPT,
        "    for held in claims_on(number):\n        try:",
        "    for held in []:\n        try:",
    ),
    "the item does not say why it was given back": (
        SCRIPT,
        "作業ツリーもこの項目のコミットもありません (着手: %s)",
        "作業ツリーもありません (着手: %s)",
    ),
    "a git hook's GIT_DIR reaches the script's git": (
        SCRIPT,
        'REPOSITORY_POINTERS = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR")',
        "REPOSITORY_POINTERS = ()",
    ),
    "a claim a day old is still too young": (
        SCRIPT,
        "STALE_HOURS = 24",
        "STALE_HOURS = 26",
    ),
    # Requirement 2.
    "the age is not looked at": (
        SCRIPT,
        "    if then is None or datetime.datetime.now(datetime.timezone.utc).timestamp() - then < STALE_HOURS * 3600:",
        "    if then is None:",
    ),
    "a claim under a day old is old enough": (
        SCRIPT,
        "STALE_HOURS = 24",
        "STALE_HOURS = 22",
    ),
    "a working copy of the holder's name is not looked at": (
        SCRIPT,
        '    if copies is None or held["by"] in copies[1]:',
        "    if copies is None:",
    ),
    "the working copy's name is compared unnormalised": (
        SCRIPT,
        "    linked = {name_of(os.path.basename(path))",
        "    linked = {os.path.basename(path)",
    ),
    "a commit naming the item is not looked at": (
        SCRIPT,
        "    return named is not None and not named.strip()",
        "    return named is not None",
    ),
    "a remote-tracking ref is not looked at": (
        SCRIPT,
        '"log", "--branches", "--remotes",',
        '"log", "--branches",',
    ),
    "git failing reads as nobody working": (
        SCRIPT,
        LEFT_BEHIND,
        LEFT_BEHIND.replace("if copies is None or held", "if copies is not None and held").replace(
            "return named is not None and not", "return named is None or not"
        ),
    ),
    "git is waited on for ever": (
        SCRIPT,
        "    timer = threading.Timer(timeout_seconds, child.kill)",
        "    timer = threading.Timer(3600, child.kill)",
    ),
}

MOVING = {
    # Requirement 3.
    "a claim never moves": (
        SCRIPT,
        '            if named and linked_here and holder == copies[0] and born_after(claims[0]["since"]):',
        "            if False:",
    ),
    "a copy older than the claim takes it over": (
        SCRIPT,
        ' and born_after(claims[0]["since"]):',
        ":",
    ),
    "the birth time is compared the wrong way": (
        SCRIPT,
        "    return born >= then",
        "    return born < then",
    ),
    "another copy's claim is taken over": (
        SCRIPT,
        "linked_here and holder == copies[0] and born_after",
        "linked_here and holder != here and born_after",
    ),
    "the main copy's claim is kept beside the moved one": (
        SCRIPT,
        '    for held in claims:\n        withdraw(held["id"])',
        "    for held in []:\n        pass",
    ),
    "the move is not said": (
        SCRIPT,
        "の着手をこの作業ツリーに移しました",
        "を取りました",
    ),
    "a claim under this copy's name is refused": (
        SCRIPT,
        "            if named and linked_here and holder == here:",
        "            if False:",
    ),
    "the main copy hands a held item to a second session of the same name": (
        SCRIPT,
        "            if named and linked_here and holder == here:",
        "            if named and holder == here:",
    ),
}

REACH = {
    # Requirement 5.
    "the command alone is not the Run use": (
        SKILL,
        "`/backlog` given alone,\nwith nothing after it, is the Run use",
        "`/backlog` asks\nwhat is left",
    ),
    "the list comes before the claim": (
        SKILL,
        "1. Take the best item nobody holds:\n\n   ```sh\n   python3 scripts/backlog.py claim\n",
        "1. Take the best item nobody holds:\n\n   ```sh\n   python3 scripts/backlog.py list\n   python3 scripts/backlog.py claim\n",
    ),
    "the claim is not moved into the working copy": (
        SKILL,
        "   is a `fix/` branch) — and, from inside it, move the claim there:\n\n   ```sh\n   python3 scripts/backlog.py claim 12\n   ```\n",
        "   is a `fix/` branch).\n",
    ),
    "a gone session's claim is released by hand again": (
        SKILL,
        "- `list` shows 「着手中」 with a name and an age. `claim` gives back on its own",
        "- `list` shows 「着手中」 with a name and an age. `release` it with that reason; nothing",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

GROUPS = {"giving": GIVING, "moving": MOVING, "reach": REACH}
if len(sys.argv) == 1:
    import subprocess

    for _group in GROUPS:
        print(f"== {_group}", flush=True)
        if subprocess.run([sys.executable, __file__, _group], check=False).returncode != 0:
            sys.exit(1)
    sys.exit(0)
if len(sys.argv) != 2 or sys.argv[1] not in GROUPS:
    sys.exit("usage: mutations.py [" + "|".join(GROUPS) + "]")
GROUP = sys.argv[1]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS[GROUP]]]
mutation_sweep.main(RUN_SECONDS, CHECKS, GROUPS[GROUP])
