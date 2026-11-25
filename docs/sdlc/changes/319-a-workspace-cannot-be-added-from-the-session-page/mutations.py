#!/usr/bin/env python3
"""The mutation sweep for change 319: the session page's column adds a
session or a project, keeps a project added there on the session page, and
draws project names one level above their rows. Runner:
scripts/mutation_sweep.py.
"""
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = ["column_319_"]
# Incremental rebuilds keep a dozen mutations inside the obligation ceiling
# (change 270, change 317).
os.environ["CARGO_INCREMENTAL"] = "1"
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

CA = ROOT / "src/app/column_add.rs"
FP = ROOT / "src/app/folder_picker.rs"
SC = ROOT / "src/app/screens.rs"

MUTATIONS = {
    # R1: the heading's + opens no menu.
    "plus-opens-nothing": (
        SC,
        "                column_add = egui::Popup::menu(&add)\n                    .id(column_add_menu_id())\n                    .show(|ui| Self::ui_column_add_items(ui, palette))\n                    .and_then(|response| response.inner);",
        "                let _ = add;",
    ),
    # R1/R2, the claim "a session or a project": the menus lose the project.
    "menus-offer-only-a-session": (
        CA,
        "pub(crate) const COLUMN_ADD_CHOICES: [ColumnAdd; 2] = [ColumnAdd::Session, ColumnAdd::Project];",
        "pub(crate) const COLUMN_ADD_CHOICES: [ColumnAdd; 1] = [ColumnAdd::Session];",
    ),
    # R2: the empty space has no menu.
    "empty-space-has-no-menu": (
        SC,
        "                if rest.height() > 0.0 {",
        "                if false {",
    ),
    # R3: 「新しいセッション…」 opens the folder picker instead of ⌘N's sheet.
    "new-session-is-not-command-n": (
        CA,
        "            ColumnAdd::Session => self.open_new_session(),",
        "            ColumnAdd::Session => self.open_folder_picker(FolderPickerTarget::SessionColumn),",
    ),
    # R4: the column's entry opens the projects page's picker.
    "column-opens-projects-picker": (
        CA,
        "            ColumnAdd::Project => self.open_folder_picker(FolderPickerTarget::SessionColumn),",
        "            ColumnAdd::Project => self.choose_project_folder(),",
    ),
    # R4, the claim "stays on the session page": the folder goes to the page.
    "column-target-leaves-the-page": (
        FP,
        "            FolderPickerTarget::SessionColumn => self.apply_column_project_folder(Some(path)),",
        "            FolderPickerTarget::SessionColumn => {\n                self.add_project_path(path);\n            }",
    ),
    # R4/R5, the claim "unfolded": the added project arrives folded.
    "added-project-stays-folded": (
        CA,
        "        if !self.session_arrangement.unfolded.contains(&project_id) {",
        "        if false {",
    ),
    # R4/R5, the claim "chosen": the workspace keeps its old project.
    "added-project-not-chosen": (
        CA,
        "        self.select_workspace_project(project_id);\n",
        "",
    ),
    # R5, the claim "not added twice … opened": the existing notice is lost.
    "registered-folder-says-added": (
        CA,
        "                (id, tr(\"そのプロジェクトは追加済みのため、開きました。\"))",
        "                (id, tr(\"プロジェクトを追加しました。\"))",
    ),
    # R6: a heading's right-click cannot start a session in that project.
    "heading-cannot-start": (
        SC,
        "                                                    if quiet_button(\n                                                        ui,\n                                                        palette,\n                                                        tr(\"セッションを開始\"),",
        "                                                    if false && quiet_button(\n                                                        ui,\n                                                        palette,\n                                                        tr(\"セッションを開始\"),",
    ),
    # R6: a heading's right-click cannot add a project.
    "heading-cannot-add": (
        SC,
        "                                                    if Self::ui_column_add_project_item(",
        "                                                    if false && Self::ui_column_add_project_item(",
    ),
    # R7, the claim "larger than the rows": names fall back to the rows' size.
    "names-at-row-size": (
        SC,
        "                                                                .size(13.5)\n                                                                .strong()\n                                                                .color(if selected {",
        "                                                                .size(12.0)\n                                                                .strong()\n                                                                .color(if selected {",
    ),
    # R8: 履歴 goes back to the framed button.
    "history-framed": (
        SC,
        "                if quiet_button(ui, palette, tr(\"履歴\")).clicked() {",
        "                if ui.button(tr(\"履歴\")).clicked() {",
    ),
    # F1: the Escape that closes the + menu reaches a terminal.
    "escape-passed-on": (
        ROOT / "src/app/launch_menu.rs",
        "            || egui::Popup::is_id_open(ui, column_add_menu_id());",
        "            || false;",
    ),
    # R9: the empty column offers no way to add.
    "empty-column-offers-nothing": (
        SC,
        "            if secondary_button(ui, ColumnAdd::Project.label()).clicked() {",
        "            if false && secondary_button(ui, ColumnAdd::Project.label()).clicked() {",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
