#!/usr/bin/env python3
"""Change 068's sweep. Copied from docs/sdlc/templates/mutations.py, which this
change added — so this is also the first use of it, and the template's shape is
what it left behind.

One mutation per requirement, plus the two claim mutations REVIEW.md asks for.
Both claim mutations leave every count and every exit code correct and make a
printed sentence untrue, which is the axis a counters-only sweep cannot see:
`damaged-plist-called-old` gives a damaged bundle the wording reserved for an
old one, and `timeout-reported-as-an-ordinary-catch` reports a sweep that gave
up waiting as though the mutation had been caught by an assertion.

Two of the mutations below are deliberately deletions rather than comment-outs.
A commented-out line still contains the substring a text guard searches for, so
commenting is a mutation that cannot fail — met in change 067's sweep and
recorded there.
"""
import os
import signal
import subprocess
import sys
from pathlib import Path

# 1. WHERE. An absolute path: this runs from anywhere, and a worktree session's
#    `GIT_DIR`/`GIT_WORK_TREE` do not point here.
ROOT = Path(__file__).resolve().parents[4]

# 2. HOW LONG one check may take before the sweep stops waiting for it. Two
#    orders of magnitude above a healthy run, because this bound is here to end
#    a hang and not to measure anything.
RUN_SECONDS = 1800

