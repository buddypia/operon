# Plan: four failures the harness could not stop from recurring

- **Spec**: `./spec.md`
- **Approved**: 2026-09-22
- **Status**: done

## What this is

Four mechanisms, one per recurring failure in `intent.md`. They share a shape —
a failure that leaves no trace a later session can read — and no code. Each
lands with its own guard, and the order below is chosen so that a piece that
fails takes nothing else down with it.

## Files that change

| File | Change |
|---|---|
| `REVIEW.md` | the claim-mutation rule; the three reviewer states |
| `.claude/skills/sdlc/SKILL.md` | the claim-mutation rule at step 10, where a session meets it while writing `mutations.py` |
| `evals/007-mutate-the-claim-not-the-counter.md` | new; seeded by lesson 041 |
| `.claude/hooks/gate-stop.sh` | says when its own file is in another checkout; reports a stale installed build |
| `scripts/package-macos.sh` | stamps the source commit into the staged bundle |
| `scripts/check-installed-build.sh` | new; compares the stamp against the newest release-binary commit |
| `docs/sdlc/templates/state.yaml` | a `release-binary` contract row |
| `src/tests.rs` | three new guards, one existing table extended |
| `docs/sdlc/lessons.md` | one entry, four guards named |

## Order of work

Smallest blast radius first, and the two paused surfaces last so that everything
else is already green when they move.

1. **The steering, and the row that holds it.** `REVIEW.md` gains the
   claim-mutation paragraph and the three reviewer states;
   `.claude/skills/sdlc/SKILL.md` gains the same rule at step 10. Extend
   `the_review_policy_names_every_policy_the_repo_enforces`'s required table
   with a row for each, **watched failing first** — add the rows before the
   prose and confirm the test names exactly what is missing. This is the one
   step where the existing guard already does the work; the rest need new ones.

2. **The eval.** `evals/007-mutate-the-claim-not-the-counter.md`. Its Check block
   is the interesting part and is verified by hand both ways: against a
   `mutations.py` that only moves counters (must fail) and one that mutates the
   claim (must pass). It cannot be run against a model here — no API key — and
   that is stated in `state.yaml` rather than reported as a pass. Then point
   lesson 041's Guard paragraph at it and confirm
   `scripts/pipeline-indicators.sh` with `--lessons` still reports every entry held.

3. **`scripts/check-installed-build.sh`**, written before anything calls it, so
   it can be exercised alone. Reads the stamp key out of
   `scripts/package-macos.sh` rather than spelling it — the guard for that is
   `the_installed_build_check_reads_its_stamp_key_out_of_the_packager`, and the
   shape is `.claude/rules/identifiers.md` applied to a pair of shell scripts,
   the same way `gate-stop.sh`'s test reads the stamp path out of the hook. At
   this point the key does not exist yet, so the script's unstamped path is the
   one that runs, which is also the path this machine will take.

4. **`scripts/package-macos.sh` — the stamp.** First paused surface. One
   `PlistBuddy` call against the staged `Info.plist` before the signature, with
   the sha from `git rev-parse HEAD`. Written before the signing step, because a
   bundle signed and then edited is a bundle with a broken signature — the
   packager already re-signs after assembling, so the stamp goes in with the
   rest of the assembly. Verify by packaging once and reading the key back out
   of `dist/Operon.app`, then re-running `check-installed-build.sh` and watching
   it move from "unstamped" to a real comparison.

5. **`.claude/hooks/gate-stop.sh`.** Second paused surface, and last because it
   is the file this branch has already had two defects in. Two additions, both
   additive and neither changing what the hook refuses:
   - the mismatched-checkout line, from `dirname "$0"` and `git rev-parse
     --show-toplevel`, compared against the tree already resolved;
   - the stale-build report, by calling step 3's script.
   Guard: `the_stop_gate_says_when_it_is_reading_another_checkout`, two fixture
   repositories, run under `/bin/bash` so the parse is measured behaviourally —
   lesson 040's rule, on the file that taught it.

