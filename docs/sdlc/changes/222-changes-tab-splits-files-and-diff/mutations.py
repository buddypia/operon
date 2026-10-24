#!/usr/bin/env python3
"""The mutation sweep for change 222."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "the_changes_sub_tab_draws_the_file_list_and_commit_box_beside_the_diff_pane",
    "clicking_a_changed_file_filters_the_diff_without_leaving_the_changes_sub_tab",
]
CHECKS = [
    ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", t]
    for t in TESTS
]

MUTATIONS = {
    "changes-tab-omits-right-diff-pane": (
        ROOT / "src/app/screens.rs",
        "                            ui.set_width(right_width);\n                            self.ui_git_diff_view(ui, project, palette);",
        "                            ui.set_width(right_width);",
    ),
    "clicking-file-switches-to-diff-tab": (
        ROOT / "src/app/screens.rs",
        "                                    if let Some(path) = chosen {\n                                        self.git_selected_file = Some(path);\n                                    }",
        "                                    if let Some(path) = chosen {\n                                        self.git_selected_file = Some(path);\n                                        self.git_view = GitView::Diff;\n                                    }",
    ),
    "refresh-on-changes-omits-diff-refresh": (
        ROOT / "src/app/screens.rs",
        "                    GitView::Changes => {\n                        self.request_git_changes(project);\n                        self.request_git_diff(project, self.git_selected_file.clone());\n                    }",
        "                    GitView::Changes => self.request_git_changes(project),",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
