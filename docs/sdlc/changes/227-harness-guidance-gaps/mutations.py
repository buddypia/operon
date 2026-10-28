#!/usr/bin/env python3
"""Meaningful evidence and claim mutations for committed change receipts."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 600
CHECKS = [["cargo", "test", "--locked", "change_receipt"]]
SCRIPT = ROOT / "scripts/change-receipt.py"
MUTATIONS = {
    "worktree-bytes-replace-committed-evidence": (
        SCRIPT, 'return git("cat-file", "blob", object_id)', 'return open(path, "rb").read()'),
    "historical-claims-become-fresh-verification": (
        SCRIPT, '"committed records; not fresh verification"', '"freshly verified"'),
    "previous-snapshot-claims-whole-change-revert": (
        SCRIPT, '"previous snapshot only; not a whole-change revert"', '"whole change reverted"'),
    "missing-state-accepted": (
        SCRIPT, 'if records["state.yaml"] is None or not records["state.yaml"]["text"].strip():', 'if False:'),
    "oversized-records-accepted": (
        SCRIPT, 'MAX_BYTES = 64 * 1024', 'MAX_BYTES = 128 * 1024'),
    "git-replace-changes-provenance": (
        SCRIPT, 'GIT_NO_REPLACE_OBJECTS="1", ', ''),
    "missing-objects-invoke-promisor-transport": (
        SCRIPT, 'GIT_NO_LAZY_FETCH="1", ', ''),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
