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
        '    command.args(["set-option", "-q", "-t", name, "remain-on-exit-format", ""]);',
        '    command.args(["set-option", "-q", "-t", name, "remain-on-exit", "on"]);',
    ),
    "dead-pane-scrollback-not-pushed-to-history": (
        TMUX,
        "                preserve_dead_pane_scrollback(name);\n                respawn_tmux_pane(name, cwd, PLAIN_TERMINAL_COMMAND)",
        "                respawn_tmux_pane(name, cwd, PLAIN_TERMINAL_COMMAND)",
    ),
    "dead-pane-killed-instead-of-respawned": (
        TMUX,
        "        TmuxState::Dead => set_tmux_session_environment(name, environment)\n"
        "            .and_then(|()| configure_tmux_session(name))\n"
        "            .and_then(|()| {\n"
        "                preserve_dead_pane_scrollback(name);\n"
        "                respawn_tmux_pane(name, cwd, PLAIN_TERMINAL_COMMAND)\n"
        "            })\n"
        "            .map_err(|error| error.to_string()),",
        "        TmuxState::Dead => {\n"
        "            kill_tmux_session(name);\n"
        "            start_empty_tmux_session(name, cwd, environment).map_err(|error| error.to_string())\n"
        "        }",
    ),
    "dead-pane-env-not-updated-before-respawn": (
        TMUX,
        "        TmuxState::Dead => set_tmux_session_environment(name, environment)\n            .and_then(|()| configure_tmux_session(name))",
        "        TmuxState::Dead => configure_tmux_session(name)",
    ),
    "exited-open-tabs-not-reopened-on-load": (
        RELAUNCH,
        "    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {\n        if !self.tools.tmux {\n            return;\n        }",
        "    pub(crate) fn reopen_exited_workspace_tabs_on_load(&mut self) {\n        if self.tools.tmux || !self.tools.tmux {\n            return;\n        }",
    ),
    "closed-exited-history-also-reopened-on-load": (
        RELAUNCH,
        "                session.status == SessionStatus::Exited\n                    && open_tabs.contains(&session.id)",
        "                session.status == SessionStatus::Exited",
    ),
    "clean-exit-keeps-stale-native-session-id": (
        RELAUNCH,
        "            stored.native_session_id = None;\n            stored.native_session_path = None;",
        "            let _ = &stored.native_session_id;",
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
        "            let forced_restoring = false;",
    ),
    "auto-restore-cooldown-not-enforced": (
        APP,
        "            let forced_restoring =\n                !first_look && can_auto_restore && self.auto_restore_ready(id, poll_now);",
        "            let forced_restoring = !first_look && can_auto_restore;",
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
        "    for managed in ([] as [&str; 0]) {",
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
