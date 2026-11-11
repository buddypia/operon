#!/usr/bin/env python3
"""Break terminal cursor cell sizing, theme block fill, glyph inversion, and unfocused outline (change 290)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    [
        "cargo",
        "test",
        "--locked",
        "--manifest-path",
        str(ROOT / "Cargo.toml"),
        "cursor_290_",
    ],
]
TERMINAL = ROOT / "src/ui/terminal.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    "wide-caret-stays-one-cell": (
        TERMINAL,
        "egui::vec2(span.columns as f32 * cell.x, cell.y),",
        "cell,",
    ),
    "trailing-column-does-not-snap-to-lead": (
        TERMINAL,
        "egui::pos2(ui.min_rect().left() + span.column as f32 * cell.x, top),",
        "egui::pos2(ui.min_rect().left() + column as f32 * cell.x, top),",
    ),
    "nfd-voiced-mark-counted-as-wide": (
        TERMINAL,
        "            | 0x3099..=0x309A\n",
        "",
    ),
    "four-byte-ivs-counted-as-narrow": (
        TERMINAL,
        "            | 0xE0100..=0xE01EF\n",
        "",
    ),
    "four-byte-emoji-supplement-stays-narrow": (
        TERMINAL,
        "            | 0x1FA00..=0x1FAFF\n",
        "",
    ),
    "widened-caret-ignores-right-edge": (
        TERMINAL,
        "                    && rect.max.x <= inner_right + slack\n",
        "",
    ),
    "focused-cursor-hardcodes-white-fill": (
        TERMINAL,
        "        ui.painter().rect_filled(caret, 0.0, palette.terminal_fg);",
        "        ui.painter().rect_filled(caret, 0.0, Color32::WHITE);",
    ),
    "focused-cursor-omits-inverted-glyph": (
        TERMINAL,
        "        if glyph.chars().any(|c| !c.is_whitespace()) {",
        "        if false && glyph.chars().any(|c| !c.is_whitespace()) {",
    ),
    "unfocused-pane-skips-hollow-outline": (
        SCREENS,
        "                        } else if let Some(caret) = caret {",
        "                        } else if terminal_response.has_focus() && caret.is_some() {\n                            let caret = caret.unwrap();",
    ),
    "inactive-session-still-draws-cursor": (
        SCREENS,
        "                    if session.status == SessionStatus::Active {\n                        // Composition text sits on the same rect the IME claims,",
        "                    if true {\n                        // Composition text sits on the same rect the IME claims,",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
