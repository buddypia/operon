#!/usr/bin/env python3
"""Mutation sweep for change 215 (operon-channel CLI)."""
import sys
from pathlib import Path

# 1. WHERE.
ROOT = Path("/Users/a13973/dev/buddypia/operon/.worktrees/feature/215-switching-between-canary-and-stable-builds-is-manual")

# 2. HOW LONG one check may take before the sweep stops waiting for it.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN.
TESTS = [
    "the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager",
    "the_channel_cli_builds_and_launches_canary_without_touching_stable",
    "the_channel_cli_promotes_canary_to_stable_and_rolls_back",
    "the_channel_cli_refuses_missing_bundles_and_unknown_subcommands",
    "the_readmes_document_the_channel_cli",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

# 4. WHAT TO BREAK.
MUTATIONS = {
    "status-omits-installed-build-freshness-claim": (
        ROOT / "scripts/operon-channel.sh",
        "printf 'installed build: %s\\n' \"$(installed_build_freshness)\"",
        "printf 'installed build: UNKNOWN\\n'",
    ),
    "canary-opens-stable-instead-of-canary": (
        ROOT / "scripts/operon-channel.sh",
        'open_bundle "$canary_app"',
        'open_bundle "$stable_app"',
    ),
    "promote-moves-canary-instead-of-staging-copy": (
        ROOT / "scripts/operon-channel.sh",
        'cp -R "$canary_app" "$stage_app"',
        'mv "$canary_app" "$stage_app"',
    ),
    "rollback-fails-to-restore-previous-bundle": (
        ROOT / "scripts/operon-channel.sh",
        'cp -R "$stable_prev_app" "$stage_app"',
        'mkdir -p "$stage_app" && printf broken > "$stage_app/marker"',
    ),
    "promote-missing-bundle-exits-zero": (
        ROOT / "scripts/operon-channel.sh",
        "echo \"error: canary bundle not found at $canary_app — run 'operon-channel.sh canary --build' first\" >&2\n    exit 66",
        "echo \"error: canary bundle not found\" >&2\n    exit 0",
    ),
    "running-operon-ignores-packaged-capital-o-process": (
        ROOT / "scripts/operon-channel.sh",
        'local names="${OPERON_PROCESS_NAMES:-Operon operon}"',
        'local names="${OPERON_PROCESS_NAMES:-operon}"',
    ),
    "readme-drops-promote-subcommand-claim": (
        ROOT / "README.md",
        "bash scripts/operon-channel.sh promote",
        "bash scripts/operon-channel.sh archive",
    ),
}

# 5. Nothing below this line is filled in.

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
