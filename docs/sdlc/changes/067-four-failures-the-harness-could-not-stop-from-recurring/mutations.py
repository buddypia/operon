#!/usr/bin/env python3
"""Watch change 067's guards fail. One mutation per requirement in spec.md, plus
one that this change's own rule demands: a mutation of what a sentence CLAIMS,
not of a counter behind it.

Two rules carried from 061's sweep, both learned there. `str.replace(old, new)`
takes every occurrence, and the target is asserted present before the edit, so a
mutation that silently matched nothing cannot read as caught. And a mutated file
is written with `write_text`, never `shutil.copy2`: copy2 preserves mtime, cargo
reuses the previous binary, and the unmutated result gets reported as the
mutated one.
"""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
REVIEW = ROOT / "REVIEW.md"
SKILL = ROOT / ".claude/skills/sdlc/SKILL.md"
EVAL = ROOT / "evals/007-mutate-the-claim-not-the-counter.md"
HOOK = ROOT / ".claude/hooks/gate-stop.sh"
PACKAGER = ROOT / "scripts/package-macos.sh"
CHECKER = ROOT / "scripts/check-installed-build.sh"
TEMPLATE = ROOT / "docs/sdlc/templates/state.yaml"
LESSONS = ROOT / "docs/sdlc/lessons.md"

TESTS = [
    "the_review_policy_names_every_policy_the_repo_enforces",
    "the_pipeline_documents_ask_their_questions_before_review_does",
    "every_eval_states_a_prompt_and_a_check",
    "the_stop_gate_says_when_it_is_reading_another_checkout",
    "the_stop_gate_reports_an_installed_build_that_is_behind",
    "the_stop_gate_reads_the_tree_the_session_is_working_in",
    "the_installed_build_check_reads_its_stamp_key_out_of_the_packager",
    "every_gate_script_is_wired_or_declares_why_not",
    "harness_documents_only_name_paths_that_exist",
]

