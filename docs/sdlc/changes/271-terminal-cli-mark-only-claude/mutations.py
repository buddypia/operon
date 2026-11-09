#!/usr/bin/env python3
"""Change 271's mutation sweep: a terminal takes the kind of the CLI seen in
its foreground, for Codex and Antigravity as for Claude. Runner and rules:
scripts/mutation_sweep.py. No sentence a person reads changed, so every
mutation breaks a claim of the reading or of the adoption.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "the_cli_in_a_terminals_foreground_is_read_from_its_processes",
    "a_terminal_takes_the_kind_of_the_cli_seen_running_in_it",
    "the_foreground_cli_of_a_live_pane_is_read_through_tmux_and_ps",
    "a_terminal_takes_the_kind_of_the_cli_running_in_it",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

TMUX = ROOT / "src" / "tmux.rs"
APP = ROOT / "src" / "app.rs"

MUTATIONS = {
    "a CLI in the background counts as the one in front": (
        TMUX,
        "(pgid == tpgid).then_some((pid, ppid, provider))",
        "Some((pid, ppid, provider))",
    ),
    "a CLI started by another CLI is taken for the person's": (
        TMUX,
        ".filter(|(pid, _, provider)| provider.is_some() && !named_parent(*pid))",
        ".filter(|(_, _, provider)| provider.is_some())",
    ),
    "two CLIs side by side are guessed between": (
        TMUX,
        "outermost.all(|other| other == first).then_some(first)",
        "Some(first)",
    ),
    "a name that only begins like a CLI's is taken for it": (
        TMUX,
        ".find(|provider| provider.executable() == executable);",
        ".find(|provider| executable.starts_with(provider.executable()));",
    ),
    "a device name that is not plain reaches ps": (
        TMUX,
        "(!device.is_empty() && device.chars().all(|c| c.is_ascii_alphanumeric())).then_some(device)",
        "(!device.is_empty()).then_some(device)",
    ),
    "the poll never reads a terminal's foreground": (
        APP,
        "let cli = (*from_terminal && observation.state == TmuxState::Alive)",
        "let cli = (false && observation.state == TmuxState::Alive)",
    ),
    "the poll's reading never reaches the session": (
        APP,
        "                        self.adopt_terminal_cli(session_id, provider);\n",
        "                        let _ = (session_id, provider);\n",
    ),
    "the previous CLI's conversation stays under the new kind": (
        APP,
        "        take_cli_kind(session, provider);\n        session.native_session_id = None;\n",
        "        take_cli_kind(session, provider);\n",
    ),
    "a session launched as a CLI loses its conversation": (
        APP,
        "                && session.agent != provider.agent()\n                && terminal_provenance(session).is_some()\n",
        "                && session.agent != provider.agent()\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
