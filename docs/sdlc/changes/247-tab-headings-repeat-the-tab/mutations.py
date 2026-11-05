#!/usr/bin/env python3
"""Change 247 mutation sweep: break each claim and watch its guard go red."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "tab_content_no_longer_repeats_its_tab_as_a_heading",
    "session_list_title_is_a_panel_label_not_a_heading",
    "agent_settings_sections_use_section_headings",
    "settings_sidebar_label_is_a_panel_label",
    "tab_headings_keep_their_controls",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

SCREEN = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "git tab repeats its tab as a heading again": (
        SCREEN,
        "        // No heading: the 変更 tab already says this is the project's\n        // changes (change 247). The row is the view switch and refresh.\n        ui.horizontal(|ui| {\n            for view in GitView::all() {",
        "        ui.horizontal(|ui| {\n            ui.heading(\"Git\");\n            for view in GitView::all() {",
    ),
    "session list title becomes a heading again": (
        SCREEN,
        "        ui.label(\n            RichText::new(tr(\"プロジェクト別セッション\"))\n                .size(12.5)\n                .color(palette.text_muted),\n        );",
        "        ui.heading(tr(\"プロジェクト別セッション\"));",
    ),
    "skills section becomes a page heading again": (
        SCREEN,
        "            ui.label(\n                RichText::new(tr(\"スキル\"))\n                    .size(15.0)",
        "            ui.label(\n                RichText::new(tr(\"スキル\"))\n                    .size(20.0)",
    ),
    "settings sidebar label becomes a page title again": (
        SCREEN,
        "                    RichText::new(tr(\"設定\"))\n                        .size(12.5)\n                        .color(palette.text_muted),",
        "                    RichText::new(tr(\"設定\"))\n                        .size(20.0)\n                        .strong()\n                        .color(palette.text_strong),",
    ),
    "git row loses its refresh control": (
        SCREEN,
        "            if icon_button(ui, palette, ICON_REFRESH, tr(\"更新\")).clicked() {\n                match self.git_view {",
        "            if icon_button(ui, palette, ICON_ADD, tr(\"更新\")).clicked() {\n                match self.git_view {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
