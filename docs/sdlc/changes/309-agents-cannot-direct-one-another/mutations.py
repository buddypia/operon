#!/usr/bin/env python3
"""Break what one agent may see of, and type into, another (change 309)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 2400
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
    "short-prefix-admitted": (C, "if wanted.chars().count() < AGENTS_ID_PREFIX_MIN {", "if wanted.is_empty() {"),
    "stale-accepted": (C, "<= AGENTS_SNAPSHOT_STALE_SECONDS =>", "<= AGENTS_SNAPSHOT_STALE_SECONDS * 10 =>"),
    "stale-edge-refused": (C, "<= AGENTS_SNAPSHOT_STALE_SECONDS =>", "< AGENTS_SNAPSHOT_STALE_SECONDS =>"),
    "caller-session-ignored": (C, "    if let Some(entry) = caller.tmux_name.as_ref().and_then(|name| {", "    if let Some(entry) = None::<String>.as_ref().and_then(|name: &String| {"),
    "read-ignores-its-count": (C, "let start = kept.len().saturating_sub(lines);", "let start = 0;"),
    "read-keeps-escapes": (C, "let text: String = strip_terminal_escapes(&text)", "let text: String = text.clone()"),
    "wait-either-ignores-prompt": (C, "WaitFor::Either => state == AGENT_STATE_IDLE || state == AGENT_STATE_WAITING,", "WaitFor::Either => state == AGENT_STATE_IDLE,"),
    "wait-never-ends-on-failure": (C, 'if matches!(current.state.as_str(), "failed" | "lost") {', "if false {"),
    "wait-ignores-timeout": (C, "if host.now() >= deadline {", "if host.now() >= deadline + 1 {"),
    "removal-not-last": (S, "        let _ = self.sender.send(AgentSnapshotWrite::Remove);\n", ""),
    "remove-does-nothing": (S, "                let _ = fs::remove_file(path);\n", ""),
    "heartbeat-never": (S, "                    snapshot.written_at = clock();\n", ""),
    "heartbeat-resurrects-after-removal": (S, "                last = None;\n", ""),
    "unchanged-rewritten-every-check": (S, "if self.agent_snapshot_written.as_deref() == Some(content.as_str()) {", "if false {"),
    "changes-never-written": (S, "if self.agent_snapshot_written.as_deref() == Some(content.as_str()) {", "if self.agent_snapshot_written.is_some() {"),
    "checked-every-frame": (S, "now.duration_since(at) < Duration::from_secs(AGENTS_SNAPSHOT_CHECK_SECONDS)", "false"),
    "wait-returns-before-the-turn-starts": (C, "} else if left || host.now().saturating_sub(started) >= AGENTS_WAIT_SETTLE_SECONDS {", "} else {"),
    "wait-never-settles": (C, "} else if left || host.now().saturating_sub(started) >= AGENTS_WAIT_SETTLE_SECONDS {", "} else if left {"),
    "list-prints-names-raw": (C, "        without_controls(&entry.name),", "        entry.name.clone(),"),
    "read-keeps-controls": (C, ".filter(|character| !character.is_control() || matches!(character, '\\n' | '\\t'))", ".filter(|_| true)"),
    "finished-sessions-published": (S, "let state = agent_state_token(self.session_status_kind_for(session))?;", 'let state = agent_state_token(self.session_status_kind_for(session)).unwrap_or("done");'),
    "logic-does-not-publish": (APP, "        self.track_agent_snapshot(Instant::now());\n", ""),
    "exit-leaves-the-file": (APP, "        self.finish_agent_snapshot();\n", ""),
    "main-does-not-dispatch": (MAIN, "        std::process::exit(run_agents_cli(&arguments[1..]));\n", ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
