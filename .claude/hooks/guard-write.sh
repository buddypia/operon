#!/usr/bin/env bash
# PreToolUse(Edit|Write|NotebookEdit): refuse writes that would corrupt a
# transaction owned by a script, and refuse writes that would put a credential
# in the diff.
#
# Stage 3 Play 4 of the AI-native SDLC: build-phase hooks are fast and scoped to
# the file being changed. Nothing here compiles or runs the suite — that belongs
# to gate-commit.sh at the commit boundary.
#
# Exit 2 blocks the tool call and the stderr text is the reason Claude reads.
set -uo pipefail

payload=$(cat)
path=$(printf '%s' "$payload" | jq -r '.tool_input.file_path // empty')

# --- generated output nobody edits by hand -----------------------------------

case "$path" in
  */dist/* )
    echo "Blocked: $path is generated output. dist/ is owned by the atomic swap in scripts/replace-macos-bundle.sh — rebuild it with 'bash scripts/package-macos.sh'." >&2
    exit 2
    ;;
  */target/* )
    echo "Blocked: $path is generated output. target/ is the cargo build cache." >&2
    exit 2
    ;;
esac

# --- files that are credentials by definition --------------------------------

base=${path##*/}
case "$base" in
  .env|.env.*|id_rsa|id_ed25519|*.pem|*.p12|*.pfx|*.keystore)
    echo "Blocked: $path is a credential file. Operon is local-first and commits no secrets (CONTRIBUTING.md). Keep it outside the repository." >&2
    exit 2
    ;;
esac

# --- credential-shaped content ------------------------------------------------
#
# This directory is exempt: the patterns below have to be written down
# somewhere, and that somewhere is this file.
case "$path" in
  */.claude/hooks/*) exit 0 ;;
esac

written=$(printf '%s' "$payload" | jq -r '
  [.tool_input.content?, .tool_input.new_string?, .tool_input.new_source?]
  | map(select(. != null)) | join("\n")')
[ -z "$written" ] && exit 0

# Each pattern requires a realistic key length, so prose that merely names a
# prefix does not trip the gate.
if printf '%s' "$written" | perl -ne '
    exit 1 if /sk-ant-[A-Za-z0-9_-]{24,}/
           || /\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}/
           || /\bgithub_pat_[A-Za-z0-9_]{40,}/
           || /\bAKIA[0-9A-Z]{16}\b/
           || /-----BEGIN [A-Z ]*PRIVATE KEY-----/
           || /\bxoxb-[0-9]{10,}-[0-9]{10,}-[A-Za-z0-9]{20,}/;
  '; then
  exit 0
fi

echo "Blocked: the text being written to $path contains something shaped like a live credential (API key, token, or private key). Operon commits no secrets — see CONTRIBUTING.md. Use a placeholder, or read the value from the environment at run time." >&2
exit 2
