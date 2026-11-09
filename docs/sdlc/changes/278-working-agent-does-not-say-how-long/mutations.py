#!/usr/bin/env python3
"""Break the working clock of change 278 and each place that says it."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "working_278_"]]
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
TIME = ROOT / "src/app/working_time.rs"
WORKING = "if self.session_status_kind_for(session) == SessionStatusKind::Working {"
MUTATIONS = {
    "every-live-session-gets-a-clock": (TIME, WORKING, "if self.session_status_kind_for(session).label() == SessionStatusKind::Working.label() {"),
    "no-session-gets-a-clock": (TIME, WORKING, "if false {"),
    "each-frame-restarts-the-clock": (TIME, "since.entry(session.id).or_insert(now);", "since.insert(session.id, now);"),
    "ended-turn-keeps-its-clock": (TIME, "                since.remove(&session.id);\n", ""),
    "gone-session-keeps-its-clock": (TIME, "        if since.len() != seen {", "        if false {"),
    "time-is-said-under-a-minute": (TIME, "        0 => None,\n", ""),
    "an-hour-is-said-in-minutes": (TIME, "        1..=59 => Some(", "        1..=60 => Some("),
    "hours-are-said-as-minutes": (TIME, "tf!(\"{p0}時間\", p0 = minutes / 60)", "tf!(\"{p0}時間\", p0 = minutes)"),
    "frame-does-not-keep-the-clocks": (APP, "        self.track_working(Instant::now());\n", ""),
    "status-hover-drops-the-time": (APP, "        view.hint = self.with_working_time(&view.hint, session.id, Instant::now());\n", ""),
    "list-row-drops-the-time": (SCREENS, "                                let state = self.with_working_time(\n                                    &status.label,\n                                    candidate.id,\n                                    Instant::now(),\n                                );", "                                let state = status.label.clone();"),
    "pane-heading-drops-the-time": (SCREENS, "        status.label = self.with_working_time(&status.label, session_id, Instant::now());\n", ""),
    "tab-hover-drops-the-time": (SCREENS, "ui.label(self.with_working_time(\n                                                                    &view.hint,\n                                                                    *id,\n                                                                    Instant::now(),\n                                                                ));", "ui.label(&view.hint);"),
    "column-hover-drops-the-time": (SCREENS, "ui.label(self.with_working_time(\n                                            &view.hint,\n                                            id,\n                                            Instant::now(),\n                                        ));", "ui.label(&view.hint);"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
