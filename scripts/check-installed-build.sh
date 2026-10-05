#!/usr/bin/env bash
set -euo pipefail

# Usage: bash scripts/check-installed-build.sh [bundle]
#
# Is the installed application built from what this repository now holds?
#
# Exit 0  — it is, or the question cannot be asked here (no bundle, no stamp, no
#           release-binary commit, no git, no readable packager, or a git that
#           could not answer). A machine that has never installed the app is not
#           behind, and neither is one whose bundle predates the stamp: an
#           unstamped bundle is UNKNOWN, not STALE. Every bundle built before
#           change 067 is unstamped, and calling those stale would make this
#           check noise from the day it landed.
# Exit 1  — the installed bundle is behind, or carries a stamp this repository
#           has never heard of, or carries one that is not an object name at all.
#
# NOTHING HERE WRITES. Every branch is a read and an `echo`; the one call that
# takes an output path sends it to /dev/null and leaves its input byte-identical,
# which was measured against both plist encodings. So the check is idempotent and
# safe to run from a gate on every stop.
#
# Read by an agent and by a person at a terminal, so the output is English like
# the rest of scripts/. `.claude/hooks/gate-stop.sh` turns a non-zero exit into
# its own Japanese line.

# Answer about the repository this script lives in, and nothing else. `git -C`
# does NOT override `GIT_DIR`/`GIT_WORK_TREE`, and a session in a worktree has
# both set — `.claude/scripts/worktree-init.mjs` writes them into the worktree's
# settings. `.claude/hooks/gate-stop.sh` may run this script from a tree those
# pointers do not name, and a stamp compared against the wrong history reads as
# "installed from another branch". Change 065's lesson, one directory over.
unset GIT_DIR GIT_WORK_TREE

# A path that begins with a hyphen is a path, and every program below would read
# one as an option. Both of these were carried out of change 067's review as
# nits, and both are silent wrong answers rather than errors, which is what makes
# them worth the two words: `dirname` on a hyphen-leading path prints NOTHING and
# exits 1, so without `--` this line becomes `cd "/.."` and succeeds, and the
# script then answers about the filesystem root instead of about itself.
# `.claude/hooks/gate-stop.sh` was written with `cd -- "$(dirname -- "$0")"` in
# the same change; this is the other half of one habit.
repo_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"
bundle="${1:-/Applications/Operon.app}"
# The same hazard one argument over. `./` makes a hyphen-leading name a path
# again without refusing a name somebody legitimately gave a bundle — `./x` and
# `x` name the same file, and they differ only in the one case this is for.
case "$bundle" in -*) bundle="./$bundle" ;; esac
packager="$repo_root/scripts/package-macos.sh"

# What counts as a release-binary source, spelled once. `src/tests.rs` is behind
# `#[cfg(test)] mod tests;` and never reaches the shipped binary — which is the
# whole reason changes 065 and 066 needed no repackaging.
release_sources=(src Cargo.toml Cargo.lock ':(exclude)src/tests.rs')

