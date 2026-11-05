#!/usr/bin/env python3
"""Change 245 mutation sweep: break each claim and watch its guard go red."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "session_column_project_is_a_heading_not_a_card",
    "session_column_project_heading_keeps_its_operations",
    "session_crumb_names_project_then_title",
    "terminal_pane_draws_the_crumb_once",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

SCREEN = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "column project becomes a card again": (
        SCREEN,
        "let card = egui::Frame::new()",
        "let card = card_frame(palette)",
    ),
    "heading loses its hairline": (
        SCREEN,
        "hairline(ui, palette);\n                            if folded {",
        "if folded {",
    ),
    "crumb loses the project": (
        SCREEN,
        'format!("{project_name} / {title}")',
        'format!("{title}")',
    ),
    "pane stops drawing the crumb": (
        SCREEN,
        "RichText::new(session_crumb(&project_name, &title))",
        "RichText::new(title.clone())",
    ),
    "selected heading becomes the hero again": (
        SCREEN,
        ".strong()\n                                                                .color(palette.text_muted)\n                                                        } else {",
        ".strong()\n                                                                .color(palette.text_strong)\n                                                        } else {",
    ),
    "session title loses its strength": (
        SCREEN,
        "RichText::new(&title)\n                                                        .size(12.5)\n                                                        .strong()\n                                                        .color(palette.text),",
        "RichText::new(&title)\n                                                        .size(12.5)\n                                                        .color(palette.text),",
    ),
    "crumb becomes the hero": (
        SCREEN,
        "RichText::new(session_crumb(&project_name, &title))\n                .size(11.5)\n                .color(palette.text_muted),",
        "RichText::new(session_crumb(&project_name, &title))\n                .size(11.5)\n                .color(palette.text_strong),",
    ),
    "heading loses session start": (
        SCREEN,
        'ICON_ADD,\n                                            tr("セッションを開始"),\n                                        )\n                                        .clicked()\n                                        {\n                                            self.open_project_session_setup(project.id);',
        'ICON_ADD,\n                                            tr("セッションを開始"),\n                                        )\n                                        .clicked()\n                                        {\n                                            self.open_new_session();',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
