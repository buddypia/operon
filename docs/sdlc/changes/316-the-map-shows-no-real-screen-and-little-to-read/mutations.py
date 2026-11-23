#!/usr/bin/env python3
"""The mutation sweep of change 316: each mutation breaks one claim the
capture, the pictures, the flow, the map's model, the page or the pipeline
make, and one of the twelve tests of spec.md (or a change-map test this
change re-pointed) must go red. Copied from docs/sdlc/templates/mutations.py;
the runner is scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 2400

TESTS = [
    "capture_plan_reaches_every_screen_with_words_the_app_draws",
    "every_screen_of_the_flow_has_its_picture_or_says_why",
    "capture_writes_each_picture_and_leaves_nothing_running",
    "capture_keeps_the_old_picture_of_a_screen_it_could_not_reach",
    "capture_interrupted_leaves_nothing_behind",
    "every_way_with_words_lands_on_one_control_of_its_picture",
    "map_page_logic_routes_walks_searches_and_scales_areas",
    "map_overview_counts_what_the_model_holds",
    "map_feature_and_domain_gather_screens_terms_code_and_changes",
    "map_glossary_by_screen_lists_the_terms_its_picture_shows",
    "change_map_survives_missing_or_broken_pictures",
    "the_pipeline_retakes_pictures_when_a_screen_changes",
    "change_map_page_loads_nothing_from_outside",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

FLOW = ROOT / "docs/features/flow.yaml"
SHOTS = ROOT / "docs/images/screens/screens.json"
CAPTURE = ROOT / "scripts/capture-screens.py"
MAP = ROOT / "scripts/change-map.py"
RUNNER = ROOT / "scripts/verify-app.py"
PAGE = ROOT / "scripts/change-map/page.js"

MUTATIONS = {
    # Requirement 1.
    "a-screen-says-neither-how-it-is-reached-nor-why-not": (
        FLOW, "    features: palette\n    reach: 検索・操作\n", "    features: palette\n"),
    "a-reach-word-the-app-does-not-draw": (
        FLOW, "    features: palette\n    reach: 検索・操作\n", "    features: palette\n    reach: 検索と操作\n"),
    "the-plan-accepts-both-reach-and-noshot": (
        CAPTURE, "if (reach is None) == (noshot is None):", "if reach is None and noshot is None:"),
    # Requirement 2.
    "an-entry-names-a-picture-that-is-not-there": (
        SHOTS, '"picture": "palette.jpg",', '"picture": "palette.png",'),
    # Requirement 3.
    "the-picture-is-taken-of-another-window": (
        CAPTURE, 'number = "".join(c for c in window if c.isdigit())', 'number = "1"'),
    "controls-are-kept-in-screen-points": (
        CAPTURE, "x, y = box[0] - wx, box[1] - wy", "x, y = box[0], box[1]"),
    "each-screen-is-left-running": (
        CAPTURE, "        temporary.unlink(missing_ok=True)\n        runner.stop(state)\n",
        "        temporary.unlink(missing_ok=True)\n"),
    # Requirement 4.
    "a-screen-not-reached-loses-its-earlier-entry": (
        CAPTURE, 'failures.append((screen["id"], str(error)))',
        'failures.append((screen["id"], str(error))); index["screens"].pop(screen["id"], None)'),
    "a-screen-not-reached-is-not-named": (
        CAPTURE, 'say(f"not taken: {sid}: {why}")', 'say("some screens were not taken")'),
    # Requirement 5.
    "sigterm-skips-the-cleanup": (CAPTURE, "signal.signal(signal.SIGTERM, interrupted)", "pass"),
    "a-half-picture-is-kept": (
        CAPTURE, "    finally:\n        temporary.unlink(missing_ok=True)\n",
        "    finally:\n        pass\n"),
    # Requirement 6.
    "an-area-lands-beside-its-control": (
        MAP, 'way["area"] = {k: c.get(k) for k in ("x", "y", "w", "h")}',
        'way["area"] = {"x": c.get("y"), "y": c.get("x"), "w": c.get("w"), "h": c.get("h")}'),
    "a-way-whose-words-answer-two-controls": (FLOW, "    press: セッション #1\n", "    press: セッション\n"),
    "the-map-matches-names-by-another-rule": (
        RUNNER, "    return exact or [at for at, name in enumerate(names) if words in name]",
        "    return exact"),
    # Requirement 7.
    "a-link-that-does-not-parse-back": (PAGE, '"glossary/screens": "glossary-screens", ', ""),
    "an-area-scales-by-the-height": (PAGE, "const f = drawnWidth / window.w;", "const f = drawnWidth / window.h;"),
    "a-new-step-keeps-what-was-ahead": (PAGE, "    this.stack = this.stack.slice(0, this.at + 1);\n", ""),
    "search-ranks-an-exact-title-last": (PAGE, "title === q ? 4 :", "title === q ? 1 :"),
    "a-key-is-gone-from-the-table": (PAGE, 'ArrowLeft: "back", ', ""),
    # Requirement 8.
    "the-overview-counts-every-screen-as-photographed": (
        MAP, '"photographed": sum(1 for s in flow["screens"] if s.get("shot")),',
        '"photographed": len(flow["screens"]),'),
    "the-overview-counts-pinned-lines-as-bare": (
        MAP, "        if not row[\"pinned\"]:\n            bare[", "        if row[\"pinned\"]:\n            bare["),
    # Requirement 9.
    "a-feature-gets-every-screen": (
        MAP, "                                   if feature[\"id\"] in s[\"features\"]]",
        "                                   ]"),
    "a-domain-gets-no-screen": (
        MAP, "if set(s[\"features\"]) & set(domain[\"features\"])]", "if False]"),
    # Requirement 10.
    "a-screen-shows-only-the-terms-by-their-own-word": (
        MAP, "for word in [t[\"term\"]] + t[\"ui\"])]", "for word in [t[\"term\"]])]"),
    # Requirement 11.
    "a-picture-of-the-wrong-size-is-shown": (
        MAP, 'screen["shot_problem"] = "写真を読めませんでした: 大きさが記録と違います"\n            continue',
        'pass'),
    "the-summary-does-not-count-missing-pictures": (MAP, '"画面 %d · 写真なし %d"', '"画面 %d · 写真 %d"'),
    "too-many-pictures-are-dropped-silently": (
        MAP, '"写真が多すぎます。%d 画面のうち %d 画面を載せています"', '"%d / %d"'),
    # Requirement 12.
    "close-forgets-to-retake": (
        ROOT / ".claude/skills/feature-pilot/SKILL.md",
        "  `reach` or `press` in `flow.yaml` and runs `python3 scripts/capture-screens.py`",
        "  `flow.yaml`"),
    "the-readme-forgets-unseen": (ROOT / "docs/features/README.md", "- `unseen` —", "- unseen —"),
    # The page's pictures are only ever inlined ones.
    "a-picture-may-be-fetched": (
        PAGE, "/^data:image\\/(jpeg|png);base64,[A-Za-z0-9+\\/=]*$/", "/./"),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
