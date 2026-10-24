#!/usr/bin/env bash
# Usage: bash scripts/install-prune-schedule.sh [--print | --uninstall]
# Installs a per-user launchd agent that trims build output once a week.
# Change 220.
#
#   (no flag)    write ~/Library/LaunchAgents/com.buddypia.operon-prune-target.plist
#                and load it; an already-loaded copy is replaced
#   --print      write the plist to stdout and touch nothing
#   --uninstall  unload the agent and remove its plist
#
# Every Monday at 10:00 it runs, from the main checkout,
#   bash scripts/prune-target.sh --all-worktrees --if-idle
# which trims the old test programs and stale incremental caches under target/ of
# the main checkout and of every worktree, and does nothing while cargo or rustc is
# running. launchd runs a job missed while the Mac slept when it wakes. Output goes
# to ~/Library/Logs/operon-prune-target.log. `make prune.unschedule` removes it.
#
# The main checkout is found from git, not from the working directory, so the agent
# points at the same script wherever this runs from (a worktree included).
set -euo pipefail

label="com.buddypia.operon-prune-target"
agent_dir="$HOME/Library/LaunchAgents"
plist_path="$agent_dir/$label.plist"
log_path="$HOME/Library/Logs/operon-prune-target.log"
mode="install"

case "${1:-}" in
  "") ;;
  --print) mode="print" ;;
  --uninstall) mode="uninstall" ;;
  *)
    echo "usage: install-prune-schedule.sh [--print | --uninstall]" >&2
    exit 64
    ;;
esac

script_dir="$(cd "$(dirname "$0")" && pwd)"
common="$(git -C "$script_dir" rev-parse --path-format=absolute --git-common-dir)"
main_root="$(cd "$common/.." && pwd -P)"

plist() {
  cat <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$label</string>
  <key>ProgramArguments</key>
  <array>
    <string>/bin/bash</string>
    <string>$main_root/scripts/prune-target.sh</string>
    <string>--all-worktrees</string>
    <string>--if-idle</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict>
    <key>Weekday</key>
    <integer>1</integer>
    <key>Hour</key>
    <integer>10</integer>
    <key>Minute</key>
    <integer>0</integer>
  </dict>
  <key>StandardOutPath</key>
  <string>$log_path</string>
  <key>StandardErrorPath</key>
  <string>$log_path</string>
</dict>
</plist>
EOF
}

case "$mode" in
  print)
    plist
    ;;
  uninstall)
    launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
    rm -f "$plist_path"
    echo "登録を解除しました。"
    ;;
  install)
    mkdir -p "$agent_dir" "$(dirname "$log_path")"
    plist > "$plist_path"
    plutil -lint "$plist_path" >/dev/null
    launchctl bootout "gui/$(id -u)/$label" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$plist_path"
    echo "毎週月曜 10:00 に target/ を掃除するよう登録しました。"
    ;;
esac

# operon: end of scripts/install-prune-schedule.sh