# The stamp key is read out of the packager rather than spelled again here. The
# packager writes it; this script only has to find it. `.claude/rules/identifiers.md`
# applied to a pair of shell scripts.
#
# Guarded by a readability test rather than left to `set -e`. Measured against
# the version before this guard: with the packager missing, `sed` printed its own
# error and the script died at exit 1 — which the header above promises means
# "the bundle is behind", so a checkout with no packager reported every bundle as
# stale-or-unknowable through a code nobody had chosen. The absence of a packager
# is a question that cannot be asked, and that is exit 0 by this file's contract.
stamp_key=""
if [ -r "$packager" ]; then
  stamp_key="$(sed -n 's/^source_commit_key="\([^"]*\)"$/\1/p' "$packager" 2>/dev/null | head -1 || true)"
fi
# The key is spent as an argument to `plutil`, so it may not be able to look like
# an option or carry a shell-significant byte. An identifier that is not one is
# treated as no declaration at all, which lands on the branch that already exists
# for that and says so in words a reader can act on.
case "$stamp_key" in
  *[!A-Za-z0-9_]*) stamp_key="" ;;
esac

if ! git -C "$repo_root" rev-parse --git-dir >/dev/null 2>&1; then
  echo "installed build: UNKNOWN — $repo_root is not a git repository"
  exit 0
fi

head_commit="$(git -C "$repo_root" log -1 --format=%H -- "${release_sources[@]}" || true)"
if [ -z "$head_commit" ]; then
  echo "installed build: UNKNOWN — no commit has touched a release-binary source yet"
  exit 0
fi

if [ ! -d "$bundle" ]; then
  echo "installed build: NONE — $bundle is not installed, so it cannot be behind"
  exit 0
fi

if [ -z "$stamp_key" ]; then
  echo "installed build: UNKNOWN — $packager does not stamp a source commit, so"
  echo "  no bundle it builds can be compared. Newest release-binary commit: $head_commit"
  exit 0
fi

plist="$bundle/Contents/Info.plist"

# Two different bundles wear the same `plutil -extract` failure: one whose
# Info.plist reads and carries no stamp, and one whose Info.plist does not read
# at all. Both end UNKNOWN and exit 0, and they have different answers — the
# first is merely older than the stamp, the second is damaged and has to be
# rebuilt today — so the sentence has to tell them apart even though the exit
# code does not. Carried out of change 067's review.
#
# `-convert … -o /dev/null` is the discriminator that names no key of its own,
# so the stamp key is still spelled once and only in the packager. `-o` is where
# the output goes; the input is not touched, which the stamp read below would
# notice if it were.
if ! plutil -convert xml1 -o /dev/null "$plist" 2>/dev/null; then
  echo "installed build: UNKNOWN — $plist is missing or does not parse as a property"
  echo "  list, so no stamp can be read from it. This bundle is damaged rather than"
  echo "  old. Rebuild it with: bash scripts/package-macos.sh"
  exit 0
fi

installed_commit="$(plutil -extract "$stamp_key" raw -o - "$plist" 2>/dev/null || true)"
if [ -z "$installed_commit" ]; then
  echo "installed build: UNKNOWN — $bundle carries no $stamp_key (built before the"
  echo "  stamp existed). Newest release-binary commit: $head_commit"
  exit 0
fi

# THE STAMP IS THE ONE VALUE HERE THAT COMES FROM OUTSIDE THE REPOSITORY. It is
# read from a property list under /Applications, which anyone who can write there
# chooses — and every line below hands it to git as a REVISION. Git's revision
# grammar is far wider than "a commit id": `HEAD`, a branch name, `@{1}` and
# `:/message` all resolve. Measured against the version before this guard, a
# bundle stamped with the four characters `HEAD` reported:
#
#   installed build: CURRENT — …/Operon.app was built from HEAD, which
#     includes the newest release-binary commit 5dae14e…
#
# which is this check answering "yes" to its own question for any bundle at all,
# including one that is genuinely years behind. So a stamp has to look like an
# object name before it is used as one: hexadecimal, and the full width of a
# sha1 or a sha256 object id. A resolvable name that is not one is refused
# exactly like an unknown commit, which is the answer that was already correct
# for "this value names nothing I can compare".
stamp_is_an_object_name=no
case "$installed_commit" in
  *[!0-9a-f]*) ;;
  *) case "${#installed_commit}" in 40 | 64) stamp_is_an_object_name=yes ;; esac ;;
esac
if [ "$stamp_is_an_object_name" = no ]; then
  echo "installed build: UNKNOWN COMMIT — $bundle carries a $stamp_key that is not an"
  echo "  object name: \"$installed_commit\". The stamp is read from outside this"
  echo "  repository and is spent as a git revision, so a value git would resolve to"
  echo "  something else — a branch, HEAD, a reflog entry — is refused rather than"
  echo "  followed. Rebuild the bundle with: bash scripts/package-macos.sh"
  exit 1
fi

if ! git -C "$repo_root" cat-file -e "${installed_commit}^{commit}" 2>/dev/null; then
  echo "installed build: UNKNOWN COMMIT — $bundle was built from $installed_commit,"
  echo "  which is not in this repository. Installed from another branch, or from a"
  echo "  commit since rewritten; nothing here can say whether it is behind."
  exit 1
fi

# `merge-base --is-ancestor` has three answers and the caller used to see two:
# 0 is "yes", 1 is "no", and anything else — 128 for a broken object, a
# permission failure, a resource limit — is "I could not tell you". Read as a
# plain `if`, that third answer printed STALE and exited 1, so a transient git
# failure became a positive claim that the bundle was behind, and the gate above
# turned it into a refusal. A check that cannot answer says so.
ancestry=0
git -C "$repo_root" merge-base --is-ancestor "$head_commit" "$installed_commit" 2>/dev/null || ancestry=$?

case "$ancestry" in
  0)
    echo "installed build: CURRENT — $bundle was built from $installed_commit, which"
    echo "  includes the newest release-binary commit $head_commit"
    exit 0
    ;;
  1)
    echo "installed build: STALE — $bundle was built from $installed_commit; the newest"
    echo "  commit touching a release-binary source is $head_commit."
    echo "  Rebuild and swap it with: bash scripts/package-macos.sh"
    exit 1
    ;;
  *)
    echo "installed build: UNKNOWN — git could not compare $installed_commit with"
    echo "  $head_commit (merge-base exit $ancestry). Both objects are present, so this"
    echo "  is the repository failing to answer rather than the bundle being behind."
    exit 0
    ;;
esac
