#!/usr/bin/env python3
"""Falsify the idle group's wording: every place it could read as waiting on
the person again must turn a guard red."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "the_idle_group_word_does_not_read_as_waiting_on_you",
    "the_documents_name_the_idle_filter_with_the_windows_word",
    "workspace_session_tab_shows_its_status_glyph",
    "every_message_id_has_a_row_in_every_table",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
WIDGETS = ROOT / "src/ui/widgets.rs"
TABLES = ROOT / "src/i18n_tables.rs"
MANUAL_JA = ROOT / "docs/MANUAL.ja.md"
MANUAL_KO = ROOT / "docs/MANUAL.ko.md"
README_KO = ROOT / "README.ko.md"
MUTATIONS = {
    "the idle chip says 指示待ち again": (
        WIDGETS,
        'SessionStatusGroup::Free => "手すき",',
        'SessionStatusGroup::Free => "指示待ち",',
    ),
    "the idle chip's hover says it is waiting": (
        WIDGETS,
        'SessionStatusGroup::Free => "ターンを終えて手が空いているセッション",',
        'SessionStatusGroup::Free => "起動していて、次の依頼を待っているセッション",',
    ),
    "an idle tab's hover says it is waiting": (
        WIDGETS,
        'SessionStatusKind::Idle => tr("ターンを終えて手が空いています").to_owned(),',
        'SessionStatusKind::Idle => tr("前提の完了待ち").to_owned(),',
    ),
    "the English tab hint says waiting": (
        TABLES,
        '("ターンを終えて手が空いています", "Turn finished; free for a new request"),',
        '("ターンを終えて手が空いています", "Waiting for the next request"),',
    ),
    "the Korean chip word says 대기 again": (
        TABLES,
        '("手すき", "유휴"),',
        '("手すき", "지시 대기"),',
    ),
    "the Korean chip hint says waiting": (
        TABLES,
        '("ターンを終えて手が空いているセッション", "턴을 마치고 손이 비어 있는 세션"),',
        '("ターンを終えて手が空いているセッション", "다음 지시를 기다리는 세션"),',
    ),
    "the manual describes the idle state as a wait": (
        MANUAL_JA,
        "| **手すき** | ターンを終えて手が空いている。次の依頼を出すかは自由 |",
        "| **手すき** | 起動していて、次の依頼を待っている |",
    ),
    "the manual's troubleshooting names 指示待ち": (
        MANUAL_JA,
        "「手すき」のまま",
        "「指示待ち」のまま",
    ),
    "the Korean manual's troubleshooting names 지시 대기": (
        MANUAL_KO,
        '"유휴"로 남음',
        '"지시 대기"로 남음',
    ),
    "the English tab hint becomes another state's": (
        TABLES,
        '("ターンを終えて手が空いています", "Turn finished; free for a new request"),',
        '("ターンを終えて手が空いています", "Needs you now"),',
    ),
    "the Korean tab hint says it is running": (
        TABLES,
        '("ターンを終えて手が空いています", "턴을 마치고 손이 비어 있습니다"),',
        '("ターンを終えて手が空いています", "실행 중입니다"),',
    ),
    "the English chip hint says running": (
        TABLES,
        '("ターンを終えて手が空いているセッション", "Sessions that finished their turn and are free"),',
        '("ターンを終えて手が空いているセッション", "Sessions that are running"),',
    ),
    "the English manual describes Idle as a wait": (
        ROOT / "docs/MANUAL.md",
        "| **Idle** | Finished its turn and free; the next request is up to you |",
        "| **Idle** | Running and waiting for your next request |",
    ),
    "the Korean README caption names 지시 대기": (
        README_KO,
        "대응 필요·실행 중·유휴·종료로 한눈에",
        "대응 필요·실행 중·지시 대기·종료로 한눈에",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
