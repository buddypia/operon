#!/usr/bin/env bash
# Switch between Operon's battle-test (canary) and stable release channels.
#
# Channels:
#   canary           dist/Operon.app (built from the current checkout for live verification)
#   stable           /Applications/Operon Stable.app (promoted from a verified canary bundle)
#
# Subcommands:
#   status                           Show repo HEAD, running process, and canary/stable stamps
#   canary [--build|-b] [--full] [--no-open]
#                                    Build (if requested or missing) and launch the canary bundle
#   stable [--promote|-p] [--rollback|-r] [--no-open]
#                                    Launch stable, promote canary to stable, or roll back stable
#   promote [--no-open] [--open]     Alias for `stable --promote`
#   rollback [--no-open]             Alias for `stable --rollback`
#   quit                             Quit any running Operon and wait for single-instance lock release
#
# Exit codes:
#   0   success
#   64  usage error
#   66  required bundle or helper script missing
#   75  running Operon process did not exit within OPERON_QUIT_WAIT_SECONDS
set -euo pipefail

unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR

stage_app=""
scratch_backup=""
cleanup_staging() {
  local status=$?
  trap - EXIT
  trap '' INT TERM
  if [ -n "$stage_app" ] && [ -e "$stage_app" ]; then
    rm -rf "$stage_app"
  fi
  if [ -n "$scratch_backup" ] && [ -e "$scratch_backup" ]; then
    rm -rf "$scratch_backup"
  fi
  return "$status"
}
trap cleanup_staging EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

