#!/usr/bin/env bash
# May this command install a build into /Applications/Operon.app?
#
# Stage 5's gate, checked rather than asked. `.claude/hooks/guard-bash.sh` used to
# stop the session here and put three questions to a person: did the three gates
# pass on this tree, did dist/Operon.app come from the packaging script unedited,
# and was the running Operon quit. All three are facts about the working directory
# at that instant, so the person answering them was either re-running the commands
# the session had just run, or answering from memory — and answering from memory is
# the failure the gate was built to prevent. A prompt asked in a session that has
# already granted blanket approval is a prompt that gets clicked through.
#
# So this establishes the same three conditions, plus a fourth the prompt never
# asked at all: that the command is the canonical swap and not some other
# invocation of the same script.
#
#   bash scripts/check-release-preconditions.sh "<the shell command being run>"
#
#   exit 0   every condition holds; stdout is one line naming what was established
#   exit 1   a condition failed; stdout says which, and what to do
#   exit 2   the check could not be run at all
#
# It fails closed. There is no flag, no environment variable, and no argument that
# returns 0 without running all four checks — a bypass that exists is a bypass that
# becomes the default, and in a transcript it is indistinguishable from a pass.
set -uo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || {
  echo "the repository root could not be resolved from ${BASH_SOURCE[0]}"
  exit 2
}
cd "$repo_root" || { echo "cannot enter $repo_root"; exit 2; }

command_line="${1-}"
if [ -z "$command_line" ] && [ ! -t 0 ]; then
  command_line=$(cat)
fi
if [ -z "$command_line" ]; then
  echo "no command to check. Usage: check-release-preconditions.sh \"<command>\""
  exit 2
fi

# Output budget. A failing gate can print a whole test run and a diff can list a
# whole bundle; both end up inside a hook's reason text, which an agent reads in
# full. Two ceilings, stated where they are enforced.
TAIL_BYTES=2000
DIFF_LINES=20

fail() { printf '%s\n' "$1"; exit 1; }

# --- 1. is this the canonical swap? ------------------------------------------
#
# guard-bash.sh matches the script name anywhere in the command, which is what
# lets it fire on a real release line — the one that prompted this change also
# piped to tail and ran codesign afterwards. But `allow` must not be handed to a
# swap whose arguments are not the ones this check went on to verify, so the three
# tokens after the script name have to be exactly the triple package-macos.sh
# produces and AGENTS.md installs.
#
# Derived the same way scripts/package-macos.sh derives them, rather than typed
# out again: a guard that restates the value it guards goes blind on the rename it
# exists to survive (lesson 004).
dist_dir="${OPERON_DIST_DIR:-$repo_root/dist}"
expected_staged="$dist_dir/Operon.app"
expected_bundle="/Applications/Operon.app"
expected_previous="$dist_dir/Operon.previous.app"

case "$command_line" in
  *replace-macos-bundle.sh*) ;;
  *) fail "this command does not invoke replace-macos-bundle.sh, so there is nothing here to authorize." ;;
esac

rest=${command_line#*replace-macos-bundle.sh}
read -r arg_staged arg_bundle arg_previous _ <<<"$rest"

for token in "$arg_staged" "$arg_bundle" "$arg_previous"; do
  case "$token" in
    '' | *[!A-Za-z0-9/._-]*)
      fail "the swap's arguments are not three plain paths, so what is being installed cannot be established from the command:
    $rest

Expected exactly:
    $expected_staged $expected_bundle $expected_previous"
      ;;
  esac
done

if [ "$arg_staged" != "$expected_staged" ] ||
  [ "$arg_bundle" != "$expected_bundle" ] ||
  [ "$arg_previous" != "$expected_previous" ]; then
  fail "this is not the canonical release swap.

    seen:     $arg_staged $arg_bundle $arg_previous
    expected: $expected_staged $expected_bundle $expected_previous

Only the swap of the packaged bundle into the installed app is checked here. Any
other invocation is a person's decision."
fi

