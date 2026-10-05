#!/usr/bin/env python3
"""Watch change 061's guards fail. Each mutation removes one claim of the fix.

Two things learned the hard way and kept here:

- `str.replace(old, new)` with no count, and the occurrence count asserted.
  Lesson 039 was a mutation that read as caught while the thing it removed was
  still present at a second call site, so the check passed on a copy of what it
  was looking for.
- Every mutated file is rewritten with `write_text`, which moves its mtime
  forward. `shutil.copy2` does not, and cargo then reuses the previous binary
  and reports the unmutated result as if it were the mutated one.
"""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HISTORY = ROOT / "src/history.rs"
VOCABULARY = ROOT / "src/transcript.rs"

TOKEN_COUNTS = "a_codex_token_count_record_is_excluded_rather_than_reported_as_unknown"
TWO_ARMS = "a_kind_that_crossed_and_a_kind_that_was_dropped_are_reported_as_different_things"
ARITHMETIC = "the_notices_printed_counts_account_for_every_record_read"
CARRIED_KINDS = "an_unclassified_record_is_restored_and_reported"
ROLE = "an_unknown_role_is_reported_as_carried_because_its_turn_crosses"
PROMISES = "the_notice_promises_no_marker_the_restored_text_does_not_carry"
DEFERRED = "an_unknown_flag_or_role_is_counted_against_what_the_record_did"

VOCABULARY_ROW = '    kind("codex", Record, "token_usage_record", Operational, "token counts for a turn and for the thread, keyed by thread/turn/response id. The same thing claude/cost-state is, and seeded from a real rollout: 32 of this machine\'s own carry it, 2385 records in one thread"),\n'

ARITHMETIC_SENTENCE = '"（元の記録 {records} 件から会話 {restored} 件・思考 {reasoning} 件を取り、運用レコード {operational} 件・未分類 {unclassified} 件・本文なし {empty} 件は除外）"'

