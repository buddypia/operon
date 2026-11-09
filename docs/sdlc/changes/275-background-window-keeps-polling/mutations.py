#!/usr/bin/env python3
"""Break the focus-dependent session reading of change 275."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "poll_275_"]]
APP = ROOT / "src/app.rs"
UTIL = ROOT / "src/util.rs"
CONFIG = ROOT / "src/config.rs"
POLL_LINE = "        self.poll_sessions_when_due(ctx.input(|input| input.focused), Instant::now());\n"
MUTATIONS = {
    "behind-reads-at-the-front-rate": (UTIL, "        crate::SESSION_POLL_INTERVAL_UNFOCUSED\n", "        crate::SESSION_POLL_INTERVAL\n"),
    "front-waits-the-long-interval": (UTIL, "        crate::SESSION_POLL_INTERVAL\n    } else {", "        crate::SESSION_POLL_INTERVAL_UNFOCUSED\n    } else {"),
    "claim-a-third-as-often-is-not-kept": (CONFIG, "    std::time::Duration::from_secs(6);", "    std::time::Duration::from_secs(3);"),
    "frame-ignores-the-focus": (APP, POLL_LINE, POLL_LINE.replace("ctx.input(|input| input.focused)", "true")),
    "frame-always-behind": (APP, POLL_LINE, POLL_LINE.replace("ctx.input(|input| input.focused)", "false")),
    "frame-never-reads": (APP, POLL_LINE, ""),
    "reading-is-not-stamped": (APP, "        self.last_session_poll = now;\n", ""),
    "reads-before-the-interval": (APP, "self.last_session_poll) < session_poll_interval(focused)", "self.last_session_poll) < Duration::ZERO"),
    "store-retry-slows-behind": (APP, "if self.last_session_refresh.elapsed() >= SESSION_POLL_INTERVAL {", "if self.last_session_refresh.elapsed() >= session_poll_interval(ctx.input(|input| input.focused)) {"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
