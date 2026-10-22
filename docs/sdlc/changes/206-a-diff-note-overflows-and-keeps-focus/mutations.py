#!/usr/bin/env python3
"""Change 206's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/206-a-diff-note-overflows-and-keeps-focus")

RUN_SECONDS = 1800

TESTS = [
    "tests::a_note_on_a_changed_line_says_so_from_the_per_file_review",
    "tests::a_long_diff_note_wraps_and_pushes_the_next_line_down",
    "tests::the_note_editor_does_not_take_the_keyboard_back",
    "tests::offsets_every_diff_row_by_the_height_of_the_rows_above_it",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

APP = ROOT / "src/app.rs"
DIFF = ROOT / "src/ui/diff.rs"
COMMENTS = ROOT / "src/git/comments.rs"
MUTATIONS = {
    "staleness reads only the whole-project parse": (
        APP,
        "[Some(comment.file.clone()), None]",
        "[None, None]",
    ),
    "the copy reads notes its own way again": (
        APP,
        "        let annotated = self.annotated_diff_comments(project, false);\n",
        "        let annotated: Vec<(DiffComment, bool)> = Vec::new();\n",
    ),
    "a note's row ignores how tall it drew": (
        DIFF,
        "note_height(file, line).unwrap_or(line_height * DIFF_COMMENT_ROW_LINES)",
        "line_height * DIFF_COMMENT_ROW_LINES",
    ),
    "a drawn note's height is never kept": (
        DIFF,
        "                                .insert((project, number), drawn);\n",
        "                                .insert((project, number), 0.0);\n",
    ),
    "a note being read runs on one line": (
        DIFF,
        "note.label(body_text);",
        "note.horizontal(|ui| ui.label(body_text));",
    ),
    "the editor takes the keyboard every frame": (
        DIFF,
        "if std::mem::take(&mut annotations.focus_pending) {",
        "if true {",
    ),
    "the editor never takes the keyboard": (
        COMMENTS,
        "        self.focus_pending = true;\n",
        "",
    ),
    "a deleted note keeps its height": (
        COMMENTS,
        "            !heights.is_empty()\n        });",
        "            true\n        });\n        let _ = comments;",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
