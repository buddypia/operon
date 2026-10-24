#!/usr/bin/env python3
"""The mutation sweep for change 223."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "the_command_palette_sits_centered_near_the_top_of_the_window",
]
CHECKS = [
    ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", t]
    for t in TESTS
]

MUTATIONS = {
    "palette-omits-top-center-anchor": (
        ROOT / "src/app/screens.rs",
        "            .resizable(false)\n            .anchor(egui::Align2::CENTER_TOP, [0.0, 68.0])\n            .default_width(520.0)",
        "            .resizable(false)\n            .default_width(520.0)",
    ),
    "palette-reverts-to-420-width": (
        ROOT / "src/app/screens.rs",
        "            .anchor(egui::Align2::CENTER_TOP, [0.0, 68.0])\n            .default_width(520.0)",
        "            .anchor(egui::Align2::CENTER_TOP, [0.0, 68.0])\n            .default_width(420.0)",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
