#!/usr/bin/env python3
"""Verify that each requirement's guard fails when its behavior is broken."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "ide_261_"],
]
GIT = ROOT / "src/git.rs"
DIFF = ROOT / "src/ui/diff.rs"
UTIL = ROOT / "src/util.rs"
FILES = ROOT / "src/files.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    # R1: single-hunk stage applies in reverse instead of staging into the index.
    "single-hunk-stage-applies-reverse": (
        GIT,
        'apply_git_hunk_patch(worktree, &patch, &["--cached"])',
        'apply_git_hunk_patch(worktree, &patch, &["--reverse"])',
    ),
    # R2: diff pane omits per-hunk Stage and Discard buttons.
    "diff-pane-omits-hunk-buttons": (
        DIFF,
        "if line.kind == DiffLineKind::Hunk && !path.is_empty() {",
        "if false && line.kind == DiffLineKind::Hunk && !path.is_empty() {",
    ),
    # R3: code context handoff fails to bound snippets at MAX_CODE_CONTEXT_SNIPPET_BYTES.
    "code-context-unbounded-snippet": (
        UTIL,
        "        MAX_CODE_CONTEXT_SNIPPET_BYTES,\n    );",
        "        usize::MAX,\n    );",
    ),
    # R4: @ mention ranking omits the @diff candidate.
    "at-mention-omits-diff-token": (
        FILES,
        'if has_git_diff && (q.is_empty() || "diff".contains(&q)) {',
        'if false && has_git_diff && (q.is_empty() || "diff".contains(&q)) {',
    ),
    # R5: external IDE line-jump drops the -g goto flag for VS Code/Windsurf/Antigravity/Void.
    "external-ide-drops-goto-flag": (
        UTIL,
        'vec!["-g".to_string(), format!("{display}:{line}")]',
        'vec![format!("{display}:{line}")]',
    ),
    # R6: parallel arena branch naming drops the agent suffix and collides.
    "arena-branch-drops-agent-suffix": (
        UTIL,
        'format!("{prefix}-{suffix}")',
        "prefix.to_string()",
    ),
    # R7: command palette omits open-document symbols.
    "palette-omits-document-symbols": (
        SCREENS,
        "for sym in extract_file_symbols(&doc.path, &doc.buffer) {",
        "for sym in Vec::<CodeSymbol>::new() {",
    ),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
