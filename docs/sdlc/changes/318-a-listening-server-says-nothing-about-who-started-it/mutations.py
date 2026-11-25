#!/usr/bin/env python3
"""Change 318's mutation sweep: who started each listening server, when, and
with what. The runner is scripts/mutation_sweep.py; its header holds the rules.
"""
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = ["ports_318_"]
# CI builds with CARGO_INCREMENTAL=0, which recompiles the whole crate for each
# mutation and runs past the obligation ceiling (changes 270, 317).
os.environ["CARGO_INCREMENTAL"] = "1"
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

PORTS = ROOT / "src/git/ports.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    # Requirement 1: details.
    "cut-last-line-kept": (PORTS, "        lines.pop();\n", "        let _ = &lines;\n"),
    "raw-command-kept-in-the-scan": (
        PORTS,
        ".map(|rest| redact_command_line(&rest.join(\" \")));",
        ".map(|rest| rest.join(\" \"));",
    ),
    "etime-day-with-24-hours-accepted": (PORTS, "hours >= 24)", "hours >= 25)"),
    # Requirement 2: the tag.
    "first-tag-token-wins": (PORTS, "            .next_back()\n", "            .next()\n"),
    "empty-tag-is-a-session": (PORTS, "            .filter(|value| !value.is_empty())\n", ""),
    "unread-pid-read-as-absent": (
        PORTS,
        "            session_tag: SessionTag::Unread,\n",
        "            session_tag: SessionTag::Absent,\n",
    ),
    "tags-never-merged": (PORTS, "tags.get(pid)", "tags.get(&0)"),
    # Requirement 3: redaction, one rule each.
    "flag-equals-value-shown": (
        PORTS,
        "Some((name, _)) if names_a_secret(name) =>",
        "Some((name, _)) if false && names_a_secret(name) =>",
    ),
    "flag-then-value-shown": (
        PORTS,
        "None => (word.to_owned(), names_a_secret(flag)),",
        "None => (word.to_owned(), false),",
    ),
    "bearer-credential-shown": (
        PORTS,
        'if lower.ends_with("bearer") ||',
        'if false && lower.ends_with("bearer") ||',
    ),
    "name-equals-value-shown": (
        PORTS,
        '&& !value.starts_with("//") && names_a_secret(name) {',
        '&& !value.starts_with("//") && false {',
    ),
    "url-user-shown": (PORTS, "rfind('@')", "rfind('\\u{0}')"),
    "token-prefix-shown": (PORTS, '    "sk-",\n', '    "sk-zzz",\n'),
    # Requirement 4: origin and leftover.
    "unread-tag-is-unknown": (
        PORTS,
        "SessionTag::Unread => PortOrigin::Unread,",
        "SessionTag::Unread => PortOrigin::Unknown,",
    ),
    "unknown-is-leftover": (
        PORTS,
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession\n",
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession | PortOrigin::Unknown\n",
    ),
    "unread-is-leftover": (
        PORTS,
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession\n",
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession | PortOrigin::Unread\n",
    ),
    "removed-session-not-leftover": (
        PORTS,
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession\n",
        "PortOrigin::Session { running: false, .. }\n",
    ),
    "exited-counts-as-running": (
        PORTS,
        "SessionStatus::Starting | SessionStatus::Active\n",
        "SessionStatus::Starting | SessionStatus::Active | SessionStatus::Exited\n",
    ),
    # Requirement 6: the badge counts the whole list.
    "badge-counts-own-folder-only": (PORTS, "            .chain(&self.elsewhere)\n", ""),
    # Requirements 5 and 7: the sentences a person reads.
    "parent-gone-said-of-every-parent": (
        SCREENS,
        "if process.parent_pid == Some(1) {",
        "if process.parent_pid.is_some() {",
    ),
    "untagged-server-called-forgotten": (
        SCREENS,
        'PortOrigin::RemovedSession => Some(tr("起動したセッションは削除されています")),',
        'PortOrigin::RemovedSession | PortOrigin::Unknown => Some(tr("起動したセッションは削除されています")),',
    ),
    "confirmation-parent-never-exited": (
        SCREENS,
        '            Some(1) => tf!("終了済み (PID {ppid})", ppid = 1),',
        '            Some(0) => tf!("終了済み (PID {ppid})", ppid = 1),',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
