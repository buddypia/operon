#!/usr/bin/env python3
"""Mutation check for 218-channel-cli-stable-bootstraps-missing-bundle."""

import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[4]

MUTATIONS = [
    (
        "stable_bootstraps_from_canary_when_missing",
        ROOT / "scripts/operon-channel.sh",
        '    if [ -d "$canary_app" ]; then\n      do_promote "$no_open"\n      return 0\n    fi',
        "    if false; then\n      do_promote \"$no_open\"\n      return 0\n    fi",
        "the_channel_cli_promotes_canary_to_stable_and_rolls_back",
    ),
    (
        "stable_does_not_overwrite_existing_bundle",
        ROOT / "scripts/operon-channel.sh",
        '  if [ ! -d "$stable_app" ]; then',
        "  if true; then",
        "the_channel_cli_promotes_canary_to_stable_and_rolls_back",
    ),
    (
        "stable_refuses_when_both_missing",
        ROOT / "scripts/operon-channel.sh",
        '    echo "error: stable bundle not found at $stable_app — run \'operon-channel.sh canary --build\' first" >&2\n    exit 66',
        '    echo "error: stable bundle not found at $stable_app — run \'operon-channel.sh canary --build\' first" >&2\n    exit 0',
        "the_channel_cli_refuses_missing_bundles_and_unknown_subcommands",
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
