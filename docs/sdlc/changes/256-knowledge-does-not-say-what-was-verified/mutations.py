#!/usr/bin/env python3
"""Mutation sweep for change 256: a knowledge entry says whether its
conversation confirmed it. Runner and rules: scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "knowledge_256_"]]

KNOWLEDGE = ROOT / "src/knowledge.rs"
TAB = ROOT / "src/app/knowledge.rs"

MUTATIONS = {
    # Req 1, and the claim: what counts as confirmed and what the flag is for.
    "prompt-stops-asking-for-verified": (
        KNOWLEDGE,
        '\\"verified\\": true | false, \\"candidate\\"',
        '\\"candidate\\"',
    ),
    "prompt-drops-what-the-flag-is-for": (
        KNOWLEDGE,
        "A later conversation relies on a verified entry \\\nwithout checking it again and checks an unverified one first, so when",
        "When",
    ),
    # Req 2.
    "parser-ignores-verified": (
        KNOWLEDGE,
        """                        verified: item
                            .get("verified")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),""",
        "                        verified: false,",
    ),
    # Req 3: a later unconfirmed repeat takes the confirmation back.
    "reinforce-downgrades": (
        KNOWLEDGE,
        "verified = MAX(verified, ?3)",
        "verified = ?3",
    ),
    "insert-drops-verified": (
        KNOWLEDGE,
        "VALUES (?1, ?2, ?3, ?4, ?4, ?4, 0, 0, 0, ?5, ?6)",
        "VALUES (?1, ?2, ?3, ?4, ?4, ?4, 0, 0, 0, 0, ?6)",
    ),
    # Req 4: an old database never gains the column.
    "old-database-not-migrated": (
        KNOWLEDGE,
        "        if !entries_have_verified(&connection).map_err(sql_error)? {",
        "        if false {",
    ),
    # Req 4, the race: the loser of two opens fails.
    "losing-alter-fails-the-open": (
        KNOWLEDGE,
        "        Err(_) if entries_have_verified(connection).unwrap_or(false) => Ok(()),\n",
        "",
    ),
    # Req 5.
    "run-drops-the-judges-flag": (
        KNOWLEDGE,
        "                    verified: entry.verified,",
        "                    verified: false,",
    ),
    # Req 6.
    "json-drops-verified": (
        KNOWLEDGE,
        '                    "verified": hit.entry.verified,\n',
        "",
    ),
    "text-always-says-unconfirmed": (
        KNOWLEDGE,
        "status = verified_label(hit.entry.verified)",
        "status = verified_label(false)",
    ),
    # Req 7, and the claim on screen: the words swapped.
    "row-loses-the-mark": (
        TAB,
        "    verified_mark(ui, palette, entry.verified);\n    let right_width",
        "    let right_width",
    ),
    "heading-loses-the-mark": (
        TAB,
        "                verified_mark(ui, palette, entry.verified);\n",
        "",
    ),
    "label-says-the-opposite": (
        KNOWLEDGE,
        '    if verified {\n        tr("検証済み")',
        '    if !verified {\n        tr("検証済み")',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
