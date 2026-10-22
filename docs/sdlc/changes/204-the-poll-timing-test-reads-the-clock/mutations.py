#!/usr/bin/env python3
"""Change 204's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/204-the-poll-timing-test-reads-the-clock")

RUN_SECONDS = 1800

TESTS = [
    "--exact",
    "tests::test_fast_subprocess_completes_without_25ms_polling_floor",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

EXEC = ROOT / "src/exec.rs"
MUTATIONS = {
    "the poll first sleeps 25 ms": (
        EXEC,
        "let mut poll_sleep = Duration::from_millis(1);",
        "let mut poll_sleep = Duration::from_millis(25);",
    ),
    "the poll sleeps a flat 1 ms forever after": (
        EXEC,
        "poll_sleep = (poll_sleep * 2).min(Duration::from_millis(25));",
        "let _ = poll_sleep;",
    ),
    "the poll sleeps a constant beside poll_sleep": (
        EXEC,
        "thread::sleep(poll_sleep.min(deadline - now));",
        "thread::sleep(Duration::from_millis(25).min(deadline - now));",
    ),
    "the poll sleeps 25 ms before its first look": (
        EXEC,
        "    let status = loop {",
        "    thread::sleep(Duration::from_millis(25));\n    let status = loop {",
    ),
    "the poll's cap rises to a second": (
        EXEC,
        "poll_sleep = (poll_sleep * 2).min(Duration::from_millis(25));",
        "poll_sleep = (poll_sleep * 2).min(Duration::from_millis(1000));",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
