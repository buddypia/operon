#!/usr/bin/env python3
"""Mutation sweep for change 235. Runner: scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/large-file-viewer")

RUN_SECONDS = 1800

TESTS = [
    "a_text_file_past",
    "viewer_rows",
    "the_viewer_highlights",
    "a_large_markdown",
    "a_document_that_shrinks",
    "a_file_the_editor_cannot_edit",
    "a_changed_file_is_reported",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

MUTATIONS = {
    "a viewer document counts as modified, so it can be saved": (
        ROOT / "src/app.rs",
        "self.read_only.is_none() && self.viewer.is_none() && self.buffer != self.disk",
        "self.read_only.is_none() && self.buffer != self.disk",
    ),
    "a large text file is refused at the old editable ceiling": (
        ROOT / "src/git.rs",
        "if metadata.len() > VIEWER_FILE_MAX_BYTES as u64 {",
        "if metadata.len() > EDITOR_FILE_MAX_BYTES as u64 {",
    ),
    "a read leaves the previous read's viewer in place": (
        ROOT / "src/app.rs",
        "        document.viewer = None;\n        match result {",
        "        match result {",
    ),
    "a Markdown viewer document stays in Preview": (
        ROOT / "src/app/screens.rs",
        "if viewer.is_some() && self.editor_view == EditorView::Preview {",
        "if false && self.editor_view == EditorView::Preview {",
    ),
    "a long line is never wrapped onto further rows": (
        ROOT / "src/ui/syntax.rs",
        "if chars == VIEWER_ROW_MAX_CHARS {",
        "if chars == usize::MAX {",
    ),
    "a drawn chunk is highlighted again on every request": (
        ROOT / "src/ui/syntax.rs",
        "if !self.chunks.contains_key(&chunk) {",
        "if true {",
    ),
    "rows are drawn uncoloured": (
        ROOT / "src/ui/syntax.rs",
        "let job = syntax_layout(text.get(base..end).unwrap_or(\"\"), language, palette);",
        "let job = syntax_layout(text.get(base..end).unwrap_or(\"\"), EditorLanguage::Plain, palette);",
    ),
    "a row is drawn as tall as a control, not as a line": (
        ROOT / "src/app/screens.rs",
        "                        ui.add(\n                            egui::Label::new(layout.job.clone())\n                                .wrap_mode(egui::TextWrapMode::Extend),\n                        );",
        "                        ui.horizontal(|ui| {\n                            ui.add(\n                                egui::Label::new(layout.job.clone())\n                                    .wrap_mode(egui::TextWrapMode::Extend),\n                            );\n                        });",
    ),
    "claim: the notice says nothing about being read-only": (
        ROOT / "src/app/screens.rs",
        "\"{p0} KiB のファイルのため、読み取り専用で表示しています。編集できるのは {p1} KiB までです。\",\n                p0 = self.open_documents[index].disk.len() / 1024,",
        "\"{p0} KiB のファイルです。\",\n                p0 = self.open_documents[index].disk.len() / 1024,",
    ),
    "claim: the too-large message drops the way to open it": (
        ROOT / "src/git.rs",
        "右上の「既定のアプリで開く」から開けます。\",\n            p0",
        "\",\n            p0",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
