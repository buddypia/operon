#!/usr/bin/env python3
"""Change 264's mutation sweep: Shift+Enter in a session terminal sends what
Ctrl+J sends, and nothing else about Enter moves. Runner and rules:
scripts/mutation_sweep.py. No sentence a person reads changed, so rule 4's
claim is the binding's own: "exactly Shift, and what Ctrl+J sends".
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = ["shift_enter_puts_a_newline_in_the_agents_input_instead_of_sending"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

TERMINAL = ROOT / "src/ui/terminal.rs"
ARM = "if key == egui::Key::Enter && modifiers.matches_exact(egui::Modifiers::SHIFT) {"

MUTATIONS = {
    "Shift+Enter still sends Enter": (TERMINAL, ARM, "if false {"),
    "Ctrl+Shift+Enter becomes a newline too": (
        TERMINAL,
        ARM,
        "if key == egui::Key::Enter && modifiers.shift_only() {",
    ),
    "Option+Shift+Enter becomes a newline too": (
        TERMINAL,
        ARM,
        "if key == egui::Key::Enter && modifiers.shift && !modifiers.ctrl {",
    ),
    "Shift+Enter sends something other than what Ctrl+J sends": (
        TERMINAL,
        "return terminal_key_binding(egui::Key::J, egui::Modifiers::CTRL);",
        'return Some("M-Enter".to_owned());',
    ),
    "plain Enter becomes a newline as well": (
        TERMINAL,
        'egui::Key::Enter => Some("Enter"),',
        'egui::Key::Enter => Some("C-j"),',
    ),
    "a Shift+Enter that confirms a conversion leaks out as a newline": (
        TERMINAL,
        "if enters_the_ime_owns > 0 && *key == egui::Key::Enter {",
        "if enters_the_ime_owns > 0 && *key == egui::Key::Enter && !modifiers.shift {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
