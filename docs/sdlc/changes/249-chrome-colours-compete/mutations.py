#!/usr/bin/env python3
"""Change 249: reject coloured chrome, lost accents and lost semantic colours."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--",
    "dark_chrome_uses_neutral_roles", "neutral_chrome_preserves_semantic_colours",
    "selected_navigation_uses_a_fine_accent_rule", "active_workspace_tab_has_an_orange_rule",
    "topbar_wordmark_is_neutral_beside_the_primary_action",
    "design_md_documents_exactly_what_the_app_paints"]]
MUTATIONS = {
    "blue-selection-returns": (ROOT / "src/theme.rs",
        "row_selected: Color32::from_rgb(56, 56, 56),", "row_selected: Color32::from_rgb(0, 92, 128),"),
    "orange-action-becomes-blue": (ROOT / "src/theme.rs",
        "accent: Color32::from_rgb(255, 148, 56),", "accent: Color32::from_rgb(120, 175, 255),"),
    "warning-loses-its-semantic-colour": (ROOT / "src/theme.rs",
        "warning: Color32::from_rgb(246, 187, 55),", "warning: Color32::from_rgb(187, 187, 187),"),
    "navigation-loses-selected-rule": (ROOT / "src/ui/widgets.rs",
        "egui::pos2(rect.left(), rect.bottom() - 1.0)", "egui::pos2(rect.left(), rect.top() + 1.0)"),
    "workspace-loses-selected-rule": (ROOT / "src/app/screens.rs",
        "let rect = tab_frame.response.rect;", "let rect = tab_frame.response.rect.translate(egui::vec2(0.0, -40.0));"),
    "wordmark-competes-with-primary-action": (ROOT / "src/app/screens.rs",
        'let wordmark = RichText::new("Operon")\n                        .size(14.0)\n                        .strong()\n                        .color(palette.text_strong);',
        'let wordmark = RichText::new("Operon")\n                        .size(14.0)\n                        .strong()\n                        .color(palette.accent_text);'),
    "document-claims-a-blue-selected-surface": (ROOT / "DESIGN.md",
        'dark-row-selected: "#383838"', 'dark-row-selected: "#005C80"'),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
