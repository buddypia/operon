#!/usr/bin/env python3
"""Break the snapshot the window publishes and the wiring around it (change 309)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 300
CHECKS = [["cargo", "test", "--locked", "agents_309_"]]
C = ROOT / "src/tmux/control.rs"
S = ROOT / "src/app/agent_snapshot.rs"
APP = ROOT / "src/app.rs"
MAIN = ROOT / "src/main.rs"
MUTATIONS = {
    "removal-not-last": (S, "        let _ = self.sender.send(AgentSnapshotWrite::Remove);\n", ""),
    "remove-does-nothing": (S, "                let _ = fs::remove_file(path);\n", ""),
    "heartbeat-never": (S, "                    snapshot.written_at = clock();\n", ""),
    "heartbeat-resurrects-after-removal": (S, "                last = None;\n", ""),
    "unchanged-rewritten-every-check": (S, "if self.agent_snapshot_written.as_deref() == Some(content.as_str()) {", "if false {"),
    "changes-never-written": (S, "if self.agent_snapshot_written.as_deref() == Some(content.as_str()) {", "if self.agent_snapshot_written.is_some() {"),
    "checked-every-frame": (S, "now.duration_since(at) < Duration::from_secs(AGENTS_SNAPSHOT_CHECK_SECONDS)", "false"),
    "finished-sessions-published": (S, "let state = agent_state_token(self.session_status_kind_for(session))?;", 'let state = agent_state_token(self.session_status_kind_for(session)).unwrap_or("done");'),
    "logic-does-not-publish": (APP, "        self.track_agent_snapshot(Instant::now());\n", ""),
    "exit-leaves-the-file": (APP, "        self.finish_agent_snapshot();\n", ""),
    "main-does-not-dispatch": (MAIN, "        std::process::exit(run_agents_cli(&arguments[1..]));\n", ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
