#!/usr/bin/env python3
"""Break which ticked tabs the broadcast reaches, and how the sheet ticks them (change 306)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "broadcast_"]]
B = ROOT / "src/app/broadcast.rs"
MUTATIONS = {
    # The claim: the line reaches exactly the tabs ticked.
    "send-ignores-the-choice": (B, "        let targets = self.broadcast_recipients(chosen);\n        for session_id", "        let targets = self.broadcast_targets();\n        for session_id"),
    "recipients-ignore-the-choice": (B, "        targets.retain(|id| chosen.contains(id));\n", ""),
    "recipients-ignore-who-is-idle": (B, "        let mut targets = self.broadcast_targets();\n        targets.retain(|id| chosen.contains(id));\n        targets", "        chosen.to_vec()"),
    # The claim: nobody is ticked when the sheet opens.
    "sheet-opens-with-everyone-ticked": (B, "            chosen: Vec::new(),\n        });", "            chosen: self.broadcast_targets(),\n        });"),
    # The claim: a tab that falls idle later is not ticked by itself.
    "late-idle-tabs-are-ticked": (B, "        // `broadcast_recipients` without a second pass over the sessions.\n", "        sheet.chosen = targets.clone();\n"),
    # Finding F1: an Enter that sends nothing keeps the keyboard in the field.
    "no-op-enter-drops-the-keyboard": (B, "            sheet.focus = true;\n        }\n        self.broadcast_sheet = Some(sheet);", "        }\n        self.broadcast_sheet = Some(sheet);"),
    # Ticking and unticking a row.
    "title-does-not-tick": (B, "                                        if boxed || title {", "                                        if boxed {"),
    "box-does-not-tick": (B, "                                        if boxed || title {", "                                        if title {"),
    "a-tick-cannot-be-taken-back": (B, "            if sheet.chosen.contains(&id) {\n                sheet.chosen.retain(|chosen| *chosen != id);\n            } else {\n                sheet.chosen.push(id);\n            }", "            if !sheet.chosen.contains(&id) {\n                sheet.chosen.push(id);\n            }"),
    # すべて選択 / 選択を解除.
    "choose-all-chooses-nobody": (B, "            Some(true) => sheet.chosen = targets,", "            Some(true) => {}"),
    "clear-clears-nothing": (B, "            Some(false) => sheet.chosen.clear(),", "            Some(false) => {}"),
    "control-never-offers-to-clear": (B, "let every_chosen = !targets.is_empty() && recipients.len() == targets.len();", "let every_chosen = false;"),
    # What the sheet says.
    "count-shows-the-listed-number-as-chosen": (B, "                                        p1 = recipients.len(),", "                                        p1 = targets.len(),"),
    "button-counts-every-idle-tab": (B, '&tf!("{p0} 件に送信", p0 = recipients.len()),', '&tf!("{p0} 件に送信", p0 = targets.len()),'),
    "button-ready-with-nobody-ticked": (B, "let ready = !recipients.is_empty() && !sheet.text.trim().is_empty();", "let ready = !targets.is_empty() && !sheet.text.trim().is_empty();"),
    "hint-says-everyone": (B, '.hint_text(tr("選んだ相手に送る内容を入力"))', '.hint_text(tr("アイドル中のエージェントへ送信"))'),
    # Every idle tab can be ticked, and the list scrolls instead of growing.
    "list-is-capped": (B, "                                for session in targets.iter().filter_map(|id| {", "                                for session in targets.iter().take(BROADCAST_VISIBLE_ROWS).filter_map(|id| {"),
    "list-grows-the-sheet": (B, "                            .max_height(row * BROADCAST_VISIBLE_ROWS as f32)\n", ""),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
