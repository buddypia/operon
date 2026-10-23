#!/usr/bin/env python3
"""Mutation check for 216-the-channel-cli-exposes-only-canary-and-stable-workflows."""

import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[4]

MUTATIONS = [
    (
        "status_omits_extra_channels",
        ROOT / "scripts/operon-channel.sh",
        '  print_channel_line "stable" "$stable_app"\n',
        '  print_channel_line "stable" "$stable_app"\n  print_channel_line "installed" "$stable_app"\n',
        "the_channel_cli_reports_status_and_reads_its_stamp_key_from_the_packager",
    ),
    (
        "stable_promote_flag",
        ROOT / "scripts/operon-channel.sh",
        "      --promote|-p) promote=1 ;;",
        "      --promote|-p) promote=0 ;;",
        "the_channel_cli_promotes_canary_to_stable_and_rolls_back",
    ),
    (
        "stable_promote_rollback_conflict",
        ROOT / "scripts/operon-channel.sh",
        '  if [ "$promote" -eq 1 ] && [ "$rollback" -eq 1 ]; then',
        "  if false; then",
        "the_channel_cli_refuses_missing_bundles_and_unknown_subcommands",
    ),
    (
        "readme_stable_promote",
        ROOT / "README.md",
        "bash scripts/operon-channel.sh stable --promote",
        "bash scripts/operon-channel.sh promote",
        "the_readmes_document_the_channel_cli",
    ),
]


def main() -> int:
    for name, path, old, new, test_filter in MUTATIONS:
        original = path.read_text(encoding="utf-8")
        if old not in original:
            print(f"error: target snippet not found for {name} in {path}", file=sys.stderr)
            return 2
        try:
            path.write_text(original.replace(old, new, 1), encoding="utf-8")
            proc = subprocess.run(
                ["cargo", "test", "--locked", test_filter],
                cwd=ROOT,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            if proc.returncode == 0:
                print(f"SURVIVED: {name} ({test_filter})", file=sys.stderr)
                return 1
            print(f"KILLED: {name} ({test_filter})")
        finally:
            path.write_text(original, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