6. **`docs/sdlc/templates/state.yaml`** gains the `release-binary` row. Last of
   the edits because `every_state_file_names_a_route_and_a_stage_that_exist`
   reads the closed sets out of this file, and a template change is the kind
   that goes wrong quietly. The row's verdict vocabulary is unchanged; only a
   row is added.

7. **`mutations.py`**, one mutation per requirement, each watched failing. At
   least one of them mutates a claim a person reads rather than a counter —
   this change writes that rule, so its own sweep obeys it or the rule is
   advice.

8. **`docs/sdlc/lessons.md`**, one entry, and re-run
   `scripts/pipeline-indicators.sh` with `--lessons`.

## Risks

| Risk | How it shows up | What catches it |
|---|---|---|
| The mismatched-checkout line fires on every ordinary session, making the hook's real output harder to read | Every stop gains two lines of noise | The guard asserts both directions: the line appears when hook and tree differ, and does **not** appear when they agree. A diagnostic that always fires is the failure mode, not the fix |
| `dirname "$0"` is not the hook's real location when the hook is invoked through a symlink or a relative path | The comparison reports a mismatch that is not one, or misses one that is | `git rev-parse --show-toplevel` is run from that directory, so the answer is a repository root either way; a relative `$0` resolves against the process's working directory, which is the main checkout — exactly the case being detected. The guard runs the hook by absolute path and by relative path |
| `PlistBuddy` fails on a bundle it cannot parse, and `set -e` takes the whole packaging run down | `package-macos.sh` stops after building | The stamp is written to the **staged** bundle inside the script's own temporary directory, before the swap. A failure there leaves the installed app untouched, which is what `scripts/replace-macos-bundle.sh` is built around |
| `check-installed-build.sh` calls every pre-stamp bundle stale, so the check is noise from the day it lands | The Stop gate reports a stale build on every machine, forever | Requirement 8: unstamped is reported as unknown, not stale, and this machine's own bundle is the fixture that proves it |
| The stamped commit is not in this repository's history — installed from another branch, or a commit since rewritten | `git merge-base --is-ancestor` fails and the script reports a false state | Spec's not-the-happy-path list: exit 1 and say the stamp is unknown here, rather than comparing against something that is not there |
| A new `evals/` file is a brand-new file, so the path rule that governs `evals/` does not fire for it | The eval is written without the policy that governs evals | `.claude/rules/` path rules fire on read, not on write — `CLAUDE.md` says so. `evals/README.md` is read first, deliberately, and the suite's `every_eval_*` checks are what enforce the format |
| Two paused surfaces in one change is exactly the shape that produced change 060's twenty rounds | Review churns | The four pieces share no code, so a finding in one is fixable without moving the others' digests — and the round ceiling for `modify` is 3, printed on every `check-review.sh` run |

## What proves it done

The Acceptance list in `spec.md`, plus: every new guard watched failing through
`mutations.py`, and `steering_bytes` recorded with what moved.

## Departures from the plan

**Three requirements had no guard, and the mutation sweep is what said so.**
The plan assumed requirement 2 was covered by the row added to
`the_review_policy_names_every_policy_the_repo_enforces`; that row reads
`REVIEW.md` and nothing else, so deleting the same rule from
`.claude/skills/sdlc/SKILL.md` was caught by nothing. Requirement 10 was in the
same position. Both are now held by
`the_pipeline_documents_ask_their_questions_before_review_does`, a three-row
table over the two documents. Requirement 9 was to be covered by
`every_gate_script_is_wired_or_declares_why_not`, and the mutation
`hook-stops-asking` — the hook keeps the `[ -f ]` test that names the script and
stops running it — passed. That guard sees a *reference*, not a call.
`the_stop_gate_reports_an_installed_build_that_is_behind` now asks
behaviourally, with a fixture check that prints something no other part of the
refusal would. **A guard that a mechanism is referenced is not a guard that it
runs**, and this change would have shipped three unguarded requirements without
the sweep.

