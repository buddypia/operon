#!/usr/bin/env python3
"""The mutation sweep for change 226."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "pane_drop_follows_the_pointer_like_an_editor_group",
    "dropping_on_an_edge_splits_the_terminal_on_that_side",
    "a_session_dropped_twice_moves_rather_than_copies",
    "a_split_past_four_or_too_small_is_refused",
    "closing_a_terminal_gives_its_room_to_the_neighbour",
    "selecting_a_session_replaces_the_focused_terminal_only",
    "two_split_sessions_are_both_drawn",
    "every_terminal_on_screen_is_polled_and_read",
    "pane_drop_hint_names_the_zone",
    "dragging_a_row_onto_a_terminal_splits_it",
]
CHECKS = [
    ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]
]

TREE = ROOT / "src/ui/session_tree.rs"
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    "an-edge-drop-lands-in-the-middle": (
        TREE,
        ".map_or(PaneDrop::Center, |(drop, _)| drop)",
        ".map_or(PaneDrop::Center, |_| PaneDrop::Center)",
    ),
    "a-right-drop-goes-left": (
        TREE,
        "PaneDrop::Right => (true, false),",
        "PaneDrop::Right => (true, true),",
    ),
    "a-session-dropped-again-is-shown-twice": (
        TREE,
        "        after.remove(moved);\n",
        "\n",
    ),
    "a-fifth-terminal-fits": (
        TREE,
        "self.sessions().len() <= PANE_MAX",
        "self.sessions().len() <= PANE_MAX + 1",
    ),
    "closing-one-closes-its-neighbour-too": (
        TREE,
        "(Some(only), None) | (None, Some(only)) => Some(only),",
        "(Some(_), None) | (None, Some(_)) => None,",
    ),
    "selecting-ignores-the-split": (
        APP,
        "        self.panes.show(self.selected_session, session_id);\n",
        "\n",
    ),
    "the-split-is-not-drawn": (
        SCREENS,
        ".then(|| self.ui_terminal_panes(&mut center, rects.center, palette));",
        ".then(PaneRects::default);",
    ),
    "only-the-focused-terminal-is-read": (
        APP,
        "&& self.terminals_on_screen().contains(&session_id)",
        "&& self.selected_session == Some(session_id)",
    ),
    "claim-the-hint-names-the-wrong-side": (
        TREE,
        'Self::Right => tr("右に分割"),',
        'Self::Right => tr("左に分割"),',
    ),
    "a-row-released-on-a-terminal-is-lost": (
        SCREENS,
        "if let Some(moved) = carried {",
        "if let Some(moved) = carried.filter(|_| self.column_drag.is_some()) {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
