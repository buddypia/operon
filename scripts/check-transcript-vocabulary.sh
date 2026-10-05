#!/usr/bin/env bash
# Which record, payload, block, and wrapper kinds the three agent CLIs are
# writing on THIS machine, and which of them TRANSCRIPT_VOCABULARY does not
# classify yet.
#
# not-wired: no hook or workflow calls this, and none should. It reads the agent
# CLI stores on THIS machine; CI has none, so an automated caller could only ever
# pass. Lesson 007: a check that can only pass is not a check. Run it by hand
# after updating an agent CLI.
#
# Run it after updating an agent CLI. A kind listed here is a kind a restore
# will carry across under an `[Unclassified: …]` label and report — nothing is
# lost, but nobody has decided yet whether it is conversation, reasoning, or
# the CLI talking to itself.
#
# Deliberately NOT a CI gate and NOT a control band. It reads the developer's
# own conversation files, which a runner does not have: on CI it would find
# nothing, report zero, and stand as a mechanism that can only ever pass. The
# guard that actually fires is the restore itself, on the machine that holds
# the data, at the moment somebody restores a conversation.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/..}" || exit 1

# `provider<TAB>layer<TAB>kind`, the same three-part key `transcript_class`
# looks up. Comparing on provider and kind alone would call a Block kind
# classified and a Record kind of the same name unclassified.
vocabulary=$(
  awk '
    /^ *kind\("/ {
      match($0, /kind\("[^"]+", [A-Za-z]+, "[^"]*"/)
      if (RSTART == 0) next
      row = substr($0, RSTART, RLENGTH)
      split(row, quoted, "\"")
      split(row, commas, ", ")
      print quoted[2] "\t" commas[2] "\t" quoted[4]
    }
  ' src/transcript.rs | LC_ALL=C sort -u
)

if [ -z "$vocabulary" ]; then
  echo "check-transcript-vocabulary: no rows parsed from src/transcript.rs" >&2
  exit 1
fi

observed=$(
  python3 - <<'PY'
import glob, json, os

seen = set()

def add(provider, layer, kind):
    if kind:
        seen.add((provider, layer, str(kind)))

def blocks(provider, content):
    if isinstance(content, list):
        for part in content:
            if isinstance(part, dict):
                add(provider, "Block", part.get("type"))

def records(pattern, recursive=False):
    for path in glob.glob(os.path.expanduser(pattern), recursive=recursive):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                for line in handle:
                    try:
                        record = json.loads(line)
                    except Exception:
                        continue
                    if isinstance(record, dict):
                        yield record
        except OSError:
            continue

for record in records("~/.claude/projects/*/*.jsonl"):
    add("claude", "Record", record.get("type"))
    for key, value in record.items():
        if key.startswith("is") and value is True:
            add("claude", "Flag", key)
    message = record.get("message")
    if isinstance(message, dict):
        blocks("claude", message.get("content"))

for record in records("~/.codex/sessions/**/*.jsonl", recursive=True):
    add("codex", "Record", record.get("type"))
    payload = record.get("payload")
    # Only response_item payloads are reachable: every other record type is
    # classified at the record layer and the reader stops there.
    if isinstance(payload, dict) and record.get("type") == "response_item":
        add("codex", "Payload", payload.get("type"))
        add("codex", "Role", payload.get("role"))
        blocks("codex", payload.get("content"))

pattern = "~/.gemini/antigravity-cli/brain/*/.system_generated/logs/transcript.jsonl"
for record in records(pattern):
    source, kind = record.get("source"), record.get("type")
    if source and kind:
        add("gemini", "Record", f"{source}/{kind}")
    for field in ("thinking", "tool_calls"):
        if record.get(field):
            add("gemini", "Block", field)

for provider, layer, kind in sorted(seen):
    print(f"{provider}\t{layer}\t{kind}")
PY
)

if [ -z "$observed" ]; then
  echo "vocabulary: no local agent-CLI transcripts found; nothing to compare"
  exit 0
fi

# Both sides must collate identically or comm reports differences that are only
# locale. Python sorts by codepoint, so the shell side has to as well.
missing=$(
  comm -23 \
    <(printf '%s\n' "$observed" | LC_ALL=C sort -u) \
    <(printf '%s\n' "$vocabulary" | LC_ALL=C sort -u)
)
observed_count=$(printf '%s\n' "$observed" | wc -l | tr -d ' ')

if [ -z "$missing" ]; then
  echo "vocabulary: $observed_count kinds observed locally, all classified"
  exit 0
fi

echo "vocabulary: $(printf '%s\n' "$missing" | wc -l | tr -d ' ') of $observed_count observed kinds have no row in TRANSCRIPT_VOCABULARY:"
printf '%s\n' "$missing" | sed 's/^/  /'
echo
echo "Add each one to TRANSCRIPT_VOCABULARY in src/transcript.rs with the reason"
echo "it is Conversation, Reasoning, or Operational. Until then a restore carries"
echo "it across labelled [Unclassified: …] and reports it in the notice."
exit 1
