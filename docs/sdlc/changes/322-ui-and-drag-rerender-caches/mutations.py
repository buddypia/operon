#!/usr/bin/env python3
"""Break each R1-R6 drag deferral and derived-state cache invariant (change 322)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
TESTS = ["perf_322_", "model_check_perf_322_"]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

TERMINAL_OUTPUT = ROOT / "src/app/terminal_output.rs"
SESSION_TREE = ROOT / "src/ui/session_tree.rs"
SESSION_ORDER = ROOT / "src/app/session_order.rs"
DIFF = ROOT / "src/ui/diff.rs"
SYNTAX = ROOT / "src/ui/syntax.rs"
MARKDOWN_VIEW = ROOT / "src/ui/markdown_view.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"
FILES = ROOT / "src/ui/files.rs"
SCREENS = ROOT / "src/app/screens.rs"

MUTATIONS = {
    "r1-drag-deferral-dropped": (
        TERMINAL_OUTPUT,
        "    pub(crate) fn flush_terminal_resize(&mut self, session_id: Uuid) -> Option<Duration> {\n        if self.is_layout_dragging() {\n            return None;\n        }",
        "    pub(crate) fn flush_terminal_resize(&mut self, session_id: Uuid) -> Option<Duration> {\n        if false && self.is_layout_dragging() {\n            return None;\n        }",
    ),
    "r1-can-move-tab-wrong-center": (
        SESSION_TREE,
        "        if drop == PaneDrop::Center {\n            if source.id == target {\n                let from = source.tabs.iter().position(|candidate| candidate == tab);\n                return from.is_some_and(|index| index != destination.tabs.len() - 1);\n            }\n            return true;\n        }",
        "        if drop == PaneDrop::Center {\n            return true;\n        }",
    ),
    "r2-workspace-sync-always-runs": (
        SESSION_ORDER,
        "        if self.workspace_sync_fingerprint == Some(fp) {\n            return;\n        }",
        "        if false && self.workspace_sync_fingerprint == Some(fp) {\n            return;\n        }",
    ),
    "r2-sidebar-cache-always-rebuilds": (
        SESSION_ORDER,
        "        if self.sidebar_view_cache.fingerprint == Some(fp) {\n            self.sidebar_view_cache.hits += 1;\n            return Arc::clone(&self.sidebar_view_cache.data);\n        }",
        "        if false && self.sidebar_view_cache.fingerprint == Some(fp) {\n            self.sidebar_view_cache.hits += 1;\n            return Arc::clone(&self.sidebar_view_cache.data);\n        }",
    ),
    "r3-diff-rows-always-rebuilt": (
        DIFF,
        "        if rc.rows_fp != Some(rows_fp) {",
        "        if true || rc.rows_fp != Some(rows_fp) {",
    ),
    "r4-editor-galley-bypassed": (
        SYNTAX,
        "        if let Some((cached_ppp, cached_width, galley)) = &self.layout.syntax_galley {\n            if *cached_ppp == ppp_bits && *cached_width == width_bits {",
        "        if let Some((cached_ppp, cached_width, galley)) = &self.layout.syntax_galley {\n            if false && *cached_ppp == ppp_bits && *cached_width == width_bits {",
    ),
    "r5-markdown-viewport-cull-disabled": (
        MARKDOWN_VIEW,
        "    let cull = document.blocks.len() > MARKDOWN_VIEWPORT_CULL_THRESHOLD;",
        "    let cull = false;",
    ),
    "r6-png-cache-miss": (
        WIDGETS,
        "    if let Ok(guard) = map.lock() {\n        if let Some(existing) = guard.get(name) {\n            return Some(Arc::clone(existing));\n        }\n    }",
        "    if let Ok(guard) = map.lock() {\n        let _ = guard.get(name);\n    }",
    ),
    "r6-tree-cache-miss": (
        FILES,
        "        let fp = h.finish();\n        if self.fingerprint == Some(fp) {\n            self.hits += 1;\n            return;\n        }",
        "        let fp = h.finish();\n        if false && self.fingerprint == Some(fp) {\n            self.hits += 1;\n            return;\n        }",
    ),
    "r6-palette-cache-miss": (
        SCREENS,
        "        let fp = self.compute_palette_fingerprint();\n        if self.palette_cache.fingerprint == Some(fp) {",
        "        let fp = self.compute_palette_fingerprint();\n        if false && self.palette_cache.fingerprint == Some(fp) {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
