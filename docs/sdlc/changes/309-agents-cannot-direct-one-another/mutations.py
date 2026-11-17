#!/usr/bin/env python3
"""Break who a send may reach and what it may carry (change 309)."""
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
    "not-started-published-as-idle": (C, "SessionStatusKind::NotStarted => AGENT_STATE_NOT_STARTED,", "SessionStatusKind::NotStarted => AGENT_STATE_IDLE,"),
    "send-skips-screen-recheck-working": (C, "AgentActivity::Working => return busy(AGENT_STATE_WORKING),", "AgentActivity::Working => {}"),
    "send-skips-screen-recheck-prompt": (C, "AgentActivity::AwaitingInput => return busy(AGENT_STATE_WAITING),", "AgentActivity::AwaitingInput => {}"),
    "screen-failure-still-sends": (C, "        Err(error) => return terminal_failure(&error),\n    }\n    match host.send", "        Err(_) => {}\n    }\n    match host.send"),
    "control-text-allowed": (C, "    if has_control {", "    if false {"),
    "control-judged-after-trim": (C, "let has_control = joined.chars().any(char::is_control);", "let has_control = joined.trim().chars().any(char::is_control);"),
    "long-text-allowed": (C, "if text.chars().count() > AGENTS_SEND_MAX_CHARS {", "if text.chars().count() > AGENTS_SEND_MAX_CHARS * 2 {"),
    "self-send-allowed": (C, "if caller.tmux_name.as_deref() == Some(entry.tmux_name.as_str()) {", "if false {"),
    "other-projects-are-targets": (C, "        .filter(|entry| entry.project_id == project)\n        .filter(|entry| entry.id.simple()", "        .filter(|entry| entry.id.simple()"),
    "other-projects-listed": (C, "        .filter(|entry| entry.project_id == project)\n        .collect();\n    if json {", "        .collect();\n    if json {"),
    "unsafe-tmux-name-admitted": (C, "[entry] if is_safe_tmux_name(&entry.tmux_name) => Ok(entry),", "[entry] => Ok(entry),"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
