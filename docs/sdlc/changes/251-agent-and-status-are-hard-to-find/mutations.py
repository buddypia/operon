#!/usr/bin/env python3
"""Re-run the guards after breaking each sidebar claim."""
import os
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
os.environ["CARGO_INCREMENTAL"] = "0"
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "sidebar_251_"]]
MUTATIONS = {
    "sidebar-controls-not-wired": (ROOT / "src/app/screens.rs", "self.ui_sidebar_session_filters(ui);", ""),
    "cli-and-status-become-a-union": (ROOT / "src/app.rs", "self.session_matches_agent_filter(session) && self.session_matches_status_filter(session)", "self.session_matches_agent_filter(session) || self.session_matches_status_filter(session)"),
    "ended-rows-still-hidden": (ROOT / "src/app/screens.rs", "filtering || group != SessionStatusGroup::Finished", "group != SessionStatusGroup::Finished"),
    "waiting-label-hides-the-promise": (ROOT / "src/app/screens.rs", 'tr("承認・回答待ち"),', 'tr("要対応"),'),
    "vendor-mark-removed-from-sidebar-row": (ROOT / "src/app/screens.rs", "agent_chip(ui, &session.agent, 12.0, palette);", "ui.add_space(16.0);"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
