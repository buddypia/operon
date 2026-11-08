#!/usr/bin/env python3
"""Break the promise that the + sits where the next tab will appear."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "session_260_"],
    ["cargo", "test", "--locked", "session_254_terminal_action"],
]
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    # R1: the tab region fills the strip again, so the + is back across the pane.
    "plus-pinned-to-right-edge": (
        SCREENS,
        '.id_salt(("workspace-tabs", group))\n                        .auto_shrink([true, true])',
        '.id_salt(("workspace-tabs", group))\n                        .auto_shrink([false, true])',
    ),
    # R2: the region may grow over the + slot, so overflowing tabs push it out of the pane.
    "plus-scrolls-out-of-pane": (
        SCREENS,
        "let width = (ui.available_width() - CONTROL_HEIGHT - SPACE_SM).max(0.0);",
        "let width = ui.available_width().max(0.0);",
    ),
    # R3: the empty workspace's + goes back to the centred right edge.
    "empty-plus-centred-right": (
        SCREENS,
        "body.horizontal(|ui| {",
        "body.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {",
    ),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
