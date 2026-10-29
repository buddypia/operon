#!/usr/bin/env python3
"""Targeted regression sweep for history navigation and bounded project headers."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 180
TESTS = [
    "home_status_entry_opens_filtered_history_with_visible_controls",
    "history_renders_each_session_once_in_the_chosen_view",
    "history_title_single_click_opens_after_the_double_click_window",
    "history_pending_title_is_cancelled_by_search_press_and_escape",
    "history_pending_title_is_cancelled_when_the_app_leaves_sessions",
    "a_column_title_double_click_renames_its_session",
    "history_grid_long_project_header_keeps_actions_inside_its_group",
    "the_session_grid_keeps_every_card_inside_its_view",
    "workspace_project_selection_leaves_history_and_new_session_uses_selected_project",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
APP = ROOT / "src/app.rs"
SCREEN = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "home-status-leaves-filter-history-hidden": (
        APP,
        "self.page = Page::Sessions;\n        self.session_library_open = true;\n        self.session_view = SessionView::List;",
        "self.page = Page::Sessions;\n        self.session_library_open = false;\n        self.session_view = SessionView::List;"),
    "history-appends-a-second-session-list": (
        SCREEN, "self.ui_sessions(&mut body);",
        "self.ui_sessions(&mut body); self.ui_terminal_session_tabs(&mut body);"),
    "title-click-opens-before-rename-window": (
        SCREEN, "ui.input(|input| input.time) + delay", "ui.input(|input| input.time)"),
    "sidebar-project-name-is-not-truncated": (
        SCREEN,
        ".min_size(egui::vec2(0.0, CONTROL_HEIGHT))\n                                .truncate(),",
        ".min_size(egui::vec2(0.0, CONTROL_HEIGHT)),"),
    "grid-project-name-claims-action-space": (
        SCREEN, "ui.set_max_width(name_width);", ""),
    "pending-title-survives-another-pointer-or-escape": (
        SCREEN,
        "|| ui.input(|input| input.pointer.any_pressed() || input.key_pressed(egui::Key::Escape))",
        "|| false"),
    "pending-title-survives-leaving-the-session-page": (
        APP,
        "if self.page != Page::Sessions {\n            self.pending_history_open = None;\n        }",
        "if false && self.page != Page::Sessions {\n            self.pending_history_open = None;\n        }"),
    "sidebar-launch-uses-old-session-project": (
        SCREEN,
        "if let Some(project) = self.selected_project {\n                self.open_project_session_setup(project);\n            } else {\n                self.open_new_session();\n            }",
        "self.open_new_session();"),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