MUTATIONS = {
    # Requirement 1 — REVIEW.md names the claim-mutation rule.
    "review-loses-the-claim-rule": (
        REVIEW, "must mutate the claim", "must mutate the counters"),
    # Requirement 2 — the skill carries it where a session meets it first.
    "skill-loses-the-claim-rule": (
        SKILL, "mutate the claim", "re-run the counters"),
    # Requirement 3 — the eval is well formed and exercises the rule.
    "eval-loses-its-check": (EVAL, "\n## Check\n", "\n## Notes\n"),
    # Requirement 4 — the hook says when it is reading another checkout. Fires
    # on every stop instead, which is the failure mode the fix has to avoid:
    # a diagnostic that always fires is noise, not a fix.
    "mismatch-line-always-fires": (
        HOOK, '[ "$hook_repository" != "$judged_tree" ]',
        '[ "$hook_repository" = "$judged_tree" ] || true'),
    # Requirement 4, the other half — the line is dropped altogether.
    "mismatch-line-dropped": (HOOK, 'hook_directory:-.', 'judged_tree:-.'),
    # Requirement 5 — `$0` is resolved before the hook cds into the tree it is
    # judging. Left relative, it resolves against the judged tree instead, and
    # the mismatch disappears exactly when the hook is invoked the way a session
    # invokes it. Only the relative case can see this one.
    "hook-location-resolved-late": (
        HOOK, 'hook_directory=$(cd -- "$(dirname -- "$0")" 2>/dev/null && pwd)',
        'hook_directory=$(dirname -- "$0")'),
    # THE CLAIM MUTATION. Not a counter and not a condition: the two roots are
    # printed the other way round, so every fact in the line is true and the
    # line tells the reader to go and fix the checkout it is reporting on.
    "roots-swapped": (
        HOOK, '      "$hook_repository" "$judged_tree"',
        '      "$judged_tree" "$hook_repository"'),
    # Requirement 6 — the three reviewer states, and the silent one by name.
    "reviewer-states-collapsed": (
        REVIEW, "finished and silent", "finished and quiet"),
    # Requirement 7 — the stamp key is spelled once, in the packager.
    "stamp-key-spelled-twice": (
        CHECKER, 'stamp_key="$(sed -n \'s/^source_commit_key="\\([^"]*\\)"$/\\1/p\' "$packager" | head -1)"',
        'stamp_key="OperonSourceCommit"'),
    # Requirement 7, the other side — the packager stops stamping at all.
    "packager-stops-stamping": (
        PACKAGER, 'source_commit_key="OperonSourceCommit"',
        'source_commit_key_unused="OperonSourceCommit"'),
    # Requirement 8 — an unstamped bundle is unknown, not stale. Every bundle
    # installed before this change is unstamped; calling those stale makes the
    # check noise from the day it lands.
    "unstamped-called-stale": (
        CHECKER,
        'echo "installed build: UNKNOWN — $bundle carries no $stamp_key (built before the"',
        'echo "installed build: STALE — $bundle carries no $stamp_key (built before the"'),
    # Requirement 9 — the Stop gate RUNS the check. The `[ -f ]` test above the
    # call still names the script, so the wiring guard stays green: this is the
    # mutation that says whether anything asks behaviourally.
    "hook-stops-asking": (
        HOOK, '"${OPERON_INSTALLED_CHECK_SECONDS:-5}" bash scripts/check-installed-build.sh 2>/dev/null)',
        '"${OPERON_INSTALLED_CHECK_SECONDS:-5}" true 2>/dev/null)'),
    # Round 1's Important, from subprocess-067. The check is asked before the
    # position is stamped, so a bundle that does not answer means the stamp is
    # never reached and the next stop re-enters the identical hang — the one
    # promise this hook's own header makes, broken on the new path.
    "check-runs-before-the-stamp": (
        HOOK,
        'printf \'%s\\n\' "$state" >> "$stamp"\n',
        ''),
    # The other half of the same finding: the bound itself.
    "bound-removed": (
        HOOK,
        '''  installed_report=$(perl -e 'alarm shift; exec @ARGV' \\
    "${OPERON_INSTALLED_CHECK_SECONDS:-5}" bash scripts/check-installed-build.sh 2>/dev/null)''',
        '  installed_report=$(bash scripts/check-installed-build.sh 2>/dev/null)'),
    # Requirement 10 — the contract row that asks on the first day. Deleted
    # rather than commented out: a commented row still contains the substring a
    # guard searches for, which is a mutation that cannot fail.
    "contract-row-dropped": (
        TEMPLATE,
        '  release-binary: "machine pending does this change touch src/ except src/tests.rs,\n'
        '    Cargo.toml, or Cargo.lock? If so it makes /Applications/Operon.app stale and\n'
        '    ends with bash scripts/package-macos.sh. Answer when writing this contract"\n',
        ''),
    # Requirement 11 — the lesson's Guard paragraph. Caught by the indicator run
    # rather than by a test, which is what `--lessons` is for: it collects from
    # the `**Guard.**` marker to the end of the entry, so an entry that stops
    # marking it names nothing the repository holds.
    "lesson-guard-unmarked": (
        LESSONS, "**Guard.** One per failure, each watched failing",
        "The guards, one per failure, each watched failing"),
}


def run_checks() -> tuple[bool, str]:
    """Every guard this change added or leans on, plus the lessons indicator.

    The indicator is here because one requirement's guard is not a test: an
    entry in `docs/sdlc/lessons.md` whose Guard names nothing the repository
    holds is caught by `scripts/pipeline-indicators.sh --lessons` and by nothing
    in the suite.
    """
    result = subprocess.run(
        ["cargo", "test", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"),
         "--", *TESTS],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return False, result.stdout + result.stderr
    indicators = subprocess.run(
        ["bash", str(ROOT / "scripts/pipeline-indicators.sh"), "--lessons"],
        capture_output=True,
        text=True,
        cwd=str(ROOT),
    )
    return indicators.returncode == 0, indicators.stdout + indicators.stderr


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
            reason = next(
                (line.strip() for line in output.splitlines()
                 if "panicked at" in line or "guard" in line.lower()),
                "(理由行が読めません)",
            )
            print(f"{name}: caught — {reason}")
finally:
    for path, text in originals.items():
        path.write_text(text)

print()
print(f"{len(MUTATIONS) - len(failures)} of {len(MUTATIONS)} caught")
sys.exit(1 if failures else 0)
