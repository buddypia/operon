#!/usr/bin/env bash
set -euo pipefail

# Usage: bash scripts/package-macos.sh [--fast]
# Produces a self-contained .app bundle under dist/.
# When --fast or OPERON_FAST_PACKAGE=1 is set, skips cargo test --locked (offloaded to CI/CD).

fast_mode=0
for arg in "$@"; do
  if [[ "$arg" == "--fast" ]]; then
    fast_mode=1
    break
  fi
done
if [[ "${OPERON_FAST_PACKAGE:-0}" == "1" ]]; then
  fast_mode=1
fi

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
dist_dir="${OPERON_DIST_DIR:-$repo_root/dist}"
bundle="$dist_dir/Operon.app"
binary="$repo_root/target/release/operon"
lock_file="$dist_dir/.operon-package.lockfile"

cd "$repo_root"
mkdir -p "$dist_dir"
# Keep one stable inode and hold its BSD advisory lock for this shell's entire
# lifetime. Unlike PID-directory recovery, the kernel releases this lock on
# crash and cannot admit two simultaneous stale-lock reclaimers.
exec 9>"$lock_file"
if ! /usr/bin/lockf -s -t 0 9; then
  echo "Another Operon packaging operation is already running." >&2
  exit 1
fi
staging_root=""
cleanup() {
  status=$?
  if [[ -n "$staging_root" && -e "$staging_root" ]]; then
    rm -rf "$staging_root"
  fi
  exec 9>&-
  return "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# The installed application is what a person runs, so it passes the same three
# gates a commit does, over the tree it is built from — which may hold work no
# commit has gated (sdlc 096). Inside the lock, so two packagers do not both
# run the suite — but with fd 9 closed for each child (`9>&-`): the suite starts
# processes that can outlive it (a tmux server, a background sleep), and one
# that inherited the lock would hold it after this script exits, refusing every
# later packager with nothing running.
#
# When fast mode is requested (--fast or OPERON_FAST_PACKAGE=1), the heavy test
# suite is skipped in favor of CI/CD execution while formatting and clippy
# lint gates still run.
cargo fmt --check 9>&-
if [[ "$fast_mode" -eq 1 ]]; then
  echo "[packager] Fast mode: skipping test gate (delegated to CI/CD); running fmt & clippy."
else
  cargo test --locked 9>&-
fi
cargo clippy --locked -- -D warnings 9>&-
cargo build --release --locked 9>&-
staging_root="$(mktemp -d "$dist_dir/.operon-package.XXXXXX")"
staged_bundle="$staging_root/Operon.app"
mkdir -p "$staged_bundle/Contents/MacOS" "$staged_bundle/Contents/Resources"
cp "$binary" "$staged_bundle/Contents/MacOS/Operon"
cp "$repo_root/assets/Operon.icns" "$staged_bundle/Contents/Resources/Operon.icns"
cp "$repo_root/assets/operon-icon-1024.png" "$staged_bundle/Contents/Resources/operon-icon-1024.png"
plist="$staged_bundle/Contents/Info.plist"
plutil -create xml1 "$plist"
plutil -insert CFBundleDisplayName -string "Operon" "$plist"
plutil -insert CFBundleExecutable -string "Operon" "$plist"
plutil -insert CFBundleIdentifier -string "local.operon" "$plist"
plutil -insert CFBundleIconFile -string "Operon" "$plist"
plutil -insert CFBundleName -string "Operon" "$plist"
plutil -insert CFBundlePackageType -string "APPL" "$plist"
plutil -insert CFBundleShortVersionString -string "0.1.0" "$plist"
plutil -insert CFBundleVersion -string "1" "$plist"
plutil -insert LSMinimumSystemVersion -string "12.0" "$plist"
plutil -insert NSHighResolutionCapable -bool true "$plist"

# Which commit this bundle was built from, so the installed application can be
# compared against the repository instead of guessed at from a diff.
# `CFBundleShortVersionString` cannot answer that: it has read 0.1.0 since the
# first build, and a stamp that does not move cannot say whether one is behind.
#
# This key is spelled HERE and nowhere else. scripts/check-installed-build.sh
# reads the name out of the line below rather than repeating it, and
# `the_installed_build_check_reads_its_stamp_key_out_of_the_packager` fails if
# the two ever drift apart.
source_commit_key="OperonSourceCommit"
# No stamp at all when the commit cannot be determined — an unstamped bundle
# reads as "unknown", while a stamp of the word `unknown` would read as a commit
# from some other repository. The tree may hold uncommitted work on top; the
# stamp says the bundle is at least this commit, which is what staleness asks.
source_commit="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || true)"
if [ -n "$source_commit" ]; then
  plutil -insert "$source_commit_key" -string "$source_commit" "$plist"
fi

# Ad-hoc signing makes a locally-built bundle launchable on macOS without an
# Apple Developer certificate. Distribution outside this Mac requires proper
# Developer ID signing and notarization.
codesign --force --deep --sign - "$staged_bundle"
plutil -lint "$plist"
codesign --verify --deep --strict "$staged_bundle"

bash "$repo_root/scripts/replace-macos-bundle.sh" \
  "$staged_bundle" \
  "$bundle" \
  "$dist_dir/Operon.previous.app"

echo "Created: $bundle"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$bundle" 2>/dev/null || true
