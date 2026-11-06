#!/usr/bin/env python3
"""Change 248: remove each leading state cue and falsify the visible words."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = ["session_tab_state_icons_are_recognizable", "session_tabs_show_live_state_before_title", "workspace_session_tab_shows_its_status_glyph"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
SCREEN = ROOT / "src/app/screens.rs"
GLYPHS = ROOT / "src/glyphs.rs"
MUTATIONS = {
    "idle goes back to an unexplained empty ring": (GLYPHS, "ICON_STATUS_IDLE: &str = icons::PAUSE_CIRCLE;", "ICON_STATUS_IDLE: &str = icons::CIRCLE;"),
    "running loses the recognizable circular arrows": (GLYPHS, "ICON_STATUS_RUNNING: &str = icons::ARROWS_CLOCKWISE;", "ICON_STATUS_RUNNING: &str = icons::CIRCLE_NOTCH;"),
    "tab state words disappear": (SCREEN, "RichText::new(tr(kind.label()))", 'RichText::new("")'),
    "busy tabs falsely claim to be idle": (SCREEN, "RichText::new(tr(kind.label()))", 'RichText::new(tr("指示待ち"))'),
    "tab hover loses the precise state explanation": (SCREEN, "                                                    ui.label(&view.hint);", '                                                    ui.label("");'),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
