#!/usr/bin/env bash
# Which fields Codex puts on every rollout record it writes on THIS machine, and
# whether the replay records Operon appends carry them.
#
# not-wired: no hook or workflow calls this, and none should. It reads the Codex
# session files on THIS machine; CI has none, so an automated caller could only
# ever pass. Lesson 007: a check that can only pass is not a check. Run it by
# hand after updating Codex.
#
# Why it exists: Codex 0.155.0 started numbering rollout records with an
# `ordinal`, its thread store reads the final record to continue the sequence,
# and Operon kept appending unnumbered replay records — so every restored thread
# refused to resume, with the only report a terminal pane that died on first
# use. The suite stayed green for at least eight days, because the only test
# over that writer builds its rollout by hand and the end-to-end test that uses
# a real Codex is `#[ignore]`d. A field Codex adds to its envelope is exactly
# the kind of drift no fixture notices, and this is the cheapest place to see
# it: the files are already on disk.
#
# The comparison is deliberately asymmetric. Within one session file, a field
# has to be on EVERY record to count as the envelope; one that appears on some
# records is payload-specific and not something an appended replay record must
# carry. Across files the sets are UNIONED, not intersected, and that is the
# load-bearing choice: of the Codex-written files on the machine this was
# written on, 390 carry `{ordinal,timestamp,type}` and 249 the older
# `{timestamp,type}`, so intersecting would have hidden `ordinal` — the exact
# drift this script exists for. What it reports is therefore "every field any
# Codex version on this disk puts on all of its records", not "what the
# installed Codex writes today".
#
# The cost of the union is the second report below: a field Codex has STOPPED
# writing stays in the union forever, so that half can name a field the writer
# is carrying needlessly but can never prove one is obsolete. Say so rather
# than let a reader infer symmetry.
set -uo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/..}" || exit 1

# The envelope Operon writes, read out of the writer rather than restated here:
# a check that retypes the value it guards goes blind on the rename it exists to
# survive (.claude/rules/identifiers.md, lesson 004).
written=$(
  awk '
    /^pub\(crate\) fn codex_replay_event\(/ { inside = 1 }
    inside && /^}/ { inside = 0 }
    inside && match($0, /record\.insert\("[^"]+"/) {
      field = substr($0, RSTART + 15, RLENGTH - 15)
      gsub(/"/, "", field)
      print field
    }
  ' src/history.rs | LC_ALL=C sort -u
)

if [ -z "$written" ]; then
  echo "check-codex-rollout-shape: no envelope fields parsed out of codex_replay_event" >&2
  exit 1
fi

observed=$(
  python3 - <<'PY'
import glob, json, os

# Rollouts Operon created carry its own originator, and including them would
# let this check read Operon's own output back as Codex's convention.
OPERON_ORIGINATOR = "xirp-copy"

always = None
files = 0
for path in sorted(glob.glob(os.path.expanduser("~/.codex/sessions/**/*.jsonl"), recursive=True)):
    per_file = None
    originator = None
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                try:
                    record = json.loads(line)
                except Exception:
                    continue
                if not isinstance(record, dict):
                    continue
                if record.get("type") == "session_meta" and isinstance(record.get("payload"), dict):
                    originator = record["payload"].get("originator")
                keys = {key for key in record if key != "payload"}
                per_file = keys if per_file is None else (per_file & keys)
    except OSError:
        continue
    if per_file is None or originator == OPERON_ORIGINATOR:
        continue
    files += 1
    always = per_file if always is None else (always | per_file)

print(files)
for field in sorted(always or ()):
    print(field)
PY
)

files=$(printf '%s\n' "$observed" | head -1)
fields=$(printf '%s\n' "$observed" | tail -n +2 | LC_ALL=C sort -u)

if [ "${files:-0}" = "0" ]; then
  echo "codex rollout shape: no Codex-written session files found; nothing to compare"
  exit 0
fi

# `payload` is excluded on both sides: it is the record, not the envelope.
missing=$(comm -23 <(printf '%s\n' "$fields") <(printf '%s\n' "$written" | grep -v '^payload$'))
unknown=$(comm -13 <(printf '%s\n' "$fields") <(printf '%s\n' "$written" | grep -v '^payload$'))

status=0
if [ -n "$missing" ]; then
  echo "codex rollout shape: $(printf '%s\n' "$missing" | wc -l | tr -d ' ') field(s) some Codex version on this disk writes on all of its records, and codex_replay_event does not:"
  printf '%s\n' "$missing" | sed 's/^/  /'
  echo
  echo "Add each one in src/history.rs, and say in the change what Codex does with"
  echo "it — a restored thread whose records are missing part of the envelope may"
  echo "refuse to resume at all."
  status=1
fi
if [ -n "$unknown" ]; then
  echo "codex rollout shape: $(printf '%s\n' "$unknown" | wc -l | tr -d ' ') field(s) codex_replay_event writes that no Codex version on this disk always carried:"
  printf '%s\n' "$unknown" | sed 's/^/  /'
  echo
  echo "Not a failure on its own: the field may be written only on some record"
  echo "types, or Operon may be inventing it. This half cannot tell you a field"
  echo "has been dropped — a field any older Codex wrote stays in the union."
fi
if [ "$status" = 0 ] && [ -z "$unknown" ]; then
  echo "codex rollout shape: $files Codex-written session files, envelope fields $(printf '%s' "$fields" | tr '\n' ' ')— all written by codex_replay_event"
fi
exit "$status"
