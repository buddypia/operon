#!/usr/bin/env python3
"""Mutation check for change 238.

Each mutant reverts or breaks one part of the fix across `scripts/seed-target.sh`,
`.github/workflows/ci.yml`, and `.github/workflows/harness.yml` and verifies that
`ci_caches_native_builds_per_runner_image_without_openssl_and_resets_restored_cmake`
fails on the mutant.
"""

from __future__ import annotations

import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[4]
TEST_NAME = (
    "tests::ci_caches_native_builds_per_runner_image_without_openssl_and_resets_restored_cmake"
)

MUTANTS: list[tuple[str, str, str, str]] = [
    (
        "seed-target-debug-only-glob",
        "scripts/seed-target.sh",
        'for cache in "$root_dir"/target/*/build/*/out/build/CMakeCache.txt; do',
        'for cache in "$root_dir"/target/debug/build/*/out/build/CMakeCache.txt; do',
    ),
    (
        "seed-target-drop-reset-only-flag",
        "scripts/seed-target.sh",
        'if [ "${1:-}" = "--reset-only" ]; then',
        'if [ "${1:-}" = "--disabled-reset-only" ]; then',
    ),
    (
        "seed-target-drop-file-symlink-guard",
        "scripts/seed-target.sh",
        '    if [ -L "$cache" ]; then',
        "    if false; then",
    ),
    (
        "seed-target-drop-parent-symlink-guard",
        "scripts/seed-target.sh",
        '      "$root_real"/target/*/build/*) ;;',
        "      *) ;;",
    ),
    (
        "ci-drop-disable-openssl",
        ".github/workflows/ci.yml",
        "  CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: ON",
        "  CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: OFF",
    ),
    (
        "ci-drop-image-key-in-test-job",
        ".github/workflows/ci.yml",
        "      - uses: Swatinem/rust-cache@v2\n        with:\n          key: ${{ env.ImageOS }}-${{ env.ImageVersion }}-no-openssl\n      - name: Reset restored native CMake caches\n        run: bash scripts/seed-target.sh --reset-only .\n      - name: Full locked test suite",
        "      - uses: Swatinem/rust-cache@v2\n      - name: Reset restored native CMake caches\n        run: bash scripts/seed-target.sh --reset-only .\n      - name: Full locked test suite",
    ),
    (
        "ci-drop-reset-step-in-package-job",
        ".github/workflows/ci.yml",
        "      - name: Reset restored native CMake caches\n        run: bash scripts/seed-target.sh --reset-only .\n      - name: Package macOS app",
        "      - name: Package macOS app",
    ),
    (
        "harness-drop-disable-openssl",
        ".github/workflows/harness.yml",
        "  CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: ON",
        "  CMAKE_DISABLE_FIND_PACKAGE_OpenSSL: OFF",
    ),
    (
        "harness-drop-reset-step",
        ".github/workflows/harness.yml",
        "      - name: Reset restored native CMake caches\n        run: bash scripts/seed-target.sh --reset-only .",
        "",
    ),
]


def run_test(binary: pathlib.Path) -> int:
    result = subprocess.run(
        [str(binary), TEST_NAME, "--exact"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if result.returncode == 0 and "1 passed;" not in result.stdout:
        raise RuntimeError(f"test filter matched 0 tests:\n{result.stdout}")
    return result.returncode


def find_test_binary() -> pathlib.Path:
    build = subprocess.run(
        ["cargo", "test", "--locked", "--no-run", "--message-format=json"],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    import json

    for line in build.stdout.splitlines():
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        executable = payload.get("executable")
        if executable and payload.get("target", {}).get("name") == "operon":
            return pathlib.Path(executable)
    raise RuntimeError("test binary not found")


def main() -> int:
    binary = find_test_binary()
    if run_test(binary) != 0:
        print("baseline failed before mutation run", file=sys.stderr)
        return 1

    survived: list[str] = []
    for name, rel_path, target, replacement in MUTANTS:
        path = ROOT / rel_path
        original = path.read_text(encoding="utf-8")
        if target not in original:
            print(f"mutant target missing for {name} in {rel_path}", file=sys.stderr)
            return 2
        try:
            path.write_text(original.replace(target, replacement, 1), encoding="utf-8")
            rc = run_test(binary)
            status = "KILLED" if rc != 0 else "SURVIVED"
            print(f"{status:8s} {name}")
            if rc == 0:
                survived.append(name)
        finally:
            path.write_text(original, encoding="utf-8")

    if survived:
        print(f"survived mutants: {', '.join(survived)}", file=sys.stderr)
        return 1
    print(f"all {len(MUTANTS)} mutants killed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
