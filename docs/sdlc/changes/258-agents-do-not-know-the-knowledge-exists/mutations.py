#!/usr/bin/env python3
"""Mutation sweep for change 258: an agent session starts knowing the project's
knowledge exists. Runner and rules: scripts/mutation_sweep.py."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 1800
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "knowledge_258_"]]

KNOWLEDGE = ROOT / "src/knowledge.rs"
HOOKS = ROOT / "src/tmux/hooks.rs"
APP = ROOT / "src/app.rs"
TAB = ROOT / "src/app/knowledge.rs"

MUTATIONS = {
    # Req 1.
    "brief-for-an-empty-base": (
        KNOWLEDGE,
        "    if entries.is_empty() {\n        return None;\n    }\n    let command = format!(",
        "    let command = format!(",
    ),
    "brief-hands-out-an-unquoted-executable": (
        KNOWLEDGE,
        "        shell_quote(&executable.display().to_string())\n    );\n    let mut brief",
        "        executable.display()\n    );\n    let mut brief",
    ),
    "brief-forgets-what-it-is-for": (
        KNOWLEDGE,
        "so that an agent does not re-derive what an earlier session already found.",
        "for reference.",
    ),
    "brief-forgets-what-verified-means": (
        KNOWLEDGE,
        "check {unverified} ones before relying on them.",
        "{unverified} ones are not.",
    ),
    "brief-includes-bodies": (
        KNOWLEDGE,
        '.join(" ")\n        );\n        let line_length',
        '.join(" ") + " " + &entry.body\n        );\n        let line_length',
    ),
    "brief-lets-a-title-span-lines": (
        KNOWLEDGE,
        'entry.title.split_whitespace().collect::<Vec<_>>().join(" ")',
        "entry.title.clone()",
    ),
    # Req 2.
    "brief-ignores-pinned": (
        KNOWLEDGE,
        "        b.pinned\n            .cmp(&a.pinned)\n            .then(b.verified.cmp(&a.verified))",
        "        b.verified\n            .cmp(&a.verified)",
    ),
    "brief-ignores-verified": (
        KNOWLEDGE,
        "            .then(b.verified.cmp(&a.verified))\n",
        "",
    ),
    "brief-stalest-first": (
        KNOWLEDGE,
        ".then(b_fresh.total_cmp(a_fresh))",
        ".then(a_fresh.total_cmp(b_fresh))",
    ),
    "brief-lists-everything": (
        KNOWLEDGE,
        "ranked.into_iter().take(KNOWLEDGE_BRIEF_ENTRIES)",
        "ranked.into_iter()",
    ),
    "brief-ignores-its-ceiling": (
        KNOWLEDGE,
        "        if length + line_length > KNOWLEDGE_BRIEF_MAX_CHARS {\n            break;\n        }\n",
        "",
    ),
    # Req 3.
    "cli-ignores-disabled": (
        KNOWLEDGE,
        "        .filter(|database| database.enabled().unwrap_or(false))\n",
        "",
    ),
    "cli-creates-databases": (
        KNOWLEDGE,
        "            KnowledgeDatabase::open_existing(&knowledge_database_path(root, project.id))\n                .ok()\n                .flatten()",
        "            KnowledgeDatabase::open(&knowledge_database_path(root, project.id)).ok()",
    ),
    "cli-follows-worktrees-through-git": (
        KNOWLEDGE,
        "    let brief = project_containing(directory, projects)\n",
        "    let brief = project_for_directory(directory, projects)\n        .as_ref()\n",
    ),
    "cli-counts-a-reference": (
        KNOWLEDGE,
        "        .and_then(|database| database.entries().ok())\n",
        "        .and_then(|database| {\n            let entries = database.entries().ok()?;\n            let ids = entries.iter().map(|entry| entry.id).collect::<Vec<_>>();\n            database.record_references(&ids, now()).ok()?;\n            Some(entries)\n        })\n",
    ),
    "cli-wrong-event-name": (
        KNOWLEDGE,
        '"hookEventName": "SessionStart",',
        '"hookEventName": "UserPromptSubmit",',
    ),
    "bounded-waits-for-ever": (
        KNOWLEDGE,
        "    receiver.recv_timeout(Duration::from_secs(seconds)).ok()",
        "    let _ = seconds;\n    receiver.recv().ok()",
    ),
    # Req 4.
    "hook-briefs-on-every-event": (
        HOOKS,
        'if [ "${HOOK_EVENT_ENV}" = SessionStart ] && [ -n',
        'if [ -n',
    ),
    "hook-hands-the-payload-to-the-brief": (
        HOOKS,
        "{KNOWLEDGE_BRIEF_SUBCOMMAND} </dev/null 2>/dev/null)",
        "{KNOWLEDGE_BRIEF_SUBCOMMAND} 2>/dev/null)",
    ),
    "hook-keeps-a-failed-brief": (
        HOOKS,
        "2>/dev/null) || brief=''",
        "2>/dev/null) || :",
    ),
    "hook-prints-anything": (
        HOOKS,
        "  '{{\"hookSpecificOutput\":'*) printf",
        "  ?*) printf",
    ),
    "hook-prints-both-answers": (
        HOOKS,
        "  '{{\"hookSpecificOutput\":'*) printf '%s\\n' \"$brief\" ;;",
        "  '{{\"hookSpecificOutput\":'*) printf '%s\\n' \"$brief\"; {usual} ;;",
    ),
    # Req 5.
    "environment-drops-the-executable": (
        HOOKS,
        "            variables.push((HOOK_EXECUTABLE_ENV, executable.display().to_string()));\n",
        "            let _ = executable;\n",
    ),
    "launch-sets-no-executable": (
        APP,
        "            executable: std::env::current_exe().ok(),",
        "            executable: None,",
    ),
    # Req 6.
    "opt-in-card-says-nothing": (
        TAB,
        "                    tr(\"・Claude Code と Codex のセッション開始時に、ピン留め・検証済み・最近使われたものを優先してナレッジの題名 (最大 10 件) と検索方法をその CLI に渡します。他の CLI との会話から作られた題名も含まれます。\"),\n",
        "",
    ),
    "enabled-tab-says-nothing": (
        TAB,
        "        // A project turned on before change 258 never saw the opt-in bullet.\n        ui.label(",
        "        // A project turned on before change 258 never saw the opt-in bullet.\n        let _ = (",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
