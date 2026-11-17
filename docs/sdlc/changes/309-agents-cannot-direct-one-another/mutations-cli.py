#!/usr/bin/env python3
"""Break how the command finds its caller, its target, and what it reads and waits for (change 309)."""
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
    "short-prefix-admitted": (C, "if wanted.chars().count() < AGENTS_ID_PREFIX_MIN {", "if wanted.is_empty() {"),
    "stale-accepted": (C, "<= AGENTS_SNAPSHOT_STALE_SECONDS =>", "<= AGENTS_SNAPSHOT_STALE_SECONDS * 10 =>"),
    "stale-edge-refused": (C, "<= AGENTS_SNAPSHOT_STALE_SECONDS =>", "< AGENTS_SNAPSHOT_STALE_SECONDS =>"),
    "caller-session-ignored": (C, "    if let Some(entry) = caller.tmux_name.as_ref().and_then(|name| {", "    if let Some(entry) = None::<String>.as_ref().and_then(|name: &String| {"),
    "read-ignores-its-count": (C, "let start = kept.len().saturating_sub(lines);", "let start = 0;"),
    "read-keeps-escapes": (C, "let text: String = strip_terminal_escapes(&text)", "let text: String = text.clone()"),
    "wait-either-ignores-prompt": (C, "WaitFor::Either => state == AGENT_STATE_IDLE || state == AGENT_STATE_WAITING,", "WaitFor::Either => state == AGENT_STATE_IDLE,"),
    "wait-never-ends-on-failure": (C, 'if matches!(current.state.as_str(), "failed" | "lost") {', "if false {"),
    "wait-ignores-timeout": (C, "if host.now() >= deadline {", "if host.now() >= deadline + 1 {"),
    "wait-returns-before-the-turn-starts": (C, "} else if left || host.now().saturating_sub(started) >= AGENTS_WAIT_SETTLE_SECONDS {", "} else {"),
    "wait-never-settles": (C, "} else if left || host.now().saturating_sub(started) >= AGENTS_WAIT_SETTLE_SECONDS {", "} else if left {"),
    "list-prints-names-raw": (C, "        without_controls(&entry.name),", "        entry.name.clone(),"),
    "read-keeps-controls": (C, ".filter(|character| !character.is_control() || matches!(character, '\\n' | '\\t'))", ".filter(|_| true)"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