# name -> (file, [(old, new, expected occurrences)], test that must go red)
MUTATIONS = {
    # Defect 1 put back: the row is gone and the kind is unknown again.
    "no-vocabulary-row": (
        VOCABULARY,
        [(VOCABULARY_ROW, "", 1)],
        TOKEN_COUNTS,
    ),
    # Defect 2 put back, in the exact shape it had: count first, decide after,
    # and call everything carried.
    "everything-is-carried": (
        HISTORY,
        [
            (
                "        None => {\n"
                "            summary.count_unclassified_kind(provider, kind, UnclassifiedOutcome::Dropped);\n"
                "            RecordOutcome::Unclassified\n"
                "        }",
                "        None => {\n"
                "            summary.count_unclassified_kind(provider, kind, UnclassifiedOutcome::Carried);\n"
                "            RecordOutcome::Unclassified\n"
                "        }",
                1,
            )
        ],
        TWO_ARMS,
    ),
    # The other direction: a record that crossed is reported as not having.
    "everything-is-dropped": (
        HISTORY,
        [
            (
                "            summary.count_unclassified_kind(provider, kind, UnclassifiedOutcome::Carried);\n"
                "            unclassified_turn(provider, kind, &text)",
                "            summary.count_unclassified_kind(provider, kind, UnclassifiedOutcome::Dropped);\n"
                "            unclassified_turn(provider, kind, &text)",
                1,
            )
        ],
        TWO_ARMS,
    ),
    # The split is kept in the struct and thrown away in the sentence: both arms
    # are named under the carried wording, which is the defect a reader sees.
    "one-sentence-for-both-arms": (
        HISTORY,
        [
            (
                "    let carried = classification.unclassified_named(UnclassifiedOutcome::Carried);",
                "    let carried = [\n"
                "        classification.unclassified_named(UnclassifiedOutcome::Carried),\n"
                "        classification.unclassified_named(UnclassifiedOutcome::Dropped),\n"
                "    ]\n"
                "    .iter()\n"
                "    .filter(|part| !part.is_empty())\n"
                "    .cloned()\n"
                "    .collect::<Vec<_>>()\n"
                "    .join(\" · \");",
                1,
            ),
            (
                "    let dropped = classification.unclassified_named(UnclassifiedOutcome::Dropped);",
                '    let dropped = String::new();',
                1,
            ),
        ],
        TWO_ARMS,
    ),
    # The filter stops filtering: every kind appears under both sentences.
    "arm-filter-ignored": (
        HISTORY,
        [
            (
                "                let count = match outcome {\n"
                "                    UnclassifiedOutcome::Carried => kind.carried,\n"
                "                    UnclassifiedOutcome::Dropped => kind.dropped,\n"
                "                };",
                "                let count = kind.met();\n"
                "                let _ = outcome;",
                1,
            )
        ],
        TWO_ARMS,
    ),
    # Defect 3 put back: the sentence names the buckets that flatter and not
    # the two that say what did not arrive.
    "arithmetic-does-not-close": (
        HISTORY,
        [
            (
                ARITHMETIC_SENTENCE,
                '"（元の記録 {records} 件から会話 {restored} 件・思考 {reasoning} 件を取り、運用レコード {operational} 件は除外）"',
                1,
            ),
            ("        unclassified = classification.unclassified,\n", "", 1),
            ("        empty = classification.empty\n", "", 1),
            ("        operational = classification.operational,\n", "        operational = classification.operational\n", 1),
        ],
        ARITHMETIC,
    ),
    # One bucket only. The five that remain still look like an accounting.
    "one-bucket-unprinted": (
        HISTORY,
        [
            (
                ARITHMETIC_SENTENCE,
                '"（元の記録 {records} 件から会話 {restored} 件・思考 {reasoning} 件を取り、運用レコード {operational} 件・未分類 {unclassified} 件は除外）"',
                1,
            ),
            ("        empty = classification.empty\n", "", 1),
            ("        unclassified = classification.unclassified,\n", "        unclassified = classification.unclassified\n", 1),
        ],
        ARITHMETIC,
    ),
    # An unknown content block always crosses — with its text, or as the bare
    # label. Filing it under `Dropped` reports a person's own screen back to
    # them as missing.
    "carried-block-called-dropped": (
        HISTORY,
        [
            (
                "                summary.count_unclassified_kind(provider, block_type, UnclassifiedOutcome::Carried);",
                "                summary.count_unclassified_kind(provider, block_type, UnclassifiedOutcome::Dropped);",
                1,
            )
        ],
        CARRIED_KINDS,
    ),
    # The same for an unknown role, which does not stop the message either. The
    # role site defers now, so the mutation is on the settlement: everything
    # deferred is called dropped, and a turn a person can see on screen is
    # reported as never having arrived.
    "deferred-settled-as-dropped": (
        HISTORY,
        [
            (
                "        let arm = match outcome {\n"
                "            RecordOutcome::Turn(..) => UnclassifiedOutcome::Carried,",
                "        let arm = match outcome {\n"
                "            RecordOutcome::Turn(..) => UnclassifiedOutcome::Dropped,",
                1,
            )
        ],
        ROLE,
    ),
    # Round 1's Important finding, put back: the carried sentence names the
    # `[Unclassified: …]` heading, which two of the four `Carried` sites never
    # attach. A person sent to look for it in the restored conversation finds
    # nothing.
    "notice-promises-a-heading-half-the-sites-do-not-write": (
        HISTORY,
        [
            (
                '            "Operon が分類を知らない記録がありました。内容は復元先へ渡しています: {kinds}。",',
                '            "Operon が分類を知らない記録がありました。内容は [Unclassified] の見出しを付けて復元先へ渡しています: {kinds}。",',
                1,
            )
        ],
        PROMISES,
    ),
    # Round 2's Important, put back at the flag site: counted on sight, before
    # the record it sits on is read and can still come back empty.
    "flag-counted-before-the-record-is-read": (
        HISTORY,
        [
            (
                "            None => summary.defer_unclassified_kind(provider, key),",
                "            None => {\n"
                "                summary.count_unclassified_kind(provider, key, UnclassifiedOutcome::Carried)\n"
                "            }",
                1,
            )
        ],
        DEFERRED,
    ),
    # The same at the role site.
    "role-counted-before-the-record-is-read": (
        HISTORY,
        [
            (
                "                None => summary.defer_unclassified_kind(provider, declared),",
                "                None => summary.count_unclassified_kind(\n"
                "                    provider,\n"
                "                    declared,\n"
                "                    UnclassifiedOutcome::Carried,\n"
                "                ),",
                1,
            )
        ],
        DEFERRED,
    ),
    # The deferral is kept and settled the wrong way: everything deferred is
    # called carried whatever the record did, which is the same lie one layer in.
    "deferred-settled-as-carried": (
        HISTORY,
        [
            (
                "            RecordOutcome::Operational | RecordOutcome::Empty | RecordOutcome::Unclassified => {\n"
                "                UnclassifiedOutcome::Dropped\n"
                "            }",
                "            RecordOutcome::Operational | RecordOutcome::Empty | RecordOutcome::Unclassified => {\n"
                "                UnclassifiedOutcome::Carried\n"
                "            }",
                1,
            )
        ],
        DEFERRED,
    ),
    # Nothing settles, so a deferred kind is never reported at all — the quiet
    # failure, where the notice simply stops naming a kind to classify.
    "deferred-never-settled": (
        HISTORY,
        [
            ("    summary.settle_deferred_kinds(&outcome);\n", "", 1),
        ],
        DEFERRED,
    ),
    # `met` stops counting both arms, so the manifest's `count` — the field that
    # predates the split and is what an existing reader looks at — silently
    # loses every dropped record.
    "met-forgets-the-dropped": (
        HISTORY,
        [
            ("        self.carried + self.dropped", "        self.carried", 1),
        ],
        TWO_ARMS,
    ),
}


def run_test(name: str) -> tuple[bool, str]:
    result = subprocess.run(
        ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), name],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0, result.stdout + result.stderr


originals = {path: path.read_text() for path in {HISTORY, VOCABULARY}}
not_caught = []
try:
    for name, (path, edits, test) in MUTATIONS.items():
        text = originals[path]
        broken = False
        for old, new, expected in edits:
            found = text.count(old)
            if found != expected:
                print(f"{name}: SETUP FAILED — {found} 箇所 (期待 {expected}):\n{old}")
                broken = True
                break
            text = text.replace(old, new)
        if broken:
            sys.exit(1)
        path.write_text(text)
        green, output = run_test(test)
        if green:
            not_caught.append(name)
            print(f"{name}: NOT CAUGHT — 変異したのに {test} が通りました")
        else:
            reason = next(
                (
                    line.strip()
                    for line in output.splitlines()
                    if "panicked at" in line or line.startswith("error[")
                ),
                "(理由行が読めません)",
            )
            print(f"{name}: caught by {test} — {reason}")
        path.write_text(originals[path])
finally:
    for path, text in originals.items():
        path.write_text(text)

print()
print("すべて捕捉されました" if not not_caught else f"捕捉できなかった変異: {not_caught}")
sys.exit(1 if not_caught else 0)
