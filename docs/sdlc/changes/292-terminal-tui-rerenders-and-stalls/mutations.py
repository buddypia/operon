#!/usr/bin/env python3
"""Mutation sweep for change 292: event-driven tmux -C stream, coalescing, incremental ANSI layout, row galley cache, and dirty-line negative-path TTL."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    [
        "cargo",
        "test",
        "--locked",
        "--manifest-path",
        str(ROOT / "Cargo.toml"),
        "--",
        "tui_perf_292_",
    ],
    [
        "cargo",
        "test",
        "--locked",
        "--manifest-path",
        str(ROOT / "Cargo.toml"),
        "--",
        "model_check_tui_292_",
    ],
]

EXEC = ROOT / "src/exec.rs"
TMUX = ROOT / "src/tmux.rs"
UI_TERM = ROOT / "src/ui/terminal.rs"
APP_TERM = ROOT / "src/app/terminal_output.rs"

MUTATIONS = {
    "watcher-rejects-unsafe-tmux-name-bypassed": (
        TMUX,
        "pub(crate) fn tmux_output_watcher_args(name: &str) -> Result<[&str; 5]> {\n    if !is_safe_tmux_name(name) {",
        "pub(crate) fn tmux_output_watcher_args(name: &str) -> Result<[&str; 5]> {\n    if false && !is_safe_tmux_name(name) {",
    ),
    "watcher-blocks-send-keys-with-read-only-flag": (
        TMUX,
        '    Ok(["-C", "attach-session", "-t", name, "-fignore-size"])',
        '    Ok(["-C", "attach-session", "-t", name, "-fread-only,ignore-size"])',
    ),
    "streaming-child-leaks-inherited-git-dir": (
        EXEC,
        "pub(crate) fn spawn_piped_streaming(command: &mut Command) -> Result<StreamingChild> {\n    command\n        .stdin(Stdio::piped())\n        .stdout(Stdio::piped())\n        .stderr(Stdio::null());\n    for pointer in INHERITED_REPOSITORY_POINTERS {\n        command.env_remove(pointer);\n    }",
        "pub(crate) fn spawn_piped_streaming(command: &mut Command) -> Result<StreamingChild> {\n    command\n        .stdin(Stdio::piped())\n        .stdout(Stdio::piped())\n        .stderr(Stdio::null());\n    for pointer in INHERITED_REPOSITORY_POINTERS {\n        let _ = pointer;\n    }",
    ),
    "coalesce-delay-ignores-frame-budget": (
        APP_TERM,
        "        Some(last) => budget.saturating_sub(now.saturating_duration_since(last)),",
        "        Some(last) => {\n            let _ = (last, now, budget);\n            Duration::ZERO\n        }",
    ),
    "live-watcher-fails-to-suppress-blind-echo-poll": (
        APP_TERM,
        "        if self.terminal_streams.is_watching(session_id) {\n            return None;\n        }",
        "        if false && self.terminal_streams.is_watching(session_id) {\n            return None;\n        }",
    ),
    "respawn-cooldown-disabled-fork-storms-missing-session": (
        APP_TERM,
        "                .is_some_and(|at| now.saturating_duration_since(*at) < SESSION_POLL_INTERVAL)",
        "                .is_some_and(|at| {\n                    let _ = (at, now);\n                    false\n                })",
    ),
    "unchanged-snapshot-not-dropped": (
        APP_TERM,
        "        if prev.cursor == capture.cursor {\n            return None;\n        }",
        "        if false && prev.cursor == capture.cursor {\n            return None;\n        }",
    ),
    "cursor-only-delta-forces-full-relayout": (
        APP_TERM,
        "    let text_unchanged = last_delivered.as_ref().is_some_and(|prev| {\n        prev.theme == theme && prev.fingerprint == fingerprint && prev.output == capture.output\n    });",
        "    let text_unchanged = last_delivered.as_ref().is_some_and(|prev| {\n        let _ = (prev, theme, fingerprint);\n        false\n    });",
    ),
    "incremental-ansi-ignores-incoming-sgr-style": (
        UI_TERM,
        "                entry.style_in == style\n                    && entry.raw_hash == raw_hash\n                    && entry.raw.as_ref() == clean_line.as_ref()",
        "                entry.raw_hash == raw_hash\n                    && entry.raw.as_ref() == clean_line.as_ref()",
    ),
    "row-galley-cache-disabled-reshapes-every-visible-row": (
        UI_TERM,
        "                    if let Some(cached) = self.row_galleys.get(&rk) {",
        "                    if let Some(cached) = None::<&Arc<egui::Galley>>.or_else(|| {\n                        let _ = &rk;\n                        None\n                    }) {",
    ),
    "static-negative-path-evicted-every-streaming-tick": (
        APP_TERM,
        "            if now.saturating_duration_since(probed_at) >= TERMINAL_NEGATIVE_PATH_TTL {",
        "            if now.saturating_duration_since(probed_at) >= Duration::ZERO {",
    ),
    "dirty-line-dynamic-file-stuck-in-negative-cache": (
        APP_TERM,
        "                Some(indices) => !indices.iter().any(|&idx| {\n                    layouts_ref\n                        .get(idx)\n                        .is_some_and(|job| job.text.contains(path.as_str()))\n                }),",
        "                Some(indices) => {\n                    let _ = (indices, layouts_ref, path);\n                    true\n                }",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
