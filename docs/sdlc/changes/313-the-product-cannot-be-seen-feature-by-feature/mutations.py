#!/usr/bin/env python3
"""The mutation sweep for change 313: the feature catalogue and its documents
resolve and are true to the code, every change names its features, the page
carries the features with their progress, and the pipeline reads and updates
the documents. Each mutation takes one of those away and a test must refuse to
pass. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = [
    "feature_catalogue",
    "feature_documents",
    "every_change_from_313",
    "a_change_that_alters",
    "the_pipeline_reads",
    "change_map_",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MAP = ROOT / "scripts/change-map.py"
INDEX = ROOT / "docs/features/index.yaml"
HOME_BRIEF = ROOT / "docs/features/home/brief.md"
HOME_SPEC = ROOT / "docs/features/home/spec.md"
STATE = Path(__file__).resolve().parent / "state.yaml"
SKILL = ROOT / ".claude/skills/feature-pilot/SKILL.md"
README = ROOT / "docs/sdlc/README.md"

MUTATIONS = {
    # Requirement 1.
    "a feature's brief loses a required heading": (
        HOME_BRIEF,
        "## 今できること",
        "## できること",
    ),
    "a catalogue entry points at a file that is not there": (
        INDEX,
        "paths: src/tmux/control.rs src/app/broadcast.rs",
        "paths: src/tmux/controls.rs src/app/broadcast.rs",
    ),
    "a feature names a screen outside the set": (
        INDEX,
        "    screen: palette\n",
        "    screen: palettes\n",
    ),
    # Requirement 2.
    "a module of the map belongs to no feature": (
        INDEX,
        "paths: src/tmux/control.rs src/app/broadcast.rs src/app/agent_snapshot.rs",
        "paths: src/tmux/control.rs src/app/agent_snapshot.rs",
    ),
    # Requirement 3.
    "a spec names a test that does not exist": (
        HOME_SPEC,
        "`home_always_says_whether_anything_waits`",
        "`home_always_says_whether_anything_waits_twice`",
    ),
    "a spec names a symbol the file does not hold": (
        HOME_SPEC,
        "`src/app/screens.rs::ui_home`",
        "`src/app/screens.rs::ui_home_removed`",
    ),
    # Requirement 4.
    "a change names a feature the catalogue lacks": (
        STATE,
        '  features: "none"\n',
        '  features: "no-such-feature"\n',
    ),
    # Requirement 5.
    "a change names a feature and does not list its spec": (
        STATE,
        '  features: "none"\n',
        '  features: "home"\n',
    ),
    # Requirement 6.
    "a change waiting for the person no longer wins over one in flight": (
        MAP,
        '            feature["badge"] = "waiting"',
        '            feature["badge"] = "changing"',
    ),
    "a feature's history is oldest first": (
        MAP,
        'feature["changes"].sort(key=lambda item: item["name"], reverse=True)',
        'feature["changes"].sort(key=lambda item: item["name"])',
    ),
    "a file claimed by many features ties a change to all of them": (
        MAP,
        "                    if len(owners) > 2:",
        "                    if len(owners) > 99:",
    ),
    "a change that is no product feature is counted as unmapped": (
        MAP,
        '        if not links and named != ["none"]:',
        "        if not links:",
    ),
    "the summary lists the steady features instead of the moving ones": (
        MAP,
        'moving = [f for f in model["features"] if f["badge"] != "steady"]',
        'moving = [f for f in model["features"] if f["badge"] == "steady"]',
    ),
    "--find ignores a feature's title": (
        MAP,
        'or word in feature["title"].lower()',
        "or False",
    ),
    "a short keyword matches the start of a longer word": (
        MAP,
        'r"(?:s|es|ed|ing)?(?![a-z0-9])"',
        'r""',
    ),
    # Requirement 7: the claims a person reads.
    "the empty history says something else": (
        MAP,
        "この機能に触れた変更の記録はありません",
        "この機能は変更されていません",
    ),
    "the estimate is presented as certain": (
        MAP,
        "外れていることがあります。",
        "正確です。",
    ),
    "the product view is not first": (
        MAP,
        '<button data-view="product" aria-pressed="true">製品</button><button data-view="changes" aria-pressed="false">変更一覧</button>',
        '<button data-view="changes" aria-pressed="false">変更一覧</button><button data-view="product" aria-pressed="true">製品</button>',
    ),
    # Requirement 8.
    "an image past its ceiling is embedded": (
        MAP,
        "if data is not None and len(data) > IMAGE_BYTES:",
        "if data is not None and len(data) > IMAGE_BYTES * 100:",
    ),
    "a catalogue past 64 entries is carried whole": (
        MAP,
        "FEATURES = 64",
        "FEATURES = 99",
    ),
    "a missing document is not reported": (
        MAP,
        'problems.append("文書がありません: " + name)',
        "pass",
    ),
    "an unreadable catalogue is not reported": (
        MAP,
        'base["catalogue_error"] = "機能の一覧を読めません: 項目がありません"',
        'base["catalogue_error"] = ""',
    ),
    # Requirement 9.
    "feature-pilot stops reading the catalogue at placement": (
        SKILL,
        "read `docs/features/index.yaml`\n  for the feature the request is about",
        "read the catalogue\n  for the feature the request is about",
    ),
    "feature-pilot's close forgets the feature documents": (
        SKILL,
        "A change that altered a feature's\n  behaviour has updated that feature's `docs/features/<id>/spec.md`",
        "A change that altered a feature's\n  behaviour has updated that feature's spec",
    ),
    "the README drops the reason for the reversal": (
        README,
        "nothing held what the product does now",
        "nothing held the product",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
