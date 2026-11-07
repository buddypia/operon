#!/usr/bin/env python3
"""Mutation sweep for change 257: a conversation that proves an entry wrong
replaces it. Runner and rules: scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "knowledge_257_"]]

KNOWLEDGE = ROOT / "src/knowledge.rs"

MUTATIONS = {
    # Req 1, and the claim: what makes an entry replaceable.
    "prompt-never-lists-existing-entries": (
        KNOWLEDGE,
        "    if !known.is_empty() {\n        prompt.push_str(",
        "    if false {\n        prompt.push_str(",
    ),
    "prompt-lets-any-related-entry-be-replaced": (
        KNOWLEDGE,
        "the conversation shows that one of them is wrong or out of date, add",
        "an item is related to one of them, add",
    ),
    # Req 2.
    "parser-accepts-numbers-outside-the-list": (
        KNOWLEDGE,
        ".filter(|number| (1..=known).contains(number))",
        ".filter(|number| (0..=known + 1).contains(number))",
    ),
    "parser-off-by-one": (
        KNOWLEDGE,
        "                            .map(|number| number - 1),",
        "                            .map(|number| number),",
    ),
    # Req 3.
    "related-offers-pinned": (
        KNOWLEDGE,
        '"SELECT id, vector FROM entries WHERE pinned = 0"',
        '"SELECT id, vector FROM entries"',
    ),
    "related-offers-unrelated": (
        KNOWLEDGE,
        "            .filter(|(_, similarity)| *similarity >= KNOWLEDGE_SEARCH_MIN_COSINE)\n            .collect::<Vec<_>>();\n        scored.sort_by(|left, right| right.1.total_cmp(&left.1));\n        scored.truncate(KNOWLEDGE_JUDGE_KNOWN_ENTRIES);",
        "            .filter(|(_, similarity)| *similarity >= -1.0)\n            .collect::<Vec<_>>();\n        scored.sort_by(|left, right| right.1.total_cmp(&left.1));\n        scored.truncate(KNOWLEDGE_JUDGE_KNOWN_ENTRIES);",
    ),
    "related-unbounded": (
        KNOWLEDGE,
        "        scored.truncate(KNOWLEDGE_JUDGE_KNOWN_ENTRIES);\n        scored\n",
        "        scored\n",
    ),
    # Req 4.
    "replace-rewrites-pinned": (
        KNOWLEDGE,
        "                 WHERE id = ?1 AND pinned = 0\",",
        "                 WHERE id = ?1\",",
    ),
    "replace-keeps-the-old-confirmation": (
        KNOWLEDGE,
        "verified = ?5, vector = ?6,",
        "verified = MAX(verified, ?5), vector = ?6,",
    ),
    "replace-drops-the-new-source": (
        KNOWLEDGE,
        "        self.insert_source(id, source, now)?;\n        Ok(true)",
        "        Ok(true)",
    ),
    # Req 5.
    "run-never-offers": (
        KNOWLEDGE,
        "    let known = if database.entry_count()? > 0 && budget.spend(candidates.len()) {",
        "    let known = if false {",
    ),
    "run-never-replaces": (
        KNOWLEDGE,
        "                Some((index, id)) if database.replace_entry(id, &new_entry, &source, now)? => {",
        "                Some((index, id)) if false && database.replace_entry(id, &new_entry, &source, now)? => {",
    ),
    "run-replaces-one-entry-twice": (
        KNOWLEDGE,
        "                .filter(|index| !replaced.contains(index))\n",
        "",
    ),
    # Req 6.
    "run-ignores-the-budget": (
        KNOWLEDGE,
        "    let known = if database.entry_count()? > 0 && budget.spend(candidates.len()) {",
        "    let known = if database.entry_count()? > 0 {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