**The hook's new lines are English, not the Japanese in `spec.md`'s Behaviour
section.** The spec said "Japanese, like the rest of `gate-stop.sh`'s refusal".
The rest of that refusal is English, because it is read by an agent rather than
by a person at a terminal — the spec was wrong about the file it was describing.
`scripts/check-installed-build.sh` is English for the same reason.

**The stamp is written with `plutil`, not `PlistBuddy`.** `package-macos.sh`
assembles the whole `Info.plist` with `plutil -insert`; a second tool for the
same job would be a second thing that can fail differently.

**`scripts/check-installed-build.sh` unsets `GIT_DIR` and `GIT_WORK_TREE`.**
Not in the plan, and found by the round-trip guard failing: `git -C` does not
override those, this suite runs inside a worktree that sets both, and the script
was answering about this repository while pointed at a fixture. The Stop gate
runs it from the judged tree, which is exactly the arrangement where the
pointers name something else. Change 065's lesson, one directory over.

**The guard asserts the order of the two roots, not only their presence.** The
mutation `roots-swapped` prints the hook's location and the judged tree the
other way round: every fact in the line stays true and the reader is sent to fix
the wrong checkout. That is this change's own rule — mutate the claim, not the
counter — applied to the change that writes it, and the assertion was added
because the mutation was not caught.

**Fourteen mutations, not eleven.** Three beyond one-per-requirement:
`mismatch-line-always-fires` (the diagnostic that fires on every stop, which is
the failure mode rather than the fix), `packager-stops-stamping`, and
`roots-swapped`. Two of the first eleven were rewritten after they passed for
the wrong reason: `contract-row-dropped` commented the row out, leaving the
substring a guard searches for, and the lesson mutation removed a name from an
entry that still held five others. **A mutation that cannot fail reads exactly
like a mutation that was caught.**

**The eval's own check passed for the wrong reason, twice, before it was
right.** `evals/007-mutate-the-claim-not-the-counter.md` was hand-verified in both directions as the plan asked, and
the first run was worthless: the suite was already red — `REVIEW.md` names the
eval and `scripts/check-installed-build.sh` did not exist yet — so "the suite
goes red under the mutation" was true without any guard existing. The check now
asserts a green baseline before mutating. The second run was worthless too: the
`perl` replacement wrote `$((orphan + 0))` unescaped, `perl` interpolated `$(`,
and the mutated script no longer parsed — so both directions were being judged
by a syntax error. The check now runs `/bin/bash -n` on what it mutated. Both
are in the eval as comments, because the next person to write a check like this
will reach for the same two shortcuts.

**Step 3 was pulled ahead of finishing step 2.** The eval's baseline assertion
needs a green suite, and `REVIEW.md` naming `evals/007-mutate-the-claim-not-the-counter.md`
made the not-yet-written `scripts/check-installed-build.sh` a red. The script
was written, then step 2's verification finished.

**Two false positives from prose, both fixed by rewording rather than by
widening a guard.** A comment added to `.claude/hooks/gate-stop.sh` named the
`.githooks` directory, and `the_review_gate_is_still_not_installed_by_anything_committed`
reads that name together with a hooks destination as evidence that something now
installs the git hook. And this change's own committed artifacts cited
`scripts/pipeline-indicators.sh` with `--lessons` inside one pair of backticks,
which `harness_documents_only_name_paths_that_exist` reads as a path — the plan
commit did not run the gates, because it changed only documents. Both are
recorded in lesson 042; the second is the argument for the first requirement of
whatever change next touches that guard.

**The round ceiling for `modify` is 2, not 3.** Read off
`docs/sdlc/review-rounds.yaml` rather than from the plan's risk table, which
said 3.
