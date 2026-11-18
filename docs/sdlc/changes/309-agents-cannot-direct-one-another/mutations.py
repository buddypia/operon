#!/usr/bin/env python3
"""Break what one agent may see of, and type into, another (change 309).

Nineteen of the thirty-seven mutations this change was swept with: all
thirty-seven were caught at 590e23a, and the CI obligation's 900 s ceiling
holds about this many rebuilds. The eighteen left out are listed in plan.md.
"""
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
    "send-ignores-state": (C, "        return busy(&entry.state);\n", ""),
    "not-started-refused": (C, "if entry.state != AGENT_STATE_IDLE && entry.state != AGENT_STATE_NOT_STARTED {", "if entry.state != AGENT_STATE_IDLE {"),
    "send-skips-screen-recheck-working": (C, "AgentActivity::Working => return busy(AGENT_STATE_WORKING),", "AgentActivity::Working => {}"),
    "control-text-allowed": (C, "    if has_control {", "    if false {"),
    "control-judged-after-trim": (C, "let has_control = joined.chars().any(char::is_control);", "let has_control = joined.trim().chars().any(char::is_control);"),
    "self-send-allowed": (C, "if caller.tmux_name.as_deref() == Some(entry.tmux_name.as_str()) {", "if false {"),
    "other-projects-are-targets": (C, "        .filter(|entry| entry.project_id == project)\n        .filter(|entry| entry.id.simple()", "        .filter(|entry| entry.id.simple()"),
    "other-projects-listed": (C, "        .filter(|entry| entry.project_id == project)\n        .collect();\n    if json {", "        .collect();\n    if json {"),
    "unsafe-tmux-name-admitted": (C, "[entry] if is_safe_tmux_name(&entry.tmux_name) => Ok(entry),", "[entry] => Ok(entry),"),
    "stale-accepted": (C, "<= AGENTS_SNAPSHOT_STALE_SECONDS =>", "<= AGENTS_SNAPSHOT_STALE_SECONDS * 10 =>"),
    "caller-session-ignored": (C, "    if let Some(entry) = caller.tmux_name.as_ref().and_then(|name| {", "    if let Some(entry) = None::<String>.as_ref().and_then(|name: &String| {"),
    "read-keeps-controls": (C, ".filter(|character| !character.is_control() || matches!(character, '\\n' | '\\t'))", ".filter(|_| true)"),
    "wait-either-ignores-prompt": (C, "WaitFor::Either => state == AGENT_STATE_IDLE || state == AGENT_STATE_WAITING,", "WaitFor::Either => state == AGENT_STATE_IDLE,"),
    "wait-returns-before-the-turn-starts": (C, "} else if left || host.now().saturating_sub(started) >= AGENTS_WAIT_SETTLE_SECONDS {", "} else {"),
    "removal-not-last": (S, "        let _ = self.sender.send(AgentSnapshotWrite::Remove);\n", ""),
    "heartbeat-never": (S, "                    snapshot.written_at = clock();\n", ""),
    "heartbeat-resurrects-after-removal": (S, "                last = None;\n", ""),
    "unchanged-rewritten-every-check": (S, "if self.agent_snapshot_written.as_deref() == Some(content.as_str()) {", "if false {"),
    "exit-leaves-the-file": (APP, "        self.finish_agent_snapshot();\n", ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
