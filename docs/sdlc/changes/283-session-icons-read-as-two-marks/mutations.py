#!/usr/bin/env python3
"""Break the one mark and its state dot (change 283)."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "session_283_"]]
WIDGETS = ROOT / "src/ui/widgets.rs"
SCREENS = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "running-terminal-wears-a-dot": (WIDGETS, "!(matches!(agent, \"terminal\" | \"existing terminal\") && kind == SessionStatusKind::Running)", "true"),
    "every-terminal-is-silent": (WIDGETS, "!(matches!(agent, \"terminal\" | \"existing terminal\") && kind == SessionStatusKind::Running)", "!matches!(agent, \"terminal\" | \"existing terminal\")"),
    "dot-loses-the-state-ink": (WIDGETS, "painter.circle_filled(center, radius, palette.status(kind.tone()));", "painter.circle_filled(center, radius, palette.text_muted);"),
    "dot-has-no-ring": (WIDGETS, "    painter.circle_filled(center, radius + STATUS_DOT_RING, surface);\n", ""),
    "waiting-dot-has-no-shape": (WIDGETS, "    if needs_you {\n        let stroke", "    if false {\n        let stroke"),
    "waiting-dot-is-not-larger": (WIDGETS, "        STATUS_DOT_ATTENTION_RATIO\n    } else {", "        STATUS_DOT_RATIO\n    } else {"),
    "tab-keeps-the-circle": (WIDGETS, "    let Some(texture) = session_mark_texture(ui.ctx(), agent) else {\n        return session_status_icon(ui, palette, kind);\n    };", "    session_status_icon(ui, palette, kind);\n    let Some(texture) = session_mark_texture(ui.ctx(), agent) else {\n        return session_status_icon(ui, palette, kind);\n    };"),
    "tab-ring-ignores-active": (SCREENS, "let surface = if active {", "let surface = if !active {"),
    "column-keeps-the-word": (SCREENS, "if session_mark_texture(ui.ctx(), &session.agent)", "if agent_icon_asset(&session.agent).is_some() && session_mark_texture(ui.ctx(), &session.agent)"),
    "column-ring-ignores-selection": (SCREENS, "                                                    palette.row_selected\n                                                } else {\n                                                    palette.panel", "                                                    palette.panel\n                                                } else {\n                                                    palette.row_selected"),
    "hover-drops-the-cli": (SCREENS, "                                                                ui.label(agent_short_label(&session.agent));\n                                                                let view", "                                                                let view"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