[ -d "$expected_staged" ] || fail "$expected_staged does not exist. Build it with 'bash scripts/package-macos.sh' first."

# --- 2. do the three gates pass on the tree as it stands? ---------------------
#
# Run now, not recalled. A tree can move between the commit that passed the gates
# and the swap that installs it, and closing that gap is the whole point of
# checking here rather than trusting gate-commit.sh.

gate() {
  local label="$1"
  shift
  local output
  if ! output=$("$@" 2>&1); then
    fail "$label failed on this tree, so the build about to be installed is not verified.

$(printf '%s' "$output" | tail -c "$TAIL_BYTES")"
  fi
}

command -v cargo >/dev/null 2>&1 || { echo "cargo is not on PATH; the three gates cannot be run"; exit 2; }

gate "cargo fmt --check" cargo fmt --check
# The suite is CI's when CI ran exactly this tree: `--worktree` refuses any
# edit or untracked file, and a passing run has to be on HEAD or on a parent
# with HEAD's tree (change 128). Anything else — local edits, a commit never
# pushed, CI out of reach — and it runs here, as it always did.
if ci_verdict=$(cd "$repo_root" && bash scripts/ci-verified.sh --worktree HEAD 2>&1); then
  suite="cargo test --locked: $ci_verdict"
else
  gate "cargo test --locked" cargo test --locked
  suite="cargo test --locked: run on this tree"
fi
gate "cargo clippy --locked -- -D warnings" cargo clippy --locked -- -D warnings

# --- 3. is the staged bundle what this tree packages? -------------------------
#
# Not "is it signed" — a valid signature only proves nothing changed after
# signing, and says nothing about which tree it was built from. Packaging is
# byte-reproducible: two runs from the same target/release/operon, into two
# different directories, produce diff -r identical bundles, because ad-hoc
# codesign embeds neither a timestamp nor a path. So re-running the script is a
# total check of both "came from the script" and "came from this tree".

scratch=""
cleanup() {
  status=$?
  if [ -n "$scratch" ] && [ -d "$scratch" ]; then
    rm -rf "$scratch"
  fi
  return "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

scratch=$(mktemp -d "${TMPDIR:-/tmp}/operon-release-check.XXXXXX") ||
  { echo "a scratch directory for the reference build could not be created"; exit 2; }

if ! package_output=$(OPERON_DIST_DIR="$scratch" bash "$repo_root/scripts/package-macos.sh" 2>&1); then
  fail "a reference build could not be packaged, so $expected_staged cannot be checked against this tree.

$(printf '%s' "$package_output" | tail -c "$TAIL_BYTES")"
fi

if ! bundle_diff=$(diff -r "$scratch/Operon.app" "$expected_staged" 2>&1); then
  fail "$expected_staged is not what this tree packages. It was either built from a different tree or edited by hand.

$(printf '%s\n' "$bundle_diff" | head -n "$DIFF_LINES")

Rebuild it with 'bash scripts/package-macos.sh'."
fi

# --- 4. is anything still running? -------------------------------------------
#
# Checked last, so the answer is as close to the swap as this script can put it.
# The installed app and a source build take the same flock on the session index,
# so a swap under a live process either verifies the old executable or hits the
# single-instance warning.
#
# Matched on the process name rather than the command line: this script is invoked
# with the whole shell command as an argument, and that command routinely names
# the bundle executable's path, so `pgrep -f` would find itself.
running=""
for name in Operon operon; do
  pids=$(pgrep -x "$name" 2>/dev/null) || continue
  for pid in $pids; do
    running="$running
    pid $pid  $(ps -p "$pid" -o comm= 2>/dev/null)"
  done
done
if [ -n "$running" ]; then
  fail "Operon is still running, so the swap would install under a live single-instance lock:
$running

Quit it, then install — the verification has to use the new executable."
fi

echo "checked: the canonical swap; cargo fmt/clippy on this tree; $suite; $expected_staged reproduced byte-for-byte by scripts/package-macos.sh; no Operon running. The six #[ignore]d tests were not run — they need a live agent CLI."
