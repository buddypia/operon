#!/usr/bin/env python3
"""Falsify the icon-only tab and its live state explanation."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", "session_tab_status_badges_are_distinct_vectors", "session_tabs_show_live_state_before_title", "workspace_session_tab_shows_its_status_glyph"]]
GLYPHS = ROOT / "src/glyphs.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"
SCREEN = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "status enclosure disappears": (GLYPHS, "painter.circle_stroke(center, 8.0 * scale, stroke);", "// removed enclosure"),
    "idle falsely uses the running mark": (WIDGETS, "SessionStatusKind::Idle => StatusMark::Pause,", "SessionStatusKind::Idle => StatusMark::Progress,"),
    "permanent tab words return": (SCREEN, "let kind = self.session_status_kind_for(session);", "let kind = self.session_status_kind_for(session); ui.label(tr(kind.label()));"),
    "tab paints stale running state": (SCREEN, "session_status_icon(ui, palette, kind)", "session_status_icon(ui, palette, SessionStatusKind::Running)"),
    "hover falsely claims idle": (SCREEN, "ui.label(&view.label);", 'ui.label(tr("指示待ち"));'),
    "precise explanation disappears": (SCREEN, "ui.label(&view.hint);", '                                                    ui.label("");'),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
