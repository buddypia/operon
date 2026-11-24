#!/usr/bin/env python3
"""The mutation sweep for change 317: the in-app folder picker — a typed path
split into a folder and a partial name, a bounded listing, opening, creating
only after a confirmation, and the entries that open it. Runner:
scripts/mutation_sweep.py.

Part 3 of 3: CI gives one sweep 900 seconds, and thirty mutations in one
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
    # R13: refused text is asked about anyway.
    "refused-text-is-listed": (
        FP,
        "        if picker.refused || picker.listing.is_some() {",
        "        if picker.listing.is_some() {",
    ),
    # R14: a drop opens the folder instead of typing it.
    "drop-bypasses-the-picker": (
        LP,
        "        if self.folder_picker.is_some() {\n            self.fill_folder_picker_from_drop(&dropped);\n            return;\n        }\n",
        "",
    ),
    # R15, the claim "never opens in a guarded folder".
    "opens-in-a-guarded-folder": (
        FP,
        "        .filter(|parent| !is_in_protected_folder(parent, home));",
        "        .filter(|_| true);",
    ),
    # R16: the picker opens over a launch in progress.
    "opens-over-a-progress-modal": (
        FP,
        "        if launching || self.restore_progress.is_some() || self.folder_picker.is_some() {",
        "        if self.restore_progress.is_some() || self.folder_picker.is_some() {",
    ),
    # R16: ⌘K and the page shortcuts reach past the picker.
    "shortcuts-reach-past-the-picker": (
        KEYS,
        "            || self.folder_picker.is_some();",
        "            || false;",
    ),
    # F1: Tab on a bare `~` writes `~name/` again.
    "tilde-completion-loses-its-slash": (
        FP,
        'let slash = if stem.ends_with(\'/\') { "" } else { "/" };',
        'let slash = "";',
    ),
    # F2: any path the disk will not answer for is offered for creation.
    "unanswered-path-read-as-missing": (
        FP,
        "        Err(_) => FolderTarget::Unreadable,\n    };",
        "        Err(_) => FolderTarget::Missing,\n    };",
    ),
    # F3: the Enter that confirms a conversion opens the folder.
    "ime-enter-opens": (
        FP,
        "        let (step, composing) = ctx.input(|input| find_bar_step(&input.events, picker.composing));",
        "        let (step, composing) = ctx.input(|input| (input.key_pressed(egui::Key::Enter).then_some(true), find_bar_step(&input.events, picker.composing).1));",
    ),
    # From review: Tab completes from a listing cut at the ceiling.
    "tab-completes-a-cut-listing": (
        FP,
        "        if listing.truncated {\n            return;\n        }\n",
        "",
    ),
    # The claim in the sentence under the field: a missing path says nothing.
    "missing-path-says-nothing": (
        FP,
        'FolderTarget::Missing => Some(tr("このフォルダはまだありません。作成して開けます。")),',
        "FolderTarget::Missing => None,",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
