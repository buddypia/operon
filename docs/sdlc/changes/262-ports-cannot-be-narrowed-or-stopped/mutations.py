#!/usr/bin/env python3
"""Break the promises that a session's header counts only its own folder,
that the rest of the project is listed apart, that a finished stop is not
undone by a scan that started before it, and that a stop signals only the
listener the person confirmed, after asking, and says truthfully what became
of it. 23 mutations, about 24 s each on CI: inside its 900 s per obligation."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [
    ["cargo", "test", "--locked", "ports_262_"],
    ["cargo", "test", "--locked", "ui_session_ports_renders_compact_popover_button_when_active"],
]
PORTS = ROOT / "src/git/ports.rs"
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    # R1: the shallowest folder wins, so the project root takes every worktree's server.
    "shallowest-folder-wins": (
        PORTS,
        ".max_by_key(|root| root.path.components().count())",
        ".min_by_key(|root| root.path.components().count())",
    ),
    # R1/R3, the claim: a worktree nobody has a session on is not a folder, so
    # the root session's badge counts its server as its own again.
    "git-worktrees-not-folders": (
        PORTS,
        "for worktree in worktrees_of(project) {",
        "for worktree in Vec::<PathBuf>::new() {",
    ),
    # R2/R3: a session stored through a link no longer finds its folder.
    "own-ignores-stored-spelling": (
        APP,
        "if worktree.is_some_and(|worktree| root.spellings.contains(worktree)) {",
        "if worktree.is_some_and(|worktree| root.path == *worktree) {",
    ),
    # R3/R4, the claim: the header counts the whole project as this folder's.
    "own-is-whole-project": (
        APP,
        "if worktree.is_some_and(|worktree| root.spellings.contains(worktree)) {",
        "if root.project_id == session.project_id {",
    ),
    # F1: a session with no worktree, running in the project checkout, owns nothing.
    "no-worktree-owns-nothing": (
        APP,
        "                .find(|project| project.id == session.project_id)\n                .map(|project| &project.path)\n",
        "                .find(|_| false)\n                .map(|project| &project.path)\n",
    ),
    # R5: the rest-of-project heading is drawn over nothing.
    "empty-section-drawn": (
        SCREENS,
        "if !ports.elsewhere.is_empty() {",
        "if true {",
    ),
    # R10: a scan that started before the stop is accepted and brings it back.
    "stale-scan-accepted": (
        APP,
        "if generation != self.ports_generation {",
        "if false {",
    ),
    "stale-scan-not-reread": (
        APP,
        "                        self.ports_scanned_at = None;\n                        continue;\n",
        "                        continue;\n",
    ),
    "generation-not-bumped": (
        APP,
        "                    self.ports_generation = self.ports_generation.wrapping_add(1);\n",
        "",
    ),
    "no-rescan-after-stop": (
        APP,
        "                    self.ports_generation = self.ports_generation.wrapping_add(1);\n                    self.ports_scanned_at = None;\n",
        "                    self.ports_generation = self.ports_generation.wrapping_add(1);\n",
    ),
    # R10: the stop drops the process's other ports and its neighbours too.
    "drop-more-than-the-stopped": (
        APP,
        "attributed.port.pid != port.pid || attributed.port.port != port.port",
        "attributed.port.pid != port.pid && attributed.port.port != port.port",
    ),
    # R5: a second stop is accepted while one is running.
    "stop-enabled-while-stopping": (
        SCREENS,
        ".add_enabled_ui(!stopping, |ui| {",
        ".add_enabled_ui(true, |ui| {",
    ),
    # R6: init, an impossible PID and Operon itself are no longer refused.
    "init-not-refused": (
        PORTS,
        "if port.pid <= 1 || port.pid > i32::MAX.unsigned_abs() || port.pid == operon_pid {",
        "if port.pid > i32::MAX.unsigned_abs() || port.pid == operon_pid {",
    ),
    "impossible-pid-not-refused": (
        PORTS,
        "if port.pid <= 1 || port.pid > i32::MAX.unsigned_abs() || port.pid == operon_pid {",
        "if port.pid <= 1 || port.pid == operon_pid {",
    ),
    "operon-not-refused": (
        PORTS,
        "if port.pid <= 1 || port.pid > i32::MAX.unsigned_abs() || port.pid == operon_pid {",
        "if port.pid <= 1 || port.pid > i32::MAX.unsigned_abs() {",
    ),
    # R7: a listener that is no longer the confirmed one is signalled anyway.
    "signal-without-match": (
        PORTS,
        "Some(ListenerRead::Renamed | ListenerRead::Free) => return PortStopOutcome::NotListening,",
        "Some(ListenerRead::Renamed | ListenerRead::Free) => {}",
    ),
    # R7: any process on that PID and port counts, whatever its name.
    "name-not-checked": (
        PORTS,
        "        let same =\n            first.process == port.process || holders.any(|seen| seen.process == port.process);",
        "        let same = true || holders.any(|seen| seen.process == port.process);",
    ),
    # R8, the claim: failed reads after the signal are reported as stopped.
    "unconfirmed-reported-stopped": (
        PORTS,
        "    } else {\n        PortStopOutcome::Unconfirmed\n    }",
        "    } else {\n        PortStopOutcome::Stopped\n    }",
    ),
    # R8: `lsof` complaining while finding nothing is read as a free port.
    "complaining-lsof-is-free": (
        PORTS,
        "(output.status.success() || quiet).then_some(ListenerRead::Free)",
        "Some(ListenerRead::Free)",
    ),
    # F2: the PID still holding the port under another name counts as stopped.
    "renamed-counts-as-free": (
        PORTS,
        "            Some(ListenerRead::Free) => return PortStopOutcome::Stopped,\n            Some(ListenerRead::Same | ListenerRead::Renamed) => seen_held = true,",
        "            Some(ListenerRead::Free | ListenerRead::Renamed) => return PortStopOutcome::Stopped,\n            Some(ListenerRead::Same) => seen_held = true,",
    ),
    # R9: cancelling the confirmation starts the stop.
    "cancel-stops": (
        SCREENS,
        "        if confirmed {\n            self.pending_port_stop = None;\n            self.stop_port(attributed.port);",
        "        if confirmed || cancelled {\n            self.pending_port_stop = None;\n            self.stop_port(attributed.port);",
    ),
    # R9, the claim: the window no longer says what will be sent or that it
    # reaches processes started outside Operon.
    "confirmation-hides-the-signal": (
        SCREENS,
        '"終了シグナル (SIGTERM) を送ります。Operon の外で起動したプロセスも含まれます。",',
        '"停止しています…",',
    ),
    # R10, the claim: a server still running is reported as stopped.
    "still-running-told-as-stopped": (
        PORTS,
        '"{process} に停止を求めましたが、まだ :{port} で動いています。",',
        '"{process} を停止しました（:{port}）。",',
    ),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
