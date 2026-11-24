#!/usr/bin/env python3
"""The mutation sweep for change 317: the in-app folder picker — a typed path
split into a folder and a partial name, a bounded listing, opening, creating
only after a confirmation, and the entries that open it. Runner:
scripts/mutation_sweep.py.

Part 1 of 3: CI gives one sweep 900 seconds, and thirty mutations in one
file ran past it. The three files hold ten each.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = ["folder_317_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

FP = ROOT / "src/app/folder_picker.rs"
LP = ROOT / "src/app/launch_projects.rs"
KEYS = ROOT / "src/app/keymap.rs"

MUTATIONS = {
    # R1: `~name` is read as the home folder.
    "tilde-name-is-home": (
        FP,
        '    } else if let Some(rest) = text.strip_prefix("~/") {',
        '    } else if let Some(rest) = text.strip_prefix("~") {',
    ),
    # R2: files are listed beside folders.
    "files-are-listed": (FP, "                if is_folder {", "                if true {"),
    # R2: hidden folders are always shown.
    "hidden-always-shown": (
        FP,
        "                if name.starts_with('.') && !needle.starts_with('.') {",
        "                if false && name.starts_with('.') {",
    ),
    # R2: the listing is in the order the disk gave.
    "listing-unsorted": (
        FP,
        "            names.sort_by_cached_key(|name| name.to_lowercase());",
        "            names.sort_by_cached_key(|name| std::cmp::Reverse(name.to_lowercase()));",
    ),
    # R3: the ceiling is not kept.
    "ceiling-not-kept": (FP, "                if read == limit {", "                if read == usize::MAX {"),
    # R3, the claim: a listing that stopped early is shown as complete.
    "truncation-not-reported": (
        FP,
        "                    truncated = true;\n                    break;",
        "                    break;",
    ),
    # R3, the claim on screen: the sentence under a cut listing is gone.
    "truncation-sentence-dropped": (
        FP,
        "                picker.note = listing.truncated.then(|| {",
        "                picker.note = false.then(|| {",
    ),
    # R4: a file is taken for a missing path, which the sheet offers to create.
    "file-read-as-missing": (
        FP,
        "        Ok(_) => FolderTarget::NotAFolder,",
        "        Ok(_) => FolderTarget::Missing,",
    ),
    # R5: the sheet's entry opens the picker for the projects page.
    "launch-entry-opens-projects-picker": (
        LP,
        "        self.open_folder_picker(FolderPickerTarget::LaunchSheet);",
        "        self.open_folder_picker(FolderPickerTarget::Projects);",
    ),
    # R6/R7: opened from the sheet, the folder goes to the projects page.
    "launch-target-opens-the-page": (
        FP,
        "            FolderPickerTarget::LaunchSheet => self.apply_launch_project_folder(Some(path)),",
        "            FolderPickerTarget::LaunchSheet => {\n                self.add_project_path(path);\n            }",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
