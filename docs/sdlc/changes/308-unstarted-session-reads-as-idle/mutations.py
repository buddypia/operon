#!/usr/bin/env python3
"""Mutation sweep for change 308: a session not yet given its first request
reads 未開始 — read from SessionStart's source, kept through a stale hook,
ended by a turn, a launch, or removal, counted and broadcast as idle, told
apart by shape, and said so in the hover, the group hint, and the manuals."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "unstarted_308_",
    "f308_",
    "the_idle_group_word_does_not_read_as_waiting_on_you",
    "session_tab_status_badges_are_distinct_vectors",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

HOOKS = ROOT / "src/tmux/hooks.rs"
APP = ROOT / "src/app.rs"
BROADCAST = ROOT / "src/app/broadcast.rs"
WIDGETS = ROOT / "src/ui/widgets.rs"
MANUAL_JA = ROOT / "docs/MANUAL.ja.md"

MUTATIONS = {
    # R1
    "a-resume-reads-as-a-new-conversation": (
        HOOKS,
        '        Some("startup" | "clear")\n',
        '        Some("startup" | "clear" | "resume")\n',
    ),
    "codex-never-starts-a-new-conversation": (
        HOOKS,
        '            "SessionStart" => {\n                boundary = true;\n'
        "                new_conversation = starts_a_new_conversation(payload);\n",
        '            "SessionStart" => {\n                boundary = true;\n'
        "                new_conversation = false && starts_a_new_conversation(payload);\n",
    ),
    # R2
    "a-turn-keeps-not-started": (
        APP,
        "        } else {\n            self.awaiting_first_request.remove(&session_id);\n        }\n",
        "        } else {\n        }\n",
    ),
    "the-flag-is-ignored": (
        WIDGETS,
        "            Some(AgentActivity::Idle) if awaiting_first_request => SessionStatusKind::NotStarted,\n",
        "            Some(AgentActivity::Idle) if false => SessionStatusKind::NotStarted,\n",
    ),
    "working-reads-not-started": (
        WIDGETS,
        "            Some(AgentActivity::Working) => SessionStatusKind::Working,\n",
        "            Some(AgentActivity::Working) if awaiting_first_request => SessionStatusKind::NotStarted,\n"
        "            Some(AgentActivity::Working) => SessionStatusKind::Working,\n",
    ),
    # F1: a late tool post the state turned away must not end the wait.
    "a-late-post-ends-not-started": (
        HOOKS,
        "        let mut unchanged = waiting.clone();\n        unchanged.received_at = now;\n",
        "        let mut unchanged = waiting.clone();\n        unchanged.received_at = now;\n"
        "        unchanged.new_conversation = false;\n",
    ),
    "the-state-forgets-a-new-conversation": (
        HOOKS,
        "            new_conversation: reading.new_conversation,\n",
        "            new_conversation: false,\n",
    ),
    # R3
    "the-stale-sweep-forgets-not-started": (
        APP,
        "            self.hook_status.remove(&id);\n            self.session_activity.remove(&id);\n",
        "            self.hook_status.remove(&id);\n            self.session_activity.remove(&id);\n"
        "            self.awaiting_first_request.remove(&id);\n",
    ),
    "a-turn-the-screen-saw-keeps-not-started": (
        APP,
        "                if tracker.settled == Some(AgentActivity::Working) {\n",
        "                if false && tracker.settled == Some(AgentActivity::Working) {\n",
    ),
    "a-launch-inherits-not-started": (
        APP,
        "        // conversation until its CLI says so.\n        self.awaiting_first_request.remove(&session_id);\n",
        "        // conversation until its CLI says so.\n",
    ),
    "removing-a-session-keeps-not-started": (
        APP,
        "        self.hook_status.remove(&session_id);\n        self.awaiting_first_request.remove(&session_id);\n",
        "        self.hook_status.remove(&session_id);\n",
    ),
    # R4
    "the-broadcast-skips-not-started": (
        BROADCAST,
        "                    && self.session_status_kind_for(session).group()\n"
        "                        == Some(SessionStatusGroup::Free)\n",
        "                    && self.session_status_kind_for(session) == SessionStatusKind::Idle\n",
    ),
    "not-started-counts-as-running": (
        WIDGETS,
        "            SessionStatusKind::Idle | SessionStatusKind::NotStarted => {\n"
        "                Some(SessionStatusGroup::Free)\n            }\n",
        "            SessionStatusKind::Idle => Some(SessionStatusGroup::Free),\n"
        "            SessionStatusKind::NotStarted => Some(SessionStatusGroup::Busy),\n",
    ),
    # R5
    "not-started-wears-the-pause-bars": (
        WIDGETS,
        "            SessionStatusKind::NotStarted => StatusMark::Dot,\n",
        "            SessionStatusKind::NotStarted => StatusMark::Pause,\n",
    ),
    # Claims a person reads (rule 4).
    "the-hover-claims-a-turn-ended": (
        WIDGETS,
        '        SessionStatusKind::NotStarted => tr("まだ最初の依頼を受けていません").to_owned(),\n',
        '        SessionStatusKind::NotStarted => tr("ターンを終え、次の依頼を出せます").to_owned(),\n',
    ),
    "the-word-is-idle-again": (
        WIDGETS,
        '            SessionStatusKind::NotStarted => "未開始",\n',
        '            SessionStatusKind::NotStarted => "アイドル",\n',
    ),
    "the-idle-tile-hover-forgets-not-started": (
        WIDGETS,
        '            Self::Free => "未開始、またはターンを終えて次の依頼を出せるセッション",\n',
        '            Self::Free => "ターンを終え、次の依頼を出せるセッション",\n',
    ),
    "the-manual-says-the-broadcast-skips-not-started": (
        MANUAL_JA,
        "終えたエージェント、またはまだ依頼を受けていない未開始のエージェントのセッション）がチェックボックス付きで並び",
        "終えたエージェントのセッション）がチェックボックス付きで並び",
    ),
}

# No __pycache__ left in scripts/ beside the runner.
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
