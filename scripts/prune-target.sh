#!/usr/bin/env bash
# Usage: bash scripts/prune-target.sh [--dry-run] [--target-dir <path>]
# Prunes obsolete test binaries and stale incremental cache under target/.
#
# Cargo never garbage-collects old binaries with past hash fingerprints, which
# causes target/ to grow to tens or hundreds of gigabytes over frequent test runs.
# This script identifies and purges obsolete build artifacts while keeping the
# latest build artifacts and dependencies intact.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
dry_run=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run=1
      ;;
    --target-dir)
      shift
      target_dir="${1:-}"
      ;;
    -h|--help)
      echo "Usage: bash scripts/prune-target.sh [--dry-run] [--target-dir <dir>]"
      echo "  --dry-run          Preview files to delete and calculated space without deleting"
      echo "  --target-dir <dir> Specify target directory (default: target/ or CARGO_TARGET_DIR)"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
  shift
done

if [ ! -d "$target_dir" ]; then
  echo "Target directory does not exist: $target_dir"
  echo "Nothing to prune."
  exit 0
fi

total_bytes=0
pruned_count=0

format_bytes() {
  local b="$1"
  if [ "$b" -ge 1073741824 ]; then
    awk "BEGIN {printf \"%.2f GB\", $b/1073741824}"
  elif [ "$b" -ge 1048576 ]; then
    awk "BEGIN {printf \"%.2f MB\", $b/1048576}"
  elif [ "$b" -ge 1024 ]; then
    awk "BEGIN {printf \"%.2f KB\", $b/1024}"
  else
    printf "%d B" "$b"
  fi
}

get_size_bytes() {
  local p="$1"
  if [ -f "$p" ]; then
    /usr/bin/stat -f "%z" "$p" 2>/dev/null || stat -c "%s" "$p" 2>/dev/null || echo 0
  elif [ -d "$p" ]; then
    du -sk "$p" 2>/dev/null | awk '{print $1 * 1024}' || echo 0
  else
    echo 0
  fi
}

# 1. Prune obsolete test executables and dSYM directories under deps/
# Find test executables matching `operon-*` (excluding .d, .rmeta, .rlib)
for profile in debug release; do
  deps_dir="$target_dir/$profile/deps"
  [ -d "$deps_dir" ] || continue

  # List test binaries sorted by modification time (newest first)
  # We keep the newest test binary for each test group and prune older ones.
  # Operon tests typically produce binaries named operon-<hash> or similar.
  test_binaries=$(find "$deps_dir" -maxdepth 1 -type f -perm +111 -name "operon-*" ! -name "*.d" ! -name "*.rmeta" ! -name "*.rlib" 2>/dev/null || true)
  if [ -n "$test_binaries" ]; then
    newest_binary=""
    newest_mtime=0
    while IFS= read -r bin; do
      [ -n "$bin" ] || continue
      mtime=$(/usr/bin/stat -f "%m" "$bin" 2>/dev/null || stat -c "%Y" "$bin" 2>/dev/null || echo 0)
      if [ "$mtime" -gt "$newest_mtime" ]; then
        newest_mtime="$mtime"
        newest_binary="$bin"
      fi
    done <<< "$test_binaries"

    while IFS= read -r bin; do
      [ -n "$bin" ] || continue
      if [ "$bin" != "$newest_binary" ]; then
        sz=$(get_size_bytes "$bin")
        total_bytes=$((total_bytes + sz))
        pruned_count=$((pruned_count + 1))
        if [ "$dry_run" -eq 1 ]; then
          echo "[dry-run] Would prune stale test binary: $bin ($(format_bytes "$sz"))"
        else
          rm -f "$bin"
          echo "Pruned stale test binary: $bin ($(format_bytes "$sz"))"
        fi

        # Associated dSYM directory if present
        dsym="${bin}.dSYM"
        if [ -d "$dsym" ]; then
          dsz=$(get_size_bytes "$dsym")
          total_bytes=$((total_bytes + dsz))
          pruned_count=$((pruned_count + 1))
          if [ "$dry_run" -eq 1 ]; then
            echo "[dry-run] Would prune test dSYM: $dsym ($(format_bytes "$dsz"))"
          else
            rm -rf "$dsym"
            echo "Pruned test dSYM: $dsym ($(format_bytes "$dsz"))"
          fi
        fi

        # Associated .d file if present
        dfile="${bin}.d"
        if [ -f "$dfile" ]; then
          rm -f "$dfile" 2>/dev/null || true
        fi
      fi
    done <<< "$test_binaries"
  fi
done

# 2. Prune old incremental compilation directories older than 7 days
for profile in debug release; do
  inc_dir="$target_dir/$profile/incremental"
  [ -d "$inc_dir" ] || continue

  # Find sessions inside incremental directory older than 7 days
  old_sessions=$(find "$inc_dir" -mindepth 2 -maxdepth 2 -type d -mtime +7 2>/dev/null || true)
  if [ -n "$old_sessions" ]; then
    while IFS= read -r session_dir; do
      [ -n "$session_dir" ] || continue
      sz=$(get_size_bytes "$session_dir")
      total_bytes=$((total_bytes + sz))
      pruned_count=$((pruned_count + 1))
      if [ "$dry_run" -eq 1 ]; then
        echo "[dry-run] Would prune old incremental session: $session_dir ($(format_bytes "$sz"))"
      else
        rm -rf "$session_dir"
        echo "Pruned old incremental session: $session_dir ($(format_bytes "$sz"))"
      fi
    done <<< "$old_sessions"
  fi
done

formatted_total=$(format_bytes "$total_bytes")
if [ "$dry_run" -eq 1 ]; then
  echo "Dry-run complete: $pruned_count item(s) would be pruned, reclaiming approximately $formatted_total."
else
  if [ "$pruned_count" -gt 0 ]; then
    echo "Prune complete: $pruned_count item(s) pruned, reclaimed approximately $formatted_total."
  else
    echo "Target directory is clean. No obsolete artifacts found (0 B reclaimed)."
  fi
fi

# operon: end of scripts/prune-target.sh
