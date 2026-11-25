#!/usr/bin/env python3
"""Break the page title, the column heading, and the session rows' room (change 320)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
TESTS = ["heading_320_", "column_320_", "sidebar_302_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
SCREENS = ROOT / "src/app/screens.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"
CONFIG = ROOT / "src/config.rs"

MUTATIONS = {
    # R1: the title falls back to the heading level.
    "page-title-at-heading-level": (
        WIDGETS,
        "                RichText::new(title)\n                    .size(24.0)",
        "                RichText::new(title)\n                    .size(20.0)",
    ),
    # R1: the title hugs the toolbar again.
    "page-title-without-room-above": (
        WIDGETS,
        "            ui.add_space(SPACE_SM);\n            ui.label(\n                RichText::new(title)",
        "            ui.add_space(2.0);\n            ui.label(\n                RichText::new(title)",
    ),
    # R2: the column heading back at the names' size.
    "column-heading-at-name-size": (
        SCREENS,
        "                RichText::new(tr(\"プロジェクト\"))\n                    .size(15.0)",
        "                RichText::new(tr(\"プロジェクト\"))\n                    .size(13.5)",
    ),
    # R2: the column heading muted again.
    "column-heading-muted": (
        SCREENS,
        "                RichText::new(tr(\"プロジェクト\"))\n                    .size(15.0)\n                    .strong()\n                    .color(palette.text_strong),",
        "                RichText::new(tr(\"プロジェクト\"))\n                    .size(15.0)\n                    .strong()\n                    .color(palette.text_muted),",
    ),
    # R3, R5: the rows packed as before.
    "rows-packed": (
        CONFIG,
        "pub(crate) const SIDEBAR_ROW_PAD_Y: f32 = 6.0;",
        "pub(crate) const SIDEBAR_ROW_PAD_Y: f32 = 2.0;",
    ),
    # R3: the padding constant no longer reaches the row.
    "row-padding-ignored": (
        SCREENS,
        "                                        top: SIDEBAR_ROW_PAD_Y as i8,\n                                        bottom: SIDEBAR_ROW_PAD_Y as i8,",
        "                                        top: 2,\n                                        bottom: 2,",
    ),
    # R4: the rows start at the column's edge again.
    "rows-not-indented": (
        SCREENS,
        "                                        left: SIDEBAR_ROW_INDENT as i8,",
        "                                        left: SPACE_XS as i8,",
    ),
    # R4: an indent that misses the name.
    "indent-off-the-name": (
        CONFIG,
        "pub(crate) const SIDEBAR_ROW_INDENT: f32 = 42.0;",
        "pub(crate) const SIDEBAR_ROW_INDENT: f32 = 30.0;",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
