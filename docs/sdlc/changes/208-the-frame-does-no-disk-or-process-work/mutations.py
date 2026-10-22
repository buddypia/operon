#!/usr/bin/env python3
"""Change 208's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/208-the-frame-does-no-disk-or-process-work")

RUN_SECONDS = 1800

TESTS = [
    "tests::a_checkpoint_diff_is_copied_from_a_background_thread",
    "tests::opening_in_an_editor_runs_off_the_frame_and_says_why_it_failed",
    "tests::the_run_script_and_the_templates_are_read_off_the_frame",
    "tests::the_staged_selection_is_settled_when_the_changes_arrive",
    "tests::a_terminal_row_is_read_for_links_only_under_the_pointer",
    "tests::the_commit_gate_leaves_the_suite_to_ci_and_the_merge_gate",
    "tests::every_translation_table_is_sorted_by_message_id",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

APP = ROOT / "src/app.rs"
SCREENS = ROOT / "src/app/screens.rs"
TERMINAL = ROOT / "src/ui/terminal.rs"
GATE = ROOT / ".claude/hooks/gate-commit.sh"
TABLES = ROOT / "src/i18n_tables.rs"
MUTATIONS = {
    "the checkpoint diff is read on the frame again": (
        SCREENS,
        "self.copy_checkpoint_diff(session_id, commit);",
        "let _ = git_diff_checkpoint(Path::new(\".\"), &commit);",
    ),
    "the diff never reaches the clipboard": (
        APP,
        "ctx.copy_text(diff);",
        "let _ = diff;",
    ),
    "a diff that could not be read is not said": (
        APP,
        'Some(tf!("差分をコピーできませんでした: {error}", error = error));',
        "None;",
    ),
    "a closed session's diff is still copied": (
        APP,
        "                        .any(|session| session.id == session_id)\n                    {\n                        continue;\n                    }\n",
        "                        .any(|session| session.id == session_id)\n                    {\n                    }\n",
    ),
    "the editor's failure is dropped": (
        APP,
        "move || open_in_external_editor(&path, editor),",
        "move || {\n                let _ = open_in_external_editor(&path, editor);\n                Ok(())\n            },",
    ),
    "the run script is read again every frame": (
        APP,
        "path == worktree && read_at.elapsed() < Duration::from_secs(2)",
        "path == worktree && read_at.elapsed() < Duration::ZERO",
    ),
    "another folder is offered this one's script": (
        APP,
        "Some((path, _, script)) if path == worktree => script.as_ref(),",
        "Some((_, _, script)) => script.as_ref(),",
    ),
    "the templates read is never kept": (
        APP,
        "self.cached_prompt_templates = Some((project_path, Instant::now(), templates));",
        "let _ = (project_path, templates);",
    ),
    "the staged selection waits for the list to be drawn": (
        APP,
        "                self.ensure_staged_selection(project_id, &snapshot.files);\n",
        "",
    ),
    "every visible row is read for links": (
        TERMINAL,
        "        .filter(|position| rect.contains(*position))?;\n",
        "        ?;\n",
    ),
    "the commit gate skips the table tests for code": (
        GATE,
        "\\.md$|^src/.*\\.rs$'",
        "\\.md$'",
    ),
    "a new row goes back to the top of the table": (
        TABLES,
        'pub(crate) const EN_TABLE: Table = &[\n',
        'pub(crate) const EN_TABLE: Table = &[\n    ("{editor} で開く", "Open in {editor}"),\n',
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
