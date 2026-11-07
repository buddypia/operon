#!/usr/bin/env python3
"""Break the naming promises and each direct rename entry point."""
import os
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
os.environ["CARGO_INCREMENTAL"] = "0"
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "session_253_"]]
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "new-name-repeats-project": (APP, "            self.next_session_name(project.id)", "            workspace_display_name(&worktree_path)"),
    "startup-keeps-old-names": (APP, "        app.number_legacy_session_names();", ""),
    "numbered-label-promises-wrong-number": (APP, 'tf!("セッション{p0}", p0 = number)', 'tf!("セッション{p0}", p0 = number + 1)'),
    "sidebar-double-click-disabled": (SCREENS, "let mut rename = row.double_clicked();", "let mut rename = false;"),
    "tab-double-click-disabled": (SCREENS, "let mut rename = label.double_clicked();", "let mut rename = false;"),
    "sidebar-right-click-disabled": (SCREENS, 'row.context_menu(|ui| {\n                                    if quiet_button', 'row.context_menu(|ui| {\n                                    if false && quiet_button'),
    "tab-right-click-disabled": (SCREENS, 'label.context_menu(|ui| {\n                                                if quiet_button', 'label.context_menu(|ui| {\n                                                if false && quiet_button'),
    "escape-saves-instead": (SCREENS, '        Some(false)\n    } else if editor.lost_focus()', '        Some(true)\n    } else if editor.lost_focus()'),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
