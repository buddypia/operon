#!/usr/bin/env python3
"""Break the Home row's state mark and its hover (change 287)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "home_287_"]]
WIDGETS = ROOT / "src/ui/widgets.rs"
SCREENS = ROOT / "src/app/screens.rs"
HOVER = "                session_status_icon(ui, palette, status.kind).on_hover_ui(|ui| {\n                    ui.label(&status.label);\n                    ui.label(&status.hint);\n                });\n"
MUTATIONS = {
    "the-word-is-back-in-the-row": (SCREENS, HOVER, "                status_chip(ui, &status, 13.5, palette);\n"),
    "the-mark-says-nothing-on-hover": (SCREENS, HOVER, "                session_status_icon(ui, palette, status.kind);\n"),
    "the-hover-does-not-name-the-state": (SCREENS, "                    ui.label(&status.label);\n                    ui.label(&status.hint);\n", "                    ui.label(&status.hint);\n"),
    "the-hover-does-not-say-which-state": (SCREENS, "                    ui.label(&status.label);\n                    ui.label(&status.hint);\n", "                    ui.label(&status.label);\n"),
    "the-mark-takes-the-rows-click": (WIDGETS, "        egui::vec2(TAB_STATUS_MARK_SIZE, CONTROL_HEIGHT_SMALL),\n        egui::Sense::hover(),", "        egui::vec2(TAB_STATUS_MARK_SIZE, CONTROL_HEIGHT_SMALL),\n        egui::Sense::click(),"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
