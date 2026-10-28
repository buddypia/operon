#!/usr/bin/env python3
"""Cache ownership, all native variants and the claim about rebuilding."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RUN_SECONDS = 600
CHECKS = [["cargo", "test", "--locked", "cloned_native_cache"]]
SCRIPT = ROOT / "scripts/seed-target.sh"
GUARD = '''    case "$parent" in
      "$worktree_real"/target/debug/build/*) ;;
      *) echo "seed-target: 共有ネイティブ構成は変更しません: $cache" >&2; continue ;;
    esac'''
BACKUP = '    backup=$(mktemp "${cache}.operon-seeded.XXXXXX") || exit 1'
MUTATIONS = {
    "old-native-configuration-stays-active": (
        SCRIPT, 'mv "$cache" "$backup"', 'cp "$cache" "$backup"'),
    "only-one-native-variant-is-reset": (
        SCRIPT, 'target/debug/build/*/out/build/CMakeCache.txt', 'target/debug/build/*test*/out/build/CMakeCache.txt'),
    "source-cache-is-edited": (
        SCRIPT, BACKUP, '    printf changed >> "$source_debug/${cache#"$worktree/target/debug/"}"\n' + BACKUP),
    "symlinked-parent-edits-another-tree": (
        SCRIPT, GUARD, '    : # parent containment removed'),
    "seeding-promises-only-changed-rust-rebuilds": (
        SCRIPT, 'ネイティブ構成は必要に応じて再生成されます。', '初回ビルドは変更分だけになります。'),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
