#!/usr/bin/env python3
"""The mutation sweep of change 315: each mutation breaks one claim the
domains, the glossary, the flow, the Pinned by lines, the page or the skills
make, and one of the ten tests of spec.md must go red. Copied from
docs/sdlc/templates/mutations.py; the runner is scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/315-the-feature-map-holds-no-domain-terms-or-screen-flow")

RUN_SECONDS = 1800

TESTS = [
    "domains_cover_every_module_once_and_every_feature",
    "glossary_terms_resolve_to_features_screen_words_and_code",
    "no_screen_word_uses_a_word_the_glossary_avoids",
    "screen_flow_reaches_every_page_and_every_way_resolves",
    "every_navigation_key_is_a_way_in_the_flow",
    "feature_behaviours_name_tests_that_exist",
    "change_map_carries_domains_terms_flow_and_requirements",
    "change_map_page_has_domain_flow_glossary_and_requirement_views",
    "change_map_survives_a_missing_domain_glossary_or_flow",
    "the_pipeline_reads_the_domain_the_glossary_and_the_flow",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

DOMAINS = ROOT / "docs/features/domains.yaml"
INDEX = ROOT / "docs/features/index.yaml"
GLOSSARY = ROOT / "docs/features/glossary.yaml"
FLOW = ROOT / "docs/features/flow.yaml"
MAP = ROOT / "scripts/change-map.py"
HOME_SPEC = ROOT / "docs/features/home/spec.md"

MUTATIONS = {
    # Requirement 1.
    "a-module-belongs-to-no-domain": (
        DOMAINS, "modules: src/store.rs src/models.rs src/exec.rs src/sys.rs",
        "modules: src/store.rs src/models.rs src/exec.rs"),
    "a-module-belongs-to-two-domains": (
        DOMAINS, "modules: src/store.rs src/models.rs src/exec.rs src/sys.rs",
        "modules: src/store.rs src/models.rs src/exec.rs src/sys.rs src/tmux.rs"),
    "a-neighbour-is-named-one-way": (
        DOMAINS, "    adjacent: agents code data\n", "    adjacent: agents code\n"),
    "a-feature-names-no-domain-that-exists": (
        INDEX, "    domain: data\n", "    domain: storage\n"),
    # Requirement 2.
    "a-screen-word-the-product-does-not-show": (
        GLOSSARY, "    ui: 要対応のセッションを見る\n", "    ui: 要対応のセッションを眺める\n"),
    "a-code-name-the-source-does-not-have": (
        GLOSSARY, "SessionStatusGroup::NeedsYou attention_count", "SessionStatusGroup::NeedsYou attention_tally"),
    "a-related-term-that-does-not-exist": (
        GLOSSARY, "    see: needs-you busy idle\n", "    see: needs-you busy idle nowhere\n"),
    "a-term-belongs-to-no-feature": (
        GLOSSARY, "    feature: needs-you\n", "    feature: needs-them\n"),
    # Requirement 3: the claim that an avoided word is on no screen.
    "an-avoided-word-is-on-the-screen": (
        GLOSSARY, "    avoid: 注意が必要\n", "    avoid: あなたを待っている\n"),
    # Requirement 4.
    "a-page-the-flow-does-not-draw": (FLOW, "    page: Settings\n", ""),
    "a-way-names-a-word-no-button-has": (FLOW, "    label: を起動\n", "    label: を発進させる\n"),
    "a-way-names-a-key-that-does-not-exist": (FLOW, "    key: panel.toggle\n", "    key: panel.flip\n"),
    "a-screen-nothing-reaches": (
        FLOW, "  - from: settings\n    to: settings-keys\n", "  - from: settings-keys\n    to: settings\n"),
    # Requirement 5: a new page key that the flow has not learned.
    "a-new-page-key-without-a-way": (
        ROOT / "src/app/keymap.rs",
        '    KeymapAction {\n        id: "page.home",',
        '    KeymapAction {\n        id: "page.elsewhere",\n        label: "ホームを開く",\n        default: "Mod+9",\n    },\n'
        '    KeymapAction {\n        id: "page.home",'),
    # Requirement 6.
    "a-behaviour-pinned-by-a-test-that-does-not-exist": (
        HOME_SPEC, "Pinned by `home_always_says_whether_anything_waits`.",
        "Pinned by `home_always_says_nothing_waits`."),
    # Requirement 7.
    "a-domain-carries-the-wrong-features": (
        MAP, 'if f.get("domain") == entry["id"]]', 'if f.get("domain") != entry["id"]]'),
    "a-term-says-a-brief-uses-it-when-it-does-not": (
        MAP, 'if word and f.get("brief") and word in f["brief"]["text"]]',
        'if word and f.get("brief")]'),
    "a-behaviour-loses-its-tests": (
        MAP, 'tests = re.findall(r"`([^`]+)`", text[at:]) if at >= 0 else []', "tests = []"),
    "find-does-not-find-a-term": (MAP, 'found.append("用語 %s', 'found.append("語 %s'),
    # Requirement 8, and the claim the basket makes about where work starts.
    "the-glossary-view-is-gone": (
        MAP, '<button data-view="glossary" aria-pressed="false">用語集</button>', ""),
    "the-basket-no-longer-starts-through-feature-pilot": (
        MAP, "Operon の次の作業を feature-pilot で進めてください（.claude/skills/feature-pilot/SKILL.md から始める）。",
        "Operon の次の作業を進めてください。"),
    "the-flow-is-not-drawn-in-the-page": (MAP, "document.createElementNS(SVG, name)", "document.createElement(name)"),
    # Requirement 9.
    "too-many-domains-are-carried-silently": (
        MAP, '"%sが多すぎます。%d 件のうち %d 件を表示しています"', '"%s: %d / %d"'),
    "a-missing-domain-file-gives-no-reason": (
        MAP, '"%s: docs/features/%s がありません" % (unreadable, name)', '""'),
    # Requirement 10.
    "close-forgets-the-flow": (
        ROOT / ".claude/skills/sdlc/SKILL.md", "  `docs/features/flow.yaml`; the suite fails", "  the flow file; the suite fails"),
    "placement-forgets-the-glossary": (
        ROOT / ".claude/skills/feature-pilot/SKILL.md",
        "label a person reads uses the words of `docs/features/glossary.yaml` and",
        "label a person reads uses the words of the product and"),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
