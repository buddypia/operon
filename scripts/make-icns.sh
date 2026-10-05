#!/usr/bin/env bash
set -euo pipefail

# Usage: bash scripts/make-icns.sh
# Regenerates assets/Operon.icns from the full-bleed 1024x1024 master PNG.
#
# The master must stay opaque edge to edge. macOS 26 composites a legacy .icns
# whose corners are transparent onto a white rounded plate, so the same bundle
# renders with a white border in some surfaces and without one in others. A
# full-bleed square lets the system apply its own squircle mask instead, which
# is identical everywhere.
#
# Uses only sips and iconutil, both part of macOS.

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
master="$repo_root/assets/operon-icon-1024.png"
target="$repo_root/assets/Operon.icns"

if [[ ! -f "$master" ]]; then
  echo "Missing icon master: $master" >&2
  exit 1
fi

read -r width height <<<"$(sips -g pixelWidth -g pixelHeight "$master" | awk '/pixelWidth/ {w=$2} /pixelHeight/ {h=$2} END {print w, h}')"
if [[ "$width" != "1024" || "$height" != "1024" ]]; then
  echo "Icon master must be 1024x1024, got ${width}x${height}." >&2
  exit 1
fi
if sips -g hasAlpha "$master" | grep "hasAlpha: yes" >/dev/null; then
  echo "Icon master must be opaque; transparent corners make macOS 26 add a white plate." >&2
  exit 1
fi

staging="$(mktemp -d "${TMPDIR:-/tmp}/operon-icns.XXXXXX")"
cleanup() {
  status=$?
  rm -rf "$staging"
  return "$status"
}
trap cleanup EXIT

iconset="$staging/Operon.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$master" --out "$iconset/icon_${size}x${size}.png" >/dev/null
  sips -z "$((size * 2))" "$((size * 2))" "$master" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done

iconutil --convert icns --output "$staging/Operon.icns" "$iconset"
mv "$staging/Operon.icns" "$target"

echo "Wrote: $target"
