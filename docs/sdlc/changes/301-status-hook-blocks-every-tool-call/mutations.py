#!/usr/bin/env python3
"""Change 301's mutation sweep: each mutation breaks one requirement or the
claim in a doc comment, and one of the four tests must go red. Runner:
scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
TESTS = [
    "claude_tool_hooks_run_async_and_every_other_handler_waits",
    "a_late_pre_tool_use_does_not_hide_a_permission_prompt",
    "a_late_tool_event_does_not_reopen_a_finished_turn",
    "a_late_tool_event_does_not_hide_a_question",
    "a_late_question_pre_tool_use_does_not_reopen_a_wait",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

HOOKS = ROOT / "src" / "tmux" / "hooks.rs"
MUTATIONS = {
    "async-on-a-fourth-event-permission-request": (
        HOOKS,
        '"PreToolUse" | "PostToolUse" | "PostToolUseFailure")',
        '"PreToolUse" | "PostToolUse" | "PostToolUseFailure" | "PermissionRequest")',
    ),
    "async-dropped-from-post-tool-use-failure": (
        HOOKS,
        '"PreToolUse" | "PostToolUse" | "PostToolUseFailure")',
        '"PreToolUse" | "PostToolUse")',
    ),
    "async-put-on-codex-too": (
        HOOKS,
        "    provider == CliProvider::Claude\n        && matches!(event,",
        "    provider != CliProvider::Gemini\n        && matches!(event,",
    ),
    "async-never-written": (
        HOOKS,
        '        handler["async"] = serde_json::Value::Bool(true);',
        "        let _ = &handler;",
    ),
    "late-request-pre-tool-use-let-through": (
        HOOKS,
        "                !reading.tool_finished\n",
        "                false\n",
    ),
    "waited-on-call-end-refused": (
        HOOKS,
        "                !reading.tool_finished\n",
        "                true\n",
    ),
    "seen-pre-tool-use-let-through": (
        HOOKS,
        "    if seen && !reading.tool_finished {\n        return true;",
        "    if seen && !reading.tool_finished {\n        return false;",
    ),
    "seen-id-ignored-while-idle": (
        HOOKS,
        "        AgentActivity::Idle => seen,",
        "        AgentActivity::Idle => false,",
    ),
    "seen-id-ignored-while-awaiting": (
        HOOKS,
        "            } else {\n                seen\n            }",
        "            } else {\n                false\n            }",
    ),
    "every-tool-event-late-while-idle-claim": (
        HOOKS,
        "        AgentActivity::Idle => seen,",
        "        AgentActivity::Idle => true,",
    ),
    "tool-id-never-remembered": (
        HOOKS,
        "    remember_tool_id(&mut seen_tool_ids, reading.tool_use_id.as_deref());",
        "    remember_tool_id(&mut seen_tool_ids, None);",
    ),
    "waiting-call-not-kept": (
        HOOKS,
        "        .filter(|_| reading.activity == AgentActivity::AwaitingInput);",
        "        .filter(|_| false);",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
