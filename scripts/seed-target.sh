#!/usr/bin/env bash
# Seed a new worktree's cargo target/ from the main checkout's. Change 191.
#
#   bash scripts/seed-target.sh <branch>
#
# `make wt.new` runs this after the worktree exists. Each worktree has its own
# target/, so the first clippy in a new one compiled all of Cargo.lock from
# nothing, llama.cpp's C++ included: 16m29s measured under the load several
# agents put on this machine, past the commit gate's 600-second hook timeout,
# which skips the gate rather than failing it. From a clone of main's
# target/debug the same clippy checked only `operon`: 15s to clone, 64s to check.
#
# `cp -c` is an APFS clone, copy-on-write: no bytes are copied until one side
# writes, and the worktree's target/ is its own from then on, so concurrent
# worktrees never contend for one build lock. Cargo relocates its own metadata,
# but native CMake caches contain absolute build and dependency paths. Keep a
# recovery copy of each cloned cache and let CMake configure the new location.
#
# Both ends are found from git, not from the working directory: worktree-new.mjs
# puts the worktree at <main checkout>/.worktrees/<branch> wherever `make wt.new`
# was run from, and the cache worth cloning is main's.
#
# Messages go to stderr: stdout of `make wt.new` is worktree-new.mjs's JSON.
# Fail-open: anything missing, an existing target, or a failed clone leaves the
# worktree to build cold, as it did before this script.
set -uo pipefail

branch="${1:-}"
[ -n "$branch" ] || { echo "usage: seed-target.sh <branch>" >&2; exit 2; }
common=$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null) || exit 0
main_root=$(dirname "$common")
main="$main_root/target/debug"
worktree="$main_root/.worktrees/$branch"

[ -d "$worktree" ] || exit 0
[ -e "$worktree/target" ] && exit 0

source_debug=""
if [ -d "$main/deps" ]; then
  if [ -f "$main_root/scripts/prune-target.sh" ]; then
    (cd "$main_root" && bash scripts/prune-target.sh >/dev/null 2>&1) || true
  fi
  source_debug="$main"
else
  for candidate in "$main_root"/.worktrees/*/*/target/debug "$main_root"/.worktrees/*/target/debug; do
    [ -d "$candidate/deps" ] || continue
    [ "$candidate" = "$worktree/target/debug" ] && continue
    source_debug="$candidate"
    break
  done
fi

if [ -z "$source_debug" ]; then
  echo "seed-target: $main がないため、初回ビルドは全依存をコンパイルします。" >&2
  exit 0
fi

mkdir -p "$worktree/target"
if cp -c -R -p "$source_debug" "$worktree/target/debug" 2>/dev/null; then
  rm -f "$worktree/target/debug/.cargo-lock"
  worktree_real=$(cd "$worktree" && pwd -P)
  for cache in "$worktree"/target/debug/build/*/out/build/CMakeCache.txt; do
    [ -f "$cache" ] || continue
    # A copied build directory may be a symlink into another tree. Reset only
    # configuration physically inside this newly owned worktree.
    parent=$(cd "$(dirname "$cache")" && pwd -P) || exit 1
    case "$parent" in
      "$worktree_real"/target/debug/build/*) ;;
      *) echo "seed-target: 共有ネイティブ構成は変更しません: $cache" >&2; continue ;;
    esac
    backup=$(mktemp "${cache}.operon-seeded.XXXXXX") || exit 1
    if ! mv "$cache" "$backup"; then
      rm "$backup"
      exit 1
    fi
  done
  echo "seed-target: $source_debug を複製しました。ネイティブ構成は必要に応じて再生成されます。" >&2
else
  rm -rf "$worktree/target/debug"
  echo "seed-target: target/debug を複製できませんでした。初回ビルドは全依存をコンパイルします。" >&2
fi
exit 0
