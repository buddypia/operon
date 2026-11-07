#!/usr/bin/env python3
"""Break the visible selection, density, action and pane-context promises."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "session_254_"]]
SCREENS = ROOT / "src/app/screens.rs"
APP = ROOT / "src/app.rs"
MUTATIONS = {
    "selected-row-disappears": (SCREENS, "palette.row_selected\n                                    } else", "palette.raised\n                                    } else"),
    "sidebar-rule-disappears": (SCREENS, "let x = row.rect.left() + 1.0;", "let x = row.rect.right() + 1.0;"),
    "selected-tab-disappears": (SCREENS, "if active { palette.row_selected }", "if active { palette.raised }"),
    "tab-text-too-large": (SCREENS, "RichText::new(&title).size(12.0)", "RichText::new(&title).size(15.0)"),
    "short-tab-wastes-width": (SCREENS, ".clamp(40.0, 132.0)", ".clamp(160.0, 160.0)"),
    "plus-does-nothing": (SCREENS, "self.launch_workspace_terminal(Some(*group));", "let _ = group;"),
    "plus-promises-wrong-action": (SCREENS, 'tr("空のターミナルを開く")', 'tr("名前を変更")'),
    "plus-ignores-pane-worktree": (APP, "self.launch_empty_session_at(project, Some(project_id), &path);", "let _ = path; self.launch_empty_session_at(project, Some(project_id), \"\");"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
