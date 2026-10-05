#!/usr/bin/env python3
"""Watch the 065 guard fail. Each mutation removes one term of the fix."""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HOOK = ROOT / ".claude/hooks/gate-stop.sh"
TEST = "the_stop_gate_reads_the_tree_the_session_is_working_in"

MUTATIONS = {
    # The defect itself, put back.
    "no-resolution": [
        (
            'running_in=$(printf \'%s\' "$payload" | jq -r \'.cwd // empty\')\n'
            '[ -n "$running_in" ] && [ -d "$running_in" ] || running_in=.\n',
            "",
        ),
        (
            'working_tree=$(cd -- "$running_in" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)\n'
            'cd -- "${working_tree:-${CLAUDE_PROJECT_DIR:-.}}" || exit 0',
            'cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0',
        ),
    ],
    # The reported path is taken on trust.
    "no-existence-check": [
        ('[ -n "$running_in" ] && [ -d "$running_in" ] || running_in=.',
         '[ -n "$running_in" ] || running_in=.'),
    ],
    # The last resort is dropped.
    "no-fallback": [
        ('cd -- "${working_tree:-${CLAUDE_PROJECT_DIR:-.}}" || exit 0',
         'cd -- "${working_tree:-.}" || exit 0'),
    ],
    # The repository pointers are dropped while resolving — right for a child
    # this crate spawns, wrong for a hook reading the environment the session
    # has.
    "pointers-dropped": [
        ('working_tree=$(cd -- "$running_in" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null)',
         'working_tree=$(cd -- "$running_in" 2>/dev/null && env -u GIT_DIR -u GIT_WORK_TREE git rev-parse --show-toplevel 2>/dev/null)'),
    ],
    # The other fix: the pattern that closes the command substitution early.
    "unparenthesised-case": [
        ('case "$status" in (done|archived) continue ;; esac',
         'case "$status" in done|archived) continue ;; esac'),
    ],
}


def run_test() -> tuple[bool, str]:
    result = subprocess.run(
        ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), TEST],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0, result.stdout + result.stderr


original = HOOK.read_text()
failures = []
try:
    for name, edits in MUTATIONS.items():
        text = original
        for old, new in edits:
            if old not in text:
                print(f"{name}: SETUP FAILED — 置換対象が見つかりません:\n{old}")
                sys.exit(1)
            text = text.replace(old, new, 1)
        HOOK.write_text(text)
        green, output = run_test()
        if green:
            failures.append(name)
            print(f"{name}: NOT CAUGHT — 変異したのにテストが通りました")
        else:
            reason = next(
                (line.strip() for line in output.splitlines() if "panicked at" in line or "終了コード" in line or "拒否文" in line),
                "(理由行が読めません)",
            )
            print(f"{name}: caught — {reason}")
finally:
    HOOK.write_text(original)

print()
print("すべて捕捉されました" if not failures else f"捕捉できなかった変異: {failures}")
sys.exit(1 if failures else 0)
