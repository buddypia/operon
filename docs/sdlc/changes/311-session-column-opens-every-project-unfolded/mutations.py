#!/usr/bin/env python3
"""Break the session column's fold default, its file, its pitch, and its filter (change 311)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
TESTS = ["column_311_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
SCREENS = ROOT / "src/app/screens.rs"
ORDER = ROOT / "src/app/session_order.rs"
TREE = ROOT / "src/ui/session_tree.rs"

MUTATIONS = {
    # R1: a project nobody opened is drawn open.
    "projects-start-unfolded": (
        SCREENS,
        "                        filtering || self.session_arrangement.unfolded.contains(&project_id);\n",
        "                        filtering || !self.session_arrangement.unfolded.contains(&project_id);\n",
    ),
    # R2: the claim "an older file folds every project" — the new list is read
    # from the key the older build wrote its folded projects under.
    "older-folded-key-read-as-unfolded": (
        TREE,
        "    #[serde(default)]\n    pub(crate) unfolded: Vec<Uuid>,",
        "    #[serde(default, rename = \"folded\")]\n    pub(crate) unfolded: Vec<Uuid>,",
    ),
    # R2: a project that is gone stays in the file.
    "gone-project-stays-unfolded": (
        ORDER,
        "        arrangement\n            .unfolded\n            .retain(|id| projects.iter().any(|project| project.id == *id));\n",
        "",
    ),
    # R3: the frame keeps its margin, so the pitch is 32px.
    "project-frame-keeps-its-margin": (
        SCREENS,
        "                        .inner_margin(egui::Margin::symmetric(SPACE_XS as i8, 0))\n                        .show(ui, |ui| {\n                            ui.set_min_width(ui.available_width());\n                            ui.spacing_mut().item_spacing.y = SIDEBAR_ROW_GAP;",
        "                        .inner_margin(egui::Margin::symmetric(SPACE_XS as i8, 2))\n                        .show(ui, |ui| {\n                            ui.set_min_width(ui.available_width());\n                            ui.spacing_mut().item_spacing.y = SIDEBAR_ROW_GAP;",
    ),
    # R3: the heading's icon button keeps the app's padding, so a heading is
    # 25px tall and the pitch 29px.
    "heading-button-keeps-the-app-padding": (
        SCREENS,
        "                                ui.spacing_mut().button_padding.y = SPACE_XS;\n",
        "",
    ),
    # R3: a folded heading keeps its rule.
    "folded-heading-keeps-its-rule": (
        SCREENS,
        "                            if !unfolded {\n                                return;\n                            }\n                            hairline(ui, palette);\n",
        "                            hairline(ui, palette);\n                            if !unfolded {\n                                return;\n                            }\n",
    ),
    # R4: a filter no longer lists the matches of a folded project.
    "filter-hides-matches-of-a-folded-project": (
        SCREENS,
        "                    let unfolded =\n                        filtering || self.session_arrangement.unfolded.contains(&project_id);\n",
        "                    let unfolded = self.session_arrangement.unfolded.contains(&project_id);\n",
    ),
    # R4: the disclosure changes and writes the fold while a filter is on.
    "filter-click-changes-the-fold": (
        SCREENS,
        "                                if small_icon_button(ui, palette, icon, tip).clicked() && !filtering\n",
        "                                if small_icon_button(ui, palette, icon, tip).clicked()\n",
    ),
    # R4, the claim a person reads: under a filter the disclosure says it
    # folds, which it does not.
    "filter-tooltip-promises-a-fold": (
        SCREENS,
        "                                    (ICON_DISCLOSURE_OPEN, tr(\"フィルタ中はすべて表示されます\"))\n",
        "                                    (ICON_DISCLOSURE_OPEN, tr(\"折りたたむ\"))\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
