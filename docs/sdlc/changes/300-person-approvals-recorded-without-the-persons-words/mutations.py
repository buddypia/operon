#!/usr/bin/env python3
"""The mutation sweep for change 300: from change 300 on, a person line in an
approvals.log carries the person's words, and approval.md says so. Each
mutation takes part of that away and a test must refuse to pass.
Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 900

TESTS = [
    "the_approval_checker_wants_the_persons_words_on_a_person_line",
    "the_approval_ledger_rows_are_the_width_the_checker_reads",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

CHECKER = ROOT / "scripts/check-approvals.sh"
RULE = ROOT / ".claude/skills/sdlc/references/approval.md"

MUTATIONS = {
    "the checker does not look for a quote": (
        CHECKER,
        'else if (f[2] == "person" && from_300(change) && !quoted(reason)) {',
        'else if (0) {',
    ),
    "the checker applies the rule to old ledgers": (
        CHECKER,
        "return n ~ /^[0-9]+$/ && n + 0 >= 300 }",
        "return 1 }",
    ),
    "the checker skips the rule everywhere": (
        CHECKER,
        "return n ~ /^[0-9]+$/ && n + 0 >= 300 }",
        "return 0 }",
    ),
    "one opening bracket is enough": (
        CHECKER,
        'if (open > 0 && index(substr(text, open + length("「")), "」") > 1) return 1',
        "if (open > 0) return 1",
    ),
    "an empty bracket quote is enough": (
        CHECKER,
        'index(substr(text, open + length("「")), "」") > 1)',
        'index(substr(text, open + length("「")), "」") > 0)',
    ),
    "one ascii quote mark is enough": (
        CHECKER,
        'return text ~ /"[^"]+"/',
        'return text ~ /"/',
    ),
    "an empty ascii quote is enough": (
        CHECKER,
        'return text ~ /"[^"]+"/',
        'return text ~ /"[^"]*"/',
    ),
    "the reason is cut at a tab again": (
        CHECKER,
        'rec = $0; sub(/^[^\\t]*\\t[^\\t]*\\t/, "", rec);',
        "rec = $3;",
    ),
    "the refusal does not name the quote": (
        CHECKER,
        'bad = "person 行には人の言葉の引用（',
        'bad = "person 行の理由が不足しています（',
    ),
    # Mutate the claim: the sentence a session reads.
    "approval.md drops the rule that a request counts only for what it asks": (
        RULE,
        "A request counts only for what it\nasks:",
        "A request counts for what it\nasks:",
    ),
    "approval.md drops the report back to the person": (
        RULE,
        "the session's final report says so in\nthe person's words",
        "the ledger line says so in\nthe person's words",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
