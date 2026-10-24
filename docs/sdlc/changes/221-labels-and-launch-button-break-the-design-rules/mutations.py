#!/usr/bin/env python3
"""The mutation sweep for change 221."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "the_git_sub_tabs_are_translated_message_ids",
    "the_idle_group_word_is_not_the_queued_session_word",
    "the_launch_button_is_the_accent_whichever_agent_is_selected",
    "the_documents_name_the_idle_filter_with_the_windows_word",
]
CHECKS = [
    ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", t]
    for t in TESTS
]

MUTATIONS = {
    "git-sub-tab-stays-english": (
        ROOT / "src/app.rs",
        'Self::Changes => tr("変更ファイル"),',
        'Self::Changes => "Changes",',
    ),
    "idle-group-keeps-queued-word": (
        ROOT / "src/ui/widgets.rs",
        'SessionStatusGroup::Free => "指示待ち",',
        'SessionStatusGroup::Free => "待機中",',
    ),
    "launch-button-uses-vendor-colour": (
        ROOT / "src/app/screens.rs",
        "if primary_button(ui, palette, ICON_START, &label).clicked() {",
        'if ui.add(egui::Button::new(RichText::new(format!("{ICON_START}  {label}")).strong()).fill(palette.agent_accent(&self.selected_agent))).clicked() {',
    ),
    "readme-filter-claim-reverts-to-old-word": (
        ROOT / "README.ja.md",
        "**要対応**・**実行中**・**指示待ち**・**終了** の 4 つのトグル",
        "**要対応**・**実行中**・**待機中**・**終了** の 4 つのトグル",
    ),
    "manual-queued-session-claim-broken": (
        ROOT / "docs/MANUAL.ja.md",
        "待機中のセッションには **今すぐ開始** が出て",
        "指示待ちのセッションには **今すぐ開始** が出て",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
