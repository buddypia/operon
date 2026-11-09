#!/usr/bin/env python3
"""Change 270: keep a terminal's kind, let a launched CLI change kind, forget or
mangle the workspace, accept a pane tree that cannot be drawn, relaunch the
wrong sessions or none, let a relaunch move the screen, miscount it, notify a
crash it undoes, say a failed write every time, and put the recursion back."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "a_terminal_takes_the_kind_of_the_cli_running_in_it",
    "a_terminal_running_claude_receives_review_notes",
    "the_workspace_comes_back_after_a_relaunch",
    "a_damaged_workspace_file_restores_only_what_is_whole",
    "a_layout_that_cannot_be_written_is_said_once",
    "sessions_running_at_the_quit_come_back_when_their_terminal_is_gone",
    "the_relaunch_says_how_many_sessions_it_reopened",
    "resuming_a_session_with_no_transcript_starts_it_fresh_once",
    "the_manual_resume_still_makes_a_new_record",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]
APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
RELAUNCH = ROOT / "src/app/relaunch.rs"
TREE = ROOT / "src/ui/session_tree.rs"
TAKES_KIND = "terminal_provenance(session).filter(|_| session.agent != reading.source.agent());"
MUTATIONS = {
    # R1
    "a terminal keeps the kind it was created with": (
        APP,
        TAKES_KIND,
        "terminal_provenance(session).filter(|_| false);",
    ),
    "the kind follows only a new conversation ID": (
        APP,
        TAKES_KIND,
        "terminal_provenance(session).filter(|_| session.native_session_id.is_none());",
    ),
    "a session launched as a CLI changes kind too": (
        RELAUNCH,
        """            Some(provenance)
        }
        _ => None,""",
        """            Some(provenance)
        }
        _ => Some(SessionOrigin::PlainTerminal),""",
    ),
    "the plain-terminal description stays": (APP, "            session.goal.clear();\n", ""),
    "a legacy record is not given its provenance": (
        APP,
        "            session.origin = Some(provenance);\n",
        "            let _ = provenance;\n",
    ),
    # R2
    "review notes skip a terminal that runs a CLI": (
        APP,
        """                    && session.status == SessionStatus::Active
                    && session.agent != "terminal\"""",
        """                    && session.status == SessionStatus::Active
                    && session.origin != Some(SessionOrigin::PlainTerminal)
                    && session.agent != "terminal\"""",
    ),
    # R3, R4
    "the layout is not restored at start": (APP, "        app.restore_workspace_layout(&layout);\n", ""),
    "an unchanged layout is written again": (
        RELAUNCH,
        "if self.saved_workspace_layout.as_ref() == Some(&snapshot) {",
        "if false && self.saved_workspace_layout.as_ref() == Some(&snapshot) {",
    ),
    "a closed tab is reopened": (
        RELAUNCH,
        "        self.workspace_seen_sessions = placed;\n",
        "        let _ = placed;\n",
    ),
    # R5
    "more panes than fit are accepted": (
        TREE,
        "        if saved_panes.len() > PANE_MAX\n",
        "        if false\n",
    ),
    "a repeated pane or group id is accepted": (TREE, "            || !distinct\n", ""),
    "a pane naming no group is accepted": (TREE, "            || !saved_panes.iter().all(known)\n", ""),
    "a group no pane shows is accepted": (
        TREE,
        """            || !saved
                .groups
                .iter()
                .all(|group| saved_panes.contains(&group.id))
""",
        "",
    ),
    "a session stays in two groups": (TREE, "exists(*id) && placed.insert(*id)", "exists(*id)"),
    "a front-tab index is not bounded": (
        TREE,
        "active: group.active.min(tabs.len() - 1),",
        "active: group.active,",
    ),
    "a share is not bounded": (TREE, "                share.clamp(0.0, 1.0)\n", "                *share\n"),
    # R6
    "nothing is relaunched": (APP, "                relaunch.push(id);\n", ""),
    "a session that dies later in the run is relaunched too": (
        APP,
        "observation.state != TmuxState::Unknown && self.relaunch_candidates.remove(&id);",
        "observation.state != TmuxState::Unknown && { self.relaunch_candidates.remove(&id); true };",
    ),
    "a crashed CLI is left stopped": (
        APP,
        "matches!(next, Some(SessionStatus::Lost | SessionStatus::Failed))",
        "matches!(next, Some(SessionStatus::Lost))",
    ),
    "a CLI the person exited is relaunched": (
        APP,
        "matches!(next, Some(SessionStatus::Lost | SessionStatus::Failed))",
        "matches!(next, Some(SessionStatus::Lost | SessionStatus::Failed | SessionStatus::Exited))",
    ),
    "a relaunched CLI is brought forward": (
        RELAUNCH,
        "        self.launch_session_record(session_id, LaunchFocus::Keep)\n    }",
        "        self.launch_session_record(session_id, LaunchFocus::Bring)\n    }",
    ),
    "a relaunched terminal is brought forward": (
        RELAUNCH,
        "return self.launch_session_record(session_id, LaunchFocus::Keep)",
        "return self.launch_session_record(session_id, LaunchFocus::Bring)",
    ),
    # R7
    "the relaunch says nothing": (
        RELAUNCH,
        "        if started > 0 {\n            self.notice_briefly(",
        "        if false && started > 0 {\n            self.notice_briefly(",
    ),
    "the relaunch counts what it could not start": (
        RELAUNCH,
        """            if self.relaunch_lost_session(id) {
                started += 1;
            }""",
        """            self.relaunch_lost_session(id);
            started += 1;""",
    ),
    "a crash about to be undone is notified": (
        APP,
        "} else if next == SessionStatus::Failed && !relaunching {",
        "} else if next == SessionStatus::Failed {",
    ),
    "a quiet launch's completion takes the screen": (
        APP,
        """                            if !quiet
                                && (self.selected_session.is_none()""",
        """                            if (self.selected_session.is_none()""",
    ),
    "a quiet launch's completion replaces the notice": (
        APP,
        "                                self.notice = shown;\n",
        "                                let _ = shown;\n",
    ),
    "the toast says the sessions ended, not that they reopened": (
        RELAUNCH,
        '"前回開いていたセッションを {count} 件再開しました。"',
        '"前回開いていたセッションを {count} 件終了しました。"',
    ),
    "the failure sentence says the layout was saved": (
        RELAUNCH,
        '"タブの配置を保存できませんでした: {error}"',
        '"タブの配置を保存しました: {error}"',
    ),
    "a failed layout write is said every time": (
        RELAUNCH,
        "                if !self.workspace_layout_save_failed {",
        "                if true {",
    ),
    # R8
    "a resume with no transcript recurses again": (
        APP,
        "                self.launch_session_record(source_id, LaunchFocus::Bring);\n",
        "                self.start_session(source_id);\n",
    ),
    "the manual resume reopens in place": (
        APP,
        """            self.resume_managed_native_session(session_id);
            return;
        }
        self.launch_session_record(session_id, LaunchFocus::Bring);""",
        """            self.launch_session_record(session_id, LaunchFocus::Bring);
            return;
        }
        self.launch_session_record(session_id, LaunchFocus::Bring);""",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
