#!/usr/bin/env python3
"""Break the keep-until-replaced rule of change 277."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "diff_277_"],
    ["cargo", "test", "--locked", "editor_diff_refreshes_after_a_second_write_with_unchanged_git_status"],
]
APP = ROOT / "src/app.rs"
KEEP = "if self.git_diff_cache.get(&key) != Some(&result) {"
MUTATIONS = {
    "refresh-discards-the-diff-again": (APP, "if let Some(file) = self.outdate_open_editor_diff(project.id) {", "if let Some(file) = self.invalidate_open_editor_diff(project.id) {"),
    "refresh-does-not-read-the-diff": (APP, "if let Some(file) = self.outdate_open_editor_diff(project.id) {", "if let Some(file) = self.outdate_open_editor_diff(project.id).filter(|_| false) {"),
    "older-read-answers-the-refresh": (APP, "        self.advance_git_diff_generation((project_id, Some(file.clone())));\n", ""),
    "unchanged-diff-is-parsed-again": (APP, KEEP, "if true {"),
    "changed-diff-is-not-shown": (APP, KEEP, "if self.git_diff_cache.get(&key).is_none() {"),
    "failed-read-is-not-shown": (APP, KEEP, "if self.git_diff_cache.get(&key) != Some(&result) && result.is_ok() {"),
    "choosing-the-view-keeps-the-old-diff": (APP, "        self.git_diff_cache.remove(&key);\n        self.diff_file_cache.remove(&key);\n        Some(file)\n", "        Some(file)\n"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
