#!/usr/bin/env python3
"""Break the corner notice/warning overlay on Page::Sessions (change 289)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "session_notice_289_"]]
APP = ROOT / "src/app.rs"

CORNER_CLOSE = (
    "                                    if small_icon_button(\n"
    "                                        ui,\n"
    "                                        palette,\n"
    "                                        ICON_CLOSE,\n"
    '                                        tr("この通知を閉じる"),\n'
    "                                    )"
)
CORNER_CLOSE_WRONG_TOOLTIP = CORNER_CLOSE.replace(
    'tr("この通知を閉じる")', '""'
)
DISMISS_BLOCK = (
    "        if dismissed {\n"
    "            self.notice = None;\n"
    "            self.brief_notice = None;\n"
    "        }"
)

MUTATIONS = {
    "sessions-still-draws-top-banner": (
        APP,
        "                        if self.page != Page::Sessions {",
        "                        if true {",
    ),
    "persistent-notice-omitted-from-corner-on-sessions": (
        APP,
        "        let show_notice = self.notice.is_some() && (brief || self.page == Page::Sessions);",
        "        let show_notice = self.notice.is_some() && brief;",
    ),
    "durability-warning-omitted-from-corner-on-sessions": (
        APP,
        "        let show_warning = self.page == Page::Sessions && self.durability_warning.is_some();",
        "        let show_warning = false;",
    ),
    "other-pages-duplicate-corner-overlay": (
        APP,
        "        let show_notice = self.notice.is_some() && (brief || self.page == Page::Sessions);",
        "        let show_notice = self.notice.is_some();",
    ),
    "overlay-anchored-at-top-instead-of-bottom-right": (
        APP,
        "            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-SPACE_LG, -SPACE_LG))",
        "            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-SPACE_LG, SPACE_LG))",
    ),
    "persistent-notice-wears-wrong-mark": (
        APP,
        "                            (ICON_NOTICE, palette.info, palette.info)",
        "                            (ICON_STATUS_DONE, palette.info, palette.info)",
    ),
    "durability-warning-wears-wrong-mark": (
        APP,
        "                                        RichText::new(ICON_ATTENTION)",
        "                                        RichText::new(ICON_NOTICE)",
    ),
    "corner-close-button-says-wrong-tooltip": (
        APP,
        CORNER_CLOSE,
        CORNER_CLOSE_WRONG_TOOLTIP,
    ),
    "corner-close-button-does-not-dismiss": (
        APP,
        DISMISS_BLOCK,
        "        if dismissed {\n            self.brief_notice = None;\n        }",
    ),
    "long-notice-does-not-wrap-inside-420px": (
        APP,
        "                                        ui.set_max_width(notice_text_w);",
        "                                        ui.set_max_width(1200.0);",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
