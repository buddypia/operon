#!/usr/bin/env python3
"""Change 219's mutation sweep. The runner and its seven rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/219-launch-modal-hides-starting-terminal")

RUN_SECONDS = 1800

TESTS = [
    "tests::launch_opens_terminal_screen_immediately_without_progress_modal",
    "tests::session_started_arms_immediate_terminal_output_polling_and_focus",
    "tests::launch_failure_surfaces_diagnostic_modal_after_starting_in_terminal",
    "tests::launch_stored_session_does_not_block_on_workspace_trust_polling",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

APP = ROOT / "src/app.rs"
TMUX = ROOT / "src/tmux.rs"

MUTATIONS = {
    "launch_empty_session does not switch to Page::Sessions immediately": (
        APP,
        "        self.select_session(id);\n        self.page = Page::Sessions;\n        self.session_library_open = false;\n        self.spawn_background(BackgroundKey::EmptySessionStart(id), move || {\n",
        "        self.spawn_background(BackgroundKey::EmptySessionStart(id), move || {\n",
    ),
    "start_session leaves launch_progress undismissed and stays off Page::Sessions": (
        APP,
        "        self.select_session(session_id);\n        self.page = Page::Sessions;\n        self.session_library_open = false;\n        self.spawn_background(BackgroundKey::SessionStart(session_id), move || {\n",
        "        self.spawn_background(BackgroundKey::SessionStart(session_id), move || {\n",
    ),
    "SessionStarted skips immediate output capture and echo poll arming": (
        APP,
        "                                self.request_session_output(session_id);\n                                self.last_terminal_input = Some(Instant::now());\n                                self.terminal_focus_request = Some(session_id);\n                            }\n                            self.persist_external_transition(\n                                tr(\"エージェントセッションを起動し、アプリ内ターミナルで開きました。\"),\n",
        "                            }\n                            self.persist_external_transition(\n                                tr(\"エージェントセッションを起動し、アプリ内ターミナルで開きました。\"),\n",
    ),
    "SessionStarted Err keeps launch_progress dismissed instead of surfacing failure modal": (
        APP,
        "                            if let Some(progress) = self\n                                .launch_progress\n                                .as_mut()\n                                .filter(|p| p.session_id == session_id)\n                            {\n                                progress.failure = Some(error.clone());\n                                progress.dismissed = false;\n                            }\n                            if self.persist() {\n                                self.notice = Some(tf!(\n                                    \"セッションを開始できませんでした: {error}\",\n",
        "                            if let Some(progress) = self\n                                .launch_progress\n                                .as_mut()\n                                .filter(|p| p.session_id == session_id)\n                            {\n                                progress.failure = Some(error.clone());\n                            }\n                            if self.persist() {\n                                self.notice = Some(tf!(\n                                    \"セッションを開始できませんでした: {error}\",\n",
    ),
    "launch_stored_session blocks synchronously on workspace trust prompt": (
        TMUX,
        "        if auto_approve_workspace_prompts {\n            let tmux_name = session.tmux_name.clone();\n            thread::spawn(move || {\n                let _ = auto_approve_workspace_trust_prompt(&tmux_name);\n            });\n        }\n",
        "        if auto_approve_workspace_prompts {\n            auto_approve_workspace_trust_prompt(&session.tmux_name)\n                .map_err(|error| error.to_string())?;\n        }\n",
    ),
    "auto_approve_workspace_trust_prompt polls 8s even when tmux session is gone": (
        TMUX,
        "            Err(error) if tmux_error_state(&error.to_string()) == TmuxState::Gone => {\n                return Ok(false);\n            }\n",
        "            Err(error) if false && tmux_error_state(&error.to_string()) == TmuxState::Gone => {\n                return Ok(false);\n            }\n",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
