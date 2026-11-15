#!/usr/bin/env python3
"""The mutation sweep for change 294: the 新しいセッション sheet adds a project
in place — a 「新しいプロジェクト…」 row, a no-project state, drops onto the
sheet, recent-first order and a name filter. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = ["launch_294_", "new_session_opens_the_sheet_over_the_current_page"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

LP = ROOT / "src/app/launch_projects.rs"
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    # R1: the list loses its 「新しいプロジェクト…」 row.
    "no-new-project-row": (
        LP,
        "                if ui.selectable_label(false, new_project).clicked() {",
        "                if false && ui.selectable_label(false, new_project).clicked() {",
    ),
    # R1, the claim "it opens the folder picker": the row switches instead.
    "new-project-row-opens-nothing": (
        LP,
        "            LaunchProjectPick::NewFolder => self.choose_launch_project_folder(),",
        "            LaunchProjectPick::NewFolder => {}",
    ),
    # R2: the sheet is not pointed at the new project.
    "new-folder-not-chosen": (
        LP,
        "        self.point_launch_sheet_at(registered.id());\n        self.notice_briefly(match registered {",
        "        self.notice_briefly(match registered {",
    ),
    # R2: the switch forgets what was typed.
    "switch-drops-the-request": (
        LP,
        "        self.goal_input = goal;\n",
        "        let _ = goal;\n",
    ),
    # R3, the claim "switched to, not refused": an existing folder is refused.
    "existing-folder-refused": (
        LP,
        "            return Ok(RegisteredProject::Existing(existing.id));",
        "            return Err(ProjectAddition::Declined);",
    ),
    # R3: the switch is announced as an addition.
    "existing-announced-as-added": (
        LP,
        "            RegisteredProject::Existing(_) => tr(\"登録済みのプロジェクトに切り替えました。\"),",
        "            RegisteredProject::Existing(_) => tr(\"プロジェクトを追加しました。\"),",
    ),
    # R5: no project jumps to the projects page again.
    "no-project-jumps-away": (
        APP,
        "            self.launch_project_filter.clear();\n            self.launch_sheet_open = true;\n            return;",
        "            self.page = Page::Projects;\n            return;",
    ),
    # R5: the sheet closes itself when there is no project.
    "no-project-sheet-closes": (
        SCREENS,
        "        if project.is_none() && !self.store.projects.is_empty() {",
        "        if project.is_none() {",
    ),
    # R5: 「フォルダを選択…」 opens nothing.
    "choose-folder-button-dead": (
        SCREENS,
        "        if choose_folder {\n            self.choose_launch_project_folder();",
        "        if false {\n            self.choose_launch_project_folder();",
    ),
    # R6: drops always go to the projects page.
    "drop-ignores-the-sheet": (
        LP,
        "        if !self.launch_sheet_shows() {\n            self.add_dropped_folders(dropped);",
        "        if true {\n            self.add_dropped_folders(dropped);",
    ),
    # R7: never-used projects outrank used ones.
    "added-order-beats-recency": (
        LP,
        "    choices.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));",
        "    choices.sort_by(|a, b| (b.1, b.0).cmp(&(a.1, a.0)));",
    ),
    # R7: the latest session is not the one that counts.
    "earliest-session-counts": (
        LP,
        "        *entry = (*entry).max(started);",
        "        *entry = (*entry).min(started);",
    ),
    # R8: the filter becomes case-sensitive.
    "filter-case-sensitive": (
        LP,
        "        .filter(|project| needle.is_empty() || project.name.to_lowercase().contains(&needle))",
        "        .filter(|project| needle.is_empty() || project.name.contains(&needle))",
    ),
    # The multi-drop message counts already-registered folders as added.
    "multi-drop-counts-existing": (
        LP,
        "                    added += usize::from(matches!(registered, RegisteredProject::New(_)));",
        "                    added += 1;",
    ),
    # A dismissed list keeps its filter.
    "dismissed-filter-kept": (
        LP,
        "        if !list_open && !self.launch_project_filter.is_empty() {",
        "        if false {",
    ),
    # R8: the field shows for any number of projects.
    "filter-always-shown": (
        LP,
        "                if self.store.projects.len() >= LAUNCH_PROJECT_FILTER_MIN {",
        "                if !self.store.projects.is_empty() {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
