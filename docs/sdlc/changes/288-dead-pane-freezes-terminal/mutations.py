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
    "dead-pane-env-not-updated-before-respawn": (
        TMUX,
        "    let mut steps = tmux_session_environment_steps(name, environment);",
        "    let mut steps = if false {\n        tmux_session_environment_steps(name, environment)\n    } else {\n        Vec::new()\n    };",
    ),
    "exited-open-tabs-not-reopened-on-load": (
        RELAUNCH,
        "    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {\n        if !self.tools.tmux {\n            return;\n        }",
        "    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {\n        if self.tools.tmux {\n            return;\n        }",
    ),
    "closed-exited-history-also-reopened-on-load": (
        RELAUNCH,
        "                session.status == SessionStatus::Exited\n                    && open_tabs.contains(&session.id)",
        "                session.status == SessionStatus::Exited\n                    && (open_tabs.contains(&session.id) || true)",
    ),
    "clean-exit-keeps-stale-native-session-id": (
        RELAUNCH,
        "            stored.native_session_id = None;\n            stored.native_session_path = None;",
        "            let _ = (&stored.native_session_id, &stored.native_session_path);",
    ),
    "run-script-converted-to-shell-on-exit": (
        RELAUNCH,
        '        "terminal" | "existing terminal" | "claude" | "codex" | "gemini" | "custom"',
        '        "terminal" | "existing terminal" | "claude" | "codex" | "gemini" | "custom" | "script"',
    ),
    "dependency-parent-converted-to-shell-before-child": (
        APP,
        "                && is_interactive_terminal_session(&self.store.sessions[idx])\n                && queued_dependency_count(&self.store.sessions, id) == 0;",
        "                && is_interactive_terminal_session(&self.store.sessions[idx]);",
    ),
    "mid-run-forced-kill-not-restored": (
        APP,
        "            let forced_restoring =\n                !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
        "            let forced_restoring =\n                false && !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
    ),
    "auto-restore-cooldown-not-enforced": (
        APP,
        "            let forced_restoring =\n                !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
        "            let forced_restoring =\n                !first_look && can_auto_restore && (self.auto_restore_ready(id, poll_now) || true);",
    ),
    "manual-start-does-not-clear-cooldown": (
        APP,
        "    pub(crate) fn start_session(&mut self, session_id: Uuid) {\n        self.clear_auto_restore_cooldown(session_id);",
        "    pub(crate) fn start_session(&mut self, session_id: Uuid) {",
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
    "stale-managed-env-not-unset-on-respawn": (
        TMUX,
        "    for managed in MANAGED_TMUX_ENV_KEYS {",
        "    for managed in MANAGED_TMUX_ENV_KEYS.into_iter().take(0) {",
    ),
    "quiet-launch-on-screen-skips-output-refresh": (
        APP,
        "                            if quiet && self.is_session_on_screen(session_id) {\n                                self.request_session_output(session_id);\n                            }",
        "                            if false && quiet && self.is_session_on_screen(session_id) {\n                                self.request_session_output(session_id);\n                            }",
    ),
    "quiet-launch-off-screen-captures-unnecessarily": (
        APP,
        "                            if quiet && self.is_session_on_screen(session_id) {\n                                self.request_session_output(session_id);\n                            }",
        "                            if quiet {\n                                self.request_session_output(session_id);\n                            }",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
