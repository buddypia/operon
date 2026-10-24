#!/usr/bin/env python3
"""The mutation sweep for change 224."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "the_reviewer_the_store_asks_for_reads_the_durability_procedure",
    "the_review_gate_requires_the_reviewers_review_md_names",
    "the_review_gate_refuses_what_it_says_it_refuses_by_route_and_base",
]
CHECKS = [
    ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", t]
    for t in TESTS
]

MUTATIONS = {
    "store-reviewer-is-not-told-to-read-the-procedure": (
        ROOT / ".claude/agents/rust-reviewer.md",
        "   `.claude/skills/durability-invariants/SKILL.md` and check the diff in the\n"
        "   order its **Reviewing a diff against this** section gives.",
        "   the durability guarantees.",
    ),
    "the-procedure-section-is-cut-from-the-skill": (
        ROOT / ".claude/skills/durability-invariants/SKILL.md",
        "## Reviewing a diff against this\n",
        "## Review\n",
    ),
    "the-store-goes-to-a-reviewer-that-does-not-read-it": (
        ROOT / "scripts/check-review.sh",
        "    store | bundle-swap) echo rust-reviewer ;;",
        "    store | bundle-swap) echo subprocess-safety-reviewer ;;",
    ),
    # The claim: REVIEW.md tells a person and a session who reviews the store.
    "review-md-still-names-the-retired-reviewer": (
        ROOT / "REVIEW.md",
        "against `.claude/skills/durability-invariants/SKILL.md` | `rust-reviewer` |",
        "against `.claude/skills/durability-invariants/SKILL.md` | `durability-reviewer` |",
    ),
    "a-full-route-still-asks-for-the-retired-reviewer": (
        ROOT / "scripts/check-review.sh",
        "  full) need rust-reviewer; need subprocess-safety-reviewer ;;",
        "  full) need rust-reviewer; need subprocess-safety-reviewer; need durability-reviewer ;;",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