# 3. WHAT SAYS THE GUARDS ARE GREEN. One command per list; the first non-zero
#    exit ends the check. Name the guards rather than running the whole suite —
#    a sweep runs this once per mutation.
TESTS = [
    "an_unknown_flag_or_role_is_counted_against_what_the_record_did",
    "the_installed_build_check_reads_a_path_that_begins_with_a_hyphen",
    "the_installed_build_check_reads_its_stamp_key_out_of_the_packager",
    "the_mutation_sweep_template_ends_a_run_that_will_not_finish",
    "the_pipeline_documents_ask_their_questions_before_review_does",
    "harness_documents_only_name_paths_that_exist",
]
CHECKS = [["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "--", *TESTS]]

HISTORY = ROOT / "src/history.rs"
CHECKER = ROOT / "scripts/check-installed-build.sh"
TEMPLATE = ROOT / "docs/sdlc/templates/mutations.py"
SKILL = ROOT / ".claude/skills/sdlc/SKILL.md"

# 4. WHAT TO BREAK. One entry per requirement in the change, plus the claim
#    mutation rule 4 asks for. The name is printed, so make it say what was
#    broken rather than which file it was in.
MUTATIONS = {
    # R1 — settle_deferred_kinds maps a record's outcome onto the arm its
    # deferred unknown kinds are counted against. Moving `Operational` to
    # `Carried` says a kind's content reached the restored conversation when the
    # record carrying it was excluded and nothing crossed, which is the sentence
    # change 061 exists to stop. It passed the whole suite and all fifteen of
    # 061's own mutations before this change added the fifth case.
    "operational-arm-carried": (
        HISTORY,
        """            RecordOutcome::Turn(..) => UnclassifiedOutcome::Carried,
            RecordOutcome::Operational | RecordOutcome::Empty | RecordOutcome::Unclassified => {
                UnclassifiedOutcome::Dropped
            }""",
        """            RecordOutcome::Turn(..) | RecordOutcome::Operational => {
                UnclassifiedOutcome::Carried
            }
            RecordOutcome::Empty | RecordOutcome::Unclassified => UnclassifiedOutcome::Dropped,""",
    ),
    # R2, `$0`. Without `--`, `dirname` refuses a hyphen-leading path, prints
    # nothing, and `cd "/.."` succeeds — repo_root becomes the filesystem root
    # and every later sentence is about a repository the script never read. A
    # silent wrong answer, which is why the guard runs the script.
    "hook-location-not-dashed": (
        CHECKER,
        'repo_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"',
        'repo_root="$(cd "$(dirname "$0")/.." && pwd)"',
    ),
    # R2, `$1`. Deleted rather than commented out.
    "bundle-argument-not-a-path": (
        CHECKER,
        'case "$bundle" in -*) bundle="./$bundle" ;; esac\n',
        "",
    ),
    # R3 — the probe that separates a plist which does not parse from one that
    # parses and carries no stamp. Deleted, so both fall into the second
    # sentence, which is the state this change found.
    "damaged-plist-probe-dropped": (
        CHECKER,
        """if ! plutil -convert xml1 -o /dev/null "$plist" 2>/dev/null; then
  echo "installed build: UNKNOWN — $plist is missing or does not parse as a property"
  echo "  list, so no stamp can be read from it. This bundle is damaged rather than"
  echo "  old. Rebuild it with: bash scripts/package-macos.sh"
  exit 0
fi
""",
        "",
    ),
    # R3, THE CLAIM. The branch stays, the exit code stays, the counters stay —
    # and the sentence tells a person with a damaged bundle that it is merely
    # older than the stamp, which sends them to look at the calendar instead of
    # at the bundle. Nothing a counter can see.
    "damaged-plist-called-old": (
        CHECKER,
        """  echo "installed build: UNKNOWN — $plist is missing or does not parse as a property"
  echo "  list, so no stamp can be read from it. This bundle is damaged rather than"
  echo "  old. Rebuild it with: bash scripts/package-macos.sh\"""",
        """  echo "installed build: UNKNOWN — $bundle carries no $stamp_key (built before the"
  echo "  stamp existed). Newest release-binary commit: $head_commit\"""",
    ),
    # R4 — the bound itself. Measured: without it the guard waits the full 30
    # seconds of the fixture's hanging check and then reports it as a mutation
    # that was NOT caught, which is what a real sweep would do to a real change.
    "sweep-bound-removed": (
        TEMPLATE,
        "            out, err = child.communicate(timeout=RUN_SECONDS)",
        "            out, err = child.communicate()",
    ),
    # R4, the half round 1 found. Bounding the WAIT is not bounding the WORK:
    # `subprocess.run(timeout=…)` — and `child.kill()`, which is the same reach —
    # ends the process Python forked and nothing under it. A sweep's check is
    # `cargo test`, which runs the test binary as a grandchild, so the sweep
    # reports on time and the hanging test stays on the machine. This mutation
    # keeps the bound and shrinks its reach, and every count it produces is
    # still correct.
    "bound-reaches-only-the-direct-child": (
        TEMPLATE,
        """            try:
                # The group id is the child's pid, by construction above.
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass""",
        "            child.kill()",
    ),
    # `start_new_session=True` one line up has NO mutation of its own, and the
    # reason is worth more than the mutation would have been: with the flag
    # removed and the killpg kept, `child.pid` is no longer a group id, and
    # killpg would name whatever group on this machine happens to hold that
    # number — a SIGKILL at a stranger, run once per sweep. The flag is covered
    # from the other side instead: `bound-reaches-only-the-direct-child` above
    # shows that the group kill is what stops the grandchild, and a group kill
    # needs the group.
    # R4, THE CLAIM. The bound still fires, the run is still counted as caught,
    # the sweep still exits 0 — and the line a person reads no longer says the
    # catch came from giving up rather than from an assertion. A reader then
    # takes a hang as evidence the guard held.
    "timeout-reported-as-an-ordinary-catch": (
        TEMPLATE,
        """                f"{TIMED_OUT} — {RUN_SECONDS} 秒で終わりませんでした: {' '.join(command)}\\n"
                "変異がチェックをハングさせています。catch ではありますが、"
                "主張が守られた証拠ではありません。\"""",
        """                f"チェックが失敗しました: {' '.join(command)}\"""",
    ),
    # R4, where a session meets it. The template can be perfect and unused: the
    # sweeps that inherited the unbounded run were each copied from whichever
    # one was nearest, and nothing pointed anywhere else.
    "skill-stops-naming-the-template": (
        SKILL,
        "**Copy the sweep from `docs/sdlc/templates/mutations.py`.**",
        "**Copy the sweep from the change that ran one most recently.**",
    ),
}

# 5. Nothing below this line is filled in.

# The marker a timed-out run prints. A timeout IS a catch — a hanging suite is
# not a green one — but it is a catch for a different reason than an assertion,
# and a reader has to be able to tell them apart to know whether the mutation
# proved anything.
TIMED_OUT = "時間切れ"


def run_checks() -> tuple[bool, str]:
    """Every check, in order. `(green, output)`.

    `start_new_session=True` puts the check in a process group of its own, and
    the bound kills the GROUP. `subprocess.run(timeout=…)` kills only the
    process Python forked, and the check a sweep actually runs is `cargo test` —
    which runs the test binary as a grandchild. Bounding the wait without
    killing the group is true and useless: the sweep finishes and reports, and
    the hanging test binary stays on the machine until somebody notices. Found
    in review, reproduced twice, and the reason this is Popen rather than the
    one-liner it looks like it should be.

    A grandchild that calls `setsid` for itself leaves the group and outlives
    this, which nothing here can prevent. No check named above does.
    """
    for command in CHECKS:
        child = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        try:
            out, err = child.communicate(timeout=RUN_SECONDS)
        except subprocess.TimeoutExpired:
            try:
                # The group id is the child's pid, by construction above.
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            # Reap it and drain the pipes, which a killed group releases.
            child.communicate()
            return False, (
                f"{TIMED_OUT} — {RUN_SECONDS} 秒で終わりませんでした: {' '.join(command)}\n"
                "変異がチェックをハングさせています。catch ではありますが、"
                "主張が守られた証拠ではありません。"
            )
        if child.returncode != 0:
            return False, out + err
    return True, ""


def reason_from(output: str) -> str:
    """The one line worth printing beside a catch."""
    for line in output.splitlines():
        stripped = line.strip()
        if TIMED_OUT in stripped or "panicked at" in stripped or "guard" in stripped.lower():
            return stripped
    return "(理由行が読めません)"


originals = {path: path.read_text() for path, _, _ in MUTATIONS.values()}
failures = []
try:
    for name, (path, old, new) in MUTATIONS.items():
        text = originals[path]
        if old not in text:
            print(f"{name}: SETUP FAILED — 置換対象が見つかりません:\n{old}")
            sys.exit(1)
        path.write_text(text.replace(old, new))
        green, output = run_checks()
        path.write_text(originals[path])
        if green:
            failures.append(name)
            print(f"{name}: NOT CAUGHT — 変異したのにチェックが通りました")
        else:
            print(f"{name}: caught — {reason_from(output)}")
finally:
    for path, text in originals.items():
        path.write_text(text)

print()
print(f"{len(MUTATIONS) - len(failures)} of {len(MUTATIONS)} caught")
sys.exit(1 if failures else 0)
