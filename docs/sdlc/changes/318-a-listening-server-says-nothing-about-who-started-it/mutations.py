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
SCREENS = ROOT / "src/app/ports_view.rs"

MUTATIONS = {
    # Requirement 1: details.
    "cut-last-record-kept": (PORTS, "        records.pop();\n", "        let _ = &records;\n"),
    "raw-command-kept-in-the-scan": (
        PORTS,
        ".map(|rest| redact_command_line(&rest.join(\" \")));",
        ".map(|rest| rest.join(\" \"));",
    ),
    "etime-day-with-24-hours-accepted": (PORTS, "hours >= 24)", "hours >= 25)"),
    # Requirement 2: the tag.
    "first-tag-token-wins": (PORTS, "            .next_back()\n", "            .next()\n"),
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
        "None => (word.to_owned(), names_a_secret(body)),",
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
    "unknown-is-leftover": (
        PORTS,
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession\n",
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession | PortOrigin::Unknown\n",
    ),
    "removed-session-not-leftover": (
        PORTS,
        "PortOrigin::Session { running: false, .. } | PortOrigin::RemovedSession\n",
        "PortOrigin::Session { running: false, .. }\n",
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
    # Review round: forged and spanning records, the session-name shape.
    "duplicate-pid-believed": (
        PORTS,
        ".filter(|(pid, _)| opened.get(pid) == Some(&1))",
        ".filter(|(pid, _)| opened.get(pid).is_some())",
    ),
    "continuation-line-dropped": (PORTS, "text.push_str(line);", "text.push_str(\"\");"),
    "any-value-taken-for-a-session": (
        PORTS,
        ".filter(|value| is_safe_tmux_name(value))",
        ".filter(|value| !value.is_empty())",
    ),
    # Review round: redaction gaps the reviewers reproduced.
    "hidden-word-not-read": (
        PORTS,
        "        hide_next = hides_next;\n",
        "        hide_next = hides_next && !hide_next;\n",
    ),
    "header-value-in-next-word-shown": (
        PORTS,
        "if word.strip_suffix(':').is_some_and(names_a_secret) {",
        "if false {",
    ),
    "nested-flag-value-shown": (
        PORTS,
        "let (value, hides_next) = redact_word(value);",
        "let (value, hides_next) = (value.to_owned(), false);",
    ),
    "query-secret-shown": (
        PORTS,
        "Some((name, value)) if names_a_secret(name) || looks_like_a_secret(value) =>",
        "Some((name, value)) if false && (names_a_secret(name) || looks_like_a_secret(value)) =>",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
