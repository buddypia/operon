#!/usr/bin/env python3
"""Watch the 066 guards fail. Each mutation removes one term of the fix."""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
ROUTES = ROOT / "docs/sdlc/routes.yaml"
ROUNDS = ROOT / "docs/sdlc/review-rounds.yaml"
GATE = ROOT / "scripts/check-review.sh"
HOOK = ROOT / ".githooks/reference-transaction"

TESTS = [
    "every_route_row_is_the_width_the_gate_that_judges_commits_reads",
    "every_route_has_a_review_round_ceiling",
]

MUTATIONS = {
    # The defect itself: a sixth value, which the gate that judges a commit
    # reads as no route at all.
    "sixth-column": (ROUTES,
                     '  bugfix: "build,test 3 .claude/hooks/gate-commit.sh .claude/skills/root-cause/SKILL.md no"',
                     '  bugfix: "build,test 3 .claude/hooks/gate-commit.sh .claude/skills/root-cause/SKILL.md no 2"'),
    # The tolerance that made the defect invisible the first time.
    "tolerant-width": (GATE, '[ "$#" -eq 5 ] || continue', '[ "$#" -ge 5 ] || continue'),
    # A route whose ceiling silently falls back to the default.
    "ceiling-dropped": (ROUNDS, '  bugfix: "2"\n', ''),
    # The premise: the hook reads main's copy of the gate. If it stops, the rule
    # above has no cause and the guard must say so rather than go on enforcing.
    "premise-gone": (HOOK, 'refs/heads/main:scripts/check-review.sh', 'scripts/check-review.sh'),
}


def run_tests() -> tuple[bool, str]:
    result = subprocess.run(
        ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--",
         *TESTS],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0, result.stdout + result.stderr


originals = {path: path.read_text() for path in {ROUTES, ROUNDS, GATE, HOOK}}
failures = []
try:
    for name, (path, old, new) in MUTATIONS.items():
        text = originals[path]
        if old not in text:
            print(f"{name}: SETUP FAILED — 置換対象が見つかりません:\n{old}")
            sys.exit(1)
        # Every occurrence, not the first: the premise appears twice in the hook
        # and replacing one left the assertion satisfied by the other.
        path.write_text(text.replace(old, new))
        green, output = run_tests()
        path.write_text(originals[path])
        if green:
            failures.append(name)
            print(f"{name}: NOT CAUGHT — 変異したのにテストが通りました")
        else:
            reason = next(
                (line.strip() for line in output.splitlines() if "panicked at" in line),
                "(理由行が読めません)",
            )
            print(f"{name}: caught — {reason}")
finally:
    for path, text in originals.items():
        path.write_text(text)

print()
print("すべて捕捉されました" if not failures else f"捕捉できなかった変異: {failures}")
sys.exit(1 if failures else 0)
