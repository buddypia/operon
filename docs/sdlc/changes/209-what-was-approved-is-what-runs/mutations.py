#!/usr/bin/env python3
"""Change 209's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/209-what-was-approved-is-what-runs")

RUN_SECONDS = 1800

TESTS = [
    "tests::the_setup_command_survives_a_worktree_path_with_a_space",
    "tests::a_setup_script_runs_end_to_end_in_a_path_with_a_space",
    "tests::the_script_that_runs_is_the_bytes_that_were_approved",
    "tests::a_script_past_the_ceiling_is_refused_by_the_read_itself",
    "tests::a_branch_git_would_refuse_is_never_landed",
    "tests::a_checkpoint_is_named_by_a_whole_object_id",
    "tests::both_script_buttons_run_the_kept_copy",
    "tests::a_fifo_at_the_script_path_is_not_waited_on",
    "tests::the_kept_copy_is_named_by_an_absolute_path",
    "tests::a_script_whose_copy_cannot_be_kept_does_not_run",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

GIT = ROOT / "src/git.rs"
SETUP = ROOT / "src/git/setup.rs"
APP = ROOT / "src/app.rs"
MUTATIONS = {
    "the repository's file runs, not the copy": (
        SETUP,
        "        shell_quote(&script.display().to_string()),\n        shell_quote(&approved.display().to_string())\n",
        "        shell_quote(&script.display().to_string()),\n        shell_quote(&script.display().to_string())\n",
    ),
    "the copy is not what was approved": (
        SETUP,
        "write_file_atomically(&path, &script.bytes)",
        "write_file_atomically(&path, &fs::read(&script.path)?)",
    ),
    "$0 names the copy": (
        SETUP,
        "\"bash -c 'approved=$1; shift; . \\\"$approved\\\"' {} {}\",",
        "\"bash {1} {0}\",",
    ),
    "the read is not bounded": (
        SETUP,
        ".take(SETUP_SCRIPT_MAX_BYTES + 1)",
        ".take(u64::MAX)",
    ),
    "the hand-written ref check comes back": (
        GIT,
        "name != \"@\" && !name.starts_with('-') && is_valid_branch_name(name)",
        "!name.is_empty() && !name.starts_with('-') && !name.contains(\"..\") && !name.contains(' ')",
    ),
    "@ is landed": (
        GIT,
        "name != \"@\" && !name.starts_with('-') && is_valid_branch_name(name)",
        "!name.starts_with('-') && is_valid_branch_name(name)",
    ),
    "an empty checkpoint id passes": (
        GIT,
        "matches!(sha.len(), 40 | 64) && sha.bytes()",
        "sha.bytes()",
    ),
    "the run button names the repository's file": (
        APP,
        "            Ok(approved) => Some(setup_command(&script.path, &approved)),",
        "            Ok(_) => Some(format!(\"bash {}\", shell_quote(&script.path.display().to_string()))),",
    ),
    "the open waits on a FIFO": (
        SETUP,
        "        .custom_flags(libc::O_NONBLOCK)\n",
        "",
    ),
    "the copy's path is relative": (
        SETUP,
        "Ok(std::path::absolute(parent)?.join(APPROVED_SCRIPTS_DIRECTORY))",
        "Ok(parent.join(APPROVED_SCRIPTS_DIRECTORY))",
    ),
    "a copy that was not kept still runs": (
        APP,
        "            Err(error) => {\n                self.notice = Some(tf!(\n                    \"承認した内容を保存できなかったため実行しませんでした: {error}\",\n                    error = format!(\"{error:#}\")\n                ));\n                None",
        "            Err(error) => {\n                self.notice = Some(tf!(\n                    \"承認した内容を保存できなかったため実行しませんでした: {error}\",\n                    error = format!(\"{error:#}\")\n                ));\n                Some(setup_command(&script.path, &script.path))",
    ),
    "the reason is dropped from the notice": (
        APP,
        "error = format!(\"{error:#}\")\n                ));\n                None",
        "error = error\n                ));\n                None",
    ),
    "a bare file name has an empty folder": (
        SETUP,
        "        .filter(|parent| !parent.as_os_str().is_empty())\n",
        "",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
