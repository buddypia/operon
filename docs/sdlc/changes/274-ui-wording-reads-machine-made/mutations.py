#!/usr/bin/env python3
"""Falsify change 274's wording: every way the reworded text could slide back —
the idle word, a hint that reads as waiting, Japanese or backslashes on an
English or Korean screen, a retired term, a document quoting an old label, a
durability notice that claims a failure — must turn a guard red."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "the_idle_group_word_does_not_read_as_waiting_on_you",
    "no_translation_carries_japanese_script_or_a_stray_backslash",
    "no_message_uses_a_retired_term",
    "the_documents_name_the_idle_filter_with_the_windows_word",
    "a_written_but_unconfirmed_notice_never_says_the_write_failed",
    "every_message_id_has_a_row_in_every_table",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
WIDGETS = ROOT / "src/ui/widgets.rs"
TABLES = ROOT / "src/i18n_tables.rs"
MUTATIONS = {
    # R1
    "the idle chip says 手すき again": (
        WIDGETS,
        'SessionStatusGroup::Free => "アイドル",',
        'SessionStatusGroup::Free => "手すき",',
    ),
    "an idle tab's hover says it is waiting": (
        WIDGETS,
        'SessionStatusKind::Idle => tr("ターンを終え、次の依頼を出せます").to_owned(),',
        'SessionStatusKind::Idle => tr("前提の完了待ち").to_owned(),',
    ),
    "Korean needs-you reads 대응 필요 again": (
        TABLES,
        '("要対応", "확인 필요"),',
        '("要対応", "대응 필요"),',
    ),
    "the English idle hint promises a wait": (
        TABLES,
        '("ターンを終え、次の依頼を出せます", "Turn finished; ready for your next request"),',
        '("ターンを終え、次の依頼を出せます", "Turn finished; waiting for your next request"),',
    ),
    "the Korean idle chip hint says 대기": (
        TABLES,
        '("ターンを終え、次の依頼を出せるセッション", "턴을 마치고 다음 요청을 보낼 수 있는 세션"),',
        '("ターンを終え、次の依頼を出せるセッション", "턴을 마치고 다음 요청을 대기하는 세션"),',
    ),
    # R2
    "a Japanese character inside a Korean word": (
        TABLES,
        '"현재 worktree의 변경 사항을 main 브랜치에 병합합니다."',
        '"현재 worktree의 변경 사항을 main 브랜치에 병合합니다."',
    ),
    "English keeps the Japanese button name": (
        TABLES,
        r'choose \"Stop\" again',
        r'press 「停止」 again',
    ),
    "a quote drawn with backslashes": (
        TABLES,
        r'Start one with \"New session\" in the toolbar.',
        r'Start one with \\\"New session\\\" in the toolbar.',
    ),
    # R3
    "Japanese says 永続化 again": (
        TABLES,
        '("{label}: 保存未確認（{error}）",',
        '("{label}: 永続化未確認（{error}）",',
    ),
    "English says land again": (
        TABLES,
        '"Merge worktree into main…"',
        '"Land worktree into main…"',
    ),
    "Korean calls a diff 차이점 again": (
        TABLES,
        '("差分", "변경 내용"),',
        '("差分", "차이점"),',
    ),
    "a row names the menu that does not exist": (
        TABLES,
        r'from \"Detect local CLI sessions\"',
        'from Discover local CLI sessions',
    ),
    # R5
    "README.ja.md lists 手すき again": (
        ROOT / "README.ja.md",
        "**要対応**・**実行中**・**アイドル**・**終了**",
        "**要対応**・**実行中**・**手すき**・**終了**",
    ),
    "the Korean manual's idle row reads as a free hand again": (
        ROOT / "docs/MANUAL.ko.md",
        "| **유휴** | 턴을 마치고 다음 요청을 보낼 수 있는 상태 |",
        "| **유휴** | 턴을 마치고 손이 비어 있는 상태. 다음 요청은 자유 |",
    ),
    "the English manual names the old merge menu": (
        ROOT / "docs/MANUAL.md",
        "**Merge worktree into main…**",
        "**Land worktree into main…**",
    ),
    "the Korean README hero caption says 대응 필요": (
        ROOT / "README.ko.md",
        "확인 필요·실행 중·유휴·종료로 한눈에",
        "대응 필요·실행 중·유휴·종료로 한눈에",
    ),
    # The claim: a notice whose data is on disk must not say the save failed.
    "a durability notice claims the save failed": (
        TABLES,
        "取り込みは書き込まれましたが、ディスクに保存されたことを確認できませんでした。",
        "取り込みは書き込まれましたが、ディスクに保存できませんでした。",
    ),
    "the English durability notice claims a failure": (
        TABLES,
        "The import was written, but Operon could not confirm it reached the disk.",
        "The import was written, but saving it to disk failed.",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
