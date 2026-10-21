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
# worktrees never contend for one build lock. Cargo carries a moved target dir
# itself (each build script's `root-output` records the old OUT_DIR); a stale
# dependency is rebuilt as it would be in any target dir.
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
if [ ! -d "$main/deps" ]; then
  echo "seed-target: $main がないため、初回ビルドは全依存をコンパイルします。" >&2
  exit 0
fi
[ -e "$worktree/target" ] && exit 0

mkdir -p "$worktree/target"
if cp -c -R -p "$main" "$worktree/target/debug" 2>/dev/null; then
  echo "seed-target: main の target/debug を複製しました。初回ビルドは変更分だけになります。" >&2
else
  rm -rf "$worktree/target/debug"
  echo "seed-target: target/debug を複製できませんでした。初回ビルドは全依存をコンパイルします。" >&2
fi
exit 0
