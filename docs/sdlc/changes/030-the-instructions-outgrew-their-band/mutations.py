#!/usr/bin/env python3
"""Watch this change's band guard fail, once per thing it claims to catch.

    python3 docs/sdlc/changes/030-the-instructions-outgrew-their-band/mutations.py

Each mutation edits `scripts/check-bands.sh`, runs the one test, restores the
file, and reports. A substitution that matches nothing is reported as this
script's own failure rather than as a green guard — a mutation that did not
apply proves nothing, and that is how a mutation run lies.

It lives in the repository, and that is the point of the file. It was written
outside it, under a developer's home directory, and `docs/sdlc/lessons.md`
entry 025 cited a count from it that nobody could re-derive: the entry said
five, listed seven, and the set had grown to ten. A number offered as evidence
has to be checkable by the next reader, which is the same finding the entry is
about — so the evidence moved in beside the claim.
"""
import pathlib
import subprocess
import sys

TREE = pathlib.Path(__file__).resolve().parents[4]
SCRIPT = TREE / "scripts" / "check-bands.sh"
TEST = "the_control_bands_say_whether_a_propose_breach_has_an_answer"

MUTATIONS = [
    (
        "the reporting is never called",
        '  if [ "$tier" = propose ]; then',
        "  if false; then",
    ),
    (
        "a mention counts as a declaration",
        "      /^\\*\\*Answers band\\*\\*:/ {",
        "      // {",
    ),
    (
        "the explanatory lines are counted as breached metrics",
        'echo "bands: $breached of $checked metrics breached"',
        'echo "bands: ${#breaches[@]} of $checked metrics breached"',
    ),
    (
        "the answer is named without its state",
        'echo "  prescribed response: ${dir##*/} (status $status)"',
        'echo "  prescribed response: ${dir##*/}"',
    ),
    (
        "a metric with no answer is reported as nothing at all",
        '  if [ "$found" -eq 0 ]; then',
        "  if false; then",
    ),
    # Round 1 of review found these two. Both were live defects in the
    # committed change, not hypotheticals.
    (
        "a shipped answer counts as one in progress",
        "      done|archived)                closed=$((closed + 1)) ;;",
        "      archived)                     closed=$((closed + 1)) ;;",
    ),
    # The old regex required a separator between the colon and the metric. Its
    # faithful mutation is the awk condition demanding the space, not the
    # tokeniser: `rest` is already taken from past the colon, so changing how it
    # is split does not reproduce the defect — a mutation that does not
    # reproduce the bug proves nothing, which is why this one is spelled out.
    (
        "a declaration with no separator is invisible to the script",
        "      /^\\*\\*Answers band\\*\\*:/ {",
        "      /^\\*\\*Answers band\\*\\*: / {",
    ),
    # Round 2. The reviewer ran these two itself and watched them survive: the
    # guard's `live` case forbade one branch's wording where a broken live
    # branch prints another's.
    (
        "a live answer is reported as having nowhere to go",
        '  elif [ "$live" -gt 0 ]; then',
        "  elif false; then",
    ),
    (
        "an answer in progress is filed as parked",
        "      *)                            live=$((live + 1)) ;;",
        "      *)                            parked=$((parked + 1)) ;;",
    ),
    # And the tokeniser in the other direction: splitting on anything that is
    # not a word character breaks the hyphen, so a change declaring a different
    # band answers this one. Rust's `split_whitespace` never did.
    (
        "a hyphenated near miss counts as a declaration",
        "        count = split(rest, parts, /[[:space:]]+/)",
        "        count = split(rest, parts, /[^A-Za-z0-9_]+/)",
    ),
    # Round 3. The arms were pinned by one representative status word each, so
    # dropping any of the other six from its arm sent it through `*)` to
    # `live` — the one outcome that prints no warning.
    (
        "a failed answer is reported as one in progress",
        "      awaiting-user|blocked|failed) parked=$((parked + 1)) ;;",
        "      awaiting-user|blocked)        parked=$((parked + 1)) ;;",
    ),
    (
        "an archived answer is reported as one in progress",
        "      done|archived)                closed=$((closed + 1)) ;;",
        "      done)                         closed=$((closed + 1)) ;;",
    ),
]


def run_test():
    done = subprocess.run(
        ["cargo", "test", "--locked", TEST],
        cwd=TREE,
        capture_output=True,
        text=True,
    )
    return done.returncode == 0


def main():
    original = SCRIPT.read_text(encoding="utf-8")

    if not run_test():
        print("refusing: the test is already red before any mutation", file=sys.stderr)
        return 1

    failures = 0
    for label, find, replace in MUTATIONS:
        if find not in original:
            print(f"[SCRIPT BUG] {label}: the text to mutate is not in the file")
            failures += 1
            continue
        SCRIPT.write_text(original.replace(find, replace, 1), encoding="utf-8")
        green = run_test()
        SCRIPT.write_text(original, encoding="utf-8")
        if green:
            print(f"[NOT CAUGHT] {label}")
            failures += 1
        else:
            print(f"[caught    ] {label}")

    if not run_test():
        print("the file was not restored correctly", file=sys.stderr)
        return 1
    print(f"\n{len(MUTATIONS) - failures} of {len(MUTATIONS)} mutations caught")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
