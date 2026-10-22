#!/usr/bin/env python3
"""Change 203's mutation sweep. The runner and its six rules are
scripts/mutation_sweep.py; `state.yaml` records the result as
`guards-verified-by-mutation`.
"""
import sys
from pathlib import Path

ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/fix/203-the-usage-reader-misses-accounts")

RUN_SECONDS = 1800

TESTS = [
    "--exact",
    "tests::the_usage_reader_reaches_every_account_and_survives_a_toggle",
    "tests::the_installed_usage_reader_posts_a_reading_end_to_end",
    "tests::the_shared_usage_marker_becomes_the_machines_own",
    "tests::the_hook_cell_follows_the_hooks_switch",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--"] + TESTS]

HOOKS = ROOT / "src/tmux/hooks.rs"
MUTATIONS = {
    "every folder shares one marker again": (
        HOOKS,
        "    let marker = statusline_marker_path(data_file, &path);\n",
        "    let marker = hook_directory(data_file).join(STATUSLINE_INSTALL_MARKER);\n",
    ),
    "Operon's own removal leaves the marker": (
        HOOKS,
        "        let _ = fs::remove_file(&marker);\n    }\n    Ok(())",
        "    }\n    Ok(())",
    ),
    "a slot somebody else holds is marked": (
        HOOKS,
        "    if statusline_slot(&settings, &reader, false) == StatusLineSlot::Ours {\n        if !installed_before {",
        "    if !remove_only {\n        if !installed_before {",
    ),
    "a clock behind the stamp is read as a tick inside the interval": (
        HOOKS,
        '[ "$now" -ge "$last" ] && ',
        "",
    ),
    "a gone account folder is not named": (
        HOOKS,
        "        .filter(|account| !account.path.is_dir())\n        .map(|account| {",
        "        .filter(|_| false)\n        .map(|account| {",
    ),
    "the settings row does not draw the notice": (
        ROOT / "src/app/screens.rs",
        "ui.label(RichText::new(notice).size(12.0).color(palette.warning));",
        "let _ = notice;",
    ),
    "the shared marker is not adopted": (
        HOOKS,
        "    adopt_shared_statusline_marker(data_file, &claude_settings_in(&claude_config_root(home)));\n",
        "",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
