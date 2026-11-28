#!/usr/bin/env python3
"""The mutation sweep of change 325: each mutation breaks one claim the map's
model, the backlog reader, the server or the page makes, and one of the twelve
tests of spec.md must go red. Copied from docs/sdlc/templates/mutations.py;
the runner is scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 2400

TESTS = [
    "map_status_counts_features_changes_and_what_waits_for_a_person",
    "map_features_carry_cover_rail_code_by_layer_and_evidence",
    "map_work_board_puts_every_card_in_exactly_one_column",
    "map_reads_the_backlog_only_when_asked_and_survives_its_failure",
    "map_process_shows_every_route_surface_and_method_the_pipeline_defines",
    "map_catalog_lists_every_palette_role_icon_and_key",
    "map_page_logic_parses_lenses_and_counts_them_faceted",
    "map_page_logic_builds_the_work_order_and_the_review_packet",
    "map_page_logic_filters_board_and_requirements_and_knows_its_keys",
    "map_page_has_status_tiles_work_process_and_catalog_views",
    "map_serves_a_fresh_page_on_loopback_and_stops",
    "map_readme_names_every_view_and_option",
    "map_page_logic_routes_walks_searches_and_scales_areas",
    "map_server_cuts_a_request_that_dribbles_past_five_seconds",
    "map_backlog_survives_an_integer_too_large_for_a_float",
    "map_backlog_deadline_holds_when_a_grandchild_keeps_stdout",
    "map_backlog_cards_have_unique_ids_when_numbers_repeat",
    "map_page_logic_reads_any_kept_basket",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MAP = ROOT / "scripts/change-map.py"
PAGE = ROOT / "scripts/change-map/page.js"
HTML = ROOT / "scripts/change-map/page.html"
README = ROOT / "docs/features/README.md"

MUTATIONS = {
    # Requirement 1.
    "a-human-check-already-passed-still-waits": (
        MAP, 'if k["kind"] == "human" and k["verdict"] == "pending"])',
        'if k["kind"] == "human"])'),
    "a-change-off-the-stages-is-lost": (
        MAP, '"off_track": [c["name"] for c in opened if c["stage"] not in STAGES],',
        '"off_track": [],'),
    "a-landed-change-counts-as-open": (
        MAP, 'return change["group"] in ("needs", "flight")', 'return change["group"] != "archived"'),
    # Requirement 2.
    "a-half-tested-feature-counts-as-tested": (
        MAP, 'else "partial" if pinned else "none")', 'else "full" if pinned else "none")'),
    "a-test-file-is-drawn-code": (MAP, '    if path == "src/tests.rs":\n        return "test"\n', ""),
    "an-estimated-link-gives-a-next-step": (
        MAP, 'for link in feature["changes"] if link["link"] != "estimated"]',
        'for link in feature["changes"]]'),
    # Requirement 3.
    "a-held-item-stays-in-the-backlog-column": (
        MAP, '"column": "held" if held else "backlog",', '"column": "backlog",'),
    "every-landed-change-gets-a-card": (
        MAP, "            if landed > LANDED_CARDS:\n                continue\n", ""),
    # Requirement 4.
    "a-slow-backlog-stops-the-map": (
        MAP, "        timer = threading.Timer(seconds, overdue)\n        timer.start()\n",
        "        timer = threading.Timer(seconds, overdue)\n"),
    "a-backlog-link-may-go-anywhere": (
        MAP, 'url.startswith("https://github.com/")', 'url.startswith("http")'),
    "a-failing-backlog-is-not-said": (
        MAP, 'return failed("終了コード %d%s"', 'return out or failed("終了コード %d%s"'),
    # Requirement 5.
    "a-route-of-the-wrong-shape-is-drawn": (
        MAP, "        if len(parts) != 5:\n            continue\n", ""),
    "an-unread-table-is-not-said": (
        MAP, '        if text is None or cut:\n            unread.append(name)\n',
        '        if text is None or cut:\n            pass\n'),
    # Requirement 6.
    "a-shadow-colour-is-lost": (MAP, '    if call == "from_black_alpha" and len(numbers) == 1:', '    if False:'),
    "an-icon-counts-its-own-definition": (
        MAP, 'relative not in ("src/glyphs.rs", "src/tests.rs")', 'relative != "src/tests.rs"'),
    # Requirement 7.
    "a-lens-press-never-clears": (
        PAGE, 'else values = chosen.length === 1 && chosen[0] === value ? [] : [value];',
        'else values = [value];'),
    "an-untested-feature-answers-nothing": (
        PAGE, 'of: f => f.cover === "empty" || !f.cover ? "none" : f.cover },', 'of: f => f.cover },'),
    "counts-ignore-the-other-lenses": (
        PAGE, "const tried = Object.assign({}, query || {}, { [lens.id]: [value] });",
        "const tried = { [lens.id]: [value] };"),
    "a-query-is-lost-from-the-link": (PAGE, 'function formatRoute(r) { return formatPath(r) + formatQuery(r.query); }',
                                      'function formatRoute(r) { return formatPath(r); }'),
    # Requirement 8.
    "a-file-is-listed-twice": (
        PAGE, "for (const file of entry.files) if (!files.includes(file)) files.push(file);",
        "for (const file of entry.files) files.push(file);"),
    "a-gone-record-stays-in-the-basket": (
        PAGE, "      if (!f) return null;\n      const files = [];", "      const files = [];"),
    "an-unread-backlog-drops-an-issue": (PAGE, "if (!item && backlog.asked) return null;", "if (!item) return null;"),
    "ok-verdicts-are-listed-to-fix": (PAGE, 'if (v.verdict !== "ok") lines', "lines"),
    "a-refused-storage-throws": (
        PAGE, "        } catch (error) { return fallback; }\n", "        } finally { }\n"),
    # Requirement 9.
    "the-board-ignores-who": (PAGE, "return owner ? cards.filter(card => card.owner === owner) : cards.slice();",
                              "return cards.slice();"),
    "untested-requirements-pass-as-tested": (
        PAGE, '(!f.tests || (f.tests === "yes") === !!row.pinned)', "true"),
    "the-basket-key-is-gone": (PAGE, ' b: "basket",', ""),
    "the-theme-never-goes-back-to-auto": (
        PAGE, "return THEMES[(at + 1) % THEMES.length][0];", "return THEMES[Math.min(at + 1, THEMES.length - 1)][0];"),
    # Requirement 10.
    "the-bar-loses-work": (HTML, '<button data-view="work">作業</button>', ""),
    "the-features-column-is-gone": (HTML, 'id="side"', 'id="aside"'),
    "a-fourth-name-in-the-browser": (
        PAGE, 'theme: "operon-map-theme" };', 'theme: "operon-map-theme", seen: "operon-seen" };'),
    # Requirement 11.
    "the-server-listens-everywhere": (MAP, 'LOOPBACK = "127.0.0.1"', 'LOOPBACK = "0.0.0.0"'),
    "every-path-gets-the-map": (MAP, 'elif self.path.split("?", 1)[0] != "/":', "elif False:"),
    "a-post-gets-the-map": (MAP, 'if self.command != "GET":', "if False:"),
    "a-cut-request-gets-the-map": (MAP, "                if not whole:\n", "                if False:\n"),
    # Found by the empirical verifier on df05c08.
    "a-dribbling-request-holds-the-server": (MAP, "            timer.start()\n            try:\n                try:", "            try:\n                try:"),
    "a-late-request-gets-the-map": (MAP, "                if late:\n", "                if False:\n"),
    # F2-F5, found by the second pass on fe1af20.
    "a-huge-integer-stops-the-map": (MAP, "    except OverflowError:\n", "    except ArithmeticError if False else ValueError:\n"),
    "the-pipe-is-read-to-its-end": (
        MAP, "if left <= 0 or not select.select([child.stdout], [], [], left)[0]:", "if False:"),
    "a-repeated-number-is-a-second-card": (MAP, " or int(number) in seen:", ":"),
    "a-number-not-whole-is-a-card": (MAP, " or number != int(number) or", " or"),
    "a-kept-null-is-taken-for-a-basket": (
        PAGE, 'const k = kept && typeof kept === "object" && !Array.isArray(kept) ? kept : {};',
        "const k = kept;"),
    # Requirement 12.
    "the-readme-forgets-the-server": (README, "- `--serve` —", "- serve —"),
    "the-readme-forgets-the-catalog": (README, "| カタログ |", "| Catalog |"),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
