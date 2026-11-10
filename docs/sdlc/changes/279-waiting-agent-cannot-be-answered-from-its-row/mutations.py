#!/usr/bin/env python3
"""Break the reply field of change 279 and what it sends."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "reply_279_"], ["cargo", "test", "--locked", "only_selections_wear_the_selection_colour"]]
REPLY = ROOT / "src/app/waiting_reply.rs"
MUTATIONS = {
    "ended-row-offers-a-field": (REPLY, "if session.status == SessionStatus::Active\n", "if true\n"),
    "no-row-offers-a-field": (REPLY, "if session.status == SessionStatus::Active\n", "if false\n"),
    "enter-does-not-send": (REPLY, "                                        reply = Some(session_id);\n", ""),
    "draft-outlives-the-wait": (ROOT / "src/app.rs", "            self.waiting_reply_drafts\n                .retain(|id, _| self.attention_since.contains_key(id));\n", ""),
    "sent-draft-stays-in-the-field": (REPLY, "if let Some(text) = drafts.remove(&session_id) {", "if let Some(text) = drafts.get(&session_id).cloned() {"),
    "reply-is-not-trimmed": (REPLY, "        let text = text.trim();\n", ""),
    "empty-reply-is-sent": (REPLY, "if text.is_empty() || !running {", "if !running {"),
    "ended-session-is-announced": (REPLY, "if text.is_empty() || !running {", "if text.is_empty() {"),
    "reply-lacks-its-enter": (REPLY, "                TerminalInput::Key(\"Enter\".into()),\n", ""),
    "send-says-nothing": (REPLY, "        self.notice_briefly(tr(\"返信を送りました\"));\n", ""),
    "confirming-enter-sends": (REPLY, "    pressed > confirmed\n", "    pressed > 0\n"),
    "second-enter-is-swallowed": (REPLY, "    pressed > confirmed\n", "    pressed > 0 && confirmed == 0\n"),
    "empty-commit-owns-an-enter": (REPLY, " if !text.is_empty())", ")"),
    "confirming-enter-drops-the-keyboard": (REPLY, "    response.request_focus();\n", ""),
    "selected-row-loses-its-fill": (REPLY, "palette.row_selected", "palette.surface"),
    "field-click-opens-the-session": (REPLY, "if row.response.clicked() {", "if row.response.clicked() || !drafts.is_empty() && ui.input(|input| input.pointer.any_click()) {"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
