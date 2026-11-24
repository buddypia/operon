#!/usr/bin/env python3
"""The mutation sweep for change 317: the in-app folder picker — a typed path
split into a folder and a partial name, a bounded listing, opening, creating
only after a confirmation, and the entries that open it. Runner:
scripts/mutation_sweep.py.

Part 2 of 3: CI gives one sweep 900 seconds, and thirty mutations in one
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
    # R7: the launch sheet is drawn under the picker.
    "sheet-drawn-under-the-picker": (
        LP,
        "            && self.folder_picker.is_none()\n",
        "",
    ),
    # R8, the claim "only after confirmation": the first press creates.
    "first-press-creates": (
        FP,
        "        if picker.question.is_some() {\n            self.confirm_folder_creation();\n            return;\n        }",
        "        if picker.listing.as_ref().is_some_and(|listing| listing.target == FolderTarget::Missing) {\n            picker.question = Some(String::new());\n            self.confirm_folder_creation();\n            return;\n        }",
    ),
    # R8: editing the path keeps the confirmation.
    "edit-keeps-the-question": (
        FP,
        "        picker.note = None;\n        picker.question = None;",
        "        picker.note = None;",
    ),
    # R8: the sheet can be closed while the folder is being made.
    "closes-while-creating": (
        FP,
        "            .is_some_and(|picker| !picker.creating)",
        "            .is_some()",
    ),
    # R9: Tab closes a prefix several names share with `/`.
    "tab-overshoots": (
        FP,
        "    if names.len() == 1 {\n        return Some(format!(\"{first}/\"));\n    }",
        "    if !names.is_empty() {\n        return Some(format!(\"{first}/\"));\n    }",
    ),
    # R9: 「上のフォルダへ」 stays where it is.
    "up-goes-nowhere": (
        FP,
        "        let text = folder_input_text(parent, home);",
        "        let text = folder_input_text(&input.dir, home);",
    ),
    # R10: a listing for other text is shown.
    "stale-listing-shown": (
        FP,
        "            Some(listing) if listing.input == picker.input => {",
        "            Some(listing) => {",
    ),
    # R10: after a stale listing the current text is not asked.
    "current-text-not-asked": (
        FP,
        "            _ => self.request_folder_listing(),",
        "            _ => {}",
    ),
    # R11: Esc closes the picker over a pending confirmation.
    "escape-skips-the-question": (
        FP,
        "                act = Some(if asking {\n                    FolderPickerAct::Withdraw",
        "                act = Some(if false {\n                    FolderPickerAct::Withdraw",
    ),
    # R12: 「Finder で選ぶ…」 does nothing.
    "finder-button-dead": (
        FP,
        "            Some(FolderPickerAct::Finder) => self.choose_folder_in_finder(),",
        "            Some(FolderPickerAct::Finder) => {}",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
