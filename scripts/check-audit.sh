#!/usr/bin/env bash
# Judge a `cargo audit --json` report by what the macOS app builds. Change 195.
#
#   bash scripts/check-audit.sh <audit.json>
#
# `cargo audit` reads all of Cargo.lock, which lists every platform's
# dependencies. The weekly scan failed on 2026-10-12 and 10-19 for
# RUSTSEC-2026-0194 and -0195 in quick-xml 0.30.0, which arrives only through
# accesskit_unix → atspi → zbus_xml: Linux accessibility, never compiled for
# macOS. A scan that is red for a crate the app does not contain is read as
# noise, and the next red that matters is read the same way.
#
# So a vulnerability fails only when its exact name and version are in the
# dependency tree cargo builds for the two Apple targets, build scripts
# included; the rest are listed as not built, so they stay visible. This is
# not an ignore list: a crate that starts to be built for macOS fails the day it
# does, with no entry to remove.
#
# Exit 0 when nothing built is vulnerable, 1 when something is, 2 when the
# report or the tree cannot be read — a scan that cannot tell is not a pass.
set -uo pipefail

report="${1:-}"
[ -f "$report" ] || { echo "usage: check-audit.sh <audit.json>" >&2; exit 2; }

# The shape first, on its own: jq reads an empty file as no input and exits 0,
# and `cargo audit --json > audit.json || true` leaves exactly that when the
# advisory database cannot be fetched — a scan that never ran, read as clean.
jq -e '.vulnerabilities.list | type == "array"' "$report" >/dev/null 2>&1 ||
  { echo "check-audit: $report is not a cargo audit JSON report" >&2; exit 2; }
found=$(jq -r '.vulnerabilities.list[] | "\(.package.name)@\(.package.version) \(.advisory.id) \(.advisory.title)"' "$report") ||
  { echo "check-audit: $report is not a cargo audit JSON report" >&2; exit 2; }

tree=$(cargo tree --locked --target aarch64-apple-darwin --target x86_64-apple-darwin \
  -e normal,build --prefix none --format '{p}') ||
  { echo "check-audit: cargo tree failed, so what the app builds is unknown" >&2; exit 2; }
built=$(printf '%s\n' "$tree" | awk 'NF >= 2 { sub(/^v/, "", $2); print $1 "@" $2 }' | sort -u)

status=0
while IFS= read -r line; do
  [ -n "$line" ] || continue
  crate=${line%% *}
  # A here-string, not a pipe: under `pipefail` a `grep -q` that stops at its
  # first match can hand the pipeline printf's SIGPIPE, and a match reads as a
  # miss. Entry 018 in docs/sdlc/lessons.md.
  if grep -qxF "$crate" <<< "$built"; then
    echo "built for macOS: $line"
    status=1
  else
    echo "not built for macOS: $line"
  fi
done <<< "$found"
[ "$status" -eq 0 ] && echo "check-audit: no vulnerability in a crate the macOS app builds"
exit "$status"
# operon: end of scripts/check-audit.sh
