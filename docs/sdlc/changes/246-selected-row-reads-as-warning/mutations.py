#!/usr/bin/env python3
"""Change 246's mutation sweep: the selected row wears the selection colour,
its words take the two selected inks, and nothing that is not a selection
wears that colour. Also the change's `empirical-mutation` obligation, so CI
runs it again — ROOT is found from this file.
"""
import sys
from pathlib import Path

# docs/sdlc/changes/<this>/mutations.py -> the checkout.
ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

CHECKS = [[
    "cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--",
    "selected_row_takes_the_selection_colour",
    "selected_row_inks_stay_readable",
    "only_selections_wear_the_selection_colour",
    "every_theme_stays_readable",
]]

THEME = ROOT / "src/theme.rs"
KNOWLEDGE = ROOT / "src/app/knowledge.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    # R1: the value itself, back to the warm tint.
    "selected-row-back-to-the-warm-tint": (
        THEME,
        "row_selected: Color32::from_rgb(0, 92, 128),",
        "row_selected: Color32::from_rgb(54, 39, 29),",
    ),
    # R2: the secondary ink back to a muted one, which measures 2.80:1 there.
    "selected-secondary-ink-back-to-muted": (
        THEME,
        "fn selected_secondary_ink(&self) -> Color32 {\n        self.text\n    }",
        "fn selected_secondary_ink(&self) -> Color32 {\n        self.text_muted\n    }",
    ),
    # R4: the map hover borrows the selection colour again.
    "map-hover-borrows-the-selection-colour": (
        KNOWLEDGE,
        "ui.painter().rect_filled(rect, 4.0, palette.card);",
        "ui.painter().rect_filled(rect, 4.0, palette.row_selected);",
    ),
    # The claim, not a counter: a warning wearing the selection colour.
    "attention-banner-wears-the-selection-colour": (
        SCREENS,
        ".fill(palette.card)\n                .stroke(egui::Stroke::new(1.0, palette.warning))",
        ".fill(palette.row_selected)\n                .stroke(egui::Stroke::new(1.0, palette.warning))",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
