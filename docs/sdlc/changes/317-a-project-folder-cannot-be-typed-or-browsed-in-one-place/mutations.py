#!/usr/bin/env python3
"""The mutation sweep for change 317: the in-app folder picker — a typed path
split into a folder and a partial name, a bounded listing, opening, creating
only after a confirmation, and the entries that open it. Runner:
scripts/mutation_sweep.py.
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
