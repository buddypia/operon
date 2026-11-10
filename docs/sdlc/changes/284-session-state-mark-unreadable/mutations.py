#!/usr/bin/env python3
"""Break the state badge and the column's generated marks (change 284)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "session_28"], ["cargo", "test", "--locked", "workspace_project_"]]
GLYPHS = ROOT / "src/glyphs.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "badge-holds-no-glyph": (WIDGETS, "    kind.status_mark()\n        .paint_badge(painter, center, radius, surface);", "    let _ = kind;"),
    "badge-glyph-in-the-state-ink": (WIDGETS, "        .paint_badge(painter, center, radius, surface);", "        .paint_badge(painter, center, radius, palette.status(kind.tone()));"),
    "every-badge-one-glyph": (WIDGETS, "    kind.status_mark()\n        .paint_badge(", "    StatusMark::Pause\n        .paint_badge("),
    "badge-back-to-a-5px-dot": (GLYPHS, "pub(crate) const STATUS_DOT_RATIO: f32 = 0.64;", "pub(crate) const STATUS_DOT_RATIO: f32 = 0.36;"),
    "glyph-spills-past-the-badge": (GLYPHS, "pub(crate) const STATUS_BADGE_GLYPH_RATIO: f32 = 0.6;", "pub(crate) const STATUS_BADGE_GLYPH_RATIO: f32 = 1.4;"),
    "glyph-drops-its-enclosure-on-the-circled-mark": (GLYPHS, "        painter.circle_stroke(center, 8.0 * scale, egui::Stroke::new(1.5 * scale, ink));\n", ""),
    "count-draws-no-mark": (WIDGETS, "        group.status_mark().paint(ui.painter(), mark, ink);", "        let _ = mark;"),
    "count-mark-loses-its-tone": (WIDGETS, "        group.status_mark().paint(ui.painter(), mark, ink);", "        group.status_mark().paint(ui.painter(), mark, palette.text_faint);"),
    "count-cannot-be-clicked": (WIDGETS, "allocate_exact_size(size, egui::Sense::click())", "allocate_exact_size(size, egui::Sense::hover())"),
    "markless-row-keeps-the-font-glyph": (SCREENS, "                                                session_status_icon(ui, palette, kind);", "                                                ui.label(RichText::new(kind.icon()).color(palette.status(kind.tone())));"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
