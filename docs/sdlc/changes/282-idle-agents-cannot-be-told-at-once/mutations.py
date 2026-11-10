#!/usr/bin/env python3
"""Break who an instruction reaches and how the sheet sends it (change 282)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "broadcast_282_"]]
B = ROOT / "src/app/broadcast.rs"
SCREENS = ROOT / "src/app/screens.rs"
APP = ROOT / "src/app.rs"
MUTATIONS = {
    "other-projects-are-reached": (B, "session.project_id == project\n                    && self.session_status_kind_for", "self.session_status_kind_for"),
    "working-agents-are-reached": (B, "== SessionStatusKind::Idle", "!= SessionStatusKind::Waiting"),
    "prompting-agents-are-reached": (B, "== SessionStatusKind::Idle", "!= SessionStatusKind::Working"),
    "selected-project-wins-over-the-workspace": (B, "self.workspace_project.or(self.selected_project)?", "self.selected_project.or(self.workspace_project)?"),
    "text-is-not-trimmed": (B, "        let text = text.trim();\n        if text.is_empty() {", "        if text.trim().is_empty() {"),
    "empty-text-is-sent": (B, "        if text.is_empty() {\n            return 0;\n        }\n", ""),
    "send-lacks-its-enter": (B, '                    TerminalInput::Key("Enter".into()),\n', ""),
    "send-says-nothing": (B, "        if !targets.is_empty() {\n            self.notice_briefly", "        if false {\n            self.notice_briefly"),
    "nobody-idle-is-announced": (B, "        if !targets.is_empty() {\n            self.notice_briefly", "        {\n            self.notice_briefly"),
    "escape-does-not-close": (B, "input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)", "input.consume_key(egui::Modifiers::COMMAND, egui::Key::Escape)"),
    "field-does-not-take-the-keyboard": (B, "if std::mem::take(&mut sheet.focus) {", "if std::mem::take(&mut sheet.focus) && false {"),
    "confirming-enter-sends": (B, "if ui.input(|input| reply_enter_sends(&input.events)) {", "if true {"),
    "enter-does-not-send": (B, "                                send = true;\n", ""),
    "sheet-stays-after-a-send": (B, "if send && self.send_broadcast(&sheet.text) > 0 {\n            return;\n        }", "if send {\n            self.send_broadcast(&sheet.text);\n        }"),
    "sheet-closes-with-nobody-idle": (B, "if send && self.send_broadcast(&sheet.text) > 0 {", "if send {\n            self.send_broadcast(&sheet.text);"),
    "recipients-are-not-named": (B, "targets.iter().take(BROADCAST_NAMED)", "targets.iter().take(0)"),
    "palette-opens-over-the-sheet": (ROOT / "src/app/keymap.rs", 'if !sheet_open && self.chord_pressed(ctx, "palette.open") {', 'if !self.launch_sheet_open && self.chord_pressed(ctx, "palette.open") {'),
    "chords-reach-past-the-sheet": (ROOT / "src/app/keymap.rs", "if !self.command_palette_open && !sheet_open {", "if !self.command_palette_open && !self.launch_sheet_open {"),
    "palette-row-opens-nothing": (SCREENS, '            PaletteKind::Action("Send to idle agents") => self.open_broadcast_sheet(),\n', ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
