#!/usr/bin/env python3
"""Break the Dock badge's count and when it is set (change 281)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "dock_281_"]]
BADGE = ROOT / "src/app/dock_badge.rs"
APP = ROOT / "src/app.rs"
MUTATIONS = {
    "zero-wears-a-badge": (BADGE, "(waiting > 0).then(|| waiting.to_string())", "Some(waiting.to_string())"),
    "one-wears-no-badge": (BADGE, "(waiting > 0).then(|| waiting.to_string())", "(waiting > 1).then(|| waiting.to_string())"),
    "dock-is-told-every-frame": (BADGE, "        if self.dock_badge_shown == waiting {\n            return None;\n        }\n", ""),
    "change-is-not-remembered": (BADGE, "        self.dock_badge_shown = waiting;\n", ""),
    "badge-counts-the-clocks-not-the-queue": (BADGE, "let waiting = self.attention_order.len();", "let waiting = self.working_since.len();"),
    "frame-does-not-keep-the-badge": (APP, "        self.track_dock_badge();\n", ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
