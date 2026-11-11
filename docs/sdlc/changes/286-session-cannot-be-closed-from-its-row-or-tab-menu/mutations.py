#!/usr/bin/env python3
"""Break 「閉じる」 in the row's and the tab's menu (change 286)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "session_286_"],
]
APP = ROOT / "src/app.rs"
ORDER = ROOT / "src/app/session_order.rs"
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "row-menu-does-not-close": (SCREENS, "                                        close_row = true;", "                                        let _ = close_row;"),
    "tab-menu-does-not-close": (SCREENS, "                                                            close_from_menu = true;", "                                                            let _ = close_from_menu;"),
    "pinned-row-offers-close": (SCREENS, "                                    if !pinned && quiet_button(ui, palette, tr(\"閉じる\")).clicked()\n", "                                    if quiet_button(ui, palette, tr(\"閉じる\")).clicked()\n"),
    "pinned-tab-offers-close": (SCREENS, "                                                        if !pinned && quiet_button(ui, palette, tr(\"閉じる\")).clicked() {", "                                                        if quiet_button(ui, palette, tr(\"閉じる\")).clicked() {"),
    "menu-says-something-else": (SCREENS, "                                    if !pinned && quiet_button(ui, palette, tr(\"閉じる\")).clicked()\n", "                                    if !pinned && quiet_button(ui, palette, tr(\"タブを閉じる\")).clicked()\n"),
    "other-project-tab-stays-open": (ORDER, "            closed |= tabs.close(&tab);", "            let _ = &tabs;"),
    "other-project-pinned-tab-closes": (ORDER, "        if self.is_session_pinned(id) {\n            return;\n        }\n        let mut closed = false;", "        let mut closed = false;"),
    "closed-tab-is-not-remembered": (APP, "        self.closed_tabs.push(tab.clone());\n        let excess", "        let excess"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
