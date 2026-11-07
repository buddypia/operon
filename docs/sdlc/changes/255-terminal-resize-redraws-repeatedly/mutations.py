#!/usr/bin/env python3
"""Break the settle-before-send and layout-reuse promises of change 255."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "session_255_"],
    ["cargo", "test", "--locked", "terminal_resize_debounces_during_divider_drag"],
]
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
TERMINAL = ROOT / "src/ui/terminal.rs"
MUTATIONS = {
    "sends-unsettled-size": (APP, "if held < TERMINAL_RESIZE_SETTLE {", "if held < Duration::ZERO {"),
    "first-size-waits": (APP, "        if applied.is_some() {\n", "        if applied.is_some() || applied.is_none() {\n"),
    "new-size-keeps-old-wait": (APP, "            self.terminal_resize_changed_at\n                .insert(session_id, Instant::now());\n        }\n        self.flush_terminal_resize(session_id)", "            self.terminal_resize_changed_at\n                .entry(session_id)\n                .or_insert_with(Instant::now);\n        }\n        self.flush_terminal_resize(session_id)"),
    "wait-not-reported": (APP, "return Some(TERMINAL_RESIZE_SETTLE - held);", "return None;"),
    "drag-end-waits": (APP, "                    self.terminal_resize_changed_at.remove(&session_id);\n", ""),
    "frame-not-woken": (SCREENS, "terminal_grid_size(terminal_rect.size(), cell),\n                    ) {\n                        ui.ctx().request_repaint_after(wait);", "terminal_grid_size(terminal_rect.size(), cell),\n                    ) {\n                        let _ = wait;"),
    "size-dirties-rows": (TERMINAL, "visible_rows: (rows.start, rows.end),\n                output_generation", "visible_rows: (rows.start, rows.end + ui.available_width() as usize),\n                output_generation"),
    "caret-lost": (TERMINAL, "caret = computed_caret;", "caret = None;"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
