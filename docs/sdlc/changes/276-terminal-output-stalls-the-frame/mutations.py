#!/usr/bin/env python3
"""Break the off-frame terminal layout of change 276."""
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "terminal_276_"]]
APP = ROOT / "src/app.rs"
OUT = ROOT / "src/app/terminal_output.rs"
MUTATIONS = {
    "frame-lays-out-anyway": (OUT, "            }) if laid_out_under == theme => {", "            }) if laid_out_under == theme && false => {"),
    "other-themes-rows-are-shown": (OUT, "            }) if laid_out_under == theme => {", "            }) if laid_out_under == theme || true => {"),
    "unchanged-text-is-laid-out": (OUT, "(known != Some(fingerprint))", "(known != Some(fingerprint) || true)"),
    "changed-text-is-not-laid-out": (OUT, "(known != Some(fingerprint))", "(known.is_none())"),
    "fingerprint-not-recorded": (OUT, "                self.terminal_output_fingerprints\n                    .insert(session_id, fingerprint);\n", ""),
    "fingerprint-outlives-its-rows": (OUT, "                self.terminal_output_fingerprints.remove(&session_id);\n", ""),
    "rows-copied-beside-themselves": (OUT, "        self.terminal_fallback_layouts.remove(&session_id);\n", "        self.terminal_fallback_layouts.insert(session_id, layouts.clone());\n"),
    "fallback-outlives-the-next-capture": (OUT, "        self.terminal_fallback_layouts.remove(&session_id);\n", ""),
    "theme-switch-blanks-the-pane": (APP, "            self.terminal_fallback_layouts\n                .extend(self.terminal_layouts.drain());\n", "            self.terminal_layouts.clear();\n"),
    "theme-switch-keeps-old-rows-current": (APP, "            self.terminal_fallback_layouts\n                .extend(self.terminal_layouts.drain());\n", ""),
    "theme-switch-keeps-fingerprints": (APP, "            self.terminal_output_fingerprints.clear();\n", ""),
    "unchanged-trusted-over-the-text": (OUT, "            .is_some_and(|existing| *existing == output)\n            && self.terminal_layouts.contains_key(&session_id)", "            .is_some_and(|existing| *existing == output || matches!(&prelaid, Some(LaidOutTerminal { layouts: None, .. })))\n            && self.terminal_layouts.contains_key(&session_id)"),
    "rows-kept-without-a-layout": (OUT, "            && self.terminal_layouts.contains_key(&session_id)\n", ""),
    "held-rows-never-get-a-fingerprint": (OUT, "                self.terminal_output_fingerprints\n                    .insert(session_id, laid_out.fingerprint);\n", ""),
    "poll-stash-is-not-emptied": (APP, "                    self.terminal_prelaid.clear();\n", ""),
    "output-stash-ignored": (APP, "                                self.terminal_prelaid.insert(session_id, laid_out);\n", "                                let _ = laid_out;\n"),
}
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep
mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
