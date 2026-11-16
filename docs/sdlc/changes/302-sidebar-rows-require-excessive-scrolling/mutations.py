#!/usr/bin/env python3
"""Break compact sidebar spacing, row heights, type scale, and drag drop gaps (change 302)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
TESTS = ["sidebar_302_", "session_column_project_is_a_heading_not_a_card"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
SCREENS = ROOT / "src/app/screens.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"

MUTATIONS = {
    "column-inherits-10px-outer-spacing": (
        SCREENS,
        "    pub(crate) fn ui_workspace_projects(&mut self, ui: &mut egui::Ui) {\n        let palette = self.store.theme.palette();\n        ui.spacing_mut().item_spacing.y = SPACE_XS;\n",
        "    pub(crate) fn ui_workspace_projects(&mut self, ui: &mut egui::Ui) {\n        let palette = self.store.theme.palette();\n",
    ),
    "agent-segment-keeps-2px-margin": (
        SCREENS,
        "        egui::Frame::NONE\n            .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle))\n            .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))\n            .inner_margin(egui::Margin::same(1))",
        "        egui::Frame::NONE\n            .stroke(egui::Stroke::new(1.0_f32, palette.border_subtle))\n            .corner_radius(egui::CornerRadius::same(RADIUS_CONTROL))\n            .inner_margin(egui::Margin::same(2))",
    ),
    "project-heading-keeps-28px-row-height": (
        SCREENS,
        "                            ui.spacing_mut().item_spacing.y = SIDEBAR_ROW_GAP;\n                            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_SMALL;\n",
        "                            ui.spacing_mut().item_spacing.y = SIDEBAR_ROW_GAP;\n",
    ),
    "session-row-keeps-28px-horizontal-min": (
        SCREENS,
        "                                        ui.spacing_mut().interact_size.y = COLUMN_SESSION_MARK_SIZE;\n",
        "",
    ),
    "session-row-overlaps-badge-ring": (
        SCREENS,
        "                                    .corner_radius(egui::CornerRadius::same(RADIUS_CARD))\n                                    .inner_margin(egui::Margin::symmetric(SPACE_XS as i8, 2))\n                                    .show(ui, |ui| {\n                                        ui.set_min_width(ui.available_width());\n                                        ui.spacing_mut().interact_size.y = COLUMN_SESSION_MARK_SIZE;",
        "                                    .corner_radius(egui::CornerRadius::same(RADIUS_CARD))\n                                    .inner_margin(egui::Margin::symmetric(SPACE_XS as i8, 1))\n                                    .show(ui, |ui| {\n                                        ui.set_min_width(ui.available_width());\n                                        ui.spacing_mut().interact_size.y = COLUMN_SESSION_MARK_SIZE;",
    ),
    "custom-row-uses-24px-status-icon": (
        SCREENS,
        "                                                session_status_icon_sized(\n                                                    ui,\n                                                    palette,\n                                                    kind,\n                                                    COLUMN_SESSION_MARK_SIZE,\n                                                );",
        "                                                session_status_icon(ui, palette, kind);",
    ),
    "session-row-uses-10px-horizontal-gap": (
        SCREENS,
        "                                        ui.horizontal(|ui| {\n                                            ui.spacing_mut().item_spacing.x = SPACE_XS;\n",
        "                                        ui.horizontal(|ui| {\n",
    ),
    "session-title-stays-12-5px": (
        SCREENS,
        "                                                ui.add(\n                                                    egui::Label::new(\n                                                        RichText::new(&title)\n                                                            .size(12.0)\n                                                            .strong()",
        "                                                ui.add(\n                                                    egui::Label::new(\n                                                        RichText::new(&title)\n                                                            .size(12.5)\n                                                            .strong()",
    ),
    "project-heading-unselected-stays-12-5px": (
        SCREENS,
        "                                                        } else {\n                                                            RichText::new(&project.name)\n                                                                .size(12.0)",
        "                                                        } else {\n                                                            RichText::new(&project.name)\n                                                                .size(12.5)",
    ),
    "status-count-chip-stays-12-5px": (
        WIDGETS,
        "            .layout_no_wrap(count.to_string(), egui::FontId::proportional(11.5), ink);",
        "            .layout_no_wrap(count.to_string(), egui::FontId::proportional(12.5), ink);",
    ),
    "project-drag-drop-line-uses-wrong-gap": (
        SCREENS,
        "                        ColumnDrag::Project(_) => (&targets, SPACE_XS),",
        "                        ColumnDrag::Project(_) => (&targets, SPACE_SM),",
    ),
    "session-drag-drop-line-uses-wrong-gap": (
        SCREENS,
        "                        ColumnDrag::Session { .. } => (&session_targets, SIDEBAR_ROW_GAP),",
        "                        ColumnDrag::Session { .. } => (&session_targets, SPACE_SM),",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
