#!/usr/bin/env bash
set -euo pipefail

# Atomically exchange a prepared macOS app bundle while retaining one last-good
# copy. macOS renameatx_np(RENAME_SWAP) guarantees that the canonical bundle
# path is never absent, including if this process is killed between steps.

if [[ "$#" -ne 3 ]]; then
  echo "Usage: replace-macos-bundle.sh STAGED_BUNDLE BUNDLE PREVIOUS_BUNDLE" >&2
  exit 64
fi

staged_bundle="$1"
bundle="$2"
previous_bundle="$3"
transaction_started=0
staged_inode=""
bundle_inode=""
previous_inode=""
had_previous=0

atomic_swap() {
  /usr/bin/swift -e '
import Darwin
let left = CommandLine.arguments[1]
let right = CommandLine.arguments[2]
if renameatx_np(AT_FDCWD, left, AT_FDCWD, right, UInt32(RENAME_SWAP)) != 0 {
    perror("renameatx_np")
    exit(1)
}
' "$1" "$2"
}

inode_of() {
  /usr/bin/stat -f '%i' "$1"
}

current_inode() {
  if [[ -e "$1" ]]; then
    inode_of "$1"
  fi
}

# A test pause ends on its own. A test that dies before signalling or releasing
# it would otherwise leave this script running for good, and inside the rollback
# it ignores TERM (sdlc 099). Returns 0 when RELEASE appeared, 1 when time ran out.
pause_for_test() {
  pause_release="$1"
  pause_deadline=$((SECONDS + ${OPERON_TEST_PAUSE_SECONDS:-60}))
  while ((SECONDS < pause_deadline)); do
    if [[ -n "$pause_release" && -e "$pause_release" ]]; then
      return 0
    fi
    sleep 0.05
  done
  return 1
}

pause_during_rollback_if_requested() {
  rollback_marker="${OPERON_TEST_DURING_ROLLBACK_MARKER:-}"
  rollback_release="${OPERON_TEST_DURING_ROLLBACK_RELEASE:-}"
  if [[ -n "$rollback_marker" && -n "$rollback_release" ]]; then
    : > "$rollback_marker"
    pause_for_test "$rollback_release" || true
  fi
}

cleanup() {
  status=$?
  trap - EXIT
  # Rollback is the critical section. A second signal must not interrupt the
  # remaining identity swaps and leave a partially restored generation.
  trap '' INT TERM
  if [[ "$status" -ne 0 && "$transaction_started" -eq 1 ]]; then
    # Shell variables cannot be updated atomically with renameatx_np. Infer the
    # completed transaction stage from the directory identities instead, so a
    # signal immediately before or after either swap cannot select the wrong
    # rollback generation.
    set +e
    rollback_ok=1
    current_staged_inode="$(current_inode "$staged_bundle")"
    current_bundle_inode="$(current_inode "$bundle")"
    current_previous_inode="$(current_inode "$previous_bundle")"

    if [[ "$current_bundle_inode" == "$staged_inode" ]]; then
      if [[ "$had_previous" -eq 1 ]]; then
        if [[ "$current_previous_inode" == "$bundle_inode" && "$current_staged_inode" == "$previous_inode" ]]; then
          if atomic_swap "$staged_bundle" "$previous_bundle"; then
            pause_during_rollback_if_requested
            current_staged_inode="$(current_inode "$staged_bundle")"
            current_previous_inode="$(current_inode "$previous_bundle")"
          else
            rollback_ok=0
          fi
        fi
        if [[ "$rollback_ok" -eq 1 && "$current_staged_inode" == "$bundle_inode" && "$current_previous_inode" == "$previous_inode" ]]; then
          if ! atomic_swap "$staged_bundle" "$bundle"; then
            rollback_ok=0
          fi
        else
          rollback_ok=0
        fi
      elif [[ -z "$current_staged_inode" && "$current_previous_inode" == "$bundle_inode" ]]; then
        if atomic_swap "$bundle" "$previous_bundle"; then
          pause_during_rollback_if_requested
          if ! mv "$previous_bundle" "$staged_bundle"; then
            rollback_ok=0
          fi
        else
          rollback_ok=0
        fi
      elif [[ "$current_staged_inode" == "$bundle_inode" && -z "$current_previous_inode" ]]; then
        if ! atomic_swap "$staged_bundle" "$bundle"; then
          rollback_ok=0
        fi
      else
        rollback_ok=0
      fi
    elif [[ "$current_bundle_inode" == "$bundle_inode" ]]; then
      if [[ "$had_previous" -eq 1 ]]; then
        if [[ "$current_staged_inode" != "$staged_inode" || "$current_previous_inode" != "$previous_inode" ]]; then
          rollback_ok=0
        fi
      elif [[ "$current_staged_inode" != "$staged_inode" || -n "$current_previous_inode" ]]; then
        rollback_ok=0
      fi
    else
      rollback_ok=0
    fi

    if [[ "$rollback_ok" -eq 1 ]]; then
      echo "Bundle replacement failed; the previous app bundle was restored." >&2
    else
      echo "Bundle replacement failed and automatic rollback was incomplete; the canonical app path was never removed." >&2
    fi
  fi
  return "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if [[ ! -e "$staged_bundle" ]]; then
  echo "Bundle replacement failed; staged bundle does not exist and the current app was left in place." >&2
  exit 66
fi

if [[ ! -e "$bundle" ]]; then
  mv "$staged_bundle" "$bundle"
  exit 0
fi

staged_inode="$(inode_of "$staged_bundle")"
bundle_inode="$(inode_of "$bundle")"
if [[ -e "$previous_bundle" ]]; then
  had_previous=1
  previous_inode="$(inode_of "$previous_bundle")"
fi
transaction_started=1

atomic_swap "$staged_bundle" "$bundle"

first_swap_marker="${OPERON_TEST_AFTER_FIRST_SWAP_MARKER:-${OPERON_TEST_AFTER_SWAP_MARKER:-}}"
if [[ -n "$first_swap_marker" ]]; then
  : > "$first_swap_marker"
  pause_for_test "" || { echo "Test pause expired; nobody signalled it." >&2; exit 1; }
fi
if [[ "${OPERON_TEST_FAIL_AFTER_FIRST_SWAP:-0}" == "1" ]]; then
  echo "Injected failure after the first atomic swap." >&2
  exit 75
fi

if [[ -e "$previous_bundle" ]]; then
  atomic_swap "$staged_bundle" "$previous_bundle"
else
  mv "$staged_bundle" "$previous_bundle"
fi

if [[ -n "${OPERON_TEST_AFTER_SECOND_SWAP_MARKER:-}" ]]; then
  : > "$OPERON_TEST_AFTER_SECOND_SWAP_MARKER"
  pause_for_test "" || { echo "Test pause expired; nobody signalled it." >&2; exit 1; }
fi
if [[ "${OPERON_TEST_FAIL_AFTER_SECOND_SWAP:-0}" == "1" ]]; then
  echo "Injected failure after the retained bundle was updated." >&2
  exit 76
fi

transaction_started=0

# After two atomic swaps the staging path contains the older retained build.
# It is no longer needed; the canonical and previous paths are both complete.
if [[ -e "$staged_bundle" ]]; then
  rm -rf "$staged_bundle"
fi
