#!/usr/bin/env python3
"""Change 267: take the CLI mark off a tab, mis-map it, mis-size it, and make its hover lie."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "the_terminal_mark_is_a_rounded_square_for_terminal_sessions_only",
    "session_tabs_show_the_cli_mark_after_the_state",
    "session_tab_mark_names_its_cli_on_hover",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
SCREEN = ROOT / "src/app/screens.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"
GLYPHS = ROOT / "src/glyphs.rs"
TERMINAL_ARM = '"terminal" | "existing terminal" => png_texture(ctx, "terminal-mark", TERMINAL_MARK),'
ABSENT = """    let Some(texture) = session_mark_texture(ui.ctx(), agent) else {
        return;
    };"""
MUTATIONS = {
    "the tab stops drawing its CLI mark": (
        SCREEN,
        "session_mark(ui, &session.agent);",
        "let _ = &session.agent;",
    ),
    "a plain terminal gets no mark": (
        WIDGETS,
        TERMINAL_ARM,
        '"existing terminal" => png_texture(ctx, "terminal-mark", TERMINAL_MARK),',
    ),
    "a recovered terminal gets no mark": (
        WIDGETS,
        TERMINAL_ARM,
        '"terminal" => png_texture(ctx, "terminal-mark", TERMINAL_MARK),',
    ),
    "a custom command is passed off as a terminal": (
        WIDGETS,
        "_ => agent_icon_texture(ctx, agent),",
        '_ => agent_icon_texture(ctx, agent).or_else(|| png_texture(ctx, "terminal-mark", TERMINAL_MARK)),',
    ),
    "a session without a mark still holds its space": (
        WIDGETS,
        ABSENT,
        ABSENT.replace("        return;", "        ui.allocate_space(egui::Vec2::splat(SESSION_MARK_SIZE));\n        return;"),
    ),
    "the mark is drawn at another size than the left column's": (
        WIDGETS,
        "egui::Vec2::splat(SESSION_MARK_SIZE),\n    )))",
        "egui::Vec2::splat(SESSION_MARK_SIZE * 1.5),\n    )))",
    ),
    "the hover says nothing": (
        WIDGETS,
        ".on_hover_text(agent_short_label(agent));",
        '.on_hover_text("");',
    ),
    "the hover names the stored id instead of the CLI": (
        WIDGETS,
        ".on_hover_text(agent_short_label(agent));",
        ".on_hover_text(agent);",
    ),
    "the terminal mark is not a rounded plate": (
        GLYPHS,
        'include_bytes!("../assets/terminal-mark-128.png")',
        'include_bytes!("../assets/operon-mark-128.png")',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