resolve_script_dir() {
  local src="${BASH_SOURCE[0]}"
  while [ -L "$src" ]; do
    local dir
    dir="$(cd -P "$(dirname "$src")" && pwd)"
    src="$(readlink "$src")"
    case "$src" in
      /*) ;;
      *) src="$dir/$src" ;;
    esac
  done
  cd -P "$(dirname "$src")" && pwd
}

script_dir="$(resolve_script_dir)"
if [ -n "${OPERON_REPO_ROOT:-}" ]; then
  repo_root="$(cd "$OPERON_REPO_ROOT" && pwd)"
else
  repo_root="$(cd "$script_dir/.." && pwd)"
fi

dist_dir="${OPERON_DIST_DIR:-$repo_root/dist}"
canary_app="${OPERON_CANARY_APP:-$dist_dir/Operon.app}"
stable_app="${OPERON_STABLE_APP:-/Applications/Operon Stable.app}"
stable_prev_app="${OPERON_STABLE_PREVIOUS_APP:-$dist_dir/Operon.stable-previous.app}"

packager="${OPERON_PACKAGER:-${OPERON_PACKAGE_SCRIPT:-$repo_root/scripts/package-macos.sh}}"
replace_script="${OPERON_REPLACE_SCRIPT:-$script_dir/replace-macos-bundle.sh}"
quit_wait_seconds="${OPERON_QUIT_WAIT_SECONDS:-10}"

case "$quit_wait_seconds" in
  ''|*[!0-9]*)
    echo "error: OPERON_QUIT_WAIT_SECONDS must be a non-negative integer (got '$quit_wait_seconds')" >&2
    exit 64
    ;;
esac

# Read the Info.plist commit stamp key from scripts/package-macos.sh (SSOT).
stamp_key=""
for candidate_packager in "$packager" "$repo_root/scripts/package-macos.sh" "$script_dir/package-macos.sh"; do
  if [ -f "$candidate_packager" ]; then
    candidate_key="$(sed -n 's/^source_commit_key="\([^"]*\)"$/\1/p' "$candidate_packager" 2>/dev/null | head -1 || true)"
    case "$candidate_key" in
      ''|*[!A-Za-z0-9_]*) candidate_key="" ;;
    esac
    if [ -n "$candidate_key" ]; then
      stamp_key="$candidate_key"
      break
    fi
  fi
done
if [ -z "$stamp_key" ]; then
  echo "error: could not read valid source_commit_key from $packager" >&2
  exit 66
fi

usage() {
  cat <<'EOF'
Usage: operon-channel.sh [subcommand] [options]

Subcommands:
  status                           Show repo HEAD, running process, and canary/stable bundles (default)
  canary [--build|-b] [--full] [--no-open]
                                   Build (when --build is passed or bundle is missing) and open canary
  stable [--promote|-p] [--rollback|-r] [--no-open]
                                   Switch to stable, promote canary to stable, or roll back stable
  promote [--no-open] [--open]     Alias for 'stable --promote'
  rollback [--no-open]             Alias for 'stable --rollback'
  quit                             Quit running Operon and wait for lock release
  help, --help, -h                 Show this help

Environment overrides:
  OPERON_DIST_DIR                  Directory for canary and rollback bundles (default: <repo>/dist)
  OPERON_CANARY_APP                Canary bundle path (default: $OPERON_DIST_DIR/Operon.app)
  OPERON_STABLE_APP                Stable bundle path (default: /Applications/Operon Stable.app)
  OPERON_STABLE_PREVIOUS_APP       Stable rollback bundle path (default: $OPERON_DIST_DIR/Operon.stable-previous.app)
  OPERON_QUIT_WAIT_SECONDS         Seconds to wait for running Operon to exit (default: 10)
EOF
}

bundle_commit() {
  local plist="$1/Contents/Info.plist"
  if [ -f "$plist" ]; then
    /usr/libexec/PlistBuddy -c "Print :$stamp_key" "$plist" 2>/dev/null || printf 'unstamped'
  else
    printf 'unstamped'
  fi
}

print_channel_line() {
  local label="$1"
  local app="$2"
  if [ ! -d "$app" ]; then
    printf '%s: %s (missing)\n' "$label" "$app"
    return
  fi
  local commit
  commit="$(bundle_commit "$app")"
  printf '%s: %s (%s)\n' "$label" "$app" "$commit"
}

# Match both packaged `.app` bundles (`Contents/MacOS/Operon`) and cargo builds (`operon`).
running_operon_pids() {
  local names="${OPERON_PROCESS_NAMES:-Operon operon}"
  local name
  for name in $names; do
    pgrep -x "$name" 2>/dev/null || true
  done | awk '!seen[$0]++'
}

open_bundle() {
  local app="$1"
  if [ -n "${OPERON_CHANNEL_OPEN_CMD:-}" ]; then
    "$OPERON_CHANNEL_OPEN_CMD" "$app"
  else
    open -a "$app"
  fi
}

quit_running_operon() {
  if [ -n "${OPERON_CHANNEL_QUIT_HOOK:-}" ]; then
    "$OPERON_CHANNEL_QUIT_HOOK"
    return 0
  fi

  local pids
  pids="$(running_operon_pids)"
  if [ -z "$pids" ]; then
    return 0
  fi

  if [ -z "${OPERON_PROCESS_NAMES:-}" ] && command -v osascript >/dev/null 2>&1; then
    osascript -e 'tell application id "local.operon" to quit' >/dev/null 2>&1 || true
  fi
  # shellcheck disable=SC2086
  kill -TERM $pids 2>/dev/null || true

  local max_ticks=$((quit_wait_seconds * 10))
  local tick=0
  while [ "$tick" -lt "$max_ticks" ]; do
    if [ -z "$(running_operon_pids)" ]; then
      return 0
    fi
    sleep 0.1
    tick=$((tick + 1))
  done

  if [ -z "$(running_operon_pids)" ]; then
    return 0
  fi

  echo "error: operon is still running after ${quit_wait_seconds}s; close it before switching channels" >&2
  exit 75
}

cmd_status() {
  if [ "$#" -gt 0 ]; then
    echo "error: 'status' takes no arguments" >&2
    usage >&2
    exit 64
  fi

  local head_sha="unknown"
  local tree_state="unknown"
  if git -C "$repo_root" rev-parse HEAD >/dev/null 2>&1; then
    head_sha="$(git -C "$repo_root" rev-parse HEAD)"
    if [ -n "$(git -C "$repo_root" status --porcelain --untracked-files=no 2>/dev/null || true)" ]; then
      tree_state="dirty"
    else
      tree_state="clean"
    fi
  fi
  printf 'head: %s (%s)\n' "$head_sha" "$tree_state"

  if [ -n "${OPERON_CHANNEL_RUNNING_OVERRIDE:-}" ]; then
    printf 'running: %s\n' "$OPERON_CHANNEL_RUNNING_OVERRIDE"
  else
    local pids
    pids="$(running_operon_pids | tr '\n' ' ' | sed 's/ *$//')"
    if [ -z "$pids" ]; then
      printf 'running: none\n'
    else
      printf 'running: pid %s\n' "$pids"
    fi
  fi

  print_channel_line "canary" "$canary_app"
  print_channel_line "stable" "$stable_app"
}

cmd_canary() {
  local need_build=0
  local full_build=0
  local no_open=0
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --build|-b) need_build=1 ;;
      --full) need_build=1; full_build=1 ;;
      --no-open) no_open=1 ;;
      *)
        echo "error: unknown option for canary: $1" >&2
        usage >&2
        exit 64
        ;;
    esac
    shift
  done

  if [ "$need_build" -eq 1 ] || [ ! -d "$canary_app" ]; then
    if [ ! -f "$packager" ]; then
      echo "error: packager script not found at $packager" >&2
      exit 66
    fi
    if [ "$full_build" -eq 1 ]; then
      OPERON_DIST_DIR="$dist_dir" OPERON_LOCKED=1 OPERON_FAST_PACKAGE=0 bash "$packager" --full
    else
      OPERON_DIST_DIR="$dist_dir" OPERON_LOCKED=1 OPERON_FAST_PACKAGE=1 bash "$packager"
    fi
  fi

  if [ ! -d "$canary_app" ]; then
    echo "error: canary bundle is missing at $canary_app" >&2
    exit 66
  fi

  if [ "$no_open" -eq 1 ]; then
    printf 'Canary ready: %s (%s)\n' "$canary_app" "$(bundle_commit "$canary_app")"
  else
    quit_running_operon
    open_bundle "$canary_app"
    printf 'Opened canary: %s (%s)\n' "$canary_app" "$(bundle_commit "$canary_app")"
  fi
}

do_promote() {
  local no_open="$1"

  if [ ! -d "$canary_app" ]; then
    echo "error: canary bundle not found at $canary_app — run 'operon-channel.sh canary --build' first" >&2
    exit 66
  fi
  if [ ! -f "$replace_script" ]; then
    echo "error: bundle replacement helper not found at $replace_script" >&2
    exit 66
  fi

  mkdir -p "$(dirname "$stable_app")" "$(dirname "$stable_prev_app")"
  stage_app="${stable_app}.staging.$$"
  rm -rf "$stage_app"
  cp -R "$canary_app" "$stage_app"

  if ! bash "$replace_script" "$stage_app" "$stable_app" "$stable_prev_app"; then
    rm -rf "$stage_app"
    stage_app=""
    echo "error: failed to promote $canary_app to $stable_app" >&2
    exit 1
  fi
  stage_app=""

  printf 'Promoted canary -> stable: %s (%s)\n' "$stable_app" "$(bundle_commit "$stable_app")"

  if [ "$no_open" -eq 0 ]; then
    quit_running_operon
    open_bundle "$stable_app"
    printf 'Opened stable: %s (%s)\n' "$stable_app" "$(bundle_commit "$stable_app")"
  fi
}

do_rollback() {
  local no_open="$1"

  if [ ! -d "$stable_prev_app" ]; then
    echo "error: rollback bundle not found at $stable_prev_app" >&2
    exit 66
  fi
  if [ ! -f "$replace_script" ]; then
    echo "error: bundle replacement helper not found at $replace_script" >&2
    exit 66
  fi

  quit_running_operon

  mkdir -p "$(dirname "$stable_app")"
  stage_app="${stable_app}.rollback-staging.$$"
  scratch_backup="${stable_app}.rolled-back.$$"
  rm -rf "$stage_app" "$scratch_backup"
  cp -R "$stable_prev_app" "$stage_app"

  if ! bash "$replace_script" "$stage_app" "$stable_app" "$scratch_backup"; then
    rm -rf "$stage_app" "$scratch_backup"
    stage_app=""
    scratch_backup=""
    echo "error: failed to roll back $stable_app from $stable_prev_app" >&2
    exit 1
  fi
  rm -rf "$scratch_backup"
  stage_app=""
  scratch_backup=""

  if [ "$no_open" -eq 1 ]; then
    printf 'Rolled back stable: %s (%s)\n' "$stable_app" "$(bundle_commit "$stable_app")"
  else
    open_bundle "$stable_app"
    printf 'Rolled back stable and opened: %s (%s)\n' "$stable_app" "$(bundle_commit "$stable_app")"
  fi
}

cmd_promote() {
  local no_open=0
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --open) no_open=0 ;;
      --no-open) no_open=1 ;;
      *)
        echo "error: unknown option for promote: $1" >&2
        usage >&2
        exit 64
        ;;
    esac
    shift
  done
  do_promote "$no_open"
}

cmd_stable() {
  local promote=0
  local rollback=0
  local no_open=0
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --promote|-p) promote=1 ;;
      --rollback|-r) rollback=1 ;;
      --no-open) no_open=1 ;;
      *)
        echo "error: unknown option for stable: $1" >&2
        usage >&2
        exit 64
        ;;
    esac
    shift
  done

  if [ "$promote" -eq 1 ] && [ "$rollback" -eq 1 ]; then
    echo "error: cannot combine --promote and --rollback" >&2
    usage >&2
    exit 64
  fi

  if [ "$promote" -eq 1 ]; then
    do_promote "$no_open"
    return 0
  fi

  if [ "$rollback" -eq 1 ]; then
    do_rollback "$no_open"
    return 0
  fi

  if [ ! -d "$stable_app" ]; then
    echo "error: stable bundle not found at $stable_app — run 'operon-channel.sh stable --promote' after verifying canary" >&2
    exit 66
  fi

  if [ "$no_open" -eq 1 ]; then
    printf 'Stable ready: %s (%s)\n' "$stable_app" "$(bundle_commit "$stable_app")"
  else
    quit_running_operon
    open_bundle "$stable_app"
    printf 'Opened stable: %s (%s)\n' "$stable_app" "$(bundle_commit "$stable_app")"
  fi
}

cmd_rollback() {
  local no_open=0
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --no-open) no_open=1 ;;
      *)
        echo "error: unknown option for rollback: $1" >&2
        usage >&2
        exit 64
        ;;
    esac
    shift
  done
  do_rollback "$no_open"
}

cmd_quit() {
  if [ "$#" -gt 0 ]; then
    echo "error: 'quit' takes no arguments" >&2
    usage >&2
    exit 64
  fi
  quit_running_operon
  printf 'Operon stopped\n'
}

subcmd="${1:-status}"
if [ "$#" -gt 0 ]; then
  shift
fi

case "$subcmd" in
  status)
    cmd_status "$@"
    ;;
  canary)
    cmd_canary "$@"
    ;;
  promote)
    cmd_promote "$@"
    ;;
  stable)
    cmd_stable "$@"
    ;;
  rollback)
    cmd_rollback "$@"
    ;;
  quit)
    cmd_quit "$@"
    ;;
  help|--help|-h)
    usage
    ;;
  *)
    echo "error: unknown subcommand '$subcmd'" >&2
    usage >&2
    exit 64
    ;;
esac
