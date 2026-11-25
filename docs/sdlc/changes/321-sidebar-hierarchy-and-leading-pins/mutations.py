#!/usr/bin/env python3
"""Break the quiet session-list label, 15px project names, and leading pin marks (change 321)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
TESTS = ["column_321_", "heading_320_", "column_319_", "sidebar_302_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
SCREENS = ROOT / "src/app/screens.rs"
I18N = ROOT / "src/i18n_tables.rs"

MUTATIONS = {
    # R1: top label reverts to プロジェクト.
    "panel-label-back-to-project": (
        SCREENS,
        "                RichText::new(DockPanel::Sessions.label())\n                    .size(12.5)",
        "                RichText::new(tr(\"プロジェクト\"))\n                    .size(12.5)",
    ),
    # R1: top label grows back to section size.
    "panel-label-at-section-size": (
        SCREENS,
        "                RichText::new(DockPanel::Sessions.label())\n                    .size(12.5)\n                    .color(palette.text_muted),",
        "                RichText::new(DockPanel::Sessions.label())\n                    .size(15.0)\n                    .color(palette.text_muted),",
    ),
    # R1: top label takes strong ink instead of muted.
    "panel-label-strong-ink": (
        SCREENS,
        "                RichText::new(DockPanel::Sessions.label())\n                    .size(12.5)\n                    .color(palette.text_muted),",
        "                RichText::new(DockPanel::Sessions.label())\n                    .size(12.5)\n                    .color(palette.text_strong),",
    ),
    # R2: project names shrink back to 13.5px.
    "project-name-back-to-13-5": (
        SCREENS,
        "                                                            RichText::new(&project.name)\n                                                                .size(15.0)",
        "                                                            RichText::new(&project.name)\n                                                                .size(13.5)",
    ),
    # R3: pinned row drops the gutter offset, breaking vertical mark alignment.
    "sidebar-pin-gutter-lost": (
        SCREENS,
        "                                        left: if pinned {\n                                            (SIDEBAR_ROW_INDENT - SESSION_PIN_SLOT_W - SPACE_XS)\n                                                as i8\n                                        } else {\n                                            SIDEBAR_ROW_INDENT as i8\n                                        },",
        "                                        left: SIDEBAR_ROW_INDENT as i8,",
    ),
    # R3: selected pinned sidebar row falls back to muted ink.
    "sidebar-pin-selected-muted": (
        SCREENS,
        "                                                        RichText::new(ICON_PIN).size(11.5).color(\n                                                            if self.selected_session == Some(id) {\n                                                                palette.selected_secondary_ink()\n                                                            } else {\n                                                                palette.text_muted\n                                                            },\n                                                        ),",
        "                                                        RichText::new(ICON_PIN).size(11.5).color(palette.text_muted),",
    ),
    # R4: active pinned central tab falls back to muted ink.
    "tab-pin-active-muted": (
        SCREENS,
        "                                                                RichText::new(ICON_PIN)\n                                                                    .size(11.5)\n                                                                    .color(if active {\n                                                                        palette.selected_secondary_ink()\n                                                                    } else {\n                                                                        palette.text_muted\n                                                                    }),",
        "                                                                RichText::new(ICON_PIN)\n                                                                    .size(11.5)\n                                                                    .color(palette.text_muted),",
    ),
    # F1: pinned sidebar row hover omits ピン留め中.
    "sidebar-pin-hover-missing": (
        SCREENS,
        "                                        if pinned {\n                                            ui.label(tr(\"ピン留め中\"));\n                                        }\n                                        ui.label(tr(\"ドラッグで並べ替え\"));",
        "                                        ui.label(tr(\"ドラッグで並べ替え\"));",
    ),
    # R5: English label falls back to lowercase "the session list".
    "session-list-english-lowercase": (
        I18N,
        "    (\"セッション一覧\", \"Session list\"),",
        "    (\"セッション一覧\", \"the session list\"),",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
