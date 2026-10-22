#!/usr/bin/env python3
"""Change 205's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/205-the-find-bar-and-the-commit-box")

RUN_SECONDS = 1800

TESTS = [
    "tests::a_commit_message_stays_with_the_project_it_was_written_for",
    "tests::the_find_bar_steps_on_enter_but_not_on_a_conversions_enter",
    "tests::esc_closes_the_find_bar_into_the_pane_but_not_mid_conversion",
    "tests::reopening_the_find_bar_counts_what_arrived_while_it_was_closed",
    "tests::no_message_spells_a_remappable_chord",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
TERMINAL = ROOT / "src/ui/terminal.rs"
MUTATIONS = {
    "a drafted message lands in one shared field": (
        APP,
        "self.commit_messages.insert(project_id, drafted);",
        "self.commit_messages.insert(Uuid::nil(), drafted);",
    ),
    "a commit empties every project's field": (
        APP,
        "self.commit_messages.remove(&project_id);",
        "self.commit_messages.clear();",
    ),
    "the field gives up the keyboard on Enter again": (
        SCREENS,
        "                    .return_key(None),\n",
        "",
    ),
    "the Enter that confirms a conversion steps": (
        TERMINAL,
        "                if enters_the_ime_owns > 0 {\n                    enters_the_ime_owns -= 1;\n                    continue;\n                }\n",
        "",
    ),
    "an Enter mid-conversion steps": (
        TERMINAL,
        "                if composing {\n                    continue;\n                }\n",
        "",
    ),
    "an Esc mid-conversion closes the bar": (
        SCREENS,
        "        if composing || conversion_moved {",
        "        if conversion_moved {",
    ),
    "an Esc beside a conversion's own events closes the bar": (
        SCREENS,
        "        if composing || conversion_moved {",
        "        if composing {",
    ),
    "the field does not take the keyboard back from egui's Esc": (
        SCREENS,
        "            if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {\n                if let Some(search) = self.terminal_search.get_mut(&session_id) {\n                    search.needs_focus = true;\n                }\n            }\n",
        "",
    ),
    "closing leaves the keyboard on a field that is gone": (
        SCREENS,
        "        self.terminal_focus_request = Some(session_id);\n    }\n",
        "    }\n",
    ),
    "reopening keeps the count from when it closed": (
        SCREENS,
        "            // count it kept is the one from when it closed.\n            self.refresh_terminal_search(session_id);\n",
        "            // count it kept is the one from when it closed.\n",
    ),
    "the empty projects page types the default key": (
        SCREENS,
        "                            \"Finder からフォルダをここにドラッグすることもできます · {chord}\",\n                            chord = chord_label(chord)\n",
        "                            \"Finder からフォルダをここにドラッグすることもできます · {chord}\",\n                            chord = \"⌘O\"\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
