#!/usr/bin/env python3
"""Break shell continuation and mid-run auto-restore for dead panes (change 288)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "pane_288_"],
]

APP = ROOT / "src/app.rs"
RELAUNCH = ROOT / "src/app/relaunch.rs"
TMUX = ROOT / "src/tmux.rs"

MUTATIONS = {
    "dead-pane-banner-format-not-cleared": (
        TMUX,
        '        TmuxStep::required(["set-option", "-q", "-t", name, "remain-on-exit-format", ""]),',
        '        TmuxStep::required(["set-option", "-q", "-t", name, "remain-on-exit", "on"]),',
    ),
    "dead-pane-scrollback-not-pushed-to-history": (
        TMUX,
        '    steps.push(TmuxStep::best_effort(["send-keys", "-R", "-t", name]));',
        '    if false {\n        steps.push(TmuxStep::best_effort(["send-keys", "-R", "-t", name]));\n    }',
    ),
    "dead-pane-killed-instead-of-respawned": (
        TMUX,
        "        TmuxState::Dead => Ok(PlainTerminalAction::RespawnDead(dead_pane_reopen_steps(\n"
        "            name,\n"
        "            cwd,\n"
        "            environment,\n"
        "        ))),",
        "        TmuxState::Dead => {\n"
        "            let _ = (name, cwd, environment);\n"
        "            Ok(PlainTerminalAction::StartNew)\n"
        "        }",
    ),
    "run-script-converted-to-shell-on-exit": (
        RELAUNCH,
        '        "terminal" | "existing terminal" | "claude" | "codex" | "gemini" | "custom"',
        '        "terminal" | "existing terminal" | "claude" | "codex" | "gemini" | "custom" | "script"',
    ),
    "exited-open-tabs-not-reopened-on-load": (
        RELAUNCH,
        "    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {\n        if !self.tools.tmux {\n            return;\n        }",
        "    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {\n        if self.tools.tmux {\n            return;\n        }",
    ),
    "mid-run-forced-kill-not-restored": (
        APP,
        "            let forced_restoring =\n                !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
        "            let forced_restoring =\n                false && !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
    ),
    "single-restore-toast-says-wrong-sentence": (
        RELAUNCH,
        '                [name] => self.notice_briefly(tf!(\n                    "強制終了されたセッションを自動で復元しました: {name}",\n                    name = name\n                )),',
        '                [name] => self.notice_briefly(tf!(\n                    "セッションが完了しました: {name}",\n                    name = name\n                )),',
    ),
    "multi-restore-toast-says-wrong-sentence": (
        RELAUNCH,
        '                many => self.notice_briefly(tf!(\n                    "強制終了されたセッションを {count} 件自動で復元しました。",\n                    count = many.len()\n                )),',
        '                many => self.notice_briefly(tf!(\n                    "前回開いていたセッションを {count} 件再開しました。",\n                    count = many.len()\n                )),',
    ),
    "auto-restore-cooldown-not-enforced": (
        APP,
        "            let forced_restoring =\n                !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
        "            let forced_restoring =\n                !first_look && can_auto_restore && (self.auto_restore_ready(id, poll_now) || true);",
    ),
    "quiet-launch-on-screen-skips-output-refresh": (
        APP,
        "                            if quiet && self.is_session_on_screen(session_id) {\n                                self.request_session_output(session_id);\n                            }",
        "                            if false && quiet && self.is_session_on_screen(session_id) {\n                                self.request_session_output(session_id);\n                            }",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
