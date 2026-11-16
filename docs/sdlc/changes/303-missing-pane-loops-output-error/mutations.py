#!/usr/bin/env python3
"""Break missing-pane guards, un-ID'd CLI recovery, open-tab fallback, and empty-pane copy (change 303)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3] if (Path(__file__).resolve().parents[3] / "src").exists() else Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [
    ["cargo", "test", "--locked", "pane_303_"],
    ["cargo", "test", "--locked", "model_check_pane_303_"],
]

APP = ROOT / "src/app.rs"
RELAUNCH = ROOT / "src/app/relaunch.rs"
TERM_OUT = ROOT / "src/app/terminal_output.rs"

MUTATIONS = {
    "missing-capture-guard-ignored-in-eligibility": (
        TERM_OUT,
        "            && !self.terminal_capture_missing.contains(&session.id)\n",
        "            && (!self.terminal_capture_missing.contains(&session.id) || true)\n",
    ),
    "queued-guard-ignored-in-eligibility": (
        TERM_OUT,
        "        session.status != SessionStatus::Queued\n",
        "        (session.status != SessionStatus::Queued || true)\n",
    ),
    "explicit-sync-flag-not-consumed": (
        TERM_OUT,
        "        let explicit = self.explicit_output_requests.remove(&session_id);",
        "        let explicit = self.explicit_output_requests.contains(&session_id);",
    ),
    "queued-placeholder-says-wrong-sentence": (
        TERM_OUT,
        '        SessionStatus::Queued => tr("開始待機中のため、まだターミナル出力はありません。"),',
        '        SessionStatus::Queued => tr("ターミナル出力を待っています…"),',
    ),
    "ended-pane-placeholder-says-wrong-sentence": (
        TERM_OUT,
        '            tr("このセッションのターミナルは終了しているため、表示できる出力はありません。")',
        '            tr("ターミナル出力を待っています…")',
    ),
    "adopted-cli-does-not-stamp-launched-at": (
        APP,
        "        take_cli_kind(session, provider);\n        session.launched_at = Some(now());",
        "        take_cli_kind(session, provider);",
    ),
    "refresh-session-output-skips-eligibility-guard": (
        APP,
        "        if eligible {\n            self.request_session_output(session_id);\n        }",
        "        if eligible || true {\n            self.request_session_output(session_id);\n        }",
    ),
    "fallback-shell-ignores-open-tab-requirement": (
        RELAUNCH,
        "    pub(crate) fn can_fallback_open_tab_to_shell(&self, session: &Session) -> bool {\n        self.session_has_a_tab(session.id)",
        "    pub(crate) fn can_fallback_open_tab_to_shell(&self, session: &Session) -> bool {\n        (self.session_has_a_tab(session.id) || true)",
    ),
    "lost-open-tabs-not-queued-on-load": (
        RELAUNCH,
        "        self.lost_tabs_on_load.extend(candidates);",
        "        let _ = candidates;",
    ),
    "fallback-shell-cooldown-not-recorded": (
        RELAUNCH,
        "                RelaunchAttempt::Launched => {\n                    self.record_auto_restore(session_id, now);\n                    return RecoveryOutcome::FallbackShell;\n                }",
        "                RelaunchAttempt::Launched => {\n                    let _ = now;\n                    return RecoveryOutcome::FallbackShell;\n                }",
    ),
    "single-fallback-toast-says-wrong-sentence": (
        RELAUNCH,
        '            [name] => self.notice_briefly(tf!(\n                "会話を復元できなかったため、ターミナルとして開き直しました: {name}",\n                name = name\n            )),',
        '            [name] => self.notice_briefly(tf!(\n                "強制終了されたセッションを自動で復元しました: {name}",\n                name = name\n            )),',
    ),
    "multi-fallback-toast-says-wrong-sentence": (
        RELAUNCH,
        '            many => self.notice_briefly(tf!(\n                "会話を復元できなかったため、{count} 件をターミナルとして開き直しました。",\n                count = many.len()\n            )),',
        '            many => self.notice_briefly(tf!(\n                "強制終了されたセッションを {count} 件自動で復元しました。",\n                count = many.len()\n            )),',
    ),
    "lost-on-load-evicted-despite-persist-failure": (
        RELAUNCH,
        "                RecoveryOutcome::NotRecovered => {\n                    self.lost_tabs_on_load.remove(&id);\n                }\n                RecoveryOutcome::PersistFailed => {}",
        "                RecoveryOutcome::NotRecovered | RecoveryOutcome::PersistFailed => {\n                    self.lost_tabs_on_load.remove(&id);\n                }",
    ),
    "refused-relaunch-wipes-resumable-cli-into-shell": (
        RELAUNCH,
        "                RelaunchAttempt::LaunchRefused => return RecoveryOutcome::NotRecovered,",
        "                RelaunchAttempt::LaunchRefused => {}",
    ),
    "echo-poll-ignores-eligibility-guard": (
        TERM_OUT,
        "        if !eligible || self.terminal_streams.is_watching(session_id) {",
        "        if (!eligible && false) || self.terminal_streams.is_watching(session_id) {",
    ),
    "open-session-does-not-clear-missing-capture": (
        APP,
        "        self.pending_history_open = None;\n        self.terminal_capture_missing.remove(&session_id);",
        "        self.pending_history_open = None;",
    ),
    "resize-spawns-while-session-launch-running": (
        TERM_OUT,
        "        if self.session_launch_running(session_id)\n",
        "        if (self.session_launch_running(session_id) && false)\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
