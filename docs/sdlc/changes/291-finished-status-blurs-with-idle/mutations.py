#!/usr/bin/env python3
"""Mutation sweep for change 291: retire 終了, mid-run auto-restore, cooldown, NeedsYou gating, and clean-exit tab close."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 900
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", "status_291_"]]

WIDGETS = ROOT / "src/ui/widgets.rs"
SCREENS = ROOT / "src/app/screens.rs"
APP = ROOT / "src/app.rs"
RELAUNCH = ROOT / "src/app/relaunch.rs"
ORDER = ROOT / "src/app/session_order.rs"

MUTATIONS = {
    "done-label-regresses-to-ambiguous-finished": (
        WIDGETS,
        '            SessionStatusKind::Done => "正常終了",',
        '            SessionStatusKind::Done => "終了",',
    ),
    "stopped-label-regresses-to-done-word": (
        WIDGETS,
        '            SessionStatusKind::Stopped => "停止済み",',
        '            SessionStatusKind::Stopped => "正常終了",',
    ),
    "sidebar-second-row-leaves-half-width-hole": (
        SCREENS,
        "                let row_width = (available - gap * (row.len() - 1) as f32) / row.len() as f32;",
        "                let row_width = (available - gap * (columns - 1) as f32) / columns as f32;",
    ),
    "midrun-crash-never-auto-restores": (
        APP,
        "            } else if forced_restoring {\n                forced_restore.push((id, self.store.sessions[idx].name.clone()));",
        "            } else if false && forced_restoring {\n                forced_restore.push((id, self.store.sessions[idx].name.clone()));",
    ),
    "midrun-restore-toast-says-startup-reopen": (
        RELAUNCH,
        '                    "強制終了されたセッションを {count} 件自動で復元しました。",',
        '                    "前回開いていたセッションを {count} 件再開しました。",',
    ),
    "declined-restore-still-claims-session-restored": (
        RELAUNCH,
        "            if launched {\n                self.record_auto_restore(id, now);\n                restored_ids.insert(id);\n                restored_names.push(name);\n            }",
        "            restored_names.push(name);\n            if launched {\n                self.record_auto_restore(id, now);\n                restored_ids.insert(id);\n            }",
    ),
    "auto-restore-toast-clobbers-durability-notice": (
        RELAUNCH,
        "        if !*persistent_notice && !self.store_retry_pending {",
        "        if true {",
    ),
    "cooldown-allows-immediate-crash-loop": (
        RELAUNCH,
        "            .is_none_or(|last| now.saturating_duration_since(*last) >= AUTO_RESTORE_COOLDOWN)",
        "            .is_none_or(|last| now.saturating_duration_since(*last) >= Duration::ZERO)",
    ),
    "closed-historical-failure-sticks-in-needs-you": (
        APP,
        "            return group == SessionStatusGroup::NeedsYou && self.session_has_a_tab(session.id);",
        "            return group == SessionStatusGroup::NeedsYou;",
    ),
    "clean-exit-leaves-unpinned-tab-open": (
        RELAUNCH,
        "        for id in exited_ids {\n            self.close_finished_session_tab(id);\n        }",
        "        for id in exited_ids {\n            let _ = id;\n        }",
    ),
    "pinned-session-closes-on-clean-exit": (
        ORDER,
        "    pub(crate) fn close_finished_session_tab(&mut self, id: Uuid) {\n        if self.is_session_pinned(id) {\n            return;\n        }",
        "    pub(crate) fn close_finished_session_tab(&mut self, id: Uuid) {",
    ),
    "explicit-stop-leaves-unread-mark-in-needs-you": (
        ORDER,
        "    pub(crate) fn clear_cancelled_unread_and_close_tab(&mut self, id: Uuid) {\n        self.unread_sessions.remove(&id);",
        "    pub(crate) fn clear_cancelled_unread_and_close_tab(&mut self, id: Uuid) {",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
