#!/usr/bin/env python3
"""The mutation sweep for change 295: Rust 1.100.0 everywhere, egui 0.37, and
the newest direct dependencies. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "every_toolchain_pin_names_one_version",
    "output_limited_command_behaves_as_before",
    "knowledge_map_joins_only_near_meanings",
    "knowledge_map_groups_related_entries_and_is_deterministic",
    "launch_294_recent_projects_come_first",
    "the_icon_face_is_registered_under_the_name_the_font_chain_names",
    "only_a_nonzero_change_count_is_emphasised",
    "a_caret_below_the_last_fully_visible_row_reports_none",
    "history_title_single_click_opens_after_the_double_click_window",
    "a_banner_keeps_a_long_path_inside_its_frame",
    "a_hidden_frame_does_not_replay_the_last_shortcut",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MUTATIONS = {
    # R1, the claim "every workflow pin": a bump that misses harness.yml.
    "harness-workflow-left-on-the-old-compiler": (
        ROOT / ".github/workflows/harness.yml",
        "          toolchain: 1.100.0",
        "          toolchain: 1.88.0",
    ),
    # rust-toolchain.toml is not mutated: any other channel makes rustup
    # install that compiler and rebuild the world inside the sweep.
    # R1: the resolver ceiling disagrees with the compiler.
    "manifest-msrv-disagrees": (
        ROOT / "Cargo.toml",
        'rust-version = "1.100.0"',
        'rust-version = "1.88.0"',
    ),
    # Lint fix in src/exec.rs: the while-let read loop must still stop at the
    # byte limit.
    "output-limit-no-longer-truncates": (
        ROOT / "src/exec.rs",
        "        let retained = remaining.min(count);",
        "        let retained = count;",
    ),
    # Lint fix in src/knowledge/map.rs: as_chunks must still pair the lanes.
    "dot-product-drops-the-chunked-lanes": (
        ROOT / "src/knowledge/map.rs",
        "            lanes[lane] += a[lane] * b[lane];",
        "            lanes[lane] += 0.0 * a[lane] * b[lane];",
    ),
    # Lint fix in src/app/launch_projects.rs: sort_by_key must stay newest first.
    "launch-projects-oldest-first": (
        ROOT / "src/app/launch_projects.rs",
        "choices.sort_by_key(|choice| std::cmp::Reverse((choice.0, choice.1)));",
        "choices.sort_by_key(|choice| (choice.0, choice.1));",
    ),
    # R3: egui-phosphor's own `add_to_fonts` is no longer called, so the icon
    # face must still be registered under the name the font chain asks for.
    "icon-face-registered-under-another-name": (
        ROOT / "src/main.rs",
        "        ICON_FACE.to_owned(),\n        egui::FontData::from_static",
        "        \"phosphor-regular\".to_owned(),\n        egui::FontData::from_static",
    ),
    # R4: egui 0.37's default button theme sets a label in Body, 14px.
    "buttons-keep-egui-default-text-size": (
        ROOT / "src/ui/widgets.rs",
        "    ctx.add_widget_theme::<egui::widget_style::ButtonStyle>(ButtonTextAtButtonSize);",
        "",
    ),
    # R4: the scroll offset egui 0.37 rounds to a pixel hides a fully visible caret.
    "caret-without-vertical-slack": (
        ROOT / "src/ui/terminal.rs",
        "                if rect.min.y >= clip.min.y - slack\n                    && rect.max.y <= clip.max.y + slack",
        "                if rect.min.y >= clip.min.y\n                    && rect.max.y <= clip.max.y",
    ),
    # R4: a title whose `Id` counts the corner's widgets loses its click the
    # frame hovering swaps the time for the close button.
    "history-title-id-counts-the-corner": (
        ROOT / "src/app/screens.rs",
        '.scope_id(ui.scope_id().with("session-row-title"))',
        '.id_salt("session-row-title")',
    ),
    # R4: a message that starts partway along the mark's wrapping row gets a
    # first-row indent egui 0.37 subtracts again after the first break.
    "banner-message-shares-the-marks-row": (
        ROOT / "src/ui/widgets.rs",
        "            ui.horizontal_top(|ui| {\n                ui.spacing_mut().item_spacing.x = SPACE_SM;\n                ui.label(RichText::new(icon).size(15.0).color(tone));\n                ui.horizontal_wrapped(|ui| {\n                    ui.spacing_mut().item_spacing.x = SPACE_SM;\n                    add_contents(ui);\n                });\n            });",
        "            ui.horizontal_wrapped(|ui| {\n                ui.spacing_mut().item_spacing.x = SPACE_SM;\n                ui.label(RichText::new(icon).size(15.0).color(tone));\n                add_contents(ui);\n            });",
    ),
    # R4: `logic` also runs while the window is hidden, on the last shown
    # frame's input; a chord read there fires again on every hidden call.
    "logic-reads-the-shortcuts": (
        ROOT / "src/app.rs",
        "        self.track_dock_badge();\n        if self.last_session_refresh",
        "        self.track_dock_badge();\n        self.handle_app_shortcuts(ctx);\n        if self.last_session_refresh",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
