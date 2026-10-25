#!/usr/bin/env python3
"""Stage 4 mutation check for change 225-knowledge-depends-on-agent-cli."""

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]

MUTATIONS = [
    (
        "knowledge_judge_command Gemini flag order",
        "src/agents.rs",
        '        CliProvider::Gemini => (\n            ANTIGRAVITY_COMMAND,\n            vec!["--sandbox".to_owned(), "--print".to_owned()],\n        ),',
        '        CliProvider::Gemini => (\n            ANTIGRAVITY_COMMAND,\n            vec!["--print".to_owned(), "--sandbox".to_owned()],\n        ),',
        "tests::judge_command_is_read_only_ephemeral_and_runs_outside_the_project",
    ),
    (
        "commit_message_command gemini flag order",
        "src/agents.rs",
        '        "gemini" => (\n            ANTIGRAVITY_COMMAND,\n            vec!["--sandbox".to_owned(), "--print".to_owned()],\n        ),',
        '        "gemini" => (\n            ANTIGRAVITY_COMMAND,\n            vec!["--print".to_owned(), "--sandbox".to_owned()],\n        ),',
        "tests::builds_a_read_only_draft_command_for_every_agent_operon_launches",
    ),
    (
        "CliJudge::judge_with failed-provider memoisation",
        "src/knowledge.rs",
        "                    if !failed.contains(&candidate) {\n                        failed.push(candidate);\n                    }",
        "                    let _ = &mut failed;",
        "tests::cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones",
    ),
    (
        "process_conversation per-provider pause isolation",
        "src/knowledge.rs",
        "    if report.capped || paused_providers.contains(&session.provider) {",
        "    if report.capped || !paused_providers.is_empty() {",
        "tests::cli_judge_falls_back_to_an_available_cli_and_skips_failed_ones",
    ),
    (
        "knowledge_status_line EmbeddingGemma 2 device claim",
        "src/app/knowledge.rs",
        '            "{n} 件 ・ 最終更新 {updated} ・ EmbeddingGemma 2 ({device}) ・ 本日の判定 {used}/{cap} 回",\n            n = snapshot.entries.len(),\n            updated = updated,\n            device = device.label(),',
        '            "{n} 件",\n            n = snapshot.entries.len(),',
        "tests::knowledge_ui_shows_embedding_gemma_device_and_available_cli_summary",
    ),
    (
        "ui_model_block local history search label claim",
        "src/app/knowledge.rs",
        '        ui.horizontal(|ui| {\n            ui.label(tr("ローカル履歴の検索"));',
        '        ui.horizontal(|ui| {\n            ui.label(tr("ローカル履歴"));',
        "tests::knowledge_ui_shows_embedding_gemma_device_and_available_cli_summary",
    ),
]


def run_test(test_name: str) -> int:
    result = subprocess.run(
        ["cargo", "test", "--locked", "--", "--exact", test_name],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    return result.returncode


def main() -> int:
    survivors = []
    for label, rel_path, target, replacement, test_name in MUTATIONS:
        path = ROOT / rel_path
        original = path.read_text(encoding="utf-8")
        if target not in original:
            print(f"ERROR: target not found for {label} in {rel_path}", file=sys.stderr)
            return 2
        try:
            path.write_text(original.replace(target, replacement, 1), encoding="utf-8")
            rc = run_test(test_name)
            if rc == 0:
                print(f"SURVIVED: {label} ({test_name})")
                survivors.append(label)
            else:
                print(f"KILLED:   {label} ({test_name})")
        finally:
            path.write_text(original, encoding="utf-8")
    return 1 if survivors else 0


if __name__ == "__main__":
    sys.exit(main())
