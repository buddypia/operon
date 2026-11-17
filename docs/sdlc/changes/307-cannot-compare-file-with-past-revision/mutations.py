#!/usr/bin/env python3
"""Change 307's mutation sweep: each mutation breaks one claim of the
side-by-side diff, the revision comparison, the annotate view, or the
remembered layout, and one of the diff_307 tests must go red. Runner:
scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = ["diff_307_", "model_check_diff_307_"]
CHECKS = [
    ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]
]

GIT = ROOT / "src" / "git.rs"
DIFF = ROOT / "src" / "ui" / "diff.rs"
STORE = ROOT / "src" / "store.rs"
MUTATIONS = {
    # An old store no longer loads once the field is required.
    "layout-field-not-defaulted": (
        STORE,
        "    #[serde(default)]\n    pub(crate) diff_layout: DiffLayout,\n",
        "    pub(crate) diff_layout: DiffLayout,\n",
    ),
    # Anything, an option included, reaches git as a revision.
    "revision-not-checked": (
        GIT,
        "    if !is_commit_id(commit) {\n        anyhow::bail!(tr(\"無効なコミット SHA です\"));\n    }\n    arguments.extend",
        "    arguments.extend",
    ),
    # Every annotated line repeats its commit in the gutter.
    "every-line-starts-a-run": (
        GIT,
        "let starts_run = lines.last().is_none_or(|last| last.commit != commit);",
        "let starts_run = true;",
    ),
    # 次の変更 runs past the last change.
    "next-change-unbounded": (
        DIFF,
        "Some(at) if forward => (at + 1).min(last),",
        "Some(at) if forward => at + 1,",
    ),
    # A narrow pane squeezes two columns anyway.
    "no-narrow-fallback": (
        DIFF,
        "if chosen == DiffLayout::SideBySide && width < DIFF_SIDE_BY_SIDE_MIN_WIDTH {",
        "if chosen == DiffLayout::SideBySide && width < 0.0 {",
    ),
    # A blame past its ceiling is shown in part.
    "too-large-shown-partially": (
        GIT,
        "        Ok((_, true)) => Ok(BlameOutcome::TooLarge),\n",
        "        Ok((output, true)) => Ok(BlameOutcome::Lines {\n"
        "            lines: Arc::new(parse_blame_porcelain(&output)),\n"
        "            widest: 0,\n"
        "        }),\n",
    ),
    # Side by side, a past revision offers stage and discard.
    "side-by-side-hunk-actions-ungated": (
        DIFF,
        "                            if editable && line.kind == DiffLineKind::Hunk {",
        "                            if line.kind == DiffLineKind::Hunk {",
    ),
    # Interleaved, a past revision offers stage and discard.
    "unified-hunk-actions-ungated": (
        DIFF,
        "if editable && line.kind == DiffLineKind::Hunk && !path.is_empty() {",
        "if line.kind == DiffLineKind::Hunk && !path.is_empty() {",
    ),
    # A rewrite's longer side loses its remainder.
    "pairing-drops-remainder": (
        DIFF,
        "for offset in 0..removed.max(added) {",
        "for offset in 0..removed.min(added) {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
