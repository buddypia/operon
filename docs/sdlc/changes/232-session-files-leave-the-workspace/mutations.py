#!/usr/bin/env python3
"""Bounded, assertion-backed guards for mixed session/document workspace tabs."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 120
TESTS = [
    "workspace_tabs_",
    "session_file_opens_in_central_workspace",
    "workspace_hidden_session_is_not_polled_or_read",
    "workspace_dirty_document_close_is_refused",
    "pane_drop_hint_names_the_zone",
    "workspace_visible_documents_do_not_reparse_unchanged_frames",
    "workspace_line_jump_waits_for_its_target_document",
    "workspace_escape_and_release_over_tab_cancel_reordering",
    "workspace_split_headers_keep_long_metadata_clear_of_controls",
    "workspace_worktree_document_tab_keeps_its_branch",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
MODEL = ROOT / "src/ui/session_tree.rs"
APP = ROOT / "src/app.rs"
SCREEN = ROOT / "src/app/screens.rs"
MUTATIONS = {
    "reopening-duplicates-document-tab": (
        MODEL, ".find(|group| group.tabs.contains(&tab))", ".find(|_| false)"),
    "edge-drop-joins-instead-of-splitting": (
        MODEL, "if drop == PaneDrop::Center {\n            let index = destination.tabs.len();",
        "if true {\n            let index = destination.tabs.len();"),
    "closing-last-tab-keeps-empty-group": (
        MODEL, "self.groups.retain(|group| !group.tabs.is_empty());", "self.groups.retain(|_| true);"),
    "too-small-split-is-accepted": (
        MODEL, "if !layout.fits(area) {", "if false && !layout.fits(area) {"),
    "hidden-session-is-treated-as-visible": (
        MODEL, "group.tabs.get(group.active).cloned()\n            })", "group.tabs.first().cloned()\n            })"),
    "file-navigation-leaves-session-page": (
        APP, "if self.page != Page::Sessions {\n            self.page = Page::Projects;",
        "if true {\n            self.page = Page::Projects;"),
    "split-header-repeats-unbounded-metadata": (
        SCREEN, "if split {\n                        // The tab already names the session.",
        "if false && split {\n                        // The tab already names the session."),
    "syntax-cache-is-shared-between-visible-documents": (
        SCREEN, "std::mem::swap(&mut self.editor_text_layout_cache, &mut caches.0);", ""),
    "markdown-cache-is-shared-between-visible-documents": (
        SCREEN, "std::mem::swap(&mut self.markdown_preview_cache, &mut caches.1);", ""),
    "line-jump-is-spent-by-another-document": (
        SCREEN, "self.workspace_editor_jumps.remove(&id)",
        "self.workspace_editor_jumps.drain().next().map(|(_, line)| line)"),
    "escape-release-still-reorders-tab": (
        SCREEN, "&& !i.key_pressed(egui::Key::Escape)", ""),
    "worktree-document-tab-hides-its-branch": (
        SCREEN, "name.push_str(branch);", "let _ = branch;"),
    "right-drop-claims-left-split": (
        MODEL, 'Self::Right => tr("右に分割"),', 'Self::Right => tr("左に分割"),'),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
