#!/usr/bin/env python3
"""Change 212's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/212-every-japanese-string-is-translated")

RUN_SECONDS = 1800

TESTS = [
    "tests::every_japanese_literal_is_a_message_or_named_data",
    "tests::every_message_row_is_one_the_code_shows",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

SCREENS = ROOT / "src/app/screens.rs"
GIT = ROOT / "src/git.rs"
MUTATIONS = {
    "a new label with no row": (
        SCREENS,
        'RichText::new(tr("● 実行中"))',
        'RichText::new("● 稼働中")',
    ),
    "the done label is written raw": (
        SCREENS,
        'RichText::new(tr("✓ 完了"))',
        'RichText::new("✓ 完了")',
    ),
    "the checkpoint name is written raw": (
        SCREENS,
        'self.capture_checkpoint(session_id, tr("手動作成"));',
        'self.capture_checkpoint(session_id, "手動作成");',
    ),
    "the truncation note is written raw": (
        GIT,
        '"{diff_text}\\n\\n{}",\n            tr("[差分が 1MB を超えたため切り詰められました]")',
        '"{diff_text}\\n\\n{}",\n            "[差分が 1MB を超えたため切り詰められました]"',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
