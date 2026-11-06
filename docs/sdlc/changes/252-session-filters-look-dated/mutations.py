#!/usr/bin/env python3
"""Re-run the sidebar guards after breaking each claim change 252 makes."""
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
os.environ["CARGO_INCREMENTAL"] = "0"
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "sidebar_25"]]
SCREENS = ROOT / "src/app/screens.rs"
APP = ROOT / "src/app.rs"
MUTATIONS = {
    # R1: four tiles, two per row at the default width.
    "tiles-never-pair": (SCREENS, "let columns = if ui.available_width() >= 2.0 * STATUS_TILE_MIN_W + gap {", "let columns = if false {"),
    # R1: clicking a tile is a filter.
    "tile-click-ignored": (SCREENS, "        if let Some(group) = toggled {\n            self.toggle_status_filter(group);\n        }\n", ""),
    # R2: counts follow the CLI choice.
    "counts-ignore-cli": (APP, "self.status_group_counts_where(|session| self.session_matches_agent_filter(session))", "self.status_group_counts_where(|_| true)"),
    # R2: 「クリア」 and the count only while filtering.
    "summary-always-shown": (SCREENS, "let filtering = self.has_session_filters();\n        let mut summarised = false;", "let filtering = true;\n        let mut summarised = false;"),
    # R2: 「クリア」 clears both dimensions.
    "clear-keeps-cli": (APP, "        self.clear_agent_filter();\n        self.clear_status_filter();\n", "        self.clear_status_filter();\n"),
    # R3: the sentence a person reads promises the waiting count.
    "waiting-count-is-a-lie": (SCREENS, "count = self.waiting_session_count()", "count = 0"),
    # R4: tiles stack where two do not fit.
    "tiles-pair-in-narrow-column": (ROOT / "src/config.rs", "STATUS_TILE_MIN_W: f32 = 104.0;", "STATUS_TILE_MIN_W: f32 = 60.0;"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
