#!/usr/bin/env python3
"""Change 207's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/207-tests-that-assert-nothing")

RUN_SECONDS = 1800

TESTS = [
    "tests::ui_session_tabs_omits_static_instruction_label_and_streamlines_card",
    "tests::ui_session_ports_renders_compact_popover_button_when_active",
    "tests::the_filtered_session_list_says_how_many_it_hides",
    "tests::every_message_row_is_one_the_code_shows",
    "tests::ci_runs_on_the_branches_the_merge_gate_asks_about",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

SCREENS = ROOT / "src/app/screens.rs"
APP = ROOT / "src/app.rs"
TABLES = ROOT / "src/i18n_tables.rs"
CI = ROOT / ".github/workflows/ci.yml"
MUTATIONS = {
    "the session list's guide line comes back": (
        SCREENS,
        "    pub(crate) fn ui_terminal_session_tabs(&mut self, ui: &mut egui::Ui) {\n",
        "    pub(crate) fn ui_terminal_session_tabs(&mut self, ui: &mut egui::Ui) {\n        ui.label(\"ダブルクリックで名前を変更 · ⇆ 復元 · × 削除\");\n",
    ),
    "one port is counted rather than named": (
        APP,
        "        let badge_text = if ports.len() == 1 {",
        "        let badge_text = if ports.is_empty() {",
    ),
    "the badge miscounts the ports": (
        APP,
        'tf!("{p0} ポート稼働中", p0 = ports.len())',
        'tf!("{p0} ポート稼働中", p0 = ports.len() + 1)',
    ),
    "the hidden count is the whole list": (
        SCREENS,
        'p0 = sessions.len().saturating_sub(shown)',
        'p0 = sessions.len()',
    ),
    "a row nothing shows is kept": (
        TABLES,
        'pub(crate) const EN_TABLE: Table = &[\n',
        'pub(crate) const EN_TABLE: Table = &[\n    ("\\n\\n… どこにも出ない行", "\\n\\n… a row nothing shows"),\n',
    ),
    "the tests are built with warnings allowed": (
        CI,
        "    env:\n      RUSTFLAGS: -D warnings\n",
        "",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
