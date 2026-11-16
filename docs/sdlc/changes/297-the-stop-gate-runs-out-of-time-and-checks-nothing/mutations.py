#!/usr/bin/env python3
"""The mutation sweep for change 297: the Stop gate reads the changes in play
rather than filtering every state file, and the approval ledger reads every
ledger in one pass. Each mutation puts a per-change cost back, or reads the
wrong set, and a test must refuse to pass. Runner: scripts/mutation_sweep.py.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]

RUN_SECONDS = 1800

TESTS = [
    "the_stop_gate_does_not_pay_for_changes_it_is_not_working_on",
    "the_stop_gate_holds_a_session_only_to_the_changes_it_is_working_on",
    "the_approval_ledger_reads_every_ledger_in_one_pass",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

STOP = ROOT / ".claude/hooks/gate-stop.sh"
APPROVALS = ROOT / "scripts/check-approvals.sh"

MUTATIONS = {
    # The claim of the change: the cost does not grow with parked changes.
    "the gate filters a walk of every state file again": (
        STOP,
        "      printf '%s\\n' \"$in_play\" | while IFS= read -r change; do\n"
        "        [ -n \"$change\" ] && printf 'docs/sdlc/changes/%s/state.yaml\\n' \"$change\"\n"
        "      done\n",
        "      for state_file in docs/sdlc/changes/*/state.yaml; do\n"
        "        printf '%s\\n' \"$in_play\" | grep -xF \"$(basename \"$(dirname \"$state_file\")\")\" >/dev/null"
        " && printf '%s\\n' \"$state_file\"\n"
        "      done\n",
    ),
    "the gate reads no state file in play": (
        STOP,
        "[ -n \"$change\" ] && printf 'docs/sdlc/changes/%s/state.yaml\\n' \"$change\"",
        ":",
    ),
    "the gate reads every state file, parked ones too": (
        STOP,
        "    if [ \"$every_change\" = 0 ]; then\n      printf '%s\\n' \"$in_play\" | while",
        "    if false; then\n      printf '%s\\n' \"$in_play\" | while",
    ),
    "with no main the gate reads nothing": (
        STOP,
        "      printf '%s\\n' docs/sdlc/changes/*/state.yaml\n",
        "      :\n",
    ),
    "the ledger is read one awk per file again": (
        APPROVALS,
        "  set --\n"
        "  for log in \"$changes_root\"/*/approvals.log; do\n"
        "    [ -f \"$log\" ] && set -- ${1+\"$@\"} \"$log\"\n"
        "  done\n"
        "  [ \"$#\" -gt 0 ] || return 0\n"
        "  awk 'FNR == 1 { change = FILENAME; sub(/\\/approvals\\.log$/, \"\", change); sub(/.*\\//, \"\", change) }\n"
        "    !/^[[:space:]]*(#|$)/ { print change \"\\t\" FNR \"\\t\" $0 }' \"$@\"\n",
        "  for log in \"$changes_root\"/*/approvals.log; do\n"
        "    [ -f \"$log\" ] || continue\n"
        "    awk -v change=\"$(basename \"$(dirname \"$log\")\")\" \\\n"
        "      '!/^[[:space:]]*(#|$)/ { print change \"\\t\" NR \"\\t\" $0 }' \"$log\"\n"
        "  done\n",
    ),
    "only the first ledger is read": (
        APPROVALS,
        "{ print change \"\\t\" FNR \"\\t\" $0 }' \"$@\"",
        "{ print change \"\\t\" FNR \"\\t\" $0 }' \"$1\"",
    ),
}

sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / "scripts"))
import mutation_sweep  # noqa: E402

mutation_sweep.main(RUN_SECONDS, CHECKS, MUTATIONS)
