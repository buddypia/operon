#!/usr/bin/env python3
"""The mutation sweep for change 293: terminal paths open in Operon on ⌘-click,
right-click offers where else to open them, and a bare name resolves to the one
scanned file with that name. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = ["terminal_293_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

PATHS = ROOT / "src/app/terminal_paths.rs"
APP = ROOT / "src/app.rs"
FILES = ROOT / "src/files.rs"

MUTATIONS = {
    # R1, and the claim "⌘-click opens it inside Operon": send it outside again.
    "cmd-click-opens-with-the-default-app": (
        PATHS,
        "        if super::screens::is_terminal_open_modifier(modifiers) {\n            self.open_terminal_target(kind);",
        "        if super::screens::is_terminal_open_modifier(modifiers) {\n            self.open_terminal_target_external(kind);",
    ),
    # R2: the menu loses its first row.
    "menu-drops-open-in-operon": (
        PATHS,
        "pub(crate) const TERMINAL_PATH_MENU: [TerminalPathMenuChoice; 4] = [\n    TerminalPathMenuChoice::OpenInOperon,\n    TerminalPathMenuChoice::OpenWithDefaultApp,",
        "pub(crate) const TERMINAL_PATH_MENU: [TerminalPathMenuChoice; 4] = [\n    TerminalPathMenuChoice::OpenWithDefaultApp,\n    TerminalPathMenuChoice::OpenWithDefaultApp,",
    ),
    # R2: the menu is not placed at the pointer.
    "menu-forgets-the-pointer": (
        PATHS,
        "            self.terminal_path_menu_at = pointer;",
        "            self.terminal_path_menu_at = None;",
    ),
    # R3: the menu's "Operon で開く" row hands the file to the default app.
    "menu-open-in-operon-goes-outside": (
        PATHS,
        "            TerminalPathMenuChoice::OpenInOperon => {\n                self.open_terminal_target(",
        "            TerminalPathMenuChoice::OpenInOperon => {\n                self.open_terminal_target_external(",
    ),
    # R4: bare names are never looked up in the scan.
    "bare-name-not-looked-up": (
        PATHS,
        "    if path.contains('/') {\n        return None;\n    }",
        "    if !path.is_empty() {\n        return None;\n    }",
    ),
    # R4: two files of the name — the first one is guessed.
    "bare-name-guesses-among-several": (
        FILES,
        "    matches.next().is_none().then_some(first.as_path())",
        "    Some(first.as_path())",
    ),
    # R5: the drain never asks for the scan.
    "probe-never-requests-the-scan": (
        APP,
        "        if scan_needed {\n            self.request_terminal_session_scan(",
        "        if scan_needed && false {\n            self.request_terminal_session_scan(",
    ),
    # R5: a landing scan keeps the old misses.
    "scan-landing-keeps-misses": (
        APP,
        "                    self.file_cache.insert(project_id, scan);\n                    self.forget_terminal_path_misses();",
        "                    self.file_cache.insert(project_id, scan);",
    ),
    # R6: Escape does not close the menu.
    "escape-keeps-the-menu-open": (
        PATHS,
        "    chosen || escape_pressed || pressed_outside",
        "    chosen || pressed_outside",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
