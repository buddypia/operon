#!/usr/bin/env python3
"""The mutation sweep for change 314: a launch apart from the person's home and
tmux, a stop that ends only what was started, steps whose verbs and words are
real, a judgement made against the window's tree, a seed the application wrote,
a whole run that keeps its evidence and leaves nothing behind, and a pipeline
that sends a painting change to the built application. Each mutation takes one
of those away and a test must refuse to pass. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = [
    "verify_app_",
    "every_step_seen_in_the_running_app",
    "the_pipeline_sends_a_change_that_paints",
    "feature_catalogue_is_complete",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

SCRIPT = ROOT / "scripts/verify-app.py"
PALETTE = ROOT / "docs/features/palette/spec.md"
PILOT = ROOT / ".claude/skills/feature-pilot/SKILL.md"
REFERENCE = ROOT / ".claude/skills/feature-pilot/references/verify-in-the-app.md"
SCREEN = ROOT / ".claude/skills/sdlc/references/screen-approval.md"

MUTATIONS = {
    # Requirement 1.
    "the planned home is under the person's own": (
        SCRIPT,
        '"HOME": str(Path(root) / "h")',
        '"HOME": str(Path.home() / ".operon-verify")',
    ),
    "the launch keeps the person's TMUX": (
        SCRIPT,
        '"unset": ["TMUX", "TMUX_PANE"]',
        '"unset": ["TMUX_PANE"]',
    ),
    "the scratch folder is too long for a unix socket": (
        SCRIPT,
        'SCRATCH_PREFIX = "opv."',
        'SCRATCH_PREFIX = "operon-verification-scratch-folder-for-one-run-of-a-feature."',
    ),
    # Requirement 2.
    "stop ends whatever holds the recorded pid": (
        SCRIPT,
        'if command and record["binary"] not in command:',
        "if False:",
    ),
    # Requirement 3.
    "a step uses a verb the runner does not have": (
        PALETTE,
        "    see 空のターミナルを開く\n",
        "    look 空のターミナルを開く\n",
    ),
    "a step names words the application no longer draws": (
        PALETTE,
        "    click 設定を開く\n",
        "    click 設定をひらく\n",
    ),
    "the runner drops a verb the documents use": (
        SCRIPT,
        'VERBS = ("start", "click", "see", "gone", "shot")',
        'VERBS = ("start", "click", "see", "shot")',
    ),
    "the script seeds through a test that does not exist": (
        SCRIPT,
        'SEED_TEST = "verify_app_seed_writes_a_store_the_app_loads"',
        'SEED_TEST = "verify_app_seed_writes_a_store"',
    ),
    "the steps sit after the code section": (
        PALETTE,
        "## Seen in the running app",
        "## Seen in the application",
    ),
    # Requirement 4.
    "run --list names every feature": (
        SCRIPT,
        "if spec.is_file() and SECTION in spec.read_text():",
        "if spec.is_file():",
    ),
    "an unknown feature is run anyway": (
        SCRIPT,
        "        if arguments.feature not in feature_ids():",
        "        if False:",
    ),
    # Requirement 5: the claim a verdict makes.
    "see holds whatever the window says": (
        SCRIPT,
        '        return (True, "") if says(tree, words) else (False, f"{words!r} is not on the window")',
        '        return (True, "")',
    ),
    "gone holds while the words are there": (
        SCRIPT,
        '        return (False, f"{words!r} is still on the window") if says(tree, words) else (True, "")',
        '        return (True, "")',
    ),
    "click presses the first of several controls": (
        SCRIPT,
        "        if len(matches) > 1:",
        "        if len(matches) > 1:\n            return matches[0][\"ref_id\"], None\n        if False:",
    ),
    "a longer name wins over the exact one": (
        SCRIPT,
        "    for matches in (exact, partial):",
        "    for matches in (partial, exact):",
    ),
    # Requirement 8.
    "the binary is started in the person's home": (
        SCRIPT,
        '    environment.update(plan["environment"])',
        "    pass",
    ),
    "the binary is handed the person's TMUX": (
        SCRIPT,
        'environment = {k: v for k, v in os.environ.items() if k not in plan["unset"]}',
        "environment = dict(os.environ)",
    ),
    "the run goes on after a step that did not hold": (
        SCRIPT,
        "        if not holds:\n            picture(record",
        "        if False:\n            picture(record",
    ),
    "stop leaves the scratch folder": (
        SCRIPT,
        "        shutil.rmtree(root, ignore_errors=True)\n    try:\n        record_path(state).unlink()",
        "        pass\n    try:\n        record_path(state).unlink()",
    ),
    "stop removes the evidence with the scratch folder": (
        SCRIPT,
        "        stop(state)\n        print(evidence)",
        "        stop(state)\n        shutil.rmtree(evidence, ignore_errors=True)\n        print(evidence)",
    ),
    "a shot keeps no picture": (
        SCRIPT,
        '                holds = picture(record, evidence / f"{number:02d}-{words}.png")',
        "                holds = True",
    ),
    # Findings 1 to 3: the verifier's two and the one a live run showed.
    "a run ends without stopping what it started": (
        SCRIPT,
        "    finally:\n        finish()",
        "    finally:\n        pass",
    ),
    "a press the driver refused still passes": (
        SCRIPT,
        "                return False, f\"the press was refused: {error.get('code')} {error.get('message')}\"",
        '                return True, ""',
    ),
    "a press that changed nothing passes": (
        SCRIPT,
        '    return False, "the press changed nothing on the window (is another window kept in front of it?)"',
        '    return True, ""',
    ),
    "a press the application never saw is not sent again": (
        SCRIPT,
        "    while attempt < 2:",
        "    while attempt < 1:",
    ),
    "a press the window answered is failed on the tool's word": (
        SCRIPT,
        "            if fresh is not None and shown(fresh) != before:\n                return True, \"\"\n            if not safe",
        "            if not safe",
    ),
    "a press the tool did not deliver is given up at once": (
        SCRIPT,
        "            if not safe or undelivered > 3:",
        "            if True:",
    ),
    # Requirement 7.
    "the entry point no longer sends a painting change to the application": (
        PILOT,
        "   `.claude/skills/feature-pilot/references/verify-in-the-app.md`.\n",
        "   see the references.\n",
    ),
    "the reference loses how to clean up": (
        REFERENCE,
        "## Cleanup",
        "## Afterwards",
    ),
    "the screen gate stops at the drawing": (
        SCREEN,
        "## After it is built",
        "## Later",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
