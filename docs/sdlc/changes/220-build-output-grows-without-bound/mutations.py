#!/usr/bin/env python3
"""Mutation sweep for 220-build-output-grows-without-bound: each mutation breaks
one requirement or the sentence the installer prints, and a prune_ test must go
red. The runner is scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800

TESTS = ["prune_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

PRUNE = ROOT / "scripts/prune-target.sh"
INSTALL = ROOT / "scripts/install-prune-schedule.sh"
MAKEFILE = ROOT / "Makefile"

MUTATIONS = {
    "keeps the stale program and removes the newest": (
        PRUNE, 'if [ "$bin" != "$newest_binary" ]; then', 'if [ "$bin" = "$newest_binary" ]; then'),
    "only the last --target-dir is trimmed": (
        PRUNE, 'explicit_dirs+=("${1:-}")', 'explicit_dirs=("${1:-}")'),
    "a run with no flags trims every worktree": (
        PRUNE, 'if [ "$all_worktrees" -eq 1 ]; then\n  # --porcelain', 'if true; then\n  # --porcelain'),
    "a worktree without target/ ends the whole pass": (
        PRUNE, '    echo "Nothing to prune."\n    return 0', '    echo "Nothing to prune."\n    exit 0'),
    "--if-idle never sees a running build": (
        PRUNE, 'if [ "$if_idle" -eq 1 ] && build_is_running; then', 'if false; then'),
    "--all-worktrees may be combined with --target-dir": (
        PRUNE, '[ "${#explicit_dirs[@]}" -gt 0 ]; then\n  echo "--all-worktrees and', '[ "${#explicit_dirs[@]}" -gt 99 ]; then\n  echo "--all-worktrees and'),
    "the schedule has no Minute, so it fires every minute of the hour": (
        INSTALL, '    <key>Minute</key>\n    <integer>0</integer>\n', ''),
    "the schedule has no error log": (
        INSTALL, '<key>StandardErrorPath</key>', '<key>StandardErrPath</key>'),
    "the scheduled run forgets --if-idle": (
        INSTALL, '    <string>--if-idle</string>\n', ''),
    "the schedule runs on the wrong weekday": (
        INSTALL, '<key>Weekday</key>\n    <integer>1</integer>', '<key>Weekday</key>\n    <integer>2</integer>'),
    "the installer promises a day the plist does not hold": (
        INSTALL, '毎週月曜 10:00 に', '毎週日曜 10:00 に'),
    "make prune.unschedule installs instead of removing": (
        MAKEFILE, '@bash scripts/install-prune-schedule.sh --uninstall', '@bash scripts/install-prune-schedule.sh'),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
